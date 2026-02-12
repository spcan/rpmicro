//! Specialized handler for the Supervisor exceptions.

pub(super) unsafe extern "C" fn call() {
    crate::log::info!("Supervisor Call triggered");
}

pub(super) unsafe extern "C" fn pend() {
    crate::log::info!("Supervisor Call pended");
}
