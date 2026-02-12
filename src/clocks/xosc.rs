//! Definition of the external crystal oscillator.
//! Must contain the nominal frequency of the oscillator in Hz.

use core::{
    ptr::{read_volatile, write_volatile},
    sync::atomic::{AtomicU32, Ordering},
};

#[repr(transparent)]
pub struct CrystalOscillator(pub Option<core::num::NonZero<u32>>);

impl CrystalOscillator {
    /// Base address of the `CrystalOscillator` registers.
    const BASE: usize = 0x40048000;

    /// Crystal Oscillator Control.
    const CTRL: usize = Self::BASE + 0x00;

    /// Crystal Oscillator Status.
    const STATUS: usize = Self::BASE + 0x04;

    /// Crystal Oscillator pause control.
    const DORMANT: usize = Self::BASE + 0x08;

    /// Controls the startup delay.
    /// This delay should be at least 1 ms.
    const STARTUP: usize = Self::BASE + 0x0C;

    /// A down counter running at the XOSC frequency which counts to zero and stops.
    const COUNT: usize = Self::BASE + 0x10;

    /// Disables the `CrystalOscillator`.
    pub(super) unsafe fn disable(&self) {
        write_volatile(Self::CTRL as *mut u32, 0xD1E << 12);
        super::ClockList::XOsc.write(0);
    }

    /// Disables the `CrystalOscillator`.
    pub(super) unsafe fn enable(&self, wait: bool) {
        // Set the frequency range and the startup delay.
        if let Some(freq) = self.0 {
            let range = match u32::from(freq) / 1_000_000 {
                0..=14 => 0xAA0,
                15..=28 => 0xAA1,
                29..=56 => 0xAA2,
                _ => 0xAA3,
            };

            let delay = (u32::from(freq) / 256_000) + 1;

            unsafe {
                write_volatile(Self::STARTUP as *mut u32, delay);
                write_volatile(Self::CTRL as *mut u32, (0xFAB << 12) | range);
            }

            // Wait for the stabilization of the oscillator.
            crate::log::info!("Started XOsc @ {} Hz. Waiting for stabilization...", freq);

            if wait {
                while !self.stable() {
                    unsafe { core::arch::asm!("nop", options(nomem, nostack, preserves_flags)) }
                }
            }

            // Report the current speed.
            super::ClockList::XOsc.write(u32::from(freq));
        }
    }

    /// Returns the programmed frequency of the `CrystalOscillator`.
    #[inline]
    pub fn freq(&self) -> u32 {
        if let Some(freq) = self.0 {
            return u32::from(freq);
        }

        0
    }

    /// Returns `true` if the `CrystalOscillator` is stable.
    #[inline]
    pub fn stable(&self) -> bool {
        unsafe { read_volatile(Self::STATUS as *const u32) & (1u32 << 31) != 0 }
    }
}
