//! Contained list of GPIOs.

use crate::{
    gpio::{GPIOAny, GPIOTyped},
    Peripheral,
};

#[cfg(not(any(feature = "QFN60", feature = "QFN80")))]
const SIZE: usize = 0;

#[cfg(feature = "QFN60")]
const SIZE: usize = 1;

#[cfg(feature = "QFN80")]
const SIZE: usize = 2;

#[cfg(not(any(feature = "QFN60", feature = "QFN80")))]
const ALL: [u32; SIZE] = [];

#[cfg(feature = "QFN60")]
const ALL: [u32; SIZE] = [0x3FFFFFFF];

#[cfg(feature = "QFN80")]
const ALL: [u32; SIZE] = [0xFFFFFFFF, 0x0000FFFF];

/// Ergonomic way to pass around a list of GPIOs.
pub struct GPIOList([u32; SIZE]);

impl GPIOList {
    /// Creates a list containing all pins in the system.
    pub(crate) unsafe fn all() -> Self {
        Self(ALL)
    }

    /// Creates an empty `GPIOList`.
    pub const fn empty() -> Self {
        Self([0; SIZE])
    }

    /// Attemps to take the given `GPIOAny` from the list.
    pub fn any(&mut self, n: u8) -> Option<GPIOAny> {
        if (self.0[n as usize / 32] & (1 << (n as usize % 32))) != 0 {
            self.0[n as usize / 32] &= !(1 << (n as usize % 32));

            return Some(GPIOAny(n));
        }

        None
    }

    /// Attemps to take the given `GPIOTyped` from the list.
    pub fn typed<const N: usize>(&mut self) -> Option<GPIOTyped<N>> {
        if (self.0[N / 32] & (1 << (N % 32))) != 0 {
            self.0[N / 32] &= !(1 << (N % 32));

            return Some(unsafe { GPIOTyped::instance() });
        }

        None
    }
}

impl<const N: usize> core::ops::Add<GPIOTyped<N>> for GPIOList {
    type Output = Self;

    fn add(mut self, _: GPIOTyped<N>) -> Self::Output {
        self.0[N / 32] |= 1 << (N % 32);
        self
    }
}

impl<const N: usize> core::ops::AddAssign<GPIOTyped<N>> for GPIOList {
    fn add_assign(&mut self, _: GPIOTyped<N>) {
        self.0[N / 32] |= 1 << (N % 32);
    }
}

impl<const N: usize, T: IntoIterator<Item = GPIOTyped<N>>> From<T> for GPIOList {
    fn from(value: T) -> GPIOList {
        value
            .into_iter()
            .fold(GPIOList::empty(), |list, gpio| list + gpio)
    }
}
