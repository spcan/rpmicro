//! Addresses of the register block.

#![allow(unused)]

/// Base address of the TRNG register block.
pub(super) const BASE: usize = 0x400F0000;

/// Address of the interrupt mask register.
pub(super) const IMR: usize = BASE + 0x100;

/// Address of the interrupt status register.
pub(super) const ISR: usize = BASE + 0x104;

/// Address of the interrupt clear register.
pub(super) const ICR: usize = BASE + 0x108;

/// Address of the chain length configuration register.
pub(super) const CONFIG: usize = BASE + 0x10C;

/// Address of the random data registers.
pub(super) const DATA: usize = BASE + 0x114;

/// Address of the enable register.
pub(super) const ENABLE: usize = BASE + 0x12C;

/// Address of the sample count register.
pub(super) const SAMPLES: usize = BASE + 0x130;

/// Address of the statistics register.
pub(super) const STATISTICS: usize = BASE + 0x134;

/// Address of the debug register.
pub(super) const DEBUG: usize = BASE + 0x138;

/// Address of the software reset register.
pub(super) const RESET: usize = BASE + 0x140;

/// Address of the debug enable register.
pub(super) const ENABLEDBG: usize = BASE + 0x1B4;

/// Address of the busy flag register.
pub(super) const BUSY: usize = BASE + 0x1B8;

/// Address of the collected bits count register.
pub(super) const BITCOUNT: usize = BASE + 0x1BC;
