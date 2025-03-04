use serde::{Deserialize, Serialize};

/// The Datatype enum is aligned with the Value enum.
/// This type stores the information about the type/encoding
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Datatype {
    /// settings with states mapped to unsigned ints. The number tells the amount of settings for this field (e.g. 2 for [On, Off])
    /// The mapping to strings is not yet defined
    Setting(u8),
    /// a integer value
    Number,
    /// a float with a division factor, e.g. pressure → 10, slope → 50, temperature → 64
    Float(u8),
    DateTime,
    Schedule,
}
