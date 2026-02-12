//! Direct Memory Access (DMA) peripheral.
//! Performs memory transfers without CPU involvement.

mod address;

use crate::AtomicRegister;

use core::ptr::{read_volatile, write_volatile};

pub struct DMAChannels(u16);

impl DMAChannels {
    /// Takes the next available DMA channel.
    pub fn take(&mut self) -> Option<DMAChannel> {
        for i in 0..16 {
            if self.0 & (1 << i) != 0 {
                self.0 &= !(1 << i);
                return Some(DMAChannel(i));
            }
        }

        None
    }

    /// Creates a list containing all the DMA channels.
    pub(crate) unsafe fn all() -> Self {
        Self(0xFFFF)
    }
}

/// Control over one of the DMA channels.
/// While this struct's instance is alive, the channel is inactive.
pub struct DMAChannel(pub(crate) u8);

impl DMAChannel {
    /// Base address of the DMA channels' registers.
    const BASE: usize = 0x50000000;

    const READ: usize = 0x00;

    const WRITE: usize = 0x04;

    const COUNT: usize = 0x08;

    const CONTROL: usize = 0x0C;

    const MULTITRIGGER: usize = 0x450;

    /// Stride between the different DMA channels registers.
    const CHSTRIDE: usize = 0x40;

    /// Sets the read address of the channel.
    pub(crate) fn read(&mut self, address: u32) {
        let register = Self::BASE + Self::READ + (Self::CHSTRIDE * self.0 as usize);
        unsafe { write_volatile(register as *mut u32, address) }
    }

    /// Sets the write address of the channel.
    pub(crate) fn write(&mut self, address: u32) {
        let register = Self::BASE + Self::WRITE + (Self::CHSTRIDE * self.0 as usize);
        unsafe { write_volatile(register as *mut u32, address) }
    }

    /// Sets the transfer count for the DMA channel.
    pub(crate) fn count(&mut self, count: TransferCount) {
        let register = Self::BASE + Self::COUNT + (Self::CHSTRIDE * self.0 as usize);
        unsafe { write_volatile(register as *mut u32, count.0) }
    }

    /// Sets the configuration for the DMA channel.
    pub(crate) fn config(&mut self, config: u32) {
        let register = Self::BASE + Self::CONTROL + (Self::CHSTRIDE * self.0 as usize);
        unsafe { write_volatile(register as *mut u32, config) }
    }

    pub(crate) fn start(&mut self) {
        let register = Self::BASE + Self::MULTITRIGGER;
        unsafe { write_volatile(register as *mut u32, 1 << self.0) }
    }
}

impl DMAChannel {
    /// Returns `true` if the `DMAChannel` is busy.
    pub fn busy(&self) -> bool {
        let address = Self::BASE + Self::CONTROL + (Self::CHSTRIDE * self.0 as usize);
        ((unsafe { read_volatile(address as *mut u32) } >> 26) & 1) == 1
    }

    /// Use the `DMAChannel` to copy one buffer to another.
    pub fn copy<T: Copy + Sized>(&mut self, src: &[T], dst: &mut [T]) {
        const CONFIG: u32 = (0x3F << 17) | (1 << 6) | (1 << 4);

        self.read(src.as_ptr() as u32);
        self.write(dst.as_mut_ptr() as u32);
        self.count(TransferCount::oneshot(
            (dst.len() * core::mem::size_of::<T>()) as u32,
        ));
        self.config(CONFIG | 1);
        self.start();
    }
}

pub struct DMAChannelActive(pub(crate) u8);

/// Common trait for all peripherals that can interact with the DMA.
pub trait DMAPeripheral {
    /// Returns the `DataRequest` associated with this peripheral.
    fn request(&self) -> DataRequest;

    fn register();
}

pub enum RWDirection {
    /// DMA Reads or Writes will not modify the Read or Write address.
    Locked = 0b00,

    /// DMA Reads or Writes will increment the Read or Write address.
    Increase = 0b01,

    /// DMA Reads or Writes will increment the Read or Write address by twice the transfer size.
    /// This causes it to Read or Write only one every two elements.
    Leapfrog = 0b10,

    /// DMA Reads or Writes will decrement the Read or Write address.
    Decrease = 0b11,
}

pub enum DataSize {
    Byte = 0,
    Half = 1,
    Word = 2,
}

pub enum DataRequest {
    PIO0TX0 = 00,
    PIO0TX1 = 01,
    PIO0TX2 = 02,
    PIO0TX3 = 03,
    PIO0RX0 = 04,
    PIO0RX1 = 05,
    PIO0RX2 = 06,
    PIO0RX3 = 07,
    PIO1TX0 = 08,
    PIO1TX1 = 09,
    PIO1TX2 = 10,
    PIO1TX3 = 11,
    PIO1RX0 = 12,
    PIO1RX1 = 13,
    PIO1RX2 = 14,
    PIO1RX3 = 15,
    PIO2TX1 = 16,
    PIO2TX2 = 17,
    PIO2TX3 = 18,
    PIO2TX4 = 19,
    PIO2RX0 = 20,
    PIO2RX1 = 21,
    PIO2RX2 = 22,
    PIO2RX3 = 23,
    SPI0TX = 24,
    SPI0RX = 25,
    SPI1TX = 26,
    SPI1RX = 27,
    UART0TX = 28,
    UART0RX = 29,
    UART1TX = 30,
    UART1RX = 31,
    PWM0 = 32,
    PWM1 = 33,
    PWM2 = 34,
    PWM3 = 35,
    PWM4 = 36,
    PWM5 = 37,
    PWM6 = 38,
    PWM7 = 39,
    PWM8 = 40,
    PWM9 = 41,
    PWM10 = 42,
    PWM11 = 43,
    I2C0TX = 44,
    I2C0RX = 45,
    I2C1TX = 46,
    I2C1RX = 47,
    ADC = 48,
    XIPSTREAM = 49,
    QMITX = 50,
    QMIRX = 51,
    HSTX = 52,
    CORESIGHT = 53,
    SHA256 = 54,

    /// Perform a transfer on every DMA Timer 0 tick.
    TIMER0 = 0x3B,

    /// Perform a transfer on every DMA Timer 1 tick.
    TIMER1 = 0x3C,

    /// Perform a transfer on every DMA Timer 2 tick.
    TIMER2 = 0x3D,

    /// Perform a transfer on every DMA Timer 3 tick.
    TIMER3 = 0x3E,

    /// Perform transfers constantly until the final count is reached (or indefinitely if unlimited).
    Permanent = 0x3F,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TransferCount(pub(crate) u32);

impl TransferCount {
    /// Creates a `TransferCount` that will trigger once.
    pub const fn oneshot(count: u32) -> Self {
        Self(count & 0x0FFFFFFF)
    }

    /// Creates a `TransferCount` that will repeat itself forever.
    /// Once the count reaches 0, the corresponding interrupt and triggers will launch and the channel restarts.
    pub const fn repeat(count: u32) -> Self {
        Self((0x1 << 28) | (count & 0x0FFFFFFF))
    }

    /// Creates a `TransferCount` that does not decrement.
    /// The DMA channel will go on forever without triggering any interrupts until ABORT is raised.
    pub const fn endless() -> Self {
        Self((0xF << 28) | 1)
    }
}
