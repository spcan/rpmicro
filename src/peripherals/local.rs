//! Set of peripherals local to each core in the system.

use crate::{
    cortex::{
        mpu::{Error, Region},
        SystemControl,
    },
    cpuid, Peripheral,
};

pub struct Local {}

impl Local {
    /// Initializes the peripherals and returns the instance controlling them.
    pub(crate) unsafe fn init() -> Self {
        crate::log::info!("Initializing Core {}...", crate::cpuid());

        // Compute the stack address.
        let stackbottom = 0x20080000 + (0x1000 * crate::cpuid());

        // Install the Vector Table and setup the interrupts and exceptions.
        unsafe {
            crate::interrupts::VectorTable::duplicate(stackbottom as *mut u32).install();
        }

        crate::interrupts::init();

        let vtor = unsafe { core::ptr::read_volatile(0xE000ED08 as *const u32) };
        crate::log::info!("[CPU{}] Current VTOR: {:#010X}", cpuid(), vtor);

        // Set the stack limit.
        let stacklim = stackbottom + 512;

        crate::log::info!("[CPU{}] Setting STACKLIM to {:#010X}", cpuid(), stacklim);

        let mut r: u32 = 0;
        unsafe {
            core::arch::asm!("mrs {}, MSP", out(reg) r, options(nomem, nostack));
        }
        crate::log::info!("[CPU{}] Current MSP: {:#010X}", cpuid(), r);
        unsafe {
            core::arch::asm!("mrs {}, PSP", out(reg) r, options(nomem, nostack));
        }
        crate::log::info!("[CPU{}] Current PSP: {:#010X}", cpuid(), r);
        unsafe {
            core::arch::asm!("mrs {}, MSPLIM", out(reg) r, options(nomem, nostack));
        }
        crate::log::info!("[CPU{}] Current MSPLIM: {:#010X}", cpuid(), r);
        unsafe {
            core::arch::asm!("mrs {}, PSPLIM", out(reg) r, options(nomem, nostack));
        }
        crate::log::info!("[CPU{}] Current PSPLIM: {:#010X}", cpuid(), r);

        // unsafe {
        //     core::arch::asm!("msr MSPLIM, {0}", in(reg) stacklim, options(nomem, nostack));
        // }

        // Generate the Cortex peripherals.
        let mut cortex = SystemControl::instance();

        // Enable the FPU, the GPIO and the Double Precision coprocessors.
        for i in [0, 4, 5, 10, 11].into_iter() {
            cortex.coprocessor.configure(i, true, true, true);
        }

        // Configure the Memory Regions.
        let mut regions = regions::REGIONS.clone();

        let range = stackbottom..(stackbottom + 0x1000);
        regions[0] = Region::new(range, regions::SRAM, 1);

        let result = cortex.mpu.configure(&attributes::ATTRIBUTES, &regions);

        match result {
            Ok(mut mpu) => {
                crate::log::warn!("Enabling the MPU...");
                mpu.disable();
            }

            Err(why) => match why {
                Error::InvalidAttribute(index, attr) => crate::log::error!(
                    "[MPU] Invalid config: Region {} has invalid attribute {} (max {})",
                    index,
                    attr,
                    attributes::ATTRIBUTES.len()
                ),
                Error::RegionOverlap(i, j) => {
                    crate::log::error!("[MPU] Invalid config: Regions {} and {} overlap", i, j)
                }
                Error::TooManyAttributes => {
                    crate::log::error!("[MPU] Invalid config: Too many attributes")
                }
                Error::TooManyRegions => {
                    crate::log::error!("[MPU] Invalid config: Too many regions")
                }
            },
        }

        Self {}
    }
}

mod attributes {
    use crate::cortex::mpu::{DeviceMemory, MemoryAttribute, NormalMemory, NormalMemoryAttributes};

    /// List of all attributes used in the RP2350.
    pub(super) const ATTRIBUTES: [MemoryAttribute; 3] = [
        // Memory attributes for peripheral memory.
        DeviceMemory::nGnRnE.generic(),
        // Memory attributes for all SRAM and ROM.
        // It also works for the non-cachable alias of external memory.
        NormalMemory::new(SRAM, SRAM).generic(),
        // Memory attributes for the external devices cachabe memory.
        NormalMemory::new(CACHE, CACHE).generic(),
    ];

    /// Memory attributes of SRAM style memory.
    pub(super) const SRAM: NormalMemoryAttributes = NormalMemoryAttributes::uncachable();

    /// Memory attributes of the external memory cached alias.
    pub(super) const CACHE: NormalMemoryAttributes =
        NormalMemoryAttributes::writethrough(true, true, true);
}

mod regions {
    use crate::cortex::mpu::{AccessPermissions, Region, RegionConfiguration, Shareability};

    /// List of all regions of memory used in the RP2350.
    pub(super) const REGIONS: [Region; 7] = [
        // Local stack, address to be modified by caller.
        Region::new(0x20080000..0x20081000, SRAM, 1),
        // Main SRAM section, SRAM 0-7.
        Region::new(0x20000000..0x20080000, SRAM, 1),
        // ROM section.
        Region::new(0x00000000..0x10000000, ROM, 1),
        // Core local peripherals and Cortex private peripherals.
        Region::new(0xD0000000..0xFFFFFF00, LOCAL, 0),
        // Device peripherals.
        Region::new(0x40000000..0xD0000000, PERIPHERALS, 0),
        // Cached XIP accesses.
        Region::new(0x10000000..0x14000000, SRAM, 2),
        // Uncached XIP accesses.
        Region::new(0x14000000..0x18000000, SRAM, 1),
    ];

    /// Region configuration for internal SRAM memory.
    pub(super) const SRAM: RegionConfiguration = RegionConfiguration::new(
        Shareability::OuterShareable,
        AccessPermissions::RWAll,
        false,
    );

    /// Region configuration for internal ROM memory.
    pub(super) const ROM: RegionConfiguration =
        RegionConfiguration::new(Shareability::NonShareable, AccessPermissions::ROAll, false);

    /// Region configuration for Core local memory.
    pub(super) const LOCAL: RegionConfiguration =
        RegionConfiguration::new(Shareability::NonShareable, AccessPermissions::RWAll, true);

    /// Region configuration for the peripheral memory.
    pub(super) const PERIPHERALS: RegionConfiguration =
        RegionConfiguration::new(Shareability::OuterShareable, AccessPermissions::RWAll, true);
}
