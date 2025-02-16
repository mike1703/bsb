use serde::{Deserialize, Serialize};

/// The Datatype enum is aligned with the Value enum.
/// This type stores the information about the type/encoding
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Datatype {
    /// Settings with states mapped to unsigned ints. The number is the maximum value of the "highest" settings for this field (e.g. 1 for [Off(0),On(1)])
    /// The mapping to strings is not yet defined
    Setting(u8),
    /// Integer value
    Number,
    /// Float with a division factor, e.g. pressure → 10, slope → 50, temperature → 64
    Float(u8),
    DateTime,
    Schedule,
}
