//! Specialized handler for the Bus fault exception.

pub(super) unsafe extern "C" fn handler() {
    // Configurable Fault Status Register.
    const CFSR: usize = 0xE000ED28;

    // MemManage Fault Address Register.
    const BFAR: usize = 0xE000ED38;

    // Get the address and status of the bus fault.
    let (address, status) = unsafe {
        (
            core::ptr::read_volatile(BFAR as *const u32),
            (core::ptr::read_volatile(CFSR as *const u32) >> 8) as u8,
        )
    };

    #[cfg(feature = "defmt")]
    defmt::error!(
        "Bus Fault @ {=u32:#010X} (Valid = [{1=7..8}])\n  IBUSERR  : [{1=0..1}]\n  PRECISERR: [{1=1..2}]\n  UNSTKERR : [{1=3..4}]\n  STKERR   : [{1=4..5}]\n  LSPER    : [{1=5..6}]",
        address,
        status,
    );

    #[cfg(not(feature = "defmt"))]
    log::error!(
        "Bus Fault @ {:#010X} (Valid = [{1=7..8}])\n  IBUSERR  : [{1=0..1}]\n  PRECISERR: [{1=1..2}]\n  UNSTKERR : [{1=3..4}]\n  STKERR   : [{1=4..5}]\n  LSPER    : [{1=5..6}]",
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
