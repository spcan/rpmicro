//! Typed abstraction over a GPIO output.

use crate::{
    gpio::{GPIOControl, GPIOTyped},
    Peripheral,
};

pub struct OutputTyped<const N: usize>;

impl<const N: usize> super::Output for OutputTyped<N>
where
    [(); N / 32]: Sized,
{
    fn toggle(&mut self) {
        unsafe { mcr::<{ N / 32 }, { functions::XORBIT }>(N) }
    }

    fn set_high(&mut self) {
        unsafe { mcr::<{ N / 32 }, { functions::SETBIT }>(N) }
    }

    fn set_low(&mut self) {
        unsafe { mcr::<{ N / 32 }, { functions::CLRBIT }>(N) }
    }
}

impl<const N: usize> Into<OutputTyped<N>> for GPIOTyped<N> {
    fn into(mut self) -> OutputTyped<N> {
        // GPIO control configuration that sets the SIO as the controller.
        const CONTROL: u32 = 0x05;

        // GPIO pad configuration that sets the SIO as output.
        const PAD: u32 = 0x33;

        // Configure the GPIO pin.
        self.control().write(CONTROL);
        self.pad().write(PAD);

        OutputTyped
    }
}

impl<const N: usize> Into<GPIOTyped<N>> for OutputTyped<N> {
    fn into(self) -> GPIOTyped<N> {
        // GPIO control configuration that sets no GPIO controller.
        const CONTROL: u32 = 0x1F;

        // GPIO pad configuration that disables the GPIO.
        const PAD: u32 = 0x80;

        // Create the GPIO instance and reset the state.
        let mut gpio = unsafe { GPIOTyped::instance() };

        gpio.control().write(CONTROL);
        gpio.pad().write(PAD);

        gpio
    }
}

/// Assembly function to execute MCR instructions.
#[inline(always)]
unsafe fn mcr<const BANK: usize, const FUNCTION: u8>(bits: usize) {
    core::arch::asm!("MCR p0, #{function}, {0}, c0, c{bank}", in(reg) bits, function = const FUNCTION, bank = const BANK, options(nostack, nomem, preserves_flags))
}

/// List of GPIO coprocessor functions.
#[allow(unused)]
mod functions {
    /// Calls the write GPIO function.
    pub(super) const PUT: u8 = 0;

    /// Calls the toggle GPIO function.
    pub(super) const XOR: u8 = 1;

    /// Calls the set GPIO function.
    pub(super) const SET: u8 = 2;

    /// Calls the clear GPIO function.
    pub(super) const CLR: u8 = 3;

    /// Calls the write GPIO function for a single IO.
    pub(super) const PUTBIT: u8 = 4;

    /// Calls the toggle GPIO function for a single IO.
    pub(super) const XORBIT: u8 = 5;

    /// Calls the set GPIO function for a single IO.
    pub(super) const SETBIT: u8 = 6;

    /// Calls the clear GPIO function for a single IO.
    pub(super) const CLRBIT: u8 = 7;
}
