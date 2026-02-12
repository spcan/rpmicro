//! Singleton control over the common and core local peripherals.

mod common;
mod local;

pub use common::Common;
pub use local::Local;

/// Common trait for all `Peripheral`s.
pub(crate) trait Peripheral: Sized {
    unsafe fn instance() -> Self;
}
