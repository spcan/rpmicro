//! Raw IRQ handler for GPIO input.

use crate::{cpuid, AtomicRegister};

use super::*;

use core::task::Waker;

pub(crate) unsafe fn handler() {
    // Mask for a full GPIO interrupt group.
    const MASK: u32 = 0xF;

    // Precompute some offsets to reduce operations in the IRQ.
    let stride = CPUSTRIDE * cpuid() as usize;

    // Map per group to reduce read operations.
    for group in 0..(crate::gpio::GPIOCOUNT / 8) {
        // Precompute this to reduce instructions in the handler.
        let groupx4 = group * 4;

        // Get the registers for this group.
        let inte = AtomicRegister::at(INTE + groupx4 + stride);
        let ints = AtomicRegister::at(INTS + groupx4 + stride).read();

        for pin in 0..8 {
            // Precompute this to reduce instructions in the handler.
            let pinx4 = pin * 4;

            // Check if an event happened on this pin.
            // If so, disable the interrupt and wake the task.
            let event = (ints >> pinx4) & MASK;
            if event != 0 {
                inte.clear(MASK << pinx4);
                if let Some(waker) = WAKERS[(group * 8) + pin].take() {
                    waker.wake();
                }
            }
        }
    }
}

#[cfg(feature = "QFN60")]
pub(super) static mut WAKERS: [Option<Waker>; crate::gpio::GPIOCOUNT] = [
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
];

#[cfg(feature = "QFN80")]
pub(super) static mut WAKERS: [Option<Waker>; crate::gpio::GPIOCOUNT] = [
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
];

#[cfg(not(any(feature = "QFN60", feature = "QFN80")))]
pub(super) static mut WAKERS: [Option<Waker>; crate::gpio::GPIOCOUNT] = [];
