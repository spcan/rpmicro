//! Vector Table and Exception and Interrupt control for the RP2350x.

use crate::{cortex::Configuration, Peripheral};

use super::{Exceptions, Interrupts, Vector};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
#[derive(Clone)]
#[repr(C)]
pub struct VectorTable {
    pub exceptions: Exceptions,
    pub interrupts: Interrupts,
}

impl VectorTable {
    /// Treats the given address as a `VectorTable` reference.
    /// UNSAFETY : No checks are performed to the address.
    pub const unsafe fn at(address: *mut u32) -> &'static mut VectorTable {
        unsafe { &mut *(address as *mut _) }
    }

    /// Returns a reference to the current `VectorTable`.
    pub unsafe fn current() -> &'static mut VectorTable {
        unsafe {
            let vtor = Configuration::instance().vtor();
            &mut *(vtor as *mut Self)
        }
    }

    /// Duplicates the current `VectorTable` to the given address.
    /// Is equivalent to `*VectorTable::at(address) = VectorTable::current().clone()`.
    pub unsafe fn duplicate(address: *mut u32) -> &'static mut VectorTable {
        // Get the reference to the new and old vector tables then clone the table.
        let new = Self::at(address);
        let current = Self::current();

        *new = current.clone();

        new
    }

    /// Installs this `VectorTable` for this core.
    pub unsafe fn install(&mut self) {
        Configuration::instance().vtrelocate(self as *mut Self as u32);
        core::arch::asm!("dsb", "isb", options(nomem, nostack, preserves_flags));
    }

    #[cfg(any(feature = "defmt", feature = "log"))]
    pub fn report() {
        let vtable = unsafe { Self::current() };

        {
            let mut exceptions = [0usize; 16];

            for i in 0..16 {
                exceptions[i] = unsafe { vtable[i].value };
            }

            crate::log::info!("Exceptions:\n{:#X}", exceptions);
        }

        {
            let mut interrrupts = [0usize; 52];

            for i in 0..52 {
                interrrupts[i] = unsafe { vtable[i + 16].value };
            }

            crate::log::info!("Interrupts:\n{:#X}", interrrupts);
        }
    }
}

impl core::ops::Index<usize> for VectorTable {
    type Output = Vector;

    fn index(&self, index: usize) -> &Self::Output {
        if index < 68 {
            return unsafe { &*((self as *const Self as usize + (4 * index)) as *const _) };
        }

        panic!("Vector Table index out of bounds {index}")
    }
}

impl core::ops::IndexMut<usize> for VectorTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < 68 {
            return unsafe { &mut *((self as *mut Self as usize + (4 * index)) as *mut _) };
        }

        panic!("Vector Table index out of bounds {index}")
    }
}
