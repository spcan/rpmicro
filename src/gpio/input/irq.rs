//! Raw IRQ handler for GPIO input.

use crate::{cpuid, AtomicRegister};

const BASE: usize = 0x40028000;

const INTE: usize = BASE + 0x248;

const INTF: usize = BASE + 0x260;

const INTS: usize = BASE + 0x278;

const CPUSTRIDE: usize = 0x290 - 0x248;

// pub(super) static mut WAKERS: [(); GPIOCOUNT] = [(); GPIOCOUNT];

pub(crate) unsafe fn handler() {
    // Precompute some offsets to reduce operations in the IRQ.
    let stride = CPUSTRIDE * cpuid() as usize;

    // Map per group to reduce read operations.
    for group in 0..6 {
        // Get the registers for this group.
        let inte = AtomicRegister::at(INTE + (group * 4) + stride);
        let ints = AtomicRegister::at(INTS + (group * 4) + stride).read();

        for pin in 0..8 {
            // Check if an event happened on this pin.
            // If so, disable the interrupt and wake the task.
            let event = (ints >> (pin * 4)) & 0xf;
            if event != 0 {
                inte.clear(0xFu32 << (pin * 4));
                // wakers[(group * 8) + pin].wake();
            }
        }
    }
}
