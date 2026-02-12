//! Access to the External Coprocessor control registers.

use crate::Peripheral;

pub struct CoprocessorControl;

impl CoprocessorControl {
    /// Address of the Coprocessor Access Control Register (CPACR).
    const CPACR: usize = 0xE000ED88;

    /// Address of the Non-Secure Access Control Register (NSACR).
    const NSACR: usize = 0xE000ED8C;

    /// Configures access to the given coprocessor `cp`.
    /// Access from thread mode and non-secure code must be explicitly enabled
    /// by setting the corresponding argument as `true`.
    pub unsafe fn configure(&mut self, cp: u8, enable: bool, thread: bool, ns: bool) {
        // Read the registers.
        let mut cpacr = core::ptr::read_volatile(Self::CPACR as *const u32);
        let mut nsacr = core::ptr::read_volatile(Self::NSACR as *const u32);

        // Erase the previous values.
        cpacr &= !(0b11 << (cp * 2));
        nsacr &= !(1 << cp);

        // If the CP is not enabled return early.
        if !enable {
            core::ptr::write_volatile(Self::CPACR as *mut u32, cpacr);
            core::ptr::write_volatile(Self::NSACR as *mut u32, nsacr);
            return;
        }

        // Set the coprocessor configuration.
        cpacr |= (((thread as u32) << 1) | 1) << (cp * 2);
        nsacr |= (ns as u32) << cp;

        // Write back the registers.
        core::ptr::write_volatile(Self::CPACR as *mut u32, cpacr);
        core::ptr::write_volatile(Self::NSACR as *mut u32, nsacr);

        core::arch::asm!("dsb", "isb", options(nomem, nostack, preserves_flags));
    }
}

impl Peripheral for CoprocessorControl {
    unsafe fn instance() -> Self {
        Self
    }
}
