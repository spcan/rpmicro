//! Common trait for objects that can be sent to an `ISRQueue`.

use core::ptr::NonNull;

use super::{ISRQueue, ISRQueueNode};

pub trait ISRObject: Sized + 'static {
    /// Output of the ISR task / job / interaction.
    type Output: Sized;

    /// Returns a reference to the global slot for the object being processed by the ISR.
    fn slot(&self) -> &'static mut Option<NonNull<ISRQueueNode<Self>>>;

    /// Returns a reference to this `ISRObject`'s queue.
    fn queue(&self) -> &'static mut ISRQueue<Self>;

    /// Triggers the object to interact with the ISR.
    fn trigger(&mut self);

    /// Collects the output of the ISR task / job / interaction.
    fn collect(&mut self) -> Self::Output;
}
