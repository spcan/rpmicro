//! Set of peripherals common to all cores in the system.

use crate::{
    clocks::Clocks,
    gpio::GPIOList,
    mem::{
        dma::DMAChannels,
        internal::{BusControl, CachePerformance, GlobalPerformance},
        ExternalMemorySlot,
    },
    system::Core1Waker,
    Peripheral,
};

pub struct Common {
    /// Control over the behavior of the device's internal bus.
    pub buscontrol: BusControl,

    /// Performance counters for the device's buses.
    pub busperf: GlobalPerformance,

    /// Performance counters for the XIP cache.
    pub cacheperf: CachePerformance,

    /// The clock system of the device.
    pub clocks: Clocks,

    /// List of all DMA Channels of the device.
    pub dma: DMAChannels,

    /// Control over the external memories.
    pub extmem: ExternalMemorySlot,

    // /// The ADC engine of the device.
    // pub adc: ADCPeripheral,
    /// List of all GPIOs of the device.
    pub gpios: GPIOList,

    /// Wakes Core 1.
    pub waker: Core1Waker,
}

impl Common {
    /// Initializes the peripherals and returns the instance controlling them.
    pub(crate) unsafe fn init() -> Self {
        Self {
            // adc: ADCPeripheral::instance(),
            buscontrol: BusControl::instance(),
            busperf: GlobalPerformance::instance(),
            // cachecontrol: CacheControl::instance(),
            cacheperf: CachePerformance::instance(),
            clocks: Clocks::instance(),
            dma: DMAChannels::all(),
            extmem: ExternalMemorySlot(1),
            gpios: GPIOList::all(),
            waker: Core1Waker::instance(),
        }
    }
}
