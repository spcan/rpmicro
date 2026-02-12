//! External memory module.

pub mod config;
pub mod psram;

mod peripheral;
mod timings;

use crate::{gpio::qspi::QMIChipSelect, interrupts::VectorTable, AtomicRegister};
use peripheral::ExtMem;

pub use config::{Config, DataWidth, RWFormat};
pub use timings::{Pagebreak, Timings};

pub static mut KGD: [u8; 6] = [0; 6];

/// Ownership over an external memory slot.
/// Allows to setup the external memory as either Flash or PSRAM.
pub struct ExternalMemorySlot(pub(crate) u8);

impl ExternalMemorySlot {
    /// Sets up this external memories slot as PSRAM.
    /// UNSAFETY : If any external memory is in use, running this function may cause a panic or hang the device.
    #[link_section = ".data"]
    #[inline(never)]
    pub unsafe fn psram(
        self,
        size: usize,
        cs: QMIChipSelect,
        config: Config,
        timings: Timings,
    ) -> ExternalMemory {
        ExternalMemory::psram(self.0 as usize, size, cs, config, timings)
    }
}

pub struct ExternalMemory {
    start: usize,
    size: usize,
    writable: bool,
}

impl ExternalMemory {
    /// Sets up one of the external memories as PSRAM.
    /// UNSAFETY : If any external memory is in use, running this function may cause a panic or hang the device.
    #[link_section = ".data"]
    #[inline(never)]
    unsafe fn psram(
        i: usize,
        size: usize,
        cs: QMIChipSelect,
        config: Config,
        timings: Timings,
    ) -> Self {
        // Forget the pin provided, it's already configured.
        core::mem::forget(cs);

        unsafe {
            // Set the QSPI Data pins.
            let mut pins = [0; 5];

            pins[0] = core::ptr::read_volatile((0x40040000 + 0x08) as *mut u32);
            pins[1] = core::ptr::read_volatile((0x40040000 + 0x0C) as *mut u32);
            pins[2] = core::ptr::read_volatile((0x40040000 + 0x10) as *mut u32);
            pins[3] = core::ptr::read_volatile((0x40040000 + 0x14) as *mut u32);
            pins[4] = core::ptr::read_volatile((0x40040000 + 0x04) as *mut u32);

            crate::log::info!("QSPI pins configuration pre-modification: {:#X}", pins);

            core::ptr::write_volatile((0x40040000 + 0x08) as *mut u32, 0x7B);
            core::ptr::write_volatile((0x40040000 + 0x0C) as *mut u32, 0x7B);
            core::ptr::write_volatile((0x40040000 + 0x10) as *mut u32, 0x7B);
            core::ptr::write_volatile((0x40040000 + 0x14) as *mut u32, 0x7B);

            // Set the QSPI SCK pin.
            core::ptr::write_volatile((0x40040000 + 0x04) as *mut u32, 0x3B);
        }

        // VectorTable::report();

        // Acquire the external memory interface in direct mode (safe configuration).
        let mut qmidirect = unsafe { ExtMem::steal().direct(10, 2) };

        crate::log::info!("QMI Direct mode successfully activated");

        // Poll until the last XIP transfer's cooldown expires.
        while qmidirect.busy() {
            unsafe { core::arch::asm!("nop", options(nomem, nostack, preserves_flags)) }
        }

        // Send the command to exit quad mode and reset the chip.
        for (iwidth, data) in config.commands.reset {
            qmidirect.write(i, *iwidth, [*data]);
        }
        // qmidirect.write(i, DataWidth::Quad, [Command::ExitQuadMode]);
        // qmidirect.write(i, DataWidth::Single, [Command::ResetEnable]);
        // qmidirect.write(i, DataWidth::Single, [Command::Reset]);

        // Read the Known Good Die (KGD) and the EID registers.
        let mut recv = [0u8; 6];
        let mut send = [0xFF; 6];
        send[0] = Command::ReadID as u8;

        qmidirect.transfer(i, DataWidth::Single, &send, &mut recv);

        crate::log::info!("KGD and EID registers have been read: {:X}", recv);

        // Enter a special mode if needed.
        // if let Some(sequence) = config.commands.mode {
        //     for (iwidth, data) in sequence {
        //         qmidirect.write(i, iwidth, [data]);
        //     }
        // }

        // Enter Dual / Quad mode if necessary.
        // if config.rdformat.data == DataWidth::Dual {
        //     //qmidirect.write(DataWidth::Single, [Command::EnterDualMode]);
        // }

        // if config.rdformat.data as usize == DataWidth::Quad as usize {
        qmidirect.write(i, DataWidth::Single, [Command::EnterQuadMode]);
        // }

        // Stop direct mode and configure the PSRAM.
        // From this point on, accesses to any memory can be performed.
        crate::log::info!("Stopping the direct mode");
        let mut psramcfg = unsafe { qmidirect.stop().mem(i) };

        crate::log::info!("Configuring the PSRAM");

        psramcfg.rdconfig(config.rdformat, Command::ReadQuad as u8);
        psramcfg.wrconfig(config.wrformat, Command::WriteQuad as u8);

        // Set default timings and enable writes to PSRAM in the XIP.
        psramcfg.timings(timings);
        AtomicRegister::at(0x400C8000).set((1u32 << (10 + i)) | 0b11);

        // Calculate the start address of the memory.
        let start = 0x10000000 + (0x01000000 * i);

        // Create the memory.
        ExternalMemory {
            start,
            size,
            writable: true,
        }
    }

