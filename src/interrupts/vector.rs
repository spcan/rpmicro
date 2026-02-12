//! Vector union.
//! Represents an entry in the Vector Table.

/// Assert the size of the vectors is 4 bytes.
const _: () = assert!(
    core::mem::size_of::<Vector>() == 4,
    "Vector size is not 4 bytes"
);

#[derive(Clone, Copy)]
#[repr(C)]
pub union Vector {
    pub(super) handler: unsafe extern "C" fn(),
    pub(super) value: usize,
}

impl Vector {
    /// Creates a reserved entry in a Vector Table. Defaults to a value of 0.
    pub const fn reserved() -> Self {
        Self { value: 0 }
    }

    /// Creates an entry with a predefined handler.
    pub const fn target(handler: unsafe extern "C" fn()) -> Self {
        Self { handler }
    }

    /// Creates an entry with a predefined value.
    pub const fn value(value: usize) -> Self {
        Self { value }
    }
}

#[cfg(feature = "log")]
impl core::fmt::Debug for Vector {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Vector({:#X})", unsafe { self.value })
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Vector {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "0x{:x}", unsafe { self.value })
    }
}
