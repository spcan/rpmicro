//! Vector table for the RP2350 interrutps.

use super::super::Vector;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
#[derive(Clone)]
#[repr(C)]
pub struct Interrupts {
    /// IRQ 0 of the TIMER 0 peripheral.
    pub tim0irq0: Vector,

    /// IRQ 1 of the TIMER 0 peripheral.
    pub tim0irq1: Vector,

    /// IRQ 2 of the TIMER 0 peripheral.
    pub tim0irq2: Vector,

    /// IRQ 3 of the TIMER 0 peripheral.
    pub tim0irq3: Vector,

    /// IRQ 0 of the TIMER 1 peripheral.
    pub tim1irq0: Vector,

    /// IRQ 1 of the TIMER 1 peripheral.
    pub tim1irq1: Vector,

    /// IRQ 2 of the TIMER 1 peripheral.
    pub tim1irq2: Vector,

    /// IRQ 3 of the TIMER 1 peripheral.
    pub tim1irq3: Vector,

    /// PWM Wrap IRQ 0.
    pub pwm0: Vector,

    /// PWM Wrap IRQ 1.
    pub pwm1: Vector,

    /// DMA IRQ0.
    pub dma0: Vector,

    /// DMA IRQ1.
    pub dma1: Vector,

    /// DMA IRQ2.
    pub dma2: Vector,

    /// DMA IRQ3.
    pub dma3: Vector,

    /// USB CTRL IRQ.
    pub usb: Vector,

    pub pio0irq0: Vector,

    pub pio0irq1: Vector,

    pub pio1irq0: Vector,

    pub pio1irq1: Vector,

    pub pio2irq0: Vector,

    pub pio2irq1: Vector,

    pub io: Vector,

    pub ions: Vector,

    pub qspi: Vector,

    pub qspins: Vector,

    pub fifo: Vector,

    pub bell: Vector,

    pub fifons: Vector,

    pub bellns: Vector,

    pub mtimecmp: Vector,

    pub clocks: Vector,

    pub spi0: Vector,

    pub spi1: Vector,

    pub uart0: Vector,

    pub uart1: Vector,

    pub adc: Vector,

    pub i2c0: Vector,

    pub i2c1: Vector,

    pub otp: Vector,

    pub trng: Vector,

    pub proc0cti: Vector,

    pub proc1cti: Vector,

    pub pllsys: Vector,

    pub pllusb: Vector,

    pub powmanpow: Vector,

    pub powmantimer: Vector,
}
