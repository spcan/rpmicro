//! Bus behaviour control and bus performance counters.

use crate::AtomicRegister;

/// Controller of the bus behaviour.
pub struct BusControl;

impl BusControl {
    /// Base address of the bus control peripehral.
    const BASE: usize = 0x40068000;

    /// Sets the priority of the given bus.
    /// <div class="warning">
    /// Changing the priority values of the buses may cause performance degradation.
    /// </div>
    pub fn prioritize(&mut self, bus: MasterBus, prio: BusPriority) {
        let register = AtomicRegister::at(Self::BASE);

        match prio {
            BusPriority::High => register.set(1u32 << (bus as u8)),
            BusPriority::Low => register.clear(1u32 << (bus as u8)),
        }
    }
}

impl crate::Peripheral for BusControl {
    unsafe fn instance() -> Self {
        Self
    }
}

/// All master buses of the device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MasterBus {
    /// CPU 0 all bus accesses.
    CPU0 = 0,

    /// CPU 1 all bus accesses.
    CPU1 = 4,

    /// DMA read bus accesses.
    DMARead = 8,

    /// DMA write bus accesses.
    DMAWrite = 12,
}

/// Possible values for bus priority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BusPriority {
    /// High bus priority.
    High,

    /// Low bus priority.
    /// The bus master will stall on accesses  contested with high priority masters.
    Low,
}
