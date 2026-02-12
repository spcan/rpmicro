//! Multicore control and access.

use core::sync::atomic::{compiler_fence, AtomicBool, Ordering};

pub struct Core<const N: usize>;

impl<const N: usize> Core<N> {
    /// Spawns `Core<1>` to run the given `main` function.
    /// Will panic if `Core<0>` tries to spawn itself.
    pub unsafe fn spawn(&mut self, main: fn() -> !) -> Result<(), ()> {
        // Reset the core.

        // Setup the stack.

        // Ensure all writes are performed before continuing.

        // Sequence of commands to send to the other core.
        // Try to send the sequence 5 times and fail if it didn't wake up yet.
        let sequence = [0, 0, 1, vtable, stack, main];
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

        // Wait until the other core has copied `entry` before returning.
        recv();

        Ok(())
    }

    /// Sends a wakeup sequence to the other core.
    fn wakeup(&self, sequence: &[u32; 6]) -> Result<(), ()> {
        for command in sequence {
            // Command `0` needs a FIFO drain.
            if *command == 0 {
                drain();
                cortex_m::asm::sev();
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
}



    // Reset the core
    let psm = pac::PSM;
    psm.frce_off().modify(|w| w.set_proc1(true));
    while !psm.frce_off().read().proc1() {
        cortex_m::asm::nop();
    }
    psm.frce_off().modify(|w| w.set_proc1(false));
