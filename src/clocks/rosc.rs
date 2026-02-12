//! Internal Ring Oscillator.

use core::ptr::{read_volatile, write_volatile};

pub struct RingOscillator;

impl RingOscillator {
    const BASE: usize = 0x400E8000;

    /// Control over the ROSC configuration.
    const CONTROL: usize = Self::BASE + 0x00;

    /// Control over the drive strength of some phases of the ROSC.
    const FREQA: usize = Self::BASE + 0x04;

    /// Control over the drive strength of some phases of the ROSC.
    const FREQB: usize = Self::BASE + 0x08;

    /// Register to change the randomiser seed.
    const RANDOM: usize = Self::BASE + 0x0C;

    /// Restores the `RingOscillator` to default enabled mode.
    pub(super) unsafe fn restore(&mut self) {
        write_volatile(Self::FREQA as *mut u32, 0);
        write_volatile(Self::FREQB as *mut u32, 0);
        write_volatile(Self::CONTROL as *mut u32, (0xFAB << 12) | 0xFA4);
    }
}