    /// Validates the memory (if configured as writable).
    /// UNSAFETY : Can corrupt the memory if it is in use.
    pub unsafe fn validate(&mut self) -> Result<(), (usize, f32)> {
        // Set the chunk size to check.
        const SIZE: usize = 256;

        // If the memory is not writable we cannot validate communication.
        if !self.writable {
            return Ok(());
        }

        // Get the whole range as uncached.
        let memory = self.uncached::<u8>();

        for chunk in memory.chunks_mut(SIZE) {
            for (index, word) in chunk.iter_mut().enumerate() {
                *word = (index & 0xFF) as u8;
            }
        }

        // Read back and check amount of errors.
        let mut errors = 0;

        for chunk in memory.chunks(SIZE) {
            for (index, word) in chunk.iter().enumerate() {
                if *word != (index & 0xFF) as u8 {
                    errors += 1;
                }
            }
        }

        // Early return if no errors were found.
        if errors == 0 {
            return Ok(());
        }

        // Calculate the percentage of errors.
        let percent = 100.0 * errors as f32 / memory.len() as f32;

        Err((errors, percent))
    }

    /// Returns the whole memory range in the cached region.
    pub unsafe fn cached<T: Sized>(&mut self) -> &'static mut [T] {
        core::slice::from_raw_parts_mut(self.start as *mut T, self.size / core::mem::size_of::<T>())
    }

    /// Returns the whole memory range in the uncached region.
    pub unsafe fn uncached<T: Sized>(&mut self) -> &'static mut [T] {
        core::slice::from_raw_parts_mut(
            (self.start + 0x04000000) as *mut T,
            self.size / core::mem::size_of::<T>(),
        )
    }
}

// /// Results for the sequential throughput test.
// #[derive(Clone, Copy, Default)]
// pub struct ThroughputResult {
//     /// `u8` test results.
//     pub byte: TypeResult<u8>,

//     /// `u16` test results.
//     pub half: TypeResult<u16>,

//     /// `u32` test results.
//     pub word: TypeResult<u32>,

//     /// `u64` test results.
//     pub double: TypeResult<u64>,

//     /// `u128` test results.
//     pub wide: TypeResult<u128>,
// }

// /// Results of a test for a given type.
// #[derive(Clone, Copy, Default)]
// pub struct TypeResult<T: Sized> {
//     /// Amount of elements in the type.
//     pub count: usize,

//     /// Result for the given type in cached memory.
//     pub cached: TestResult,

//     /// Result for the given type in uncached memory.
//     pub uncached: TestResult,

//     _phantom: core::marker::PhantomData<T>,
// }

// impl<T: 'static + Sized + Copy + Default> TypeResult<T> {
//     pub fn bytes(&self) -> usize {
//         self.count * core::mem::size_of::<T>()
//     }

//     pub fn test(&mut self, memory: &mut ExternalMemory, count: usize) {
//         // Store the count to have it available in the tests.
//         self.count = count;

//         // Test the cached memory.
//         let cached = unsafe { &mut memory.cached()[0..count] };

//         self.cached.read = self.read(cached);
//         if memory.writable {
//             self.cached.write = self.write(cached);
//         }

//         // Test the uncached memory.
//         let uncached = unsafe { &mut memory.uncached()[0..count] };

//         self.uncached.read = self.read(uncached);
//         if memory.writable {
//             self.uncached.write = self.write(uncached);
//         }
//     }

//     fn read(&self, array: &[T]) -> f32 {
//         let base = array.as_ptr();

//         let start = Instant::now();

//         for i in 0..self.count {
//             let _ = unsafe { core::ptr::read_volatile(base.offset(i as isize)) };
//         }

//         let end = Instant::now();

//         (self.count * core::mem::size_of::<T>()) as f32 / (end - start).as_micros() as f32
//     }

//     fn write(&self, array: &mut [T]) -> f32 {
//         let base = array.as_mut_ptr();
//         let default = Default::default();

//         let start = Instant::now();

//         for i in 0..self.count {
//             unsafe { core::ptr::write_volatile(base.offset(i as isize), default) };
//         }

//         let end = Instant::now();

//         (self.count * core::mem::size_of::<T>()) as f32 / (end - start).as_micros() as f32
//     }
// }

// /// Results of a test for a given type in a given memory (cached or uncached).
// #[derive(Clone, Copy, Default)]
// pub struct TestResult {
//     /// Read performance of the test in MiB / s.
//     pub read: f32,

//     /// Write performance of the test in MiB / s.
//     pub write: f32,
// }

/// Default commands for PSRAM devices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]

pub enum Command {
    /// Write in SPI mode, fast frequency.
    Write = 0x02,

    /// Read in SPI mode, slow frequency.
    ReadSlow = 0x03,

    /// Read in SPI mode, fast frequency.
    ReadFast = 0x0B,

    /// Sets the chip in Quad SPI mode.
    EnterQuadMode = 0x35,

    /// Writes in Quad SPI mode, fast frequency.
    WriteQuad = 0x38,

    /// Enables the chip to be reset by the next command.
    /// This effect gets cancelled if the next command is not a `Reset`.
    ResetEnable = 0x66,

    /// Resets the chip.
    /// Must be preceded by a `ResetEnable` command to take effect.
    Reset = 0x99,

    /// Reads the KGD and EID registers.
    ReadID = 0x9F,

    /// Read in Quad SPI mode, fast frequency.
    ReadQuad = 0xEB,

    /// Default command to exit Quad Mode I/O.
    ExitQuadMode = 0xF5,
}

impl Into<u16> for Command {
    fn into(self) -> u16 {
        self as u16
    }
}

pub trait Device {
    fn config() -> config::Config;
}
