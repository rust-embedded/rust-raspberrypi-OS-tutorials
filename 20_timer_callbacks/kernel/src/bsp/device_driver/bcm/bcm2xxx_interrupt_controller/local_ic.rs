// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

//! Local Interrupt Controller Driver.
//!
//! # Resources
//!
//! - <https://datasheets.raspberrypi.com/bcm2836/bcm2836-peripherals.pdf>

use super::{LocalIRQ, PendingIRQs};
use crate::{
    bsp::device_driver::common::MMIODerefWrapper,
    exception,
    memory::{Address, Virtual},
    synchronization,
    synchronization::{IRQSafeNullLock, InitStateLock},
};
use alloc::vec::Vec;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_structs,
    registers::{ReadOnly, WriteOnly},
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

register_structs! {
    #[allow(non_snake_case)]
    WORegisterBlock {
        (0x00 => _reserved1),
        (0x40 => CORE0_TIMER_INTERRUPT_CONTROL: WriteOnly<u32>),
        (0x44 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    RORegisterBlock {
        (0x00 => _reserved1),
        (0x60 => CORE0_INTERRUPT_SOURCE: ReadOnly<u32>),
        (0x64 => @END),
    }
}

/// Abstraction for the WriteOnly parts of the associated MMIO registers.
type WriteOnlyRegisters = MMIODerefWrapper<WORegisterBlock>;

/// Abstraction for the ReadOnly parts of the associated MMIO registers.
type ReadOnlyRegisters = MMIODerefWrapper<RORegisterBlock>;

type HandlerTable = Vec<Option<exception::asynchronous::IRQHandlerDescriptor<LocalIRQ>>>;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Representation of the peripheral interrupt controller.
pub struct LocalIC {
    /// Access to write registers is guarded with a lock.
    wo_registers: IRQSafeNullLock<WriteOnlyRegisters>,

    /// Register read access is unguarded.
    ro_registers: ReadOnlyRegisters,

    /// Stores registered IRQ handlers. Writable only during kernel init. RO afterwards.
    handler_table: InitStateLock<HandlerTable>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl LocalIC {
    // See datasheet.
    const PERIPH_IRQ_MASK: u32 = (1 << 8);

    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(mmio_start_addr: Address<Virtual>) -> Self {
        Self {
            wo_registers: IRQSafeNullLock::new(WriteOnlyRegisters::new(mmio_start_addr)),
            ro_registers: ReadOnlyRegisters::new(mmio_start_addr),
            handler_table: InitStateLock::new(Vec::new()),
        }
    }

    /// Called by the kernel to bring up the device.
    pub fn init(&self) {
        self.handler_table
            .write(|table| table.resize(LocalIRQ::MAX_INCLUSIVE + 1, None));
    }

    /// Query the list of pending IRQs.
    fn pending_irqs(&self) -> PendingIRQs {
        // Ignore the indicator bit for a peripheral IRQ.
        PendingIRQs::new(
            (self.ro_registers.CORE0_INTERRUPT_SOURCE.get() & !Self::PERIPH_IRQ_MASK).into(),
        )
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
use synchronization::interface::{Mutex, ReadWriteEx};

impl exception::asynchronous::interface::IRQManager for LocalIC {
    type IRQNumberType = LocalIRQ;

    fn register_handler(
        &self,
        irq_handler_descriptor: exception::asynchronous::IRQHandlerDescriptor<Self::IRQNumberType>,
    ) -> Result<(), &'static str> {
        self.handler_table.write(|table| {
            let irq_number = irq_handler_descriptor.number().get();

            if table[irq_number].is_some() {
                return Err("IRQ handler already registered");
            }

            table[irq_number] = Some(irq_handler_descriptor);

            Ok(())
        })
    }

    fn enable(&self, irq: &Self::IRQNumberType) {
        self.wo_registers.lock(|regs| {
            let enable_bit: u32 = 1 << (irq.get());

            // Writing a 1 to a bit will set the corresponding IRQ enable bit. All other IRQ enable
            // bits are unaffected. So we don't need read and OR'ing here.
            regs.CORE0_TIMER_INTERRUPT_CONTROL.set(enable_bit);
        });
    }

    fn handle_pending_irqs<'irq_context>(
        &'irq_context self,
        _ic: &exception::asynchronous::IRQContext<'irq_context>,
    ) {
        self.handler_table.read(|table| {
            for irq_number in self.pending_irqs() {
                match table[irq_number] {
                    None => panic!("No handler registered for IRQ {}", irq_number),
                    Some(descriptor) => {
                        // Call the IRQ handler. Panics on failure.
                        descriptor.handler().handle().expect("Error handling IRQ");
                    }
                }
            }
        })
    }

    fn print_handler(&self) {
        use crate::info;

        info!("      Local handler:");

        self.handler_table.read(|table| {
            for (i, opt) in table.iter().enumerate() {
                if let Some(handler) = opt {
                    info!("            {: >3}. {}", i, handler.name());
                }
            }
        });
    }
}
