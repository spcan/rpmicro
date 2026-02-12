//! Specialized handler for the Hard fault exception.

use crate::cpuid;

pub(super) unsafe extern "C" fn handler() {
    // Usage Fault Status Register.
    const HFSR: usize = 0xE000ED2C;

    // Get the status of the hard fault.
    let status = unsafe { core::ptr::read_volatile(HFSR as *const u32) };

    crate::log::error!(
        "[CPU{}] Hard Fault\n  VECTTBL : [{1=0..1}]\n  FORCED  : [{1=30..31}]\n  DEBUGEVT: [{1=31..32}]",
        cpuid(),
        status,
    );

    // Configurable Fault Status Register.
    const CFSR: usize = 0xE000ED28;

    // Get the status of the usage fault.
    let status = unsafe { (core::ptr::read_volatile(CFSR as *const u32) >> 16) as u16 };

    crate::log::error!(
        "[CPU{}] Usage Fault\n  UNDEFINSTR: [{1=0..1}]\n  INVSTATE  : [{1=1..2}]\n  INVPC     : [{1=2..3}]\n  NOCP      : [{1=3..4}]\n  STKOF     : [{1=4..5}]\n  UNALIGNED : [{1=8..9}]\n  DIVBYZERO : [{1=9..10}]",
        cpuid(),
        status,
    );

    // MemManage Fault Address Register.
    const BFAR: usize = 0xE000ED38;

    // Get the address and status of the bus fault.
    let (address, status) = unsafe {
        (
            core::ptr::read_volatile(BFAR as *const u32),
            (core::ptr::read_volatile(CFSR as *const u32) >> 8) as u8,
        )
    };

    crate::log::error!(
        "[CPU{}] Bus Fault @ {=u32:#010X} (Valid = [{2=7..8}])\n  IBUSERR  : [{2=0..1}]\n  PRECISERR: [{2=1..2}]\n  UNSTKERR : [{2=3..4}]\n  STKERR   : [{2=4..5}]\n  LSPER    : [{2=5..6}]",
        cpuid(),
        address,
        status,
    );

    // MemManage Fault Address Register.
    const MMFAR: usize = 0xE000ED34;

    // Get the address and status of the memory fault.
    let (address, status) = unsafe {
        (
            core::ptr::read_volatile(MMFAR as *const u32),
            core::ptr::read_volatile(CFSR as *const u32) as u8,
        )
    };

    crate::log::error!(
        "[CPU{}] MemManage Fault @ {=u32:#010X} (Valid = [{2=7..8}])\n  IACCVIOL : [{2=0..1}]\n  DACCVIOL : [{2=1..2}]\n  MUNSTKERR: [{2=3..4}]\n  MSTKERR  : [{2=4..5}]\n  MLSPER   : [{2=5..6}]",
        cpuid(),
        address,
        status,
    );

    loop {
        core::arch::asm!("nop", options(nomem, nostack, preserves_flags));
    }
}
