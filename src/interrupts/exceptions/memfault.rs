//! Handler for the MemFault exception.

/// Specialized handler for the MemManage fault exception.
pub(super) unsafe extern "C" fn handler() {
    // Configurable Fault Status Register.
    const CFSR: usize = 0xE000ED28;

    // MemManage Fault Address Register.
    const MMFAR: usize = 0xE000ED34;

    // Get the address and status of the memory fault.
    let (address, status) = unsafe {
        (
            core::ptr::read_volatile(MMFAR as *const u32),
            core::ptr::read_volatile(CFSR as *const u32) as u8,
        )
    };

    #[cfg(feature = "defmt")]
    defmt::error!(
        "MemManage Fault @ {=u32:#010X} (Valid = [{1=7..8}])\n  IACCVIOL : [{1=0..1}]\n  DACCVIOL : [{1=1..2}]\n  MUNSTKERR: [{1=3..4}]\n  MSTKERR  : [{1=4..5}]\n  MLSPER   : [{1=5..6}]",
        address,
        status,
    );

    #[cfg(feature = "log")]
    log::error!(
        "MemManage Fault @ {:#010X} (Valid = [{1=7..8}])\n  IACCVIOL : [{1=0..1}]\n  DACCVIOL : [{1=1..2}]\n  MUNSTKERR: [{1=3..4}]\n  MSTKERR  : [{1=4..5}]\n  MLSPER   : [{1=5..6}]",
        address,
        ((status >> 7) & 1) == 1,
        (status >> 0) & 1,
        (status >> 1) & 1,
        (status >> 3) & 1,
        (status >> 4) & 1,
        (status >> 5) & 1,
    );

    loop {
        core::arch::asm!("nop", options(nomem, nostack, preserves_flags));
    }
}
