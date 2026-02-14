//! External memory module.

pub mod config;
pub mod psram;

mod qmi;
mod timings;

use crate::{gpio::qspi::QMIChipSelect, AtomicRegister};

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
        // Forget the pin provided, it's already configured.
        core::mem::forget(cs);

        ExternalMemory::psram(self.0 as usize, size, config, timings)
    }
}

/// Exposes control over one of the external memory slots of the device.
pub struct ExternalMemory {
    /// Indicates which slot this memory belongs to.
    slot: usize,

    /// The size in bytes of the external memory.
    size: usize,

    /// Indicates if this memory is writable.
    writable: bool,
}

impl ExternalMemory {
    const BASE: usize = 0x400D0000;

    const TIMINGS: usize = Self::BASE + 0x0C;

    const RDFORMAT: usize = Self::BASE + 0x10;

    const RWSTRIDE: usize = 0x08;

    const SLOTSTRIDE: usize = 0x14;

    /// Sets up one of the external memories as PSRAM.
    /// UNSAFETY : If any external memory is in use, running this function may cause a panic or hang the device.
    #[link_section = ".data"]
    #[inline(never)]
    unsafe fn psram(slot: usize, size: usize, config: Config, timings: Timings) -> Self {
        unsafe {
            // Set the QSPI Data pins.
            core::ptr::write_volatile((0x40040000 + 0x08) as *mut u32, 0x56);
            core::ptr::write_volatile((0x40040000 + 0x0C) as *mut u32, 0x56);
            core::ptr::write_volatile((0x40040000 + 0x10) as *mut u32, 0x5A);
            core::ptr::write_volatile((0x40040000 + 0x14) as *mut u32, 0x5A);

            // Set the QSPI SCK pin.
            core::ptr::write_volatile((0x40040000 + 0x04) as *mut u32, 0x56);
        }

        // Acquire the external memory interface in direct mode (safe configuration).
        crate::log::warn!("Activating QMI direct mode. Flash accesses from now on will panic!");
        let mut qmidirect = unsafe { qmi::QMIDirect::create(slot) };

        // Poll until the last XIP transfer's cooldown expires.
        while qmidirect.busy() {
            unsafe { core::arch::asm!("nop", options(nomem, nostack, preserves_flags)) }
        }

        // Send the command to exit quad mode and reset the chip.
        for (iwidth, data) in config.commands.reset {
            qmidirect.write(*iwidth, &[*data]);
        }

        // Read the Known Good Die (KGD) and the EID registers.
        let mut recv = [0u8; 6];
        let mut send = [0x00; 6];
        send[0] = 0x9F;

        qmidirect.transfer(DataWidth::Single, &send, &mut recv);

        // Enter a special mode if needed.
        if let Some(sequence) = config.commands.mode {
            for (iwidth, data) in sequence {
                qmidirect.write(*iwidth, &[*data]);
            }
        }

        // Stop direct mode. From this point on, accesses to any memory can be performed.
        qmidirect.stop();

        crate::asm::dmb();
        crate::asm::dsb();
        crate::asm::isb();

        crate::log::debug!("KGD and EID registers: {:X}", recv);

        // Configure the PSRAM, set the RW format and timings.
        let mut psram = Self {
            slot,
            size,
            writable: true,
        };

        crate::log::debug!("Configuring the PSRAM...");

        psram.rdconfig(config.rdformat, config.commands.read as u8);
        psram.wrconfig(config.wrformat, config.commands.write as u8);

        psram.timings(timings);

        // Enable writes to PSRAM in the XIP.
        AtomicRegister::at(0x400C8000).set((1u32 << (10 + slot)) | 0b11);

        psram
    }

    /// Sets the read configuration of this external memory.
    #[inline(always)]
    pub fn rdconfig(&mut self, fmt: RWFormat, cmd: u8) {
        self.format::<0>(fmt, cmd);
    }

    /// Sets the write configuration of this external memory.
    #[inline(always)]
    pub fn wrconfig(&mut self, fmt: RWFormat, cmd: u8) {
        self.format::<1>(fmt, cmd);
    }

    /// Common function to write a R/W configuration.
    /// Used for code compactness.
    #[inline(always)]
    fn format<const RW: usize>(&mut self, fmt: RWFormat, cmd: u8) {
        // Calculate the register in which to store this configuration.
        let register = Self::RDFORMAT + (Self::RWSTRIDE * RW) + (Self::SLOTSTRIDE * self.slot);

        unsafe {
            core::ptr::write_volatile(register as *mut u32, fmt.0);
            core::ptr::write_volatile((register + 0x04) as *mut u32, cmd as u32);
        }
    }

    /// Configure the timings of this interface.
    /// UNSAFETY : Setting this method with untested `Timings` can render this memory unusable.
    pub unsafe fn timings(&mut self, timings: Timings) {
        core::ptr::write_volatile(
            (Self::TIMINGS + (Self::SLOTSTRIDE * self.slot)) as *mut u32,
            timings.into(),
        );
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

        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

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
        // Base address for the cached XIP memory.
        const BASE: usize = 0x10000000;

        let start = BASE + (0x01000000 * self.slot);

        core::slice::from_raw_parts_mut(start as *mut T, self.size / core::mem::size_of::<T>())
    }

    /// Returns the whole memory range in the uncached region.
    pub unsafe fn uncached<T: Sized>(&mut self) -> &'static mut [T] {
        // Base address for the cached XIP memory.
        const BASE: usize = 0x14000000;

        let start = BASE + (0x01000000 * self.slot);

        core::slice::from_raw_parts_mut(start as *mut T, self.size / core::mem::size_of::<T>())
    }
}

/// Generic trait for external memory devices. This trait is implementable by
/// the user or other libraries to generate predetermined configurations.
pub trait Device {
    /// Creates a configuration for the device.
    fn config() -> config::Config;
}
