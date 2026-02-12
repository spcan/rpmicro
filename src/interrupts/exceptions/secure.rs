//! Specialized handler for the Secure fault exception.

pub(super) unsafe extern "C" fn handler() {
    crate::log::info!("Secure fault triggered");

    loop {
        crate::asm::nop();
    }
}
