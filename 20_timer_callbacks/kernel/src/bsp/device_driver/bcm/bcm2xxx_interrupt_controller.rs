// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>

//! Interrupt Controller Driver.

mod local_ic;
mod peripheral_ic;

use crate::{
    bsp::device_driver::common::BoundedUsize,
    driver,
    exception::{self, asynchronous::IRQHandlerDescriptor},
    memory::{Address, Virtual},
};
use core::fmt;

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

pub type LocalIRQ = BoundedUsize<{ InterruptController::MAX_LOCAL_IRQ_NUMBER }>;
pub type PeripheralIRQ = BoundedUsize<{ InterruptController::MAX_PERIPHERAL_IRQ_NUMBER }>;

/// Used for the associated type of trait [`exception::asynchronous::interface::IRQManager`].
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum IRQNumber {
    Local(LocalIRQ),
    Peripheral(PeripheralIRQ),
}

/// Representation of the Interrupt Controller.
pub struct InterruptController {
    local: local_ic::LocalIC,
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
        if self.bitmask == 0 {
            return None;
        }

        let next = self.bitmask.trailing_zeros() as usize;
        self.bitmask &= self.bitmask.wrapping_sub(1);
        Some(next)
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl fmt::Display for IRQNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Local(number) => write!(f, "Local({})", number),
            Self::Peripheral(number) => write!(f, "Peripheral({})", number),
        }
    }
}

impl InterruptController {
    // Restrict to 3 for now. This makes the code for local_ic.rs more straight forward.
    const MAX_LOCAL_IRQ_NUMBER: usize = 3;
    const MAX_PERIPHERAL_IRQ_NUMBER: usize = 63;

    pub const COMPATIBLE: &'static str = "BCM Interrupt Controller";

    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(
        local_mmio_start_addr: Address<Virtual>,
        periph_mmio_start_addr: Address<Virtual>,
    ) -> Self {
        Self {
            local: local_ic::LocalIC::new(local_mmio_start_addr),
            periph: peripheral_ic::PeripheralIC::new(periph_mmio_start_addr),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

impl driver::interface::DeviceDriver for InterruptController {
    type IRQNumberType = IRQNumber;

    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        self.local.init();
        self.periph.init();

        Ok(())
    }
}

impl exception::asynchronous::interface::IRQManager for InterruptController {
    type IRQNumberType = IRQNumber;

    fn register_handler(
        &self,
        irq_handler_descriptor: exception::asynchronous::IRQHandlerDescriptor<Self::IRQNumberType>,
    ) -> Result<(), &'static str> {
        match irq_handler_descriptor.number() {
            IRQNumber::Local(lirq) => {
                let local_descriptor = IRQHandlerDescriptor::new(
                    lirq,
                    irq_handler_descriptor.name(),
                    irq_handler_descriptor.handler(),
                );

                self.local.register_handler(local_descriptor)
            }
            IRQNumber::Peripheral(pirq) => {
                let periph_descriptor = IRQHandlerDescriptor::new(
                    pirq,
                    irq_handler_descriptor.name(),
                    irq_handler_descriptor.handler(),
                );

                self.periph.register_handler(periph_descriptor)
            }
        }
    }

    fn enable(&self, irq: &Self::IRQNumberType) {
        match irq {
            IRQNumber::Local(lirq) => self.local.enable(lirq),
            IRQNumber::Peripheral(pirq) => self.periph.enable(pirq),
        }
    }

    fn handle_pending_irqs<'irq_context>(
        &'irq_context self,
        ic: &exception::asynchronous::IRQContext<'irq_context>,
    ) {
        self.local.handle_pending_irqs(ic);
        self.periph.handle_pending_irqs(ic)
    }

    fn print_handler(&self) {
        self.local.print_handler();
        self.periph.print_handler();
    }
}
