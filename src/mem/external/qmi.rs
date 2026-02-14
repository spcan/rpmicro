//! Handles direct QMI reads and writes.

use crate::{mem::external::DataWidth, AtomicRegister};

pub(super) trait Data: Copy + Sized + Into<u16> {}

impl Data for u8 {}
impl Data for u16 {}

pub struct QMIDirect(usize);

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
    pub(super) unsafe fn create(slot: usize) -> Self {
        const CONFIG: u32 = (10 << 30) | (2 << 22) | 1;

        AtomicRegister::at(Self::CSR).write(CONFIG);

        Self(slot)
    }

    /// Stops the QMI direct mode and returns the `ExtMem` instance.
    #[inline(always)]
    pub fn stop(self) {
        AtomicRegister::at(Self::CSR).write(0u32);
    }

    /// Sends a set of bytes to the selected target and receives the same amount of data.
    #[inline(always)]
    pub fn transfer<'a, T: 'a + Data>(&mut self, width: DataWidth, send: &[T], recv: &mut [T]) {
        // Drain the current RX FIFO.
        self.drain();

        // The command used for the transfer.
        let command =
            (1 << 19) | (((core::mem::size_of::<T>() == 1) as u32) << 18) | ((width as u32) << 16);

        // Start the transmission.
        let tx = AtomicRegister::at(Self::TX);
        self.assert();

        for (s, r) in send.iter().zip(recv.iter_mut()) {
            tx.write(command | ((*s).into() as u32));

            // Wait until all data has been shifted out.
            while !self.txempty() {
                crate::asm::nop();
            }

            // Wait until all data has been received, then read the new data.
            while self.busy() {
                crate::asm::nop();
            }

            *r = unsafe { core::ptr::read_volatile(Self::RX as *const T) };
        }

        // Wait for the end of the transfer.
        while self.busy() {
            crate::asm::nop();
        }

        self.deassert();
    }

    /// Sends a set of bytes to the selected target.
    #[inline(always)]
    pub fn write<'a, T: 'a + Data>(&mut self, width: DataWidth, send: &[T]) {
        // Drain the current RX FIFO.
        self.drain();

        // The command used for the transfer.
        let command =
            (1 << 19) | (((core::mem::size_of::<T>() == 1) as u32) << 18) | ((width as u32) << 16);

        // Start the transmission.
        let tx = AtomicRegister::at(Self::TX);
        self.assert();

        for word in send.iter() {
            tx.write(command | ((*word).into() as u32));

            // Wait until all data has been sent.
            while self.busy() {
                crate::asm::nop();
            }
        }

        // Wait for the end of the transfer.
        while self.busy() {
            crate::asm::nop();
        }

        self.deassert();
    }

    /// Drains the RX FIFO of the QMI.
    #[inline(always)]
    pub fn drain(&mut self) {
        // Wait until all data has been received, then read the new data.
        while !self.rxempty() {
            let _ = unsafe { core::ptr::read_volatile(Self::RX as *const u32) };
        }
    }

    /// Returns `true` if the interface is busy with a transaction.
    #[inline(always)]
    pub fn busy(&self) -> bool {
        (AtomicRegister::at(Self::CSR).read() & (1 << 1)) != 0
    }

    /// Returns `true` if the interface's RX FIFO is empty.
    #[inline(always)]
    pub fn rxempty(&self) -> bool {
        (AtomicRegister::at(Self::CSR).read() & (1 << 16)) != 0
    }

    /// Returns `true` if the interface's TX FIFO is empty.
    #[inline(always)]
    pub fn txempty(&self) -> bool {
        (AtomicRegister::at(Self::CSR).read() & (1 << 16)) != 0
    }

    /// Asserts the CS pin of the selected memory.
    #[inline(always)]
    fn assert(&mut self) {
        AtomicRegister::at(Self::CSR).set(1u32 << (2 + self.0));
    }

    /// Deasserts the CS pin of the selected memory.
    #[inline(always)]
    fn deassert(&mut self) {
        AtomicRegister::at(Self::CSR).clear(1u32 << (2 + self.0));
    }
}

impl Drop for QMIDirect {
    fn drop(&mut self) {
        AtomicRegister::at(Self::CSR).write(0u32);
    }
}
