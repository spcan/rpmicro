//! Configuration and data related to the ADC operations.

#[cfg(any(feature = "QFN60", feature = "QFN80"))]
use crate::hal::gpio::GPIOPin;

/// The types of operations that the ADC can perform.
#[cfg_attr(feature = "log", derive(Debug))]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Operation {
    /// Performs a single conversion on a given analog `Channel`.
    Oneshot(Channel),

    /// Performs continuous conversions (up to a limit) on the given `ChannelList`.
    Continuous(ChannelList),
}

#[cfg(feature = "defmt")]
#[rustfmt::skip]
impl defmt::Format for Operation {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Operation::Oneshot(channel) => defmt::write!(fmt, "Oneshot({})", channel),
            Operation::Continuous(channels) => defmt::write!(fmt, "Continuous({})", channels.bits()),
        }
    }
}

/// Selects the bit precision of the ADC conversion.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Precision {
    /// Performs a 11-bit reading.
    Full,

    /// Performs a compressed 8-bit reading.
    Compressed,
}

/// All channels available for analog measurements.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Channel {
    /// Measures from an external voltage channel.
    External(ExternalChannel),

    /// Measures from the internal temperature.
    Temperature,
}

impl Into<ChannelList> for Channel {
    fn into(self) -> ChannelList {
        match self {
            Channel::External(external) => external.into(),
            Channel::Temperature => ChannelList::Temperature,
        }
    }
}

impl Into<u32> for Channel {
    fn into(self) -> u32 {
        match self {
            Channel::External(external) => external.into(),

            #[cfg(not(feature = "QFN80"))]
            Channel::Temperature => 4,

            #[cfg(feature = "QFN80")]
            Channel::Temperature => 8,
        }
    }
}

/// All channels available for analog measurements.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ExternalChannel {
    Ch0 = 0,
    Ch1 = 1,
    Ch2 = 2,
    Ch3 = 3,
    #[cfg(feature = "QFN80")]
    Ch4 = 4,
    #[cfg(feature = "QFN80")]
    Ch5 = 5,
    #[cfg(feature = "QFN80")]
    Ch6 = 6,
    #[cfg(feature = "QFN80")]
    Ch7 = 7,
}

impl Into<ChannelList> for ExternalChannel {
    fn into(self) -> ChannelList {
        match self {
            ExternalChannel::Ch0 => ChannelList::Ch0,
            ExternalChannel::Ch1 => ChannelList::Ch1,
            ExternalChannel::Ch2 => ChannelList::Ch2,
            ExternalChannel::Ch3 => ChannelList::Ch3,
            #[cfg(feature = "QFN80")]
            ExternalChannel::Ch4 => ChannelList::Ch4,
            #[cfg(feature = "QFN80")]
            ExternalChannel::Ch5 => ChannelList::Ch5,
            #[cfg(feature = "QFN80")]
            ExternalChannel::Ch6 => ChannelList::Ch6,
            #[cfg(feature = "QFN80")]
            ExternalChannel::Ch7 => ChannelList::Ch7,
        }
    }
}

impl Into<u32> for ExternalChannel {
    fn into(self) -> u32 {
        match self {
            ExternalChannel::Ch0 => 0,
            ExternalChannel::Ch1 => 1,
            ExternalChannel::Ch2 => 2,
            ExternalChannel::Ch3 => 3,
            #[cfg(feature = "QFN80")]
            ExternalChannel::Ch4 => 4,
            #[cfg(feature = "QFN80")]
            ExternalChannel::Ch5 => 5,
            #[cfg(feature = "QFN80")]
            ExternalChannel::Ch6 => 6,
            #[cfg(feature = "QFN80")]
            ExternalChannel::Ch7 => 7,
        }
    }
}

