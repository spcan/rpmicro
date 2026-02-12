//! Specialized handler for the Usage fault exception.

pub(super) unsafe extern "C" fn handler() {
    // Configurable Fault Status Register.
    const CFSR: usize = 0xE000ED28;

    // Get the status of the usage fault.
    let status = unsafe { (core::ptr::read_volatile(CFSR as *const u32) >> 16) as u16 };

    #[cfg(feature = "defmt")]
    defmt::error!(
        "Usage Fault\n  UNDEFINSTR: [{0=0..1}]\n  INVSTATE  : [{0=1..2}]\n  INVPC     : [{0=2..3}]\n  NOCP      : [{0=3..4}]\n  STKOF     : [{0=4..5}]\n  UNALIGNED : [{0=8..9}]\n  DIVBYZERO : [{0=9..10}]",
        status,
    );

    #[cfg(not(feature = "defmt"))]
    log::error!(
        "Usage Fault\n  UNDEFINSTR: [{0=0..1}]\n  INVSTATE  : [{0=1..2}]\n  INVPC     : [{0=2..3}]\n  NOCP      : [{0=3..4}]\n  STKOF     : [{0=4..5}]\n  UNALIGNED : [{0=8..9}]\n  DIVBYZERO : [{0=9..10}]",
        (status >> 0) & 1,
        (status >> 1) & 1,
        (status >> 2) & 1,
        (status >> 3) & 1,
        (status >> 4) & 1,
        (status >> 8) & 1,
        (status >> 9) & 1,
    );

    loop {
        core::arch::asm!("nop", options(nomem, nostack, preserves_flags));
    }
}
