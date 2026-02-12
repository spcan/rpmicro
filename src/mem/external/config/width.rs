//! Indicates the interface width of a (Quad / Dual) SPI transfer.

#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(Debug))]
pub enum DataWidth {
    /// Transfer performed in SPI mode.
    Single = 0,

    /// Transfer performed in Dual SPI mode.
    Dual = 1,

    /// Transfer performed in Quad SPI mode.
    Quad = 2,
}
