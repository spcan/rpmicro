//! Configures the log crate of the system.

#![allow(unused)]

#[cfg(all(feature = "defmt", not(feature = "log")))]
pub(crate) use defmt::{debug, error, info, trace, warn};

#[cfg(all(feature = "log", not(feature = "defmt")))]
pub(crate) use log::{debug, error, info, trace, warn};

#[cfg(all(feature = "defmt", feature = "log"))]
compile_error!("Features \"defmt\" and \"log\" cannot be enabled simultaneously");

#[cfg(not(any(feature = "defmt", feature = "log")))]
mod dummy {
    #[macro_export]
    macro_rules! trace {
        ($($dummy:tt)*) => {};
    }

    #[macro_export]
    macro_rules! debug {
        ($($dummy:tt)*) => {};
    }

    #[macro_export]
    macro_rules! info {
        ($($dummy:tt)*) => {};
    }

    #[macro_export]
    macro_rules! warn {
        ($($dummy:tt)*) => {};
    }

    #[macro_export]
    macro_rules! error {
        ($($dummy:tt)*) => {};
    }
}
