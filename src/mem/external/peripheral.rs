//! Abstraction over the QMI peripheral.

use super::{DataWidth, RWFormat, Timings};
use crate::AtomicRegister;

/// Control over the external memory interface (QSPI Memory Interface).
pub struct ExtMem;

impl ExtMem {
    /// Unsafely create an instance of `ExtMem`.
    /// SAFETY : You must ensure that you're only using one instance of this type at a time.
    pub const unsafe fn steal() -> Self {
        Self
    }

    /// Stop default memory interface and run the QSPI manually.
    /// SAFETY : This will stop all external memory, make sure you run this function from SRAM.
    pub unsafe fn direct(self, clkdiv: u32, rxdelay: u32) -> QMIDirect {
        QMIDirect::create(clkdiv, rxdelay)
    }

    /// Takes the external memory configuration for the given bank.
    /// SAFETY : You must ensure that you're only using one instance of this type at a time.
    pub unsafe fn mem(&mut self, i: usize) -> ExtMemConfig {
        ExtMemConfig::create(i & 1)
    }
}

/// Direct control over the QSPI Memory Interface.
pub struct QMIDirect;

impl QMIDirect {
    /// The base address of the QMI peripheral.
    const BASE: usize = 0x400D0000;

    /// Address of the DIRECT CSR register.
    const CSR: usize = Self::BASE + 0x00;

    /// Address of the DIRECT TX register.
    const TX: usize = Self::BASE + 0x04;

    /// Address of the DIRECT RX register.
    const RX: usize = Self::BASE + 0x08;

    /// Creates a `QMIDirect` instance with the given configuration.
    #[inline(always)]
    pub(self) fn create(clkdiv: u32, rxdelay: u32) -> Self {
        AtomicRegister::at(Self::CSR).write(((rxdelay & 0x3) << 30) | ((clkdiv & 0xFF) << 22) | 1);
        Self
    }

    /// Stops the QMI direct mode and returns the `ExtMem` instance.
    #[inline(always)]
    pub fn stop(self) -> ExtMem {
        AtomicRegister::at(Self::CSR).write(0u32);

        ExtMem
    }

    /// Sends a set of bytes to the selected target and receives the same amount of data.
    #[inline(always)]
    pub fn transfer<'a, T: 'a + Copy + Sized + Into<u16>>(
        &mut self,
        mem: usize,
        iwidth: DataWidth,
        send: &[T],
        recv: &mut [T],
    ) {
        // The command used for the transfer.
        const COMMAND: u32 = 1 << 19;

        // Get the data and interface width.
        let dwidth = if core::mem::size_of::<T>() == 1 { 0 } else { 1 };

        // Assert the CS pin.
        Self::assert(mem);

        let tx = AtomicRegister::at(Self::TX);

        for (s, r) in send.into_iter().zip(recv.into_iter()) {
            tx.write(COMMAND | (dwidth << 18) | ((iwidth as u32) << 16) | ((*s).into() as u32));

            // Wait until all data has been shifted out.
            while !self.txempty() {
                crate::asm::nop();
            }

            // Wait until all data has been received, then read the new data.
            while self.busy() && !self.rxempty() {
                crate::asm::nop();
            }

            *r = unsafe { core::ptr::read_volatile(Self::RX as *const T) };
        }

        while self.busy() {
            crate::asm::nop();
        }

        Self::deassert(mem);
    }

    /// Sends a set of bytes to the selected target.
    #[inline(always)]
    pub fn write<T: Sized + Into<u16>, W: IntoIterator<Item = T>>(
        &mut self,
        mem: usize,
        iwidth: DataWidth,
        data: W,
    ) {
        // The command used for the transfer.
        const COMMAND: u32 = (1 << 20) | (1 << 19);

        // Get the data and interface width.
        let dwidth = if core::mem::size_of::<T>() == 1 { 0 } else { 1 };

        // Assert the CS pin.
        Self::assert(mem);

        let tx = AtomicRegister::at(Self::TX);

        for word in data.into_iter() {
            tx.write(COMMAND | (dwidth << 18) | ((iwidth as u32) << 16) | (word.into() as u32))
        }

        while self.busy() {
            crate::asm::nop();
        }

        Self::deassert(mem);
    }

    /// Returns `true` if the interface is busy with a transaction.
    #[inline(always)]
    pub fn busy(&self) -> bool {
        (AtomicRegister::at(Self::CSR).read() & (1 << 1)) != 0
    }

    /// Returns `true` if the TX buffer is empty.
    #[inline(always)]
    pub fn txempty(&self) -> bool {
        (AtomicRegister::at(Self::CSR).read() & (1 << 11)) != 0
    }

    /// Returns `true` if the RX buffer is empty.
    #[inline(always)]
    pub fn rxempty(&self) -> bool {
        (AtomicRegister::at(Self::CSR).read() & (1 << 16)) != 0
    }

    /// Asserts the CS pin of the selected memory.
    #[inline(always)]
    fn assert(mem: usize) {
        AtomicRegister::at(Self::CSR).set(1u32 << (2 + mem));
    }

    /// Deasserts the CS pin of the selected memory.
    #[inline(always)]
    fn deassert(mem: usize) {
        AtomicRegister::at(Self::CSR).clear(1u32 << (2 + mem));
    }
}

/// Configuration of an external memory.
pub struct ExtMemConfig(*mut u32);

impl ExtMemConfig {
    /// Base address of the QMI peripheral.
    const BASE: usize = 0x400D0000;

    /// Offset of the first configuration register.
    const OFFSET: usize = 0x0C;

    /// Size in bytes of each configuration register.
    const SIZE: usize = 20;

    /// Creater an `ExtMemConfig` i
    /// nstance for the given memory bank.
    pub(self) unsafe fn create(mem: usize) -> Self {
        Self((Self::BASE + Self::OFFSET + (Self::SIZE * mem)) as *mut u32)
    }

    /// Configure the timings of this interface.
    pub fn timings(&mut self, timings: Timings) {
        unsafe {
            core::ptr::write_volatile(self.0, timings.into());
        }
    }

    /// Sets the write configuration of this external memory.
    pub fn rdconfig(&mut self, fmt: RWFormat, cmd: u8) {
        // Offset to the Write Format register.
        const RDFMT: isize = 1;

        // Offset to the Write Command register.
        const RDCMD: isize = 2;

        self.config::<RDFMT, RDCMD>(fmt, cmd);
    }

    /// Sets the write configuration of this external memory.
    pub fn wrconfig(&mut self, fmt: RWFormat, cmd: u8) {
        // Offset to the Write Format register.
        const WRFMT: isize = 3;

        // Offset to the Write Command register.
        const WRCMD: isize = 4;

        self.config::<WRFMT, WRCMD>(fmt, cmd);
    }

    /// Common function to write a R/W configuration.
    /// Used for code compactness.
    #[inline(always)]
    fn config<const FMT: isize, const CMD: isize>(&self, fmt: RWFormat, cmd: u8) {
        unsafe {
            core::ptr::write_volatile(self.0.offset(FMT), fmt.0);
            core::ptr::write_volatile(self.0.offset(CMD), cmd as u32);
        }
    }
}
