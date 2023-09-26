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

#[cfg(feature = "bsp_rpi3")]
pub(in crate::bsp) mod irq_map {
    use super::bsp::device_driver::{IRQNumber, PeripheralIRQ};

    pub const PL011_UART: IRQNumber = IRQNumber::Peripheral(PeripheralIRQ::new(57));
}

#[cfg(feature = "bsp_rpi4")]
pub(in crate::bsp) mod irq_map {
    use super::bsp::device_driver::IRQNumber;

    pub const PL011_UART: IRQNumber = IRQNumber::new(153);
}
