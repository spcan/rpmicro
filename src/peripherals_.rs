//! Singleton control over the common and core local peripherals.

use crate::{
    // adc::ADCPeripheral,
    cortex::{
        mpu::{DeviceMemory, MemoryAttribute, NormalMemory, Region},
        SystemControl,
    },
    cpuid,
    gpio::GPIOList,
};

mod attributes {
    use crate::cortex::mpu::NormalMemoryAttributes;

    /// Memory attributes of SRAM style memory.
    pub const SRAM: NormalMemoryAttributes = NormalMemoryAttributes::uncachable();

    /// Memory attributes of the external memory cached alias.
    pub const CACHED: NormalMemoryAttributes =
        NormalMemoryAttributes::writethrough(true, true, true);
}

/// List of all attributes used in the RP2350.
const ATTRIBUTES: [MemoryAttribute; 3] = [
    // Memory attributes for peripheral memory.
    DeviceMemory::nGnRnE.generic(),
    // Memory attributes for all SRAM and ROM.
    // It also works for the non-cachable alias of external memory.
    NormalMemory::new(attributes::SRAM, attributes::SRAM).generic(),
    // Memory attributes for the external devices cachabe memory.
    NormalMemory::new(attributes::CACHED, attributes::CACHED).generic(),
];

mod regioncfg {
    use crate::cortex::mpu::{AccessPermissions, RegionConfiguration, Shareability};

    pub const SRAM: RegionConfiguration = RegionConfiguration::new(
        Shareability::OuterShareable,
        AccessPermissions::RWAll,
        false,
    );

    pub const ROM: RegionConfiguration =
        RegionConfiguration::new(Shareability::NonShareable, AccessPermissions::ROAll, false);

    pub const LOCAL: RegionConfiguration =
        RegionConfiguration::new(Shareability::NonShareable, AccessPermissions::RWAll, true);

    pub const PERIPHERALS: RegionConfiguration =
        RegionConfiguration::new(Shareability::OuterShareable, AccessPermissions::RWAll, true);
}

/// List of all reagions of memory used in the RP2350.
const REGIONS: [Region; 7] = [
    // Local stack, address to be modified by caller.
    Region::new(0x20080000..0x20081000, regioncfg::SRAM, 1),
    // Main SRAM section, SRAM 0-7.
    Region::new(0x20000000..0x20080000, regioncfg::SRAM, 1),
    // ROM section.
    Region::new(0x00000000..0x10000000, regioncfg::ROM, 1),
    // Core local peripherals and Cortex private peripherals.
    Region::new(0xD0000000..0xFFFFFF00, regioncfg::LOCAL, 0),
    // Device peripherals.
    Region::new(0x40000000..0xD0000000, regioncfg::PERIPHERALS, 0),
    // Cached XIP accesses.
    Region::new(0x10000000..0x14000000, regioncfg::SRAM, 2),
    // Uncached XIP accesses.
    Region::new(0x14000000..0x18000000, regioncfg::SRAM, 1),
];

pub struct Common {
    // /// The ADC engine of the device.
    // pub adc: ADCPeripheral,
    /// List of all GPIOs of the device.
    pub gpios: crate::gpio::GPIOList,
}

impl Common {
    /// Initializes the peripherals and returns the instance controlling them.
    pub(super) unsafe fn init() -> Self {
        Self {
            // adc: ADCPeripheral::instance(),
            gpios: GPIOList::all(),
        }
    }
}

pub struct Local {}

impl Local {
    /// Initializes the peripherals and returns the instance controlling them.
    pub(super) unsafe fn init() -> Self {
        // Generate the Cortex peripherals.
        let mut cortex = SystemControl::instance();

        // Enable the FPU, the GPIO and the Double Precision coprocessors.
        for i in [0, 4, 10, 11].into_iter() {
            cortex.coprocessor.configure(i, true, true, true);
        }

        // Configure the Memory Regions.
        let mut regions = REGIONS.clone();

        let address = (0x20080000 + (0x1000 * cpuid()))..(0x20081000 + (0x1000 * cpuid()));
        regions[0] = Region::new(address, regioncfg::SRAM, 1);

        let mut mpu = cortex
            .mpu
            .configure(&ATTRIBUTES, &regions)
            .expect("Prevalidated MPU configuration");

        crate::log::warn!("Enabling the MPU...");

        mpu.enable();

        Self {}
    }
}

/// Common trait for all `Peripheral`s.
pub(crate) trait Peripheral: Sized {
    unsafe fn instance() -> Self;
}
