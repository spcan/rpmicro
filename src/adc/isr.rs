//! Interrupt Service Routine (ISR) for the ADC.

use core::ptr::NonNull;

use crate::hal::{
    interrupts::{queue::ISRQueueNode, Interrupt},
    AtomicRegister,
};

use super::Operation;

/// Descriptor of the current ADC operation.
#[link_section = ".data"]
pub(super) static mut OPERATION: Option<NonNull<ISRQueueNode<super::ADCOperation>>> = None;

pub(crate) unsafe extern "C" fn handler() {
    // Clear the interrupt in the ADC.
    AtomicRegister::at(super::address::INTE).write(0u32);
    <super::ADCEngine as Interrupt>::clear();

    // Check if there is a descriptor to service. If there isn't check for new descriptors.
    let node = match OPERATION {
        Some(mut pointer) => pointer.as_mut(),
        None => return disable(),
    };

    let descriptor = node.object();

    // Read the control register.
    // TODO : Check that there is data ready.
    // let control = core::ptr::read_volatile(super::address::CONTROL as *const u32);

    match descriptor.operation {
        Operation::Continuous(_) => {
            crate::log::error!("ADC ISR would perform continous conversion.");
        }

        // Check if the latest conversion succeeded, write the result in the descriptor and stop the ADC engine.
        Operation::Oneshot(_) => {}
    }

    // Disable the ISR, it will be enabled by the next operation.
    disable();

    // Mark the descriptor as finished and wake it.
    node.finish();
    node.wake();
}

/// Disables the ADC engine and masks this interrupt.
#[inline(always)]
unsafe fn disable() {
    core::ptr::write_volatile(super::address::INTE as *mut u32, 0);
    <super::ADCEngine as Interrupt>::disable();
}
