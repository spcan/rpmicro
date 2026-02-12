//! RP clock tree control.

pub mod pll;
mod reference;
mod rosc;
mod xosc;

use core::sync::atomic::AtomicU32;

pub use reference::ReferenceClock;
pub use rosc::RingOscillator;
pub use xosc::CrystalOscillator;

/// Base register of the clocks register block.
pub(self) const BASE: usize = 0x40010000;

pub(self) static CLOCKS: [AtomicU32; 4] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];

pub trait ClockTrait {
    /// The index of the clock within the clock list.
    const CLOCKID: Clock;

    /// Gets the current frequency of this clock.
    fn current() -> u32 {
        CLOCKS[Self::CLOCKID as usize].load(core::sync::atomic::Ordering::Relaxed)
    }
}

#[repr(u8)]
pub enum Clock {
    Reference = 0,
    XOsc = 1,
    ROsc = 2,
    PLLSystem = 3,
    PLLUSB = 4,
    System = 5,
    USB = 6,
}

pub struct Clocks {
    /// PLL dedicated to the general system.
    /// Has a nominal maximum frequency of 150 MHz.
    pllsys: pll::PLL<0>,

    /// PLL dedicated to the USB system.
    /// Has a nominal maximum frequency of 48 MHz.
    pllusb: pll::PLL<1>,

    rosc: RingOscillator,

    reference: ReferenceClock,

    system: SystemClock,
}

impl Clocks {
    const BASE: usize = 0x40010000;

    const REFCLK: usize = Self::BASE + 0x30;

    const SYSCLK: usize = Self::BASE + 0x3C;

    const XOSC: usize = 0x40048000;

    const XOSCRANGE: usize = Self::XOSC + 0x00;

    const XOSCDELAY: usize = Self::XOSC + 0x0C;

    /// Applies the given clock tree to the device.
    /// UNSAFETY : External peripherals may glitch when running this operation.
    pub unsafe fn apply<'a>(&mut self, tree: &'a ClockTree) {
        // Enable the ROSC in basic mode, swap REFERENCE to it and set SYSTEM to REFERENCE.
        self.rosc.restore();
        self.reference.restore();
        self.system.restore();

        // After reaching a stable state, kill all other clocks.
        // Do not kill XOSC now, as it takes a while to start up.
        // Do not disable the PLLs fully, as they take a while to start up.

        // Configure the oscillators (ROSC, XOSC, LPOSC).
        // TODO : Implement the ROSC.

        // TODO : Implement the LPOSC.

        // If the XOsc is enabled in the clock tree, enable it and setup the PLLs.
        match tree.xosc.0.clone() {
            None => tree.xosc.disable(),
            _ => {
                tree.xosc.enable(true);

                if let Some(config) = tree.pllsys {
                    self.pllsys.freeze(u32::from(tree.xosc.freq()), config);
                }

                if let Some(config) = tree.pllusb {
                    self.pllusb.freeze(u32::from(tree.xosc.freq()), config);
                }
            }
        }

        // Set the reference clock and system clocks.
        self.reference.configure(tree.reference);
        self.system.configure(tree.system);
    }
}

impl crate::Peripheral for Clocks {
    unsafe fn instance() -> Self {
        Self {
            pllsys: pll::PLL::<0>::instance(),
            pllusb: pll::PLL::<1>::instance(),
            rosc: RingOscillator,
            reference: ReferenceClock,
            system: SystemClock,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ClockList {
    ROsc = 0,
    XOsc = 1,
    PLLSystem = 2,
    PLLUSB = 3,
}

impl ClockList {
    /// Returns the current frequency of the given clock.
    pub fn read(&self) -> u32 {
        CLOCKS[*self as usize].load(core::sync::atomic::Ordering::Relaxed)
    }

    /// Updates the clock's current frequency.
    pub(self) fn write(&self, val: u32) {
        CLOCKS[*self as usize].store(val, core::sync::atomic::Ordering::Relaxed)
    }
}

pub struct SystemClock;

impl SystemClock {
    pub(super) unsafe fn restore(&mut self) {
        core::ptr::write_volatile(Self::CONTROL as *mut u32, 0);
    }

    pub(super) fn configure(&mut self, source: SourceClock) {
        unsafe {
            core::ptr::write_volatile(Self::CONTROL as *mut u32, source as u32);
        }
    }
}

impl ClockControl for SystemClock {
    const BASE: usize = 0x40010000 + 0x3C;
}

/// Common trait that defines the register offsets for the common clocks.
pub(super) trait ClockControl {
    const BASE: usize;

    const CONTROL: usize = Self::BASE + 0x00;

    const DIV: usize = Self::BASE + 0x04;

    const SELECTED: usize = Self::BASE + 0x08;
}

/// Common trait for all base clocks.
/// Base clock sources are those which will not glitch or fail in case of device failure.
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum BaseClock {
    RingOscillator = 0,
    CrystalOscillator = 2,
    LowPowerOscillator = 3,
}

/// Common trait for all source clocks.
/// Source clocks can be fed to any of the main clocks of the device.
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum SourceClock {
    ReferenceClock = 0,
    PLLSystem = 1,
    PLLUSB = (0x1 << 5) | 1,
    RingOscillator = (0x2 << 5) | 1,
    CrystalOscillator = (0x3 << 5) | 1,
}

/// Definition of an external clock.
/// Must contain the nominal frequency of the clock in Hz.
pub struct ExternalClock<const N: usize>(pub u32);

/// Definition of the Low-Power Oscillator.
pub struct LPOscillator;

/// Definition of a configurable clock tree.
/// Any clock tree can be configured and swapped at runtime.
pub struct ClockTree<'a> {
    /// External crystal oscillator.
    pub xosc: &'a CrystalOscillator,

    /// System PLL configuration.
    pub pllsys: Option<&'a pll::PLLConfig>,

    /// USB PLL configuration.
    pub pllusb: Option<&'a pll::PLLConfig>,

    /// The reference clock used by the whole device.
    pub reference: BaseClock,

    /// The system clock feeds the CPU cores and general logic.
    /// The peripheral clock can also be derived from this clock.
    pub system: SourceClock,
}
