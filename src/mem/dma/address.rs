//! Addresses of the DMA registers.

#![allow(unused)]

/// Base address of the DMA register block.
pub const BASE: usize = 0x50000000;

/// Stride between different DMA channels registers.
pub const STRIDE: usize = 0x40;

pub const CONTROL: usize = 0x0C;
