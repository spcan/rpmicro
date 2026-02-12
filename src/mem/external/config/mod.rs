//! Configuration of commonly used Flash and PSRAM chips.

mod commands;
mod format;
mod width;

pub use commands::Commands;
pub use format::RWFormat;
pub use width::DataWidth;

/// Configuration of an external memory device.
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
pub struct Config {
    /// List of commands to interact with the chip.
    pub commands: Commands,

    /// The format for read operations.
    pub rdformat: RWFormat,

    /// The format for write operations.
    pub wrformat: RWFormat,
}
