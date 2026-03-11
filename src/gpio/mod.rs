//! General Purpose Input / Output (GPIO) module.

#![allow(unused)]

pub mod analog;
pub mod input;
pub mod output;
pub mod qspi;

mod pins;

use crate::AtomicRegister;

pub use analog::Analog;
pub use output::Output;
pub use pins::{GPIOAny, GPIOList, GPIOTyped};

/// The amount of GPIO pins in the device.
#[cfg(feature = "QFN60")]
pub(crate) const GPIOCOUNT: usize = 32;

/// The amount of GPIO pins in the device.
#[cfg(feature = "QFN80")]
pub(crate) const GPIOCOUNT: usize = 48;

/// The amount of GPIO pins in the device.
#[cfg(not(any(feature = "QFN60", feature = "QFN80")))]
pub(crate) const GPIOCOUNT: usize = 0;

/// Common trait for all abstractions interacting with the GPIO pins.
pub(crate) trait GPIOControl {
    /// Returns the `AtomicRegister` containing the status of the GPIO.
    fn status(&mut self) -> &mut AtomicRegister;

    /// Returns the `AtomicRegister` containing the control of the GPIO.
    fn control(&mut self) -> &mut AtomicRegister;

    /// Returns the `AtomicRegister` containing the pad configuration of the GPIO.
    fn pad(&mut self) -> &mut AtomicRegister;
}
