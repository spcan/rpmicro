//! Analog to Digital Converter (ADC) module.

pub(crate) mod isr;

mod address;
mod config;
mod operation;

use crate::hal::{
    interrupts::{
        queue::{ISRQueue, ISRQueueNode},
        Interrupt,
    },
    AtomicRegister, Peripheral,
};

pub use config::{Channel, ChannelList, Operation, Precision};

pub(self) use operation::ADCOperation;

/// Global queue for ADC operations.
#[link_section = ".data"]
pub(self) static mut QUEUE: ISRQueue<ADCOperation> = ISRQueue::new();

/// The descriptor type of the ADC ISR.
pub type Descriptor = ISRQueueNode<ADCOperation>;

#[derive(Clone, Copy)]
pub struct ADCEngine;

impl Interrupt for ADCEngine {
    const NUMBER: usize = 35;
}

impl ADCEngine {
    /// Creates a new oneshot conversion.
    pub fn oneshot(&mut self, channel: impl Into<Channel>, precision: Precision) -> Descriptor {
        let descriptor = ADCOperation {
            operation: Operation::Oneshot(channel.into()),
            precision,
        };

        Descriptor::new(descriptor)
    }

    /// Performs `n` conversions of the given channels list.
    pub fn continuous(&mut self, channels: ChannelList, precision: Precision) -> Descriptor {
        let descriptor = ADCOperation {
            operation: Operation::Continuous(channels),
            precision,
        };

        Descriptor::new(descriptor)
    }
}

impl Peripheral for ADCEngine {
    unsafe fn instance() -> Self {
        Self
    }
}

/// Initial instance of the `ADCPeripheral`. Must be initialized to convert into an `ADCEngine`.
pub struct ADCPeripheral;

impl ADCPeripheral {
    /// Initializes the ADC engine.
    pub fn init(self) -> ADCEngine {
        // Enable the ADC and wait until it's ready.
        let control = AtomicRegister::at(address::CONTROL);
        control.set(0b11u32);

        while (control.read() & (1 << 8)) == 0 {
            cortex_m::asm::nop();
        }

        ADCEngine
    }
}

impl crate::hal::Peripheral for ADCPeripheral {
    unsafe fn instance() -> Self {
        Self
    }
}

/// List of errors possible in an ADC conversion.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Error {
    /// The ADC experienced an error during conversion.
    Conversion,

    /// The ADC FIFO was overrun and this value is not valid.
    Overrun,
}
