//! Exceptions table and handling.

mod busfault;
mod hardfault;
mod memfault;
mod nmi;
mod secure;
mod supervisor;
mod systick;
mod usefault;
mod vtable;

pub(super) use vtable::Exceptions;

use super::{Vector, VectorTable};

/// Initialize the exception table for the current core.
pub(super) fn init() {
    let vtable = unsafe { VectorTable::current() };

    vtable[2] = Vector::target(nmi::handler);
    vtable[3] = Vector::target(hardfault::handler);
    vtable[4] = Vector::target(memfault::handler);
    vtable[5] = Vector::target(busfault::handler);
    vtable[6] = Vector::target(usefault::handler);
    vtable[7] = Vector::target(secure::handler);
    vtable[11] = Vector::target(supervisor::call);
    vtable[14] = Vector::target(supervisor::pend);
    vtable[15] = Vector::target(systick::handler);
}
