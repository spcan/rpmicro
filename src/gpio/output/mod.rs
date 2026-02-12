//! Control over GPIO pins in Output mode.

mod any;
mod typed;

pub use any::OutputAny;
pub use typed::OutputTyped;

/// Common trait for all abstractions interacting with a GPIO pin's output.
pub trait Output {
    /// Toggles ON / OFF the output level. Does not care for the previous output state.
    fn toggle(&mut self);

    fn set_high(&mut self);

    fn set_low(&mut self);
}
