//! High level Cortex-M peripherals.

pub mod cp;
pub mod mpu;

pub use cp::CoprocessorControl;
pub use mpu::MemoryProtection;

use crate::Peripheral;

pub struct SystemControl {
    /// Access to the general configuration registers.
    pub configuration: Configuration,

    /// Access to the External Coprocessor control registers.
    pub coprocessor: CoprocessorControl,

    /// Access to the Memory Protection Unit control registers.
    pub mpu: MemoryProtection,
}

impl Peripheral for SystemControl {
    unsafe fn instance() -> Self {
        Self {
            configuration: Configuration::instance(),
            coprocessor: CoprocessorControl::instance(),
            mpu: MemoryProtection::instance(),
        }
    }
}

pub struct Configuration;

impl Configuration {
    /// Vector Table Offset Register.
    const VTOR: usize = 0xE000ED08;

    /// System Control Register.
    const SCR: usize = 0xE000ED10;

    /// Configuration and Control Register.
    const CCR: usize = 0xE000ED14;

    /// Internal method to set a specific bit in a register.
    fn set<const ADDRESS: usize, const OFFSET: u8>(value: bool) {
        unsafe {
            let register = core::ptr::read_volatile(ADDRESS as *const u32);
            core::ptr::write_volatile(
                ADDRESS as *mut u32,
                (register & !(1 << OFFSET)) | ((value as u32) << OFFSET),
            );
        }
    }

    /// Controls the behaviour of the Usage Fault for Division by Zero exception.
    /// If set, dividing by zero will generate an exception.
    pub fn divbyzero(&mut self, forbid: bool) {
        Self::set::<{ Self::CCR }, 4>(forbid)
    }

    /// Controls the behaviour of the Memory Fault for Unaligned Access exception.
    /// If set, unaligned accesses will generate an exception.
    pub fn unaligned(&mut self, forbid: bool) {
        Self::set::<{ Self::CCR }, 3>(forbid)
    }

    /// Controls the behaviour of the Secure Fault for User Setting Pending exception.
    /// If set, unpriviledged code attempting to set a pending bit will generate an exception.
    pub fn userpend(&mut self, forbid: bool) {
        Self::set::<{ Self::CCR }, 1>(forbid)
    }

    /// Controls the sleep behaviour of the system.
    /// If set, the system will enter into deep sleep mode instead of normal sleep mode.
    pub fn deepsleep(&mut self, enabled: bool) {
        Self::set::<{ Self::SCR }, 2>(enabled)
    }

    /// Relocates the Vector Table address.
    pub fn vtrelocate(&mut self, address: u32) {
        unsafe {
            core::ptr::write_volatile(Self::VTOR as *mut u32, address);
        }
    }

    /// Returns the current VTOR address.
    pub fn vtor(&self) -> u32 {
        unsafe { core::ptr::read_volatile(Self::VTOR as *const u32) }
    }
}

impl Peripheral for Configuration {
    unsafe fn instance() -> Self {
        Self
    }
}
