//! Specialized handler for the Systick exception.

pub(super) unsafe extern "C" fn handler() {
    crate::log::info!("Systick triggered");
}
