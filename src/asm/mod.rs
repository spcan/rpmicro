//! Widely used assembly instructions.

#![allow(unused)]

use core::arch::asm;

#[inline(always)]
pub fn nop() {
    unsafe { asm!("nop", options(nomem, nostack, preserves_flags)) }
}

#[inline(always)]
pub fn sev() {
    unsafe { asm!("sev", options(nomem, nostack, preserves_flags)) }
}

#[inline(always)]
pub fn bkpt() {
    unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) }
}
