//! XIP cache control and performance counters.

/// Performance counters for the XIP cache.
pub struct CachePerformance;

impl CachePerformance {
    /// Base address of the XIP Control.
    const BASE: usize = 0x400C8000;

    /// Base offset of the hit Access register.
    const HIT: usize = Self::BASE + 0x0C;

    /// Base offset of the Counter Access register.
    const ACCESS: usize = Self::BASE + 0x10;

    /// Reads the total amount of accesses to the XIP cache since the last time this counter was cleared.
    pub fn access(&self) -> u32 {
        unsafe { core::ptr::read_volatile(Self::ACCESS as *const u32) }
    }

    /// Reads the amount of successful accesses to the XIP cache since the last time this counter was cleared.
    pub fn hit(&self) -> u32 {
        unsafe { core::ptr::read_volatile(Self::HIT as *const u32) }
    }

    /// Clears all the XIP performance counters.
    /// UNSAFETY : If an IRQ triggers in between operations, the caches will not be cleared at the same time.
    pub fn reset(&mut self) {
        unsafe { core::ptr::write_volatile(Self::ACCESS as *mut u32, 1) }
        unsafe { core::ptr::write_volatile(Self::HIT as *mut u32, 1) }
    }
}

impl crate::Peripheral for CachePerformance {
    unsafe fn instance() -> Self {
        Self
    }
}
