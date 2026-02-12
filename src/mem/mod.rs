//! Memory module.
//! Contains all functionality of the memory of the device, including external memory.

pub mod dma;
pub mod external;
pub mod internal;

pub use dma::{DMAChannel, DMAChannels};
pub use external::{ExternalMemory, ExternalMemorySlot};
