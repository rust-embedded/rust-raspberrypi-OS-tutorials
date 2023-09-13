// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

//! Null IRQ Manager.

use super::{interface, IRQContext, IRQHandlerDescriptor};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub struct NullIRQManager;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

pub static NULL_IRQ_MANAGER: NullIRQManager = NullIRQManager {};

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl interface::IRQManager for NullIRQManager {
    type IRQNumberType = super::IRQNumber;

    fn register_handler(
        &self,
        _descriptor: IRQHandlerDescriptor<Self::IRQNumberType>,
    ) -> Result<(), &'static str> {
        panic!("No IRQ Manager registered yet");
    }

    fn enable(&self, _irq_number: &Self::IRQNumberType) {
        panic!("No IRQ Manager registered yet");
    }

    fn handle_pending_irqs<'irq_context>(&'irq_context self, _ic: &IRQContext<'irq_context>) {
        panic!("No IRQ Manager registered yet");
    }
}
