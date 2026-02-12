const TYPE: usize = 0xE000ED90;

use core::{
    ops::Range,
    ptr::{read_volatile, write_volatile},
};

use crate::Peripheral;

/// Controls a valid and possibly active Memory Protection Unit.
/// Enabes adding additional regions or attributes but no reconfiguration.
pub struct ValidMemoryProtection {
    attributes: usize,

    regions: usize,
}

impl ValidMemoryProtection {
    /// The MPU Control Register.
    const CTRL: usize = 0xE000ED94;

    pub fn new(attributes: usize, regions: usize) -> Self {
        Self {
            attributes,
            regions,
        }
    }

    /// Enables the Memory Protection Unit.
    /// May cause memory faults with an incorrect configuration.
    pub unsafe fn enable(&mut self) {
        write_volatile(Self::CTRL as *mut u32, 1);
    }

    /// Enables the Memory Protection Unit.
    pub unsafe fn disable(&mut self) {
        write_volatile(Self::CTRL as *mut u32, 0);
    }
}

/// Controls the Memory Protection Unit. This abstraction will always represent an inactive MPU.
pub struct MemoryProtection;

impl MemoryProtection {
    /// MPU Type Register.
    const TYPE: usize = 0xE000ED90;

    /// Region Number Register.
    const RNR: usize = 0xE000ED98;

    /// Region Base Address Register.
    const RBAR: usize = 0xE000ED9C;

    /// Region Limit Address Register.
    const RLAR: usize = 0xE000EDA0;

    /// Memory Attributes Indirection Registers.
    const MAIR: [usize; 2] = [0xE000EDC0, 0xE000EDC4];

    /// Configures the MPU with the given attributes and regions.
    /// Will return a `ValidMemoryProtection` or an error.
    pub fn configure(
        mut self,
        attrs: &[MemoryAttribute],
        regions: &[Region],
    ) -> Result<ValidMemoryProtection, Error> {
        // Validate that there are not too many regions or attributes.
        let typereg = unsafe { read_volatile(Self::TYPE as *const u32) };

        let maxregions = ((typereg >> 8) & 0xFF) as usize;

        if regions.len() > maxregions {
            return Err(Error::TooManyRegions);
        }

        if attrs.len() > 8 {
            return Err(Error::TooManyAttributes);
        }

        // Validate that all regions point to a valid attribute.
        for (index, region) in regions.iter().enumerate() {
            let attr = region.attr();

            if attr >= attrs.len() {
                return Err(Error::InvalidAttribute(index, attr));
            }
        }

        // Validate that all regions do not overlap.
        for i in 0..(regions.len() - 1) {
            for j in (i + 1)..regions.len() {
                if regions[i].overlaps(&regions[j]) {
                    return Err(Error::RegionOverlap(i, j));
                }
            }
        }

        // Configure the attributes, then the memory regions.
        self.attributes(attrs);
        self.regions(regions);

        // Return the `ValidMemoryProtection`.
        Ok(ValidMemoryProtection::new(attrs.len(), regions.len()))
    }

    /// Configures the `MemoryAttribute`s of the MPU.
    fn attributes(&mut self, attrs: &[MemoryAttribute]) {
        // Clear both MAIR.
        unsafe {
            write_volatile(Self::MAIR[0] as *mut u32, 0);
            write_volatile(Self::MAIR[1] as *mut u32, 0);
        }

        for (register, list) in attrs.chunks(4).enumerate() {
            // Build the MAIR with the 4 attributes.
            let mut mair = 0;

            for (index, attr) in list.iter().enumerate() {
                mair |= (attr.raw() as u32) << (index * 8);
            }

            unsafe {
                write_volatile(Self::MAIR[register] as *mut u32, mair);
            }
        }
    }

    /// Configures the `Region`s of the MPU.
    fn regions(&mut self, regions: &[Region]) {
        // Disable all regions.
        let typereg = unsafe { read_volatile(Self::TYPE as *const u32) };

        let maxregions = (typereg >> 8) & 0xFF;

        for region in 0..maxregions {
            unsafe {
                write_volatile(Self::RNR as *mut u32, region);
                write_volatile(Self::RBAR as *mut u32, 0);
                write_volatile(Self::RLAR as *mut u32, 0);
            }
        }

        // Configure each region.
        for (index, region) in regions.iter().enumerate() {
            unsafe {
                write_volatile(Self::RNR as *mut u32, index as u32);
                write_volatile(Self::RBAR as *mut u32, region.rbar);
                write_volatile(Self::RLAR as *mut u32, region.rlar);
            }
        }
    }
}

