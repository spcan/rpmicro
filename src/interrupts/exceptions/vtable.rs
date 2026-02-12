//! Vector table for the Cortex-M33 exceptions.

use super::super::Vector;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
#[derive(Clone)]
#[repr(C)]
pub struct Exceptions {
    /// Stack pointer after reset.
    sp: Vector,

    /// Reset vector. First function to be execute after reset.
    reset: Vector,

    /// Non-maskable interrupt.
    nmi: Vector,

    /// Generic hardfault.
    hardfault: Vector,

    /// Memory management fault.
    memmanage: Vector,

    /// Bus access fault.
    busfault: Vector,

    /// Usage fault.
    usagefault: Vector,

    #[doc(hidden)]
    reserved7: Vector,

    #[doc(hidden)]
    reserved8: Vector,

    #[doc(hidden)]
    reserved9: Vector,

    #[doc(hidden)]
    reserved10: Vector,

    /// Synchronous exception to call the supervisor routine.
    svcall: Vector,

    /// Debug monitor call.
    debug: Vector,

    #[doc(hidden)]
    reserve13: Vector,

    /// Pending a SV call.
    pendsv: Vector,

    /// System Tick exception.
    systick: Vector,
}
