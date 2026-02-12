//! TRNG Interrupt Service Routine.

use core::ptr::{read_volatile, write_volatile, NonNull};

use crate::hal::interrupts::{queue::ISRQueueNode, Interrupt};

/// Descriptor of the current TRNG operation.
#[link_section = ".data"]
pub(super) static mut OPERATION: Option<NonNull<ISRQueueNode<super::operation::Descriptor>>> = None;

pub(crate) unsafe extern "C" fn handler() {
    // Get the status and check why this ISR was triggered, then clear the interrupts that triggered this routine.
    let status = read_volatile(super::address::ISR as *const u32);
    write_volatile(super::address::ICR as *mut u32, status);
    <super::TRNGenerator as Interrupt>::clear();

    // Ensure there is a descriptor to service. If there isn't stop the engine and exit the handler.
    let node = match OPERATION {
        Some(mut pointer) => pointer.as_mut(),
        None => return disable(),
    };

    let descriptor = node.object();

    // If any of the Von Neumman, CRNGT or autocorrelation errors are set restart the process and exit the handler.
    if (status & 0b1110) != 0 {
        crate::log::trace!("Failed RNG tests. Restarting...");
        return super::TRNGenerator::reset();
    }

    // If there is valid data write it to the current subscriber and exit.
    if (status & 1) == 1 {
        // Read out the random data.
        let iterator = (0..6).zip(descriptor.buffer()[descriptor.current..].chunks_mut(4));

        for (rngindex, chunk) in iterator {
            let rng = read_volatile((super::address::DATA as *mut u32).offset(rngindex));

            for (dst, src) in chunk.iter_mut().zip(rng.to_ne_bytes()) {
                descriptor.current += 1;
                *dst = src;
            }
        }

        // If the request is finished, stop the engine and get the next descriptor.
        if descriptor.current >= descriptor.size {
            disable();
            node.finish();
            node.wake();
        }
    }
}

/// Disables the TRN interrupts.
#[inline(always)]
unsafe fn disable() {
    write_volatile(super::address::IMR as *mut u32, 0xF);
    <super::TRNGenerator as Interrupt>::disable();
}
