// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>

//! BSP asynchronous exception handling.

use crate::bsp;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Export for reuse in generic asynchronous.rs.
pub use bsp::device_driver::IRQNumber;

/// The IRQ map.
#[cfg(feature = "bsp_rpi3")]
pub mod irq_map {
    use super::bsp::device_driver::{IRQNumber, LocalIRQ, PeripheralIRQ};

    /// The non-secure physical timer IRQ number.
    pub const ARM_NS_PHYSICAL_TIMER: IRQNumber = IRQNumber::Local(LocalIRQ::new(1));

    pub(in crate::bsp) const PL011_UART: IRQNumber = IRQNumber::Peripheral(PeripheralIRQ::new(57));
}

/// The IRQ map.
#[cfg(feature = "bsp_rpi4")]
pub mod irq_map {
    use super::bsp::device_driver::IRQNumber;

    /// The non-secure physical timer IRQ number.
    pub const ARM_NS_PHYSICAL_TIMER: IRQNumber = IRQNumber::new(30);

    pub(in crate::bsp) const PL011_UART: IRQNumber = IRQNumber::new(153);
}
