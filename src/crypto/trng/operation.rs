//! Descriptors and queues for TRNG control and access.

use crate::hal::interrupts::{
    queue::{ISRObject, ISRQueue, ISRQueueNode},
    Interrupt,
};

use core::{
    future::Future,
    pin::Pin,
    ptr::write_volatile,
    task::{Context, Poll},
};

pub struct TRNGOperation<'a> {
    /// Array to fill with random data.
    #[allow(unused)]
    pub(super) buffer: &'a mut [u8],

    /// The `ISRQueueNode` for this operation.
    node: ISRQueueNode<Descriptor>,
}

impl<'a> TRNGOperation<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        let descriptor = Descriptor::new(buffer);

        Self {
            buffer,
            node: ISRQueueNode::new(descriptor),
        }
    }
}

impl<'a> Future for TRNGOperation<'a> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        core::pin::pin!(&mut self.node).poll(cx)
    }
}

pub struct Descriptor {
    /// Current amount of bytes added to the buffer.
    pub(super) current: usize,

    /// Address of the buffer.s
    pub(super) address: usize,

    /// Size of the buffer.
    pub(super) size: usize,
}

impl Descriptor {
    pub(self) fn new(buffer: &mut [u8]) -> Self {
        Self {
            current: 0,
            address: core::ptr::without_provenance::<u8>(buffer.as_ptr() as usize) as usize,
            size: buffer.len(),
        }
    }

    pub(super) fn buffer<'a>(&self) -> &'a mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.address as *mut u8, self.size) }
    }
}

impl ISRObject for Descriptor {
    type Output = ();

    #[allow(static_mut_refs)]
    fn queue(&self) -> &'static mut ISRQueue<Self> {
        unsafe { &mut super::QUEUE }
    }

    #[allow(static_mut_refs)]
    fn slot(&self) -> &'static mut Option<core::ptr::NonNull<ISRQueueNode<Self>>> {
        unsafe { &mut super::isr::OPERATION }
    }

    fn trigger(&mut self) {
        unsafe {
            // Enable the TRNG.
            super::TRNGenerator::reset();

            // Enable the interrupts.
            <super::TRNGenerator as Interrupt>::enable();
            // <super::TRNGenerator as Interrupt>::pend();
        }
    }

    fn collect(&mut self) -> Self::Output {
        ()
    }
}
