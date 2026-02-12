//! Traits for all pins that implement QSPI functions.

use super::{GPIOControl, GPIOTyped};

/// Force enable the output and select QSPI function.
const CONTROL: u32 = (0x3 << 14) | 0x09;

/// 12 mA strength, enable pull-up, enable Schmitt trigger, fast slew rate.
const PADS: u32 = 0x3B;

/// Represents an anonimous QSPI CS pin.
pub struct QMIChipSelect;

unsafe impl Send for QMIChipSelect {}
unsafe impl Sync for QMIChipSelect {}

impl Into<QMIChipSelect> for GPIOTyped<0> {
    fn into(mut self) -> QMIChipSelect {
        self.control().write(CONTROL);
        self.pad().write(PADS);
        QMIChipSelect
    }
}

impl Into<QMIChipSelect> for GPIOTyped<8> {
    fn into(mut self) -> QMIChipSelect {
        self.control().write(CONTROL);
        self.pad().write(PADS);
        QMIChipSelect
    }
}

impl Into<QMIChipSelect> for GPIOTyped<19> {
    fn into(mut self) -> QMIChipSelect {
        self.control().write(CONTROL);
        self.pad().write(PADS);
        QMIChipSelect
    }
}

impl Into<QMIChipSelect> for GPIOTyped<47> {
    fn into(mut self) -> QMIChipSelect {
        self.control().write(CONTROL);
        self.pad().write(PADS);
        QMIChipSelect
    }
}
