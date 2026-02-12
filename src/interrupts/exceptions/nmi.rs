//! Specialized handler for the Non-Maskable Interrupt exception.

pub(super) unsafe extern "C" fn handler() {
    crate::log::info!("NMI triggered");

    loop {
        crate::asm::nop();
    }
}