impl Peripheral for MemoryProtection {
    unsafe fn instance() -> Self {
        Self
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Region {
    /// The Region Base Address, Contains the base address, the sahreability definition, the access permissions and the execute enable bit.
    pub(self) rbar: u32,

    /// The Region Limit Address. Contains the maximum address, the attribute index and the enable bit.
    pub(self) rlar: u32,
}

impl Region {
    /// Creates a new `Region` for the given address range (must be 32 byte aligned)
    /// with the given configuration and the indirectly targeted memory attributes.
    /// The validity of the indiret attributes will be checked at runtime.
    pub const fn new(address: Range<u32>, config: RegionConfiguration, attr: u32) -> Self {
        // Ensure the addresses are 32 byte aligned.
        if (address.start & 0x1F) != 0 {
            panic!("Start address of a Memory Region must be 32 byte aligned")
        }

        if (address.end & 0x1F) != 0 {
            panic!("End address of a Memory Region must be 32 byte aligned")
        }

        // Create the RBAR and RLAR.
        let rbar = address.start | config.0;
        let rlar = ((address.end - 1) & !0x1F) | ((attr & 0b111) << 1) | 1;

        Self { rbar, rlar }
    }

    /// Returns the attribute index of this region.
    pub const fn attr(&self) -> usize {
        ((self.rlar >> 1) & 0b111) as usize
    }

    /// Returns `true` if the given region overlaps with this region.
    /// Region overlap is disallowed in ARMV8-M.
    pub fn overlaps(&self, rhs: &Self) -> bool {
        let lhs = (self.rbar & !0x1F)..(self.rlar & !0x1F);
        let rhs = (rhs.rbar & !0x1F)..(rhs.rlar & !0x1F);

        (lhs.start <= rhs.end) && (rhs.start <= lhs.end)
    }
}

pub struct RegionConfiguration(u32);

impl RegionConfiguration {
    /// Creates a new `RegionConfiguration`.
    /// Takes the `Shareability`, `AccessPermissions` and a flag indicating if this region is not executable (eXecute Never).
    pub const fn new(share: Shareability, access: AccessPermissions, xn: bool) -> Self {
        Self(((share as u32) << 3) | ((access as u32) << 1) | (xn as u32))
    }
}

/// Indicates if the memory region is shared between different bus masters.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub enum Shareability {
    /// Private memory.
    NonShareable = 0b00,

    /// Shareable with all bus masters.
    #[default]
    OuterShareable = 0b10,

    /// Shareable with internal bus masters.
    InnerShareable = 0b11,
}

/// Indicates access permissions of the region.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub enum AccessPermissions {
    /// Read-Write region accessible only by privileged code.
    RWPrivileged = 0b00,

    /// Read-Write region accessible by all modes.
    #[default]
    RWAll = 0b01,

    /// Read-Only region accessible only by privileged code.
    ROPrivileged = 0b10,

    /// Read-Only region accessible by all modes.
    ROAll = 0b11,
}

/// Defines the memory attributes of a region.
#[derive(Clone, Copy)]
pub union MemoryAttribute {
    /// Defines a Device Memory region attribute.
    device: DeviceMemory,

    /// Defines a Normal Memory region attribute.
    normal: NormalMemory,
}

impl MemoryAttribute {
    /// Returns the raw `u8` value of this `MemoryAttribute`.
    pub fn raw(&self) -> u8 {
        unsafe { self.normal.0 }
    }
}

/// Types of Device Memory allowed by the ARMV8-M MPU.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum DeviceMemory {
    /// No Gather, no Reorder, no Early ACK.
    /// Most restrictive Device Memory type.
    /// Default and recommended for peripheral memory.
    #[default]
    nGnRnE = 0b0000,

    /// No Gather, no Reorder, Early ACK allowed.
    nGnRE = 0b0100,

    /// No Gather, Reorder allowed, Early ACK allowed.
    nGRE = 0b1000,

    /// Gather allowed, Reorder allowed, Early ACK allowed.
    /// Least restrictive Device Memory type.
    GRE = 0b1100,
}

impl DeviceMemory {
    pub const fn generic(self) -> MemoryAttribute {
        MemoryAttribute { device: self }
    }
}

/// Defines the Read and Write attributes for a Normal Memory region.
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(transparent)]
pub struct NormalMemory(u8);

impl NormalMemory {
    /// Creates a new Normal Memory region attributes with inner and outer definitions.
    pub const fn new(outer: NormalMemoryAttributes, inner: NormalMemoryAttributes) -> Self {
        Self((outer.0 << 4) | inner.0)
    }

    pub const fn generic(self) -> MemoryAttribute {
        MemoryAttribute { normal: self }
    }
}

impl Default for NormalMemory {
    fn default() -> Self {
        Self(0b01000100)
    }
}

/// Defines the Read or Write attributes for a Normal Memory region.
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(transparent)]
pub struct NormalMemoryAttributes(u8);

impl NormalMemoryAttributes {
    /// Creates a Normal Memory definition for uncachable memory.
    pub const fn uncachable() -> Self {
        Self(0b0100)
    }

    /// Creates a Normal Memory definition for a Write Back cache.
    /// Write Back means that all writes to the cache will first be sent to the
    /// memory and then be read back into the cache.
    /// `read` and `write` indicate if the corresponding memory access will
    /// cause allocations in the cache.
    pub const fn writeback(transient: bool, read: bool, write: bool) -> Self {
        // Marks the memory as write back.
        const WRITEBACK: u8 = 0b0100;

        let mut mem = Self(((read as u8) << 1) | (write as u8) | WRITEBACK);

        if !transient {
            mem.0 |= 1 << 3;
        }

        mem
    }

    /// Creates a Normal Memory definition for a Write Through cache.
    /// Write Through means that all writes to the cache will first register in
    /// the cache then be sent to the memory.
    /// `read` and `write` indicate if the corresponding memory access will
    /// cause allocations in the cache.
    pub const fn writethrough(transient: bool, read: bool, write: bool) -> Self {
        let mut mem = Self(((read as u8) << 1) | (write as u8));

        if !transient {
            mem.0 |= 1 << 3;
        }

        mem
    }
}

impl Default for NormalMemoryAttributes {
    fn default() -> Self {
        Self::uncachable()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Attempted to configure more attributes than allowed by the MPU.
    TooManyAttributes,

    /// Attempted to configure more regions than allowed by the MPU.
    TooManyRegions,

    /// The given memory region points to an invalid or unset Attribute.
    /// Returns the index of the region and the invalid attribute index.
    InvalidAttribute(usize, usize),

    /// The two given regions overlap.
    RegionOverlap(usize, usize),
}
