//! DMA module.

mod address;
mod isr;

/// Const-style DMA channel.
pub struct ConstChannel<const N: usize>;

/// Dyn-style DMA channel.
pub struct AnyChannel(pub(crate) u8);

/// Public trait for all DMA channel types.
pub trait DMAChannel {}

/// Crate-only trait for all DMA channel types.
pub(crate) trait DMAChannelInner {
    /// Sets the read address of the channel.
    fn read(&mut self, address: u32);

    /// Sets the write address of the channel.
    fn write(&mut self, address: u32);

    /// Sets the transfer count of the channel.
    fn count(&mut self, count: TransferCount);
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TransferCount(pub(crate) u32);

impl TransferCount {
    /// Creates a `TransferCount` that will trigger once.
    pub const fn oneshot(count: u32) -> Self {
        Self(count & 0x0FFFFFFF)
    }

    /// Creates a `TransferCount` that will repeat itself forever.
    /// Once the count reaches 0, the corresponding interrupt and triggers will launch and the channel restarts.
    pub const fn repeat(count: u32) -> Self {
        Self((0x1 << 28) | (count & 0x0FFFFFFF))
    }

    /// Creates a `TransferCount` that does not decrement.
    /// The DMA channel will go on forever without triggering any interrupts until ABORT is raised.
    pub const fn endless() -> Self {
        Self((0xF << 28) | 1)
    }
}
