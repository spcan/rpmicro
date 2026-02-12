//! Performance monitoring for the device's buses.

///Contains all the performance counters for the buses of the device.
pub struct GlobalPerformance {
    pub a: BusPerformance<0>,
    pub b: BusPerformance<1>,
    pub c: BusPerformance<2>,
    pub d: BusPerformance<3>,
}

impl GlobalPerformance {
    /// Base address of the bus control peripehral.
    const BASE: usize = 0x40068000;

    /// Base offset of the enable registers.
    const ENABLE: usize = Self::BASE + 0x08;

    pub unsafe fn instance() -> Self {
        Self {
            a: BusPerformance,
            b: BusPerformance,
            c: BusPerformance,
            d: BusPerformance,
        }
    }

    /// Starts all performance counters.
    pub fn start(&mut self) {
        unsafe {
            core::ptr::write_volatile(Self::ENABLE as *mut u32, 1);
        }
    }

    /// Stops all performance counters.
    pub fn stop(&mut self) {
        unsafe {
            core::ptr::write_volatile(Self::ENABLE as *mut u32, 0);
        }
    }

    /// Resets all performance counters.
    pub fn reset(&mut self) {
        self.a.reset();
        self.b.reset();
        self.c.reset();
        self.d.reset();
    }
}

impl crate::Peripheral for GlobalPerformance {
    unsafe fn instance() -> Self {
        Self {
            a: BusPerformance,
            b: BusPerformance,
            c: BusPerformance,
            d: BusPerformance,
        }
    }
}

/// Performance counters for the bus.
pub struct BusPerformance<const N: usize>;

impl<const N: usize> BusPerformance<N> {
    /// Base address of the bus control peripehral.
    const BASE: usize = 0x40068000;

    /// Base offset of the counter registers.
    const COUNTER: usize = Self::BASE + 0x0C + (N * 8);

    /// Base offset of the select registers.
    const SELECT: usize = Self::BASE + 0x10 + (N * 8);

    /// Resets the counter to 0.
    pub fn reset(&mut self) {
        unsafe {
            core::ptr::write_volatile(Self::COUNTER as *mut u8, 1);
        }
    }

    /// Reads the current value of the performance counter.
    pub fn read(&self) -> u32 {
        unsafe { core::ptr::read_volatile(Self::COUNTER as *const u32) }
    }

    /// Selects the BusEvent tracked by this `BusPerformance`.
    pub fn select(&mut self, event: BusEvent) {
        unsafe {
            core::ptr::write_volatile(Self::SELECT as *mut u8, event.into());
        }
    }

    /// Returns the BusEvent this performance counter is tracking.
    pub fn tracking(&self) -> BusEvent {
        BusEvent::from(unsafe { core::ptr::read_volatile(Self::SELECT as *const u32) } as u8)
    }
}

/// All buses that can be monitored with their selected BusEvent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BusEvent {
    /// Processor 1 SIO access.
    SIOProc1(EventKind),

    /// Processor 0 SIO access.
    SIOProc0(EventKind),

    /// Downstream bus containing the slow peripherals.
    APB(EventKind),

    /// Bus containing all fast peripherals.
    FastPeripheral(EventKind),

    /// Bus accesses to the SRAM 9.
    SRAM9(EventKind),

    /// Bus accesses to the SRAM 8.
    SRAM8(EventKind),

    /// Bus accesses to the SRAM 7.
    SRAM7(EventKind),

    /// Bus accesses to the SRAM 6.
    SRAM6(EventKind),

    /// Bus accesses to the SRAM 5.
    SRAM5(EventKind),

    /// Bus accesses to the SRAM 4.
    SRAM4(EventKind),

    /// Bus accesses to the SRAM 3.
    SRAM3(EventKind),

    /// Bus accesses to the SRAM 2.
    SRAM2(EventKind),

    /// Bus accesses to the SRAM 1.
    SRAM1(EventKind),

    /// Bus accesses to the SRAM 0.
    SRAM0(EventKind),

    /// Bus accesses to the XIP chip 0.
    XIP0(EventKind),

    /// Bus accesses to the XIP chip 1.
    XIP1(EventKind),

    /// Bus accesses to the ROM.
    ROM(EventKind),

    Unknown,
}

impl From<u8> for BusEvent {
    fn from(value: u8) -> BusEvent {
        let kind = EventKind::from(value % 4);

        match value / 4 {
            00 => BusEvent::SIOProc1(kind),
            01 => BusEvent::SIOProc0(kind),
            02 => BusEvent::APB(kind),
            03 => BusEvent::FastPeripheral(kind),
            04 => BusEvent::SRAM9(kind),
            05 => BusEvent::SRAM8(kind),
            06 => BusEvent::SRAM7(kind),
            07 => BusEvent::SRAM6(kind),
            08 => BusEvent::SRAM5(kind),
            09 => BusEvent::SRAM4(kind),
            10 => BusEvent::SRAM3(kind),
            11 => BusEvent::SRAM2(kind),
            12 => BusEvent::SRAM1(kind),
            13 => BusEvent::SRAM0(kind),
            14 => BusEvent::XIP1(kind),
            15 => BusEvent::XIP0(kind),
            16 => BusEvent::ROM(kind),

            _ => BusEvent::Unknown,
        }
    }
}

impl Into<u8> for BusEvent {
    fn into(self) -> u8 {
        match self {
            BusEvent::SIOProc1(kind) => 0x00 + (kind as u8),
            BusEvent::SIOProc0(kind) => 0x04 + (kind as u8),
            BusEvent::APB(kind) => 0x08 + (kind as u8),
            BusEvent::FastPeripheral(kind) => 0x0C + (kind as u8),
            BusEvent::SRAM9(kind) => 0x10 + (kind as u8),
            BusEvent::SRAM8(kind) => 0x14 + (kind as u8),
            BusEvent::SRAM7(kind) => 0x18 + (kind as u8),
            BusEvent::SRAM6(kind) => 0x1C + (kind as u8),
            BusEvent::SRAM5(kind) => 0x20 + (kind as u8),
            BusEvent::SRAM4(kind) => 0x24 + (kind as u8),
            BusEvent::SRAM3(kind) => 0x28 + (kind as u8),
            BusEvent::SRAM2(kind) => 0x2C + (kind as u8),
            BusEvent::SRAM1(kind) => 0x30 + (kind as u8),
            BusEvent::SRAM0(kind) => 0x34 + (kind as u8),
            BusEvent::XIP1(kind) => 0x38 + (kind as u8),
            BusEvent::XIP0(kind) => 0x3C + (kind as u8),
            BusEvent::ROM(kind) => 0x40 + (kind as u8),

            BusEvent::Unknown => 0,
        }
    }
}

/// All kinds of events that can be measured for all buses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    /// Cycles stalled due to any reason, including master contention.
    StallUpstream = 0,

    /// Cycles stalled due to the slave stalling.
    StallDownstream = 1,

    /// All accesses that previously were stalled by another bus accessing the same slave.
    AccessContested = 2,

    /// All accesses to the bus.
    Access = 3,
}

impl From<u8> for EventKind {
    fn from(value: u8) -> Self {
        match value {
            0 => EventKind::StallUpstream,
            1 => EventKind::StallDownstream,
            2 => EventKind::AccessContested,
            _ => EventKind::Access,
        }
    }
}
