//! List of commands and command sequences used for this chip.

use crate::mem::external::config::DataWidth;

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
pub struct Commands {
    /// Command sequence used to reset the device.
    pub reset: &'static [(DataWidth, u8)],

    /// Command sequence used to enter the desired R/W mode.
    pub mode: Option<&'static [(DataWidth, u8)]>,

    /// Command to issue a read.
    pub read: u8,

    /// Command to issue a write.
    pub write: u8,
}
