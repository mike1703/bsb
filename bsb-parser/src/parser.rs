use nom::bytes::streaming::{tag, take, take_till};
use nom::combinator::{map, verify};
use nom::error::context;
use nom::number::streaming::{be_u16, be_u32, u8};
use nom::Parser as _;
use nom_language::error::{VerboseError, VerboseErrorKind};
use strum_macros::{EnumString, IntoStaticStr};

use crate::{Frame, SOF};

pub enum ParseResult<'a> {
    /// Successfully parsed frame and unparsed rest
    Ok { rest: &'a [u8], frame: Frame },
    /// Not enough data, please provide more bytes
    Incomplete,
    /// Unrecoverable Error, broken data and unparsed rest
    Failure {
        rest: &'a [u8],
        broken_data: &'a [u8],
        error: ParseErrorKind,
    },
}

#[derive(EnumString, IntoStaticStr)]
pub enum ParseErrorKind {
    ChecksumError,
    InvalidLength,
}

pub type NomParseResult<T, U> = nom::IResult<T, U, VerboseError<T>>;

/// Match until SOF
fn take_until_sof(input: &[u8]) -> NomParseResult<&[u8], &[u8]> {
    take_till(|b| b == SOF)(input)
}

/// Parse a bsb frame and throw away any garbage at the beginning.
/// Returns the remaining/unparsed bytes and the `Frame` if successfull or a `VerboseError`
pub fn frame_parser(data: &[u8]) -> NomParseResult<&[u8], Frame> {
    // message_start is the message beginning with the SYNCBYTE
    let (message_start, _) = take_until_sof(data)?;
    let (input, _) = tag(&[SOF][..]).parse(message_start)?;
    let (input, source_address) = map(u8, |source| source ^ 0x80).parse(input)?;
    let (input, destination_address) = u8(input)?;
    let (input, header_length) = context(
        ParseErrorKind::InvalidLength.into(),
        // at least 11 (required for minimum message) but max 70 (arbitrary max length)
        verify(u8, |&header_length| {
            (4 + 4 + 2 + 1..70).contains(&header_length)
        }),
    )
    .parse(input)?;
    let payload_len = header_length - 4 - 4 - 2 - 1; // -4 header -4 field id -2 CRC -1 SOF byte
    let (input, packet_type) = u8(input)?;
    let (input, field_id) = map(be_u32, |field_id| {
        if packet_type == 3 || packet_type == 6 {
            // during sets (3) and gets (6) these id bytes are reversed
            (field_id & 0x0000_ffff)
                | ((field_id >> 8) & 0x00ff_0000)
                | ((field_id << 8) & 0xff00_0000)
        } else {
            field_id
        }
    })
    .parse(input)?;
    let (input, payload) = take(payload_len)(input)?;
    let (_, message_without_checksum) = take(header_length - 2)(message_start)?;
    let calculated_crc = crc16::State::<crc16::XMODEM>::calculate(message_without_checksum);
    let (input, _crc) = context(
        ParseErrorKind::ChecksumError.into(),
        verify(be_u16, |&crc| crc == calculated_crc),
    )
    .parse(input)?;

    Ok((
        input,
        Frame {
            destination_address,
            source_address,
            packet_type,
            field_id,
            payload: payload.to_vec(),
        },
    ))
}

/// Parse the `input` slice into `Ok(remaining_bytes, Frame)`, `Incomplete` or `Error`
pub fn parser(input: &[u8]) -> ParseResult {
    match frame_parser(input) {
        Ok((rest, frame)) => ParseResult::Ok { rest, frame },
        Err(error) => match error {
            nom::Err::Incomplete(_n) => ParseResult::Incomplete,
            // treat recoverable errors and failures the same
            nom::Err::Error(error) | nom::Err::Failure(error) => {
                // the last error contains the real error
                let (rest, error) = error.errors.last().unwrap();
                let error = match error {
                    // unfortunately errors can only be reported with context strings... but this code is backed with enums
                    VerboseErrorKind::Context(context) => {
                        ParseErrorKind::try_from(*context).unwrap()
                    }
                    // the next two parsers cannot happen due to parser construction
                    VerboseErrorKind::Char(_) | VerboseErrorKind::Nom(_) => unimplemented!(),
                };
                ParseResult::Failure {
                    rest,
                    broken_data: input,
                    error,
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use nom_language::error::VerboseErrorKind;

    use crate::{
        parser::{frame_parser, ParseErrorKind},
        Frame,
    };

    #[test]
    fn test_parse_get_message() {
        let data = &[220, 194, 0, 11, 6, 61, 5, 25, 240, 36, 62];
        let want = Frame {
            destination_address: 0,
            source_address: 66,
            packet_type: 6,
            field_id: 87890416,
            payload: vec![],
        };
        let (rest, broetje) = frame_parser(data).unwrap();
        assert_eq!(want, broetje);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_parse_ret_message() {
        let data = &[220, 128, 66, 14, 7, 5, 61, 25, 240, 0, 0, 15, 29, 116];
        let want = Frame {
            destination_address: 66,
            source_address: 0,
            packet_type: 7,
            field_id: 87890416,
            payload: vec![0, 0, 15],
        };
        let (rest, broetje) = frame_parser(data).unwrap();
        assert_eq!(want, broetje);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_frame_too_short_minimum() {
        let data = &[220, 1, 2, 11, 4, 5, 6, 7, 8, 9];
        let error = frame_parser(data).expect_err("fail");
        assert_eq!(
            error,
            nom::Err::Incomplete(nom::Needed::Size(std::num::NonZeroUsize::new(1).unwrap()))
        );
    }

    #[test]
    fn test_frame_too_short() {
        let data = &[220, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let error = frame_parser(data).expect_err("fail");
        assert_eq!(
            error,
            nom::Err::Incomplete(nom::Needed::Size(std::num::NonZeroUsize::new(1).unwrap()))
        );
    }

    #[test]
    fn test_header_length_invalid_low() {
        let data = &[220, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let nom::Err::Error(result) = frame_parser(data).expect_err("fail") else {
            panic!()
        };
        assert_eq!(
            result.errors[1].1,
            VerboseErrorKind::Context(ParseErrorKind::InvalidLength.into())
        );
    }

    #[test]
    fn test_header_length_invalid_high() {
        let data = &[220, 0, 0, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let nom::Err::Error(result) = frame_parser(data).expect_err("fail") else {
            panic!()
        };
        assert_eq!(
            result.errors[1].1,
            VerboseErrorKind::Context(ParseErrorKind::InvalidLength.into())
        );
    }

    #[test]
    fn test_no_sof() {
        let data = &[0xBB, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0];
        let error = frame_parser(data).expect_err("fail");
        assert_eq!(
            error,
            nom::Err::Incomplete(nom::Needed::Size(std::num::NonZeroUsize::new(1).unwrap()))
        );
    }

    #[test]
    fn test_leading_garbage_then_ok() {
        let data = &[0, 1, 2, 3, 220, 194, 0, 11, 6, 61, 5, 25, 240, 36, 62];
        let want = Frame {
            destination_address: 0,
            source_address: 66,
            packet_type: 6,
            field_id: 87890416,
            payload: vec![],
        };
        let (rest, broetje) = frame_parser(data).unwrap();
        assert_eq!(want, broetje);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_frame_crc_error() {
        let data = &[220, 0, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let nom::Err::Error(result) = frame_parser(data).expect_err("fail") else {
            panic!()
        };
        assert_eq!(
            result.errors[1].1,
            VerboseErrorKind::Context(ParseErrorKind::ChecksumError.into())
        );
    }
}
