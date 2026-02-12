//! Addresses of the register block.

#![allow(unused)]

/// Base address of the DMA register block.
pub(super) const BASE: usize = 0x50000000;

/// Size of a DMA register block in bytes.
pub(super) const BLOCKSIZE: usize = 0x10;

/// Number of alias addresses per DMA register block.
pub(super) const NALIAS: usize = 4;

pub(super) const INTRAW: usize = BASE + 0x400;

pub(super) const INTENABLE: usize = BASE + 0x404;

pub(super) const INTFORCE: usize = BASE + 0x408;

pub(super) const INTSTATUS: usize = BASE + 0x40C;

pub(super) const INTCPUSTRIDE: usize = 0x20;

pub(super) const INTHALFSTRIDE: usize = 0x20;
