//! Typed GPIO pin abstraction.

use crate::{gpio::GPIOControl, AtomicRegister};

pub struct GPIOTyped<const N: usize>;

impl<const N: usize> GPIOControl for GPIOTyped<N> {
    fn control(&mut self) -> &mut AtomicRegister {
        AtomicRegister::at(super::address::IOUSER + (0x08 * N) + 0x04)
    }

    fn status(&mut self) -> &mut AtomicRegister {
        AtomicRegister::at(super::address::IOUSER + (0x08 * N))
    }

    fn pad(&mut self) -> &mut AtomicRegister {
        AtomicRegister::at(super::address::PADUSER + (0x04 * N) + 0x04)
    }
}

impl<const N: usize> crate::Peripheral for GPIOTyped<N> {
    unsafe fn instance() -> Self {
        Self
    }
}
