//! Atomic register properties of some registers in the RP series.

#![allow(unused)]

#[repr(transparent)]
pub struct AtomicRegister(pub u32);

impl AtomicRegister {
    /// Maps the register at the given location.
    #[inline(always)]
    pub(crate) const fn at<'a>(address: usize) -> &'a mut AtomicRegister {
        unsafe { &mut *(address as *mut Self) }
    }

    /// Reads the `AtomicRegister`s contents.
    #[inline(always)]
    pub fn read(&self) -> u32 {
        unsafe { core::ptr::read_volatile(self as *const Self as *const u32) }
    }

    /// Writes to the `AtomicRegister`s contents.
    #[inline(always)]
    pub fn write<V: Into<u32>>(&mut self, value: V) {
        unsafe { core::ptr::write_volatile(self as *mut Self as *mut u32, value.into()) }
    }

    /// Clears the given bitmask in the register.
    #[inline(always)]
    pub fn clear<B: Into<u32>>(&mut self, bitmask: B) {
        unsafe {
            core::ptr::write_volatile(
                (self as *mut Self as *mut u32 as usize + 0x3000) as *mut u32,
                bitmask.into(),
            );
        }
    }

    /// Sets the given bitmask in the register.
    #[inline(always)]
    pub fn set<B: Into<u32>>(&mut self, bitmask: B) {
        unsafe {
            core::ptr::write_volatile(
                (self as *mut Self as *mut u32 as usize + 0x2000) as *mut u32,
                bitmask.into(),
            );
        }
    }

    /// Toggles the given bitmask in the register. Equivalent to XOR.
    #[inline(always)]
    pub fn toggle<B: Into<u32>>(&mut self, bitmask: B) {
        unsafe {
            core::ptr::write_volatile(
                (self as *mut Self as *mut u32 as usize + 0x1000) as *mut u32,
                bitmask.into(),
            );
        }
    }
}
