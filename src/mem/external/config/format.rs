//! Format for Read and Write operations in the QMI.

use super::DataWidth;

#[derive(Clone, Eq, PartialEq)]
pub struct RWFormat(pub(crate) u32);

impl RWFormat {
    /// Returns a new empty format configuration.
    pub const fn new() -> Self {
        Self(0)
    }

    /// Sets the SPI configuration of the first byte sent in a transfer.
    pub const fn prefix(mut self, width: DataWidth) -> Self {
        self.0 &= !((0x3 << 0) | (1 << 12));
        self.0 |= (1 << 12) | ((width as u32) << 0);
        self
    }

    /// Sets the SPI configuration of the address.
    pub const fn address(mut self, width: DataWidth) -> Self {
        self.0 &= !(0x3 << 2);
        self.0 |= (width as u32) << 2;
        self
    }

    /// Sets the SPI configuration of the byte sent after the address.
    pub const fn suffix(mut self, width: DataWidth) -> Self {
        self.0 &= !((0x3 << 4) | (3 << 14));
        self.0 |= (2 << 14) | ((width as u32) << 4);
        self
    }

    /// Sets the amount of dummy SPI cycles to send after the address.
    pub const fn dummy(mut self, cycles: u8) -> Self {
        const ZERO: u32 = (0 << 16) | ((DataWidth::Single as u32) << 6);

        const CYCLES: [u32; 32] = [
            ZERO,
            (1 << 16) | ((DataWidth::Quad as u32) << 6),
            (2 << 16) | ((DataWidth::Quad as u32) << 6),
            (3 << 16) | ((DataWidth::Quad as u32) << 6),
            (4 << 16) | ((DataWidth::Quad as u32) << 6),
            (5 << 16) | ((DataWidth::Quad as u32) << 6),
            (6 << 16) | ((DataWidth::Quad as u32) << 6),
            (7 << 16) | ((DataWidth::Quad as u32) << 6),
            (2 << 16) | ((DataWidth::Single as u32) << 6),
            ZERO,
            (5 << 16) | ((DataWidth::Dual as u32) << 6),
            ZERO,
            (3 << 16) | ((DataWidth::Single as u32) << 6),
            ZERO,
            (7 << 16) | ((DataWidth::Dual as u32) << 6),
            ZERO,
            (4 << 16) | ((DataWidth::Single as u32) << 6),
            ZERO,
            ZERO,
            ZERO,
            (5 << 16) | ((DataWidth::Single as u32) << 6),
            ZERO,
            ZERO,
            ZERO,
            (6 << 16) | ((DataWidth::Single as u32) << 6),
            ZERO,
            ZERO,
            ZERO,
            (7 << 16) | ((DataWidth::Single as u32) << 6),
            ZERO,
            ZERO,
            ZERO,
        ];

        self.0 &= !((0x3 << 6) | (0x7 << 16));
        self.0 |= CYCLES[(cycles & 0x1F) as usize];
        self
    }

    /// Sets the SPI configuration of the data phase.
    pub const fn data(mut self, width: DataWidth) -> Self {
        self.0 &= !(0x3 << 8);
        self.0 |= (width as u32) << 8;
        self
    }

    /// Sets if the QMI should use Double Data Rate for the transfers.
    pub const fn ddr(mut self, enable: bool) -> Self {
        self.0 &= !(1 << 28);
        self.0 |= (enable as u32) << 28;
        self
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for RWFormat {
    fn format(&self, fmt: defmt::Formatter) {
        let format_width = |w: u32| -> &'static str {
            if w == DataWidth::Single as u32 {
                "Single"
            } else if w == DataWidth::Dual as u32 {
                "Dual"
            } else if w == DataWidth::Quad as u32 {
                "Quad"
            } else {
                "Unknown"
            }
        };

        defmt::write!(fmt, "RWFormat {{ prefix: ");
        if (self.0 & (1 << 12)) != 0 {
            defmt::write!(fmt, "Some({})", format_width((self.0 >> 0) & 0x3));
        } else {
            defmt::write!(fmt, "None");
        }

        defmt::write!(fmt, ", address: {}", format_width((self.0 >> 2) & 0x3));

        defmt::write!(fmt, ", suffix: ");
        if (self.0 & (3 << 14)) == (2 << 14) {
            defmt::write!(fmt, "Some({})", format_width((self.0 >> 4) & 0x3));
        } else {
            defmt::write!(fmt, "None");
        }

        defmt::write!(fmt, ", dummy: ");
        let d_width = (self.0 >> 6) & 0x3;
        let d_val = (self.0 >> 16) & 0x7;
        let cycles = if d_width == DataWidth::Quad as u32 {
            Some(d_val as u8)
        } else if d_width == DataWidth::Dual as u32 {
            match d_val {
                4 => Some(8),
                5 => Some(10),
                6 => Some(12),
                7 => Some(14),
                _ => None,
            }
        } else if d_width == DataWidth::Single as u32 {
            match d_val {
                4 => Some(16),
                5 => Some(20),
                6 => Some(24),
                7 => Some(28),
                _ => None,
            }
        } else {
            None
        };

        if let Some(c) = cycles {
            defmt::write!(fmt, "Some({})", c);
        } else {
            defmt::write!(fmt, "None");
        }

        defmt::write!(fmt, ", data: {}", format_width((self.0 >> 8) & 0x3));
        defmt::write!(fmt, ", ddr: {}", (self.0 & (1 << 28)) != 0);
        defmt::write!(fmt, " }}");
    }
}

#[cfg(feature = "log")]
impl std::fmt::Debug for RWFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RWFormat")
            .field("prefix", &self.prefix())
            .field("address", &self.address())
            .field("suffix", &self.suffix())
            .field("dummy", &self.dummy())
            .field("data", &self.data())
            .field("ddr", &self.ddr())
            .finish()
    }
}
