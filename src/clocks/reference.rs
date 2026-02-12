//! Definition of the internal reference clock.

use super::{Clock, ClockControl, ClockTrait};

pub struct ReferenceClock;

impl ReferenceClock {
    /// Restores the reference clock to the default state.
    pub(super) unsafe fn restore(&mut self) {
        core::ptr::write_volatile(Self::CONTROL as *mut u32, 0);
    }

    /// Configures the `ReferenceClock` to source the time from the given clock source.
    pub(super) fn configure(&mut self, source: super::BaseClock) {
        unsafe {
            core::ptr::write_volatile(Self::CONTROL as *mut u32, source as u32);
        }
    }
}

impl ClockTrait for ReferenceClock {
    const CLOCKID: Clock = Clock::Reference;
}

impl ClockControl for ReferenceClock {
    const BASE: usize = 0x40010000 + 0x30;
}
