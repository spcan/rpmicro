//! Customized Hardware Abstraction Layer (HAL) for the RP2350x.

#![no_std]
#![feature(generic_const_exprs)]
#![feature(likely_unlikely)]

// Public modules.
// pub mod adc;
pub mod clocks;
pub mod cortex;
// pub mod crypto;
pub mod gpio;
pub mod interrupts;
pub mod mem;
pub mod system;

// Internal only modules.
pub(crate) mod asm;
pub(crate) mod log;

mod peripherals;
mod register;

// Publicly exported items.
pub use peripherals::{Common, Local};

// Internally exported items.
pub(crate) use peripherals::Peripheral;
pub(crate) use register::AtomicRegister;

use core::sync::atomic::{AtomicBool, Ordering};

/// Global flag indicating if the HAL's common peripherals have already been initialized.
static COMMON: AtomicBool = AtomicBool::new(false);

/// Global flag indicating if the HAL's core local peripherals have already been initialized.
static LOCAL: [AtomicBool; 2] = [AtomicBool::new(false), AtomicBool::new(false)];

/// Reads the current CPU's ID.
#[inline(always)]
pub fn cpuid() -> u32 {
    unsafe { core::ptr::read_volatile(0xD0000000 as *const u32) }
}

/// Initializes the HAL's common peripherals and returns the `Common` peripherals control.
pub fn common() -> Common {
    // First check if the HAL is already initialized.
    if COMMON.swap(true, Ordering::SeqCst) {
        crate::log::error!("[Core {}] Common HAL is already initialized", cpuid());
        panic!("Common HAL is already initialized");
    }

    unsafe { Common::init() }
}

/// Initialize the HAL's core local peripherals and returns the `Local` peripherals control.
pub fn local() -> Local {
    // First check if the HAL is already initialized.
    if LOCAL[cpuid() as usize].swap(true, Ordering::SeqCst) {
        crate::log::error!("[Core {}] Local HAL is already initialized", cpuid());
        panic!("Local HAL is already initialized");
    }

    unsafe { Local::init() }
}
