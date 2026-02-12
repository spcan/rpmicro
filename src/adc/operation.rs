//! Descriptors and queues for ADC control and access.

use crate::hal::{
    interrupts::{
        queue::{ISRObject, ISRQueue, ISRQueueNode},
        Interrupt,
    },
    AtomicRegister,
};

use super::{Channel, Error, Operation, Precision};

pub struct ADCOperation {
    /// The kind of operation performed.
    pub(super) operation: Operation,

    /// The bit-depth precision to use in this operation.
    pub(super) precision: Precision,
}

impl ISRObject for ADCOperation {
    type Output = Result<u16, (Error, u16)>;

    #[allow(static_mut_refs)]
    fn queue(&self) -> &'static mut ISRQueue<Self> {
        unsafe { &mut super::QUEUE }
    }

    #[allow(static_mut_refs)]
    fn slot(&self) -> &'static mut Option<core::ptr::NonNull<ISRQueueNode<Self>>> {
        unsafe { &mut super::isr::OPERATION }
    }

    fn trigger(&mut self) {
        // Pre-create the atomic register that will be used.
        let control = AtomicRegister::at(super::address::CONTROL);
        let fcs = AtomicRegister::at(super::address::FCS);
        let inte = AtomicRegister::at(super::address::INTE);

        // Clear previous errors.
        control.set(1u32 << 10);

        // Configure the conversion.
        match self.operation {
            Operation::Oneshot(channel) => {
                // Select the channel to measure.
                control.clear(0xFu32 << 12);
                control.set(<Channel as Into<u32>>::into(channel) << 12);
            }

            Operation::Continuous(_) => {
                // Select the channels to measure.
                control.clear(0xFu32 << 12);
            }
        }

        // Prepare the interrupts and set the precision level.
        fcs.write((1 << 24) | 1u32);

        if self.precision == Precision::Compressed {
            fcs.set(1u32 << 1);
        }

        // Enable the ADC and the ADC interrupt in the peripheral and the NVIC.
        unsafe {
            <super::ADCEngine as Interrupt>::enable();
        }
        control.set((1 << 2) | 1u32);
        inte.write(1u32);
    }

    fn collect(&mut self) -> Self::Output {
        let result = (AtomicRegister::at(super::address::RESULT).read() & 0xFFF) as u16;

        if (AtomicRegister::at(super::address::FIFO).read() >> 15) & 1 != 0 {
            return Err((Error::Conversion, result));
        }

        Ok(result)
    }
}
