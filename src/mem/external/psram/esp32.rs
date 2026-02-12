//! Implementation of the ESP32PSRAM64H chips.

use crate::mem::external::{
    config::{Commands, Config, DataWidth, RWFormat},
    Device,
};

/// Commands used to reset the chip.
#[link_section = ".data"]
static RESET: [(DataWidth, u8); 3] = [
    (DataWidth::Quad, Command::ExitQuadMode as u8),
    (DataWidth::Single, Command::ResetEnable as u8),
    (DataWidth::Single, Command::Reset as u8),
];

mod single {
    use super::*;

    /// Read format in SPI mode.
    pub const RDFORMAT: RWFormat = RWFormat::new()
        .prefix(DataWidth::Single)
        .dummy(8)
        .address(DataWidth::Single)
        .data(DataWidth::Single)
        .ddr(false);

    /// Write format in SPI mode.
    pub const WRFORMAT: RWFormat = RWFormat::new()
        .prefix(DataWidth::Single)
        .dummy(0)
        .address(DataWidth::Single)
        .data(DataWidth::Single)
        .ddr(false);
}

mod quad {
    use super::*;

    /// Commands used to enter Quad SPI mode.
    #[link_section = ".data"]
    pub static ENTERQUAD: [(DataWidth, u8); 1] =
        [(DataWidth::Single, Command::EnterQuadMode as u8)];

    /// Read format in Quad SPI mode.
    pub const RDFORMAT: RWFormat = RWFormat::new()
        .prefix(DataWidth::Quad)
        .dummy(6)
        .address(DataWidth::Quad)
        .data(DataWidth::Quad)
        .ddr(false);

    /// Write format in Quad SPI mode.
    pub const WRFORMAT: RWFormat = RWFormat::new()
        .prefix(DataWidth::Quad)
        .dummy(0)
        .address(DataWidth::Quad)
        .data(DataWidth::Quad)
        .ddr(false);
}

/// Implementation of the ESP32PSRAM64H chip in SPI mode.
pub struct PSRAMSingle;

impl Device for PSRAMSingle {
    fn config() -> Config {
        let commands = Commands {
            reset: &RESET,
            mode: None,
            read: Command::ReadFast as u8,
            write: Command::Write as u8,
        };

        let rdformat = single::RDFORMAT;
        let wrformat = single::WRFORMAT;

        Config {
            commands,
            rdformat,
            wrformat,
        }
    }
}

/// Implementation of the ESP32PSRAM64H chip in QuadSPI mode.
pub struct PSRAMQuad;

impl Device for PSRAMQuad {
    fn config() -> Config {
        let commands = Commands {
            reset: &RESET,
            mode: Some(&quad::ENTERQUAD),
            read: Command::ReadQuad as u8,
            write: Command::WriteQuad as u8,
        };

        let rdformat = quad::RDFORMAT;
        let wrformat = quad::WRFORMAT;

        Config {
            commands,
            rdformat,
            wrformat,
        }
    }
}

/// Default commands for PSRAM devices.
#[derive(Clone, Copy, Debug)]
#[allow(unused)]
pub(super) enum Command {
    /// Write in SPI mode, fast frequency.
    Write = 0x02,

    /// Read in SPI mode, slow frequency.
    ReadSlow = 0x03,

    /// Read in SPI mode, fast frequency.
    ReadFast = 0x0B,

    /// Sets the chip in Quad SPI mode.
    EnterQuadMode = 0x35,

    /// Writes in Quad SPI mode, fast frequency.
    WriteQuad = 0x38,

    /// Enables the chip to be reset by the next command.
    /// This effect gets cancelled if the next command is not a `Reset`.
    ResetEnable = 0x66,

    /// Resets the chip.
    /// Must be preceded by a `ResetEnable` command to take effect.
    Reset = 0x99,

    /// Reads the KGD and EID registers.
    ReadID = 0x9F,

    /// Read in Quad SPI mode, fast frequency.
    ReadQuad = 0xEB,

    /// Default command to exit Quad Mode I/O.
    ExitQuadMode = 0xF5,
}
