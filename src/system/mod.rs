//! System control and configuration.

// mod psm;

// pub use psm::PowerState;

use core::sync::atomic::{compiler_fence, Ordering};

use crate::{interrupts::VectorTable, AtomicRegister, Peripheral};

/// The bottom of the stack for Core 0 and Core 1.
const STACKBOTTOM: [u32; 2] = [0x20080000, 0x20081000];

/// The top of the stack for Core 0 and Core 1.
const STACKTOP: [u32; 2] = [0x20081000 - 4, 0x20082000 - 4];

/// Specific utility used to wake up Core 1 from Core 0.
pub struct Core1Waker;

impl Core1Waker {
    /// Spawns `Core<1>` to run the given `main` function.
    /// This function will reset CPU 1.
    pub unsafe fn spawn(&mut self, main: fn() -> !) -> Result<(), ()> {
        // Reset the core via the PSM.
        Self::reset();

        // Duplicate the vector table for the other core.
        VectorTable::duplicate(STACKBOTTOM[1] as *mut u32);

        // Ensure all writes have gone through by the time we wake up the core.
        compiler_fence(Ordering::Release);

        // Sequence of commands to send to the other core.
        // Try to send the sequence 5 times and fail if it didn't wake up yet.
        let pointer = trampoline as *const () as u32;
        let sequence = [0, 0, 1, STACKBOTTOM[1], STACKTOP[1], pointer];
        let mut success = false;

        for i in 0..5 {
            if let Ok(_) = self.wakeup(&sequence) {
                success = true;
                break;
            }
        }

        if !success {
            crate::log::error!("Core 1 is not responding to wakeup attempts");
            return Err(());
        }

        // Wait until the other core is ready before sending the main function.
        let _ = recv();
        send(main as u32);

        Ok(())
    }

    /// Sends a wakeup sequence to Core 1.
    fn wakeup(&self, sequence: &[u32; 6]) -> Result<(), ()> {
        for command in sequence {
            // Command `0` needs a FIFO drain.
            if *command == 0 {
                Self::drain();
                crate::asm::sev();
            }

            // Send the command to the other core.
            send(*command);

            // Wait for the response from the other core and check agains the current command.
            let response = recv();

            if response != *command {
                return Err(());
            }
        }

        Ok(())
    }

    /// Resets Core 1.
    fn reset() {
        // Get the registers to force on and off.
        let on = AtomicRegister::at(0x40018000);
        let off = AtomicRegister::at(0x40018004);

        // Force Core 1 off.
        on.clear(1u32 << 24);
        off.set(1u32 << 24);

        // Allow the reset to happen.
        crate::asm::nop();

        // Force Core 1 on.
        off.clear(1u32 << 24);
        on.set(1u32 << 24);
    }

    /// Drains the current mailbox.
    fn drain() {
        while valid() {
            let _ = recv();
        }
    }
}

impl Peripheral for Core1Waker {
    unsafe fn instance() -> Self {
        Self {}
    }
}

/// Core 1 trampoline function to jump to main code.
fn trampoline() -> ! {
    crate::log::info!("[CPU1] Hello world!");

    // Signal that it's okay for Core 0 to send the main function then read it.
    send(1);
    crate::log::info!("[CPU1] Waiting for main function");
    let main = recv();

    // Jump to the main function.
    crate::log::info!("[CPU1] Jumping to main function");
    unsafe { core::mem::transmute::<usize, fn() -> !>(main as usize)() }
}

/// Reads one value from the mailbox.
fn recv() -> u32 {
    // Wait for the FIFO to have enough space.
    while !valid() {
        crate::asm::nop();
    }

    // Send data and an event to the other core to wake up.
    unsafe { core::ptr::read_volatile(0xD0000058 as *const u32) }
}

/// Sends the given `value` to the other core.
fn send(value: u32) {
    // Wait for the FIFO to have enough space.
    while !ready() {
        crate::asm::nop();
    }

    // Send data and an event to the other core to wake up.
    unsafe {
        core::ptr::write_volatile(0xD0000054 as *mut u32, value);
        crate::asm::sev();
    }
}

/// Returns `true` if the mailbox is ready to send more data.
fn ready() -> bool {
    let status = unsafe { core::ptr::read_volatile(0xD0000050 as *const u32) };
    (status & (1 << 1)) != 0
}

/// Returns `true` if the mailbox has valid data.
fn valid() -> bool {
    let status = unsafe { core::ptr::read_volatile(0xD0000050 as *const u32) };
    (status & (1 << 0)) != 0
}
