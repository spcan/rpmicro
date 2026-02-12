//! TRNG Interrupt Service Routine.

use core::ptr::{read_volatile, write_volatile};

use super::address::*;

/// List of wakers for all the DMA channels.
pub(super) static mut DMAWAKERS: [Option<()>; 16] = [None; 16];

pub(crate) unsafe extern "C" fn handler<const CPU: usize, const HALF: usize>() {
    // Precompute the STATUS register.
    let register = INTSTATUS + (INTCPUSTRIDE * CPU) + (INTHALFSTRIDE * HALF);

    // Clear the interrupt in the NVIC.

    // Read the corresponding ISR list and clear the pending interrupts.
    let status = read_volatile(register as *const u32);
    write_volatile(register as *mut u32, status);

    let offset = HALF * 8;

    for i in 0..8 {
        let mask = 1 << (i + offset);

        if (status & mask) == 1 {
            if let Some(waker) = &DMAWAKERS[i] {}
        }
    }
}
