//! True Random Number Generator.

mod address;
mod isr;
mod operation;

use core::ptr::{read_volatile, write_volatile};

use crate::hal::interrupts::{queue::ISRQueue, Interrupt};

pub(self) use operation::TRNGOperation;

/// Global queue for TRNG operations.
#[link_section = ".data"]
pub(self) static mut QUEUE: ISRQueue<operation::Descriptor> = ISRQueue::new();

/// Control over the TRN peripheral.
/// Enables multicore control over the TRN source after initialization.
pub struct TRNGPeripheral;

impl TRNGPeripheral {
    /// Unsafe method to create an instance of this peripheral out of thin air.
    /// Should only be used during initialziation.
    pub(crate) unsafe fn instance() -> Self {
        Self
    }

    /// Initializes the peripheral for all modes of operation.
    pub fn init(self) -> TRNGenerator {
        unsafe {
            // Disable interrupts, stop the engine and reset it.
            write_volatile(address::IMR as *mut u32, 0xF);
            write_volatile(address::ENABLE as *mut u32, 0);
            write_volatile(address::RESET as *mut u32, 1);

            // Configure the engine.
            write_volatile(address::CONFIG as *mut u32, 4);
            write_volatile(address::SAMPLES as *mut u32, 50);

            // Install the ISR in the vector table and enable it in the NVIC.
            <TRNGenerator as Interrupt>::install(isr::handler);
            <TRNGenerator as Interrupt>::enable();

            // Reset all interrupts and enable them.
            write_volatile(address::ICR as *mut u32, 0xF);
            write_volatile(address::IMR as *mut u32, 0);
        }

        TRNGenerator
    }
}

/// Multicore safe source of random numbers.
#[derive(Clone, Copy)]
pub struct TRNGenerator;

impl TRNGenerator {
    /// Creates a descriptor to fill the buffer with random data.
    pub fn fill<'a>(&mut self, data: &'a mut [u8]) -> TRNGOperation<'a> {
        TRNGOperation::new(data)
    }

    /// Internal method to reset the TRNG peripheral.
    pub(self) unsafe fn reset() {
        // Stop the RNG.
        write_volatile(address::ENABLE as *mut u32, 0);
        write_volatile(address::BITCOUNT as *mut u32, 0);

        // Copy the current configuration.
        let samples = read_volatile(address::SAMPLES as *const u32);
        let chain = read_volatile(address::CONFIG as *const u32);
        let debug = read_volatile(address::DEBUG as *const u32);

        // Reset the RNG block and wait for one cycle (reset is not immediate).
        write_volatile(address::RESET as *mut u32, 1);
        cortex_m::asm::nop();
        cortex_m::asm::nop();

        // Reconfigure the TRNG peripheral.
        write_volatile(address::ENABLEDBG as *mut u32, 1);
        write_volatile(address::SAMPLES as *mut u32, samples);
        write_volatile(address::CONFIG as *mut u32, chain);
        write_volatile(address::DEBUG as *mut u32, debug);

        // Reset all interrupts and enable them.
        write_volatile(address::ICR as *mut u32, 0xF);
        write_volatile(address::IMR as *mut u32, 0);

        // Enable the TRNG.
        write_volatile(address::ENABLE as *mut u32, 1);
    }
}

impl Interrupt for TRNGenerator {
    const NUMBER: usize = 39;
}
