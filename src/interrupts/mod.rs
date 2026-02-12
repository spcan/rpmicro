//! General module for interrupts, exceptions and service routine handlers.

pub mod exceptions;
pub mod interrupts;
pub mod queue;
pub mod vector;
pub mod vtable;

pub(self) use exceptions::Exceptions;
pub(self) use interrupts::Interrupts;
pub(self) use vector::Vector;

pub use interrupts::Interrupt;
pub use vtable::VectorTable;

pub(crate) fn init() {
    // Initialize the exceptions' handlers.
    exceptions::init();

    // Initialize the ISR handlers for the common peripherals.
    unsafe {
        // <crate::hal::adc::ADCEngine as Interrupt>::install(crate::hal::adc::isr::handler);
    }
}
