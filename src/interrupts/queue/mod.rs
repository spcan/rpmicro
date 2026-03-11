//! ISR safe queues used to enqueue work in peripheral ISRs.

mod node;
mod object;

pub use node::ISRQueueNode;
pub use object::ISRObject;

use core::{
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
};

/// Item agnostic multithread friendly queue.
pub struct ISRQueue<T: ISRObject> {
    /// Queue lock for multicore access.
    ready: AtomicBool,

    /// The current head of the queue.
    head: Option<NonNull<ISRQueueNode<T>>>,

    /// The current tail of the queue.
    tail: Option<NonNull<ISRQueueNode<T>>>,
}

impl<T: ISRObject> ISRQueue<T> {
    /// Static initializer of an `ISRQueue`.
    pub const fn new() -> Self {
        Self {
            ready: AtomicBool::new(true),
            head: None,
            tail: None,
        }
    }

    /// Pushes a new item into the queue.
    /// If it succeeds to lock the queue it will return `true` if the item inserted was the first in the queue.
    /// This method is safe to access from multiple threads at the sasme time as it has internal synchronization.
    pub unsafe fn push(&mut self, value: NonNull<ISRQueueNode<T>>) -> Result<bool, ()> {
        // Attempt to lock the queue. Leave the failure handling to the caller.
        if core::hint::unlikely(!self.lock()) {
            crate::log::warn!("[Core {}] (PUSH) Queue lock contention", crate::cpuid());
            return Err(());
        }

        // Check if this is the first item in the queue.
        let first = match self.head {
            // There is already an item at the head, so set this item as the tail.
            Some(_) => {
                let old = self.tail.replace(value);

                if let Some(mut tail) = old {
                    tail.as_mut().next = Some(value);
                }

                false
            }

            // There is no head, set both the head and tail of the queue.
            None => {
                self.head.replace(value);
                self.tail.replace(value);

                true
            }
        };

        // Unlock the queue and return the 'first in queue' conditional.
        self.unlock();
        return Ok(first);
    }

    /// Pops the head of the queue, returning the popped item.
    pub unsafe fn pop(&mut self) -> Result<Option<NonNull<ISRQueueNode<T>>>, ()> {
        // Attempt to lock the queue. Leave the failure handling to the caller.
        if core::hint::unlikely(!self.lock()) {
            crate::log::warn!("[Core {}] (POP ) Queue lock contention", crate::cpuid());
            return Err(());
        }

        // Take the head and set the next item as the head.
        let output = self.head.take();

        if let Some(output) = output {
            self.head = output.as_ref().next;

            // If the queue is now empty, clear the tail as well.
            if self.head.is_none() {
                self.tail = None;
            }
        }

        // Unlock the queue and return the next item.
        self.unlock();
        return Ok(output);
    }

    /// Internal function to lock the queue.
    fn lock(&self) -> bool {
        self.ready.swap(false, Ordering::Acquire)
    }

    /// Internal function to unlock the queue.
    fn unlock(&self) {
        self.ready.store(true, Ordering::Release)
    }
}

unsafe impl<T: ISRObject> Send for ISRQueue<T> {}
unsafe impl<T: ISRObject> Sync for ISRQueue<T> {}
