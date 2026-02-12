//! Untyped, dynamic GPIO pin abstraction.

use crate::{
    gpio::{GPIOControl, GPIOTyped},
    AtomicRegister,
};

pub struct GPIOAny(pub(crate) u8);

impl GPIOControl for GPIOAny {
    fn control(&mut self) -> &mut AtomicRegister {
        AtomicRegister::at(super::address::IOUSER + (0x08 * self.0 as usize) + 0x04)
    }

    fn status(&mut self) -> &mut AtomicRegister {
        AtomicRegister::at(super::address::IOUSER + (0x08 * self.0 as usize))
    }

    fn pad(&mut self) -> &mut AtomicRegister {
        AtomicRegister::at(super::address::PADUSER + (0x04 * self.0 as usize) + 0x04)
    }
}

impl<const N: usize> Into<GPIOAny> for GPIOTyped<N> {
    fn into(self) -> GPIOAny {
        GPIOAny(N as u8)
    }
}
