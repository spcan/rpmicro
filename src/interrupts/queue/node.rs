//! Implementation of a node within a `ISRQueue`.

use core::{
    future::Future,
    pin::Pin,
    ptr::NonNull,
    task::{Context, Poll, Waker},
};

use super::ISRObject;

pub struct ISRQueueNode<T: ISRObject> {
    /// The next operation in an `ISRQueue`. Necessary for the `QueueNode` trait.
    pub(super) next: Option<NonNull<Self>>,

    /// Set of flags to track the progress of this `ADCOperation`.
    flags: Flags,

    /// The waker of the context that requested the data.
    waker: Option<Waker>,

    /// The internal object contained in this node.
    object: T,
}

impl<T: ISRObject> ISRQueueNode<T> {
    pub fn new(object: T) -> Self {
        Self {
            object,
            flags: Flags::empty(),
            next: None,
            waker: None,
        }
    }

    /// Marks this `ISRQueueNode`'s `Future` as finished.
    pub unsafe fn finish(&mut self) {
        self.flags |= Flags::FINISHED;
    }

    /// Returns a mutable reference to this `ISRQueueNode`'s contained object.
    pub fn object(&mut self) -> &mut T {
        &mut self.object
    }

    pub fn triggered(&self) -> bool {
        self.flags.contains(Flags::ACTIVE)
    }

    /// Wakes up this `ISRQueueNode`'s `Future`.
    pub fn wake(&mut self) {
        if let Some(waker) = &self.waker {
            waker.wake_by_ref();
        }
    }
}

impl<T: ISRObject> Future for ISRQueueNode<T>
where
    Self: Unpin,
{
    type Output = T::Output;

    #[rustfmt::skip]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Register the waker if not done yet, this will happen before the ISR gets called so it's safe.
        if self.waker.is_none() {
            self.waker = Some(cx.waker().clone());
        }

        // Register the descriptor into the queue if not done yet.
        if !self.flags.contains(Flags::REGISTERED) {
            // Get as a reference because the compiler complains.
            // Safe because we know this is a valid `ADCOperation`.
            let reference = unsafe { NonNull::new_unchecked(self.as_mut().get_unchecked_mut() as *mut Self) };

            // Attempt to register this `ADCOperation` in the ADC queue.
            match unsafe { self.object.queue().push(reference) } {
                // Registration succeeded, if it's the first item in the queue, set it to run immediately.
                Ok(first) => if first {
                    *self.object.slot() = Some(reference);
                    self.flags |= Flags::ACTIVE;
                },

                // Registration failed, wake again the task and yield to the executor.
                Err(_) => {
                    crate::log::error!("Failed to register node @ {:#010X}", reference);
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
            }

            self.as_mut().flags |= Flags::REGISTERED;
        }

        // Check if this is the active `ADCOperation` and skip if not.
        if !self.flags.contains(Flags::ACTIVE) {
            return Poll::Pending;
        }

        // If not finished, configure the ADC to do the selected operation.
        if !self.flags.contains(Flags::FINISHED) {
            self.as_mut().object.trigger();
            return Poll::Pending;
        }

        // The operation is finished, dequeue this operation.
        if let Err(_) = unsafe { self.object.queue().pop() } {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        // If there is a chained operation, queue it and wake it.
        unsafe {
            *self.object.slot() = self.next;

            if let Some(mut operation) = self.next {
                operation.as_mut().flags |= Flags::ACTIVE;
                core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
                operation.as_mut().waker.as_ref().unwrap().wake_by_ref();
            }
        }

        Poll::Ready(self.as_mut().object.collect())
    }
}

unsafe impl<T: ISRObject> Send for ISRQueueNode<T> {}
unsafe impl<T: ISRObject> Sync for ISRQueueNode<T> {}

bitflags::bitflags! {
    /// Flags of an `ISRQueueNode`.
    pub(super) struct Flags: u8 {
        /// Marks if this `ISRQueueNode` has been registered in the queue.
        const REGISTERED = 1 << 0;

        /// Marks if this `ISRQueueNode` is being serviced right now.
        const ACTIVE = 1 << 1;

        /// Marks if this `ISRQueueNode` has finished.
        const FINISHED = 1 << 2;
    }
}
