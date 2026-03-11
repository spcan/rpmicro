//! Module for GPIO as input.

mod irq;

use crate::{cpuid, gpio::GPIOAny, AtomicRegister};

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

const BASE: usize = 0x40028000;

const INTR: usize = BASE + 0x230;

const INTE: usize = BASE + 0x248;

const INTF: usize = BASE + 0x260;

const INTS: usize = BASE + 0x278;

const CPUSTRIDE: usize = 0x290 - 0x248;

/// Untyped, anonimous abstraction over a GPIO input.
pub struct Input(u8);

impl Input {
    /// Wait for a specific event. The `Input` can be reused after this method is called.
    pub async fn wait(&mut self, event: Event) {
        WaitEvent::new(self.0, event).await;
    }

    /// Consumes the `Input` to generate a permanent `WaitEvent` that cannot be modified.
    /// This generates a fater abstraction for future `await`s but does not have the flexibility
    /// to change the `Event` to wait for.
    /// The `WaitEvent` can later be consumed to regenerate the `Input`.
    pub fn signal(self, event: Event) -> WaitEvent {
        WaitEvent::new(self.0, event)
    }

    /// Consumes the `Input` to regenerate the `GPIOAny` for the same pin.
    pub fn pin(self) -> GPIOAny {
        GPIOAny(self.0)
    }
}

/// `Future` that waits for a specific event in an input GPIO pin.
pub struct WaitEvent {
    /// The pin used as input for this future.
    pin: u8,

    /// The event tracked by this `Future`.
    event: Event,

    /// Tracks if the waker for this `Future` has been registered with the IRQ handler.
    registered: bool,
}

impl WaitEvent {
    /// Creates a new `WaitEvent` for the given pin.
    pub(self) fn new(pin: u8, event: Event) -> Self {
        const MASK: u32 = 0xF;

        // Check what register and offset this GPIO is.
        let stride = CPUSTRIDE * cpuid() as usize;
        let register = (pin as usize / 8) * 4;
        let offset = pin % 8;

        let inte = AtomicRegister::at(INTE + register + stride);

        // Clear previous triggers and disable the interrupt.
        inte.clear(MASK << offset);

        Self {
            pin,
            event,
            registered: false,
        }
    }

    /// Consumes the `WaitEvent` to regenerate the `Input` that created it.
    pub fn input(self) -> Input {
        Input(self.pin)
    }
}

impl Future for WaitEvent {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        const MASK: u32 = 0xF;

        // Check what register and offset this GPIO is.
        let stride = CPUSTRIDE * cpuid() as usize;
        let register = (self.pin as usize / 8) * 4;
        let offset = self.pin % 8;

        let intr = AtomicRegister::at(INTR + register);
        let inte = AtomicRegister::at(INTE + register + stride);

        // Take the struct out of pin to use in the poll.
        let this = self.get_mut();

        // Register this future if not done already.
        // This ensures there is a waker registered before enabling the interrupt.
        if !this.registered {
            unsafe { irq::WAKERS[this.pin as usize] = Some(cx.waker().clone()) };
            this.registered = true;

            // Ensure the waker is written before the interrupt is enabled.
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

            // Remove previous triggers and enable only the requested event.
            intr.clear(MASK << offset);
            inte.set((this.event as u32) << offset);
        }

        // If the interrupt is disabled, the IRQ handler triggered us.
        if ((inte.read() >> offset) & 0xF) == 0 {
            this.registered = false;

            return Poll::Ready(());
        }

        Poll::Pending
    }
}

/// List of events that can be captured from a GPIO input.
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
#[repr(u8)]
pub enum Event {
    /// Wait for a low level (logical 0).
    LevelLow = 0b0001,

    /// Wait for a high level (logical 1).
    LevelHigh = 0b0010,

    /// Wait for a transition from high to low.
    EdgeLow = 0b0100,

    /// Wait for a transition from low to high.
    EdgeHigh = 0b1000,

    /// Waits for any transition (low to high or high to low).
    EdgeAny = 0b1100,
}