bitflags::bitflags! {
    /// Bit collection of enabled channels for a continuous conversion.
    #[cfg_attr(feature = "log", derive(Debug))]
    #[derive(Clone, Copy, Eq, PartialEq)]
    pub struct ChannelList: u16 {
        const Ch0 = 1 << 0;
        const Ch1 = 1 << 1;
        const Ch2 = 1 << 2;
        const Ch3 = 1 << 3;
        #[cfg(feature = "QFN80")]
        const Ch4 = 1 << 4;
        #[cfg(feature = "QFN80")]
        const Ch5 = 1 << 5;
        #[cfg(feature = "QFN80")]
        const Ch6 = 1 << 6;
        #[cfg(feature = "QFN80")]
        const Ch7 = 1 << 7;

        #[cfg(not(feature = "QFN80"))]
        const Temperature = 1 << 4;
        #[cfg(feature = "QFN80")]
        const Temperature = 1 << 8;
    }
}

impl<T: IntoIterator<Item = Channel>> From<T> for ChannelList {
    fn from(value: T) -> ChannelList {
        value
            .into_iter()
            .fold(ChannelList::empty(), |list, channel| list | channel.into())
    }
}

#[cfg(feature = "QFN60")]
impl Into<ExternalChannel> for GPIOPin<26> {
    fn into(self) -> ExternalChannel {
        self.control().write(0x1Fu32);
        self.pad().write(1u32 << 7);
        ExternalChannel::Ch0
    }
}

#[cfg(feature = "QFN60")]
impl Analog for GPIOPin<27> {
    fn into(self) -> ExternalChannel {
        self.control().write(0x1Fu32);
        self.pad().write(1u32 << 7);
        ExternalChannel::Ch1
    }
}

#[cfg(feature = "QFN60")]
impl Analog for GPIOPin<28> {
    fn into(self) -> ExternalChannel {
        self.control().write(0x1Fu32);
        self.pad().write(1u32 << 7);
        ExternalChannel::Ch2
    }
}

#[cfg(feature = "QFN60")]
impl Analog for GPIOPin<29> {
    fn into(self) -> ExternalChannel {
        self.control().write(0x1Fu32);
        self.pad().write(1u32 << 7);
        ExternalChannel::Ch3
    }
}

#[cfg(feature = "QFN80")]
impl Analog for GPIOPin<40> {
    fn into(self) -> ExternalChannel {
        self.control().write(0x1Fu32);
        self.pad().write(1u32 << 7);
        ExternalChannel::Ch0
    }
}

#[cfg(feature = "QFN80")]
impl Analog for GPIOPin<41> {
    fn into(self) -> ExternalChannel {
        self.control().write(0x1Fu32);
        self.pad().write(1u32 << 7);
        ExternalChannel::Ch1
    }
}

#[cfg(feature = "QFN80")]
impl Analog for GPIOPin<42> {
    fn into(self) -> ExternalChannel {
        self.control().write(0x1Fu32);
        self.pad().write(1u32 << 7);
        ExternalChannel::Ch2
    }
}

#[cfg(feature = "QFN80")]
impl Analog for GPIOPin<43> {
    fn into(self) -> ExternalChannel {
        self.control().write(0x1Fu32);
        self.pad().write(1u32 << 7);
        ExternalChannel::Ch3
    }
}

#[cfg(feature = "QFN80")]
impl Analog for GPIOPin<44> {
    fn into(self) -> ExternalChannel {
        self.control().write(0x1Fu32);
        self.pad().write(1u32 << 7);
        ExternalChannel::Ch4
    }
}

#[cfg(feature = "QFN80")]
impl Analog for GPIOPin<45> {
    fn into(self) -> ExternalChannel {
        self.control().write(0x1Fu32);
        self.pad().write(1u32 << 7);
        ExternalChannel::Ch5
    }
}

#[cfg(feature = "QFN80")]
impl Analog for GPIOPin<46> {
    fn into(self) -> ExternalChannel {
        self.control().write(0x1Fu32);
        self.pad().write(1u32 << 7);
        ExternalChannel::Ch6
    }
}

#[cfg(feature = "QFN80")]
impl Analog for GPIOPin<47> {
    fn into(self) -> ExternalChannel {
        self.control().write(0x1Fu32);
        self.pad().write(1u32 << 7);
        ExternalChannel::Ch7
    }
}
