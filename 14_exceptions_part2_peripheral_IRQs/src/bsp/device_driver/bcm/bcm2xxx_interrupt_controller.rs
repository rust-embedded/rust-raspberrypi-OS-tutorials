// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

//! Interrupt Controller Driver.

mod peripheral_ic;

use crate::{driver, exception};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// Wrapper struct for a bitmask indicating pending IRQ numbers.
struct PendingIRQs {
    bitmask: u64,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub type LocalIRQ =
    exception::asynchronous::IRQNumber<{ InterruptController::MAX_LOCAL_IRQ_NUMBER }>;
pub type PeripheralIRQ =
    exception::asynchronous::IRQNumber<{ InterruptController::MAX_PERIPHERAL_IRQ_NUMBER }>;

/// Used for the associated type of trait  [`exception::asynchronous::interface::IRQManager`].
#[derive(Copy, Clone)]
pub enum IRQNumber {
    Local(LocalIRQ),
    Peripheral(PeripheralIRQ),
}

/// Representation of the Interrupt Controller.
pub struct InterruptController {
    periph: peripheral_ic::PeripheralIC,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl PendingIRQs {
    pub fn new(bitmask: u64) -> Self {
        Self { bitmask }
    }
}

impl Iterator for PendingIRQs {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        use core::intrinsics::cttz;

        let next = cttz(self.bitmask);
        if next == 64 {
            return None;
        }

        self.bitmask &= !(1 << next);

        Some(next as usize)
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl InterruptController {
    const MAX_LOCAL_IRQ_NUMBER: usize = 11;
    const MAX_PERIPHERAL_IRQ_NUMBER: usize = 63;
    const NUM_PERIPHERAL_IRQS: usize = Self::MAX_PERIPHERAL_IRQ_NUMBER + 1;

    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide the correct `base_addr`.
    pub const unsafe fn new(_local_base_addr: usize, periph_base_addr: usize) -> Self {
        Self {
            periph: peripheral_ic::PeripheralIC::new(periph_base_addr),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

impl driver::interface::DeviceDriver for InterruptController {
    fn compatible(&self) -> &str {
        "BCM Interrupt Controller"
    }
}

impl exception::asynchronous::interface::IRQManager for InterruptController {
    type IRQNumberType = IRQNumber;

    fn register_handler(
        &self,
        irq: Self::IRQNumberType,
        descriptor: exception::asynchronous::IRQDescriptor,
    ) -> Result<(), &'static str> {
        match irq {
            IRQNumber::Local(_) => unimplemented!("Local IRQ controller not implemented."),
            IRQNumber::Peripheral(pirq) => self.periph.register_handler(pirq, descriptor),
        }
    }

    fn enable(&self, irq: Self::IRQNumberType) {
        match irq {
            IRQNumber::Local(_) => unimplemented!("Local IRQ controller not implemented."),
            IRQNumber::Peripheral(pirq) => self.periph.enable(pirq),
        }
    }

    fn handle_pending_irqs<'irq_context>(
        &'irq_context self,
        ic: &exception::asynchronous::IRQContext<'irq_context>,
    ) {
        // It can only be a peripheral IRQ pending because enable() does not support local IRQs yet.
        self.periph.handle_pending_irqs(ic)
    }

    fn print_handler(&self) {
        self.periph.print_handler();
    }
}
