//! The timings of an external memory.

#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
pub struct Timings {
    /// Indicates the cooldown period (in units of 64 system clock cycles) after
    /// the end of a transfer in which the QMI will keep an external memory selected.
    /// The maximum value of this cooldown period is 3 (192 system clocks).
    /// This value will be forced to at least 1 if `pagebreak` is enabled.
    /// This cooldown period will be terminated if:
    ///  - A new transfer to a different memory bank begins
    ///  - A new transfer in a different direction (Read or Write) begins
    ///  - Page boundary limits is reached (as selected by `pagebreak`)
    ///  - Maximum select time has been reached (as selected by `select` configuration)
    pub cooldown: Option<u8>,

    /// Indicates the boundary alignment that cannot be crossed in a continuous transfer.
    pub pagebreak: Option<Pagebreak>,

    /// Indicates delays between the beginning and end of a transfer and select signal.
    /// First delay is the start delay, which has a fixed duration of 1 system clock (not SPI clock).
    /// Second delay is the end delay, which can be between 0 and 3 system clock cycles (not SPI clock).
    pub csdelay: Option<(bool, u8)>,

    /// Indicates the maximum select time for a SPI interface in units of 64 system clock cycles (not SPI clock).
    /// Might be needed for RAM memories in order to do refresh operations.
    /// Maximum value is 64 (4096 system clock cycles)
    pub select: Option<u8>,

    /// Indicates the minimum deselect time for a SPI interface in system clock cycles (not SPI clock).
    /// Might be needed for RAM memories in order to do refresh operations.
    /// Maximum value is 31 system clock cycles.
    pub deselect: Option<u8>,

    /// Indicates the delay between SCK high and data read, in system clock cycles (not SPI clock).
    /// Higher frequencies might need some delays in order to account for signal transmission.
    /// Maximum RX delay is 7 system clock cycles.
    pub rxdelay: u8,

    /// Indicates the clock divisor for the SPI interface. A divisor of 256 is represented as 0.
    pub clkdiv: u8,
}

impl Default for Timings {
    fn default() -> Self {
        Timings {
            cooldown: None,
            pagebreak: Some(Pagebreak::Small),
            csdelay: Some((true, 3)),
            select: Some(1),
            deselect: Some(31),
            rxdelay: 0,
            clkdiv: 2,
        }
    }
}

impl Into<u32> for Timings {
    fn into(self) -> u32 {
        // Create the output with the guaranteed existing data.
        let mut out = ((self.rxdelay as u32) << 8) | (self.clkdiv as u32);

        // Set the interdependent data: cooldown and pagebreak.
        match (self.cooldown, self.pagebreak) {
            (Some(cooldown), Some(pagebreak)) => {
                out |= ((cooldown.min(1) as u32 & 0x3) << 30) | ((pagebreak as u32) << 28)
            }
            (Some(cooldown), None) => out |= (cooldown as u32 & 0x3) << 30,
            (None, Some(pagebreak)) => out |= (1 << 30) | ((pagebreak as u32) << 28),
            (None, None) => (),
        }

        // Include the optional data: CS delay, select and deselect.
        if let Some((setup, hold)) = self.csdelay {
            out |= (hold as u32 & 0x3) << 23;

            if setup {
                out |= 1 << 25;
            }
        }

        if let Some(select) = self.select {
            out |= (select as u32 & 0x3F) << 17;
        }

        if let Some(deselect) = self.deselect {
            out |= (deselect as u32 & 0x1F) << 12;
        }

        out
    }
}

/// Boundaries for the page break option.
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
pub enum Pagebreak {
    /// Corresponds with a 256-byte page boundary.
    Small = 1,

    /// Corresponds with a 1024-byte page boundary.
    Big = 2,

    /// Corresponds with a 4096-byte page boundary.
    Huge = 3,
}
