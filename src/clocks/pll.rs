//! PLL Clock control.

use super::{Clock, ClockTrait};

/// The PLL controlling the System clock.
pub type PLLSystem = PLL<0>;

/// The PLL controlling the USB clock.
pub type PLLUSB = PLL<1>;

/// Control over a PLL.
pub struct PLL<const N: usize>;

impl<const N: usize> PLL<N> {
    const BASE: usize = 0x40050000 + (0x8000 * N);

    const CS: usize = Self::BASE + 0x00;

    const POWER: usize = Self::BASE + 0x04;

    const FBDIV: usize = Self::BASE + 0x08;

    const POSTDIV: usize = Self::BASE + 0x0C;

    /// Returns the final achieved frequency or `None` if it cannot be reached.
    pub(super) unsafe fn freeze(&mut self, xosc: u32, config: &PLLConfig) -> Option<u32> {
        let (fbdiv, refdiv, postdiv1, postdiv2) = config.configuration(xosc)?;

        // Calculate the final output.
        let vco = (xosc / refdiv) * fbdiv;
        let output = vco / (postdiv1 as u32 * postdiv2 as u32);

        defmt::debug!(
            "PLL Low Jitter (150 MHz): 12 * {} / {} = {} ({} / {}) = {}",
            fbdiv,
            refdiv,
            vco,
            postdiv1,
            postdiv2,
            output
        );

        // Configure the PLL.
        unsafe {
            core::ptr::write_volatile(Self::CS as *mut u32, refdiv);
            core::ptr::write_volatile(Self::FBDIV as *mut u32, fbdiv);
            core::ptr::write_volatile(Self::POWER as *mut u32, 0b001100);

            // Configure the postidividers.
            core::ptr::write_volatile(
                Self::POSTDIV as *mut u32,
                ((postdiv1 as u32) << 16) | ((postdiv2 as u32) << 12),
            );
            core::ptr::write_volatile(Self::POWER as *mut u32, 0b000100);
        }

        // Store the current frequency to the global clock list.
        super::CLOCKS[Self::CLOCKID as usize].store(output, core::sync::atomic::Ordering::Relaxed);

        Some(output)
    }
}

impl<const BASE: usize> ClockTrait for PLL<BASE> {
    const CLOCKID: Clock = if BASE == 0 {
        Clock::PLLSystem
    } else {
        Clock::PLLUSB
    };
}

impl<const BASE: usize> crate::Peripheral for PLL<BASE> {
    unsafe fn instance() -> Self {
        Self
    }
}

/// Configuration of one of the PLLs of the system.
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
pub struct PLLConfig {
    /// Selects which power mode to use for the PLL.
    pub mode: PLLPowerMode,

    /// Selects the target frequency of the PLL in Hz.
    pub target: u32,
}

impl PLLConfig {
    /// Calculates the PLL configuration and returns the feedback divider, referencce divider and the postdividers.
    pub(crate) fn configuration(&self, xosc: u32) -> Option<(u32, u32, u8, u8)> {
        // All possible post divider values.
        #[rustfmt::skip]
        const POSTDIV: [(u8, u8); 50] = [
            // 0 to 4 ratio.
            (1, 1), (1, 1), (2, 1), (3, 1), (4, 1),
            // 5 to 9 ratio.
            (5, 1), (6, 1), (7, 1), (4, 2), (3, 3),
            // 10 to 14 ratio.
            (5, 2), (6, 2), (6, 2), (7, 2), (7, 2),
            // 15 to 19 ratio.
            (5, 3), (4, 4), (6, 3), (6, 3), (5, 4),
            // 20 to 24 ratio.
            (5, 4), (7, 3), (6, 4), (6, 4), (6, 4),
            // 25 to 29 ratio.
            (5, 5), (7, 4), (7, 4), (7, 4), (6, 5),
            // 30 to 34 ratio.
            (6, 5), (7, 5), (7, 5), (7, 5), (7, 5),
            // 35 to 39 ratio.
            (7, 5), (6, 6), (7, 6), (7, 6), (7, 6),
            // 40 to 44 ratio.
            (7, 6), (7, 6), (7, 6), (7, 7), (7, 7),
            // 45 to 49 ratio.
            (7, 7), (7, 7), (7, 7), (7, 7), (7, 7),
        ];

        // Store the best configuration.
        let mut best = None;
        let mut residue = u32::MAX;

        // Calculate REFDIV according to RP datasheet.
        let refdiv = (xosc / 75) + 1;

        let propagate = |freq: u32, diva: u8, divb: u8| -> Option<(u32, u32)> {
            // Calculate the VCO and FBDIV for this post divider configuration.
            let vco = freq * (diva as u32) * (divb as u32);
            let fbdiv = vco * refdiv / xosc;

            // If the FBDIV is less or greater than the limits allowed this config is invalid.
            if (fbdiv < 16) || (fbdiv > 320) {
                return None;
            }

            // Get the residue for this configuration.
            let output = (xosc / refdiv) * fbdiv / (diva * divb) as u32;

            let residue = if output > freq {
                output - freq
            } else {
                freq - output
            };

            // Return the calculated residue and the FBDIV.
            Some((fbdiv, residue))
        };

        match self.mode {
            // For low jitter mode we need the highest possible VCO frequency, therefore the highest postdividers.
            PLLPowerMode::LowJitter => {
                for (diva, divb) in POSTDIV.iter().rev() {
                    if let Some((fbdiv, diff)) = propagate(self.target, *diva, *divb) {
                        if diff < residue {
                            best = Some((fbdiv, *diva, *divb));
                            residue = diff;
                        }
                    }
                }
            }

            // For low power mode we need the lowest possible VCO frequency, therefore the lowest postdividers.
            PLLPowerMode::LowPower => {
                for (diva, divb) in POSTDIV.iter() {
                    if let Some((fbdiv, diff)) = propagate(self.target, *diva, *divb) {
                        if diff < residue {
                            best = Some((fbdiv, *diva, *divb));
                            residue = diff;
                        }
                    }
                }
            }
        }

        // Returns the best configuration found.
        if let Some((fbdiv, diva, divb)) = best {
            return Some((fbdiv, refdiv, diva, divb));
        }

        None
    }
}

/// Selection of the power mode of a PLL.
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
pub enum PLLPowerMode {
    /// Reduces the power consumption at the expense of more clock jitter.
    LowPower,

    /// Reduces the clock jitter at the expense of more power consumption.
    LowJitter,
}
