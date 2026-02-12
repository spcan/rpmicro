//! Untyped, anonimous abstraction over a GPIO output.

use crate::{
    gpio::{GPIOAny, GPIOControl, GPIOTyped},
    Peripheral,
};

pub struct OutputAny(u8);

impl super::Output for OutputAny {
    fn set_high(&mut self) {
        // Addresses for the atomic SET register.
        const ADDRESS: [usize; 2] = [0xD0000018, 0xD000001C];

        unsafe {
            core::ptr::write_volatile(
                ADDRESS[(self.0 / 32) as usize] as *mut u32,
                1u32 << (self.0 % 32),
            )
        }
    }

    fn set_low(&mut self) {
        // Addresses for the atomic CLR register.
        const ADDRESS: [usize; 2] = [0xD0000020, 0xD0000024];

        unsafe {
            core::ptr::write_volatile(
                ADDRESS[(self.0 / 32) as usize] as *mut u32,
                1u32 << (self.0 % 32),
            )
        }
    }

    fn toggle(&mut self) {
        // Addresses for the atomic XOR register.
        const ADDRESS: [usize; 2] = [0xD0000028, 0xD000002C];

        unsafe {
            core::ptr::write_volatile(
                ADDRESS[(self.0 / 32) as usize] as *mut u32,
                1u32 << (self.0 % 32),
            )
        }
    }
}

impl Into<OutputAny> for GPIOAny {
    fn into(mut self) -> OutputAny {
        // GPIO control configuration that sets the SIO as the controller.
        const CONTROL: u32 = (0x3 << 14) | 0x05;

        // GPIO pad configuration that sets the SIO as output.
        const PAD: u32 = 0x33;

        // Configure the GPIO pin.
        self.control().write(CONTROL);
        self.pad().write(PAD);

        OutputAny(self.0)
    }
}

impl Into<GPIOAny> for OutputAny {
    fn into(self) -> GPIOAny {
        // GPIO control configuration that sets no GPIO controller.
        const CONTROL: u32 = 0x1F;

        // GPIO pad configuration that disables the GPIO.
        const PAD: u32 = 0x80;

        // Create the GPIO instance and reset the state.
        let mut gpio = GPIOAny(self.0);

        gpio.control().write(CONTROL);
        gpio.pad().write(PAD);

        gpio
    }
}

impl<const N: usize> TryInto<GPIOTyped<N>> for OutputAny {
    type Error = ();

    fn try_into(self) -> Result<GPIOTyped<N>, Self::Error> {
        // GPIO control configuration that sets no GPIO controller.
        const CONTROL: u32 = 0x1F;

        // GPIO pad configuration that disables the GPIO.
        const PAD: u32 = 0x80;

        // If the GPIO index does not match, fail the conversion early.
        if N != self.0 as usize {
            return Err(());
        }

        // Create the GPIO instance and reset the state.
        let mut gpio = unsafe { GPIOTyped::instance() };

        gpio.control().write(CONTROL);
        gpio.pad().write(PAD);

        Ok(gpio)
    }
}
