//! Interrupts table and handling.

mod vtable;

pub(super) use vtable::Interrupts;

use super::{Vector, VectorTable};

/// Common trait for all the peripherals that interact with an interrupt.
pub trait Interrupt {
    /// The interrupt number.
    /// Sets the hardware offsets of the control registers and the position in the vector table.
    const NUMBER: usize;

    /// Enables the interrupt in the NVIC.
    unsafe fn enable() {
        const ISER: usize = 0xE000E100;

        unsafe {
            core::ptr::write_volatile(
                (ISER + (4 * (Self::NUMBER / 32))) as *mut u32,
                1u32 << (Self::NUMBER % 32),
            );
        }

        core::arch::asm!("dsb", "isb", options(nomem, nostack, preserves_flags));
    }

    /// Disables the interrupt in the NVIC.
    unsafe fn disable() {
        const ICER: usize = 0xE000E180;

        unsafe {
            core::ptr::write_volatile(
                (ICER + (4 * (Self::NUMBER / 32))) as *mut u32,
                1 << (Self::NUMBER % 32),
            );
        }

        core::arch::asm!("dsb", "isb", options(nomem, nostack, preserves_flags));
    }

    /// Clears the pending interrupt in the NVIC.
    unsafe fn clear() {
        const ICPR: usize = 0xE000E280;

        unsafe {
            core::ptr::write_volatile(
                (ICPR + (4 * (Self::NUMBER / 32))) as *mut u32,
                1 << (Self::NUMBER % 32),
            );
        }

        core::arch::asm!("dsb", "isb", options(nomem, nostack, preserves_flags));
    }

    /// Pends the interrupt in the NVIC.
    unsafe fn pend() {
        const ISPR: usize = 0xE000E200;

        unsafe {
            core::ptr::write_volatile(
                (ISPR + (4 * (Self::NUMBER / 32))) as *mut u32,
                1 << (Self::NUMBER % 32),
            );
        }

        core::arch::asm!("dsb", "isb", options(nomem, nostack, preserves_flags));
    }

    /// Sets the handler function for this interrupt.
    unsafe fn install(handler: unsafe extern "C" fn()) {
        VectorTable::current()[Self::NUMBER + 16] = Vector::target(handler);
        core::arch::asm!("dsb", "isb", options(nomem, nostack, preserves_flags));
    }
}
