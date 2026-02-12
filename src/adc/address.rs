//! Register addresses of the ADC.

#![allow(unused)]

pub(super) const BASE: usize = 0x400A0000;

pub(super) const CONTROL: usize = BASE + 0x00;

pub(super) const RESULT: usize = BASE + 0x04;

pub(super) const FCS: usize = BASE + 0x08;

pub(super) const FIFO: usize = BASE + 0x0C;

pub(super) const DIV: usize = BASE + 0x10;

pub(super) const INTR: usize = BASE + 0x14;

pub(super) const INTE: usize = BASE + 0x18;

pub(super) const INTF: usize = BASE + 0x1C;

pub(super) const INTS: usize = BASE + 0x20;
