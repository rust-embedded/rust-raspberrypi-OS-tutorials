// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

//! Peripheral Interrupt Controller Driver.

use super::{InterruptController, PendingIRQs, PeripheralIRQ};
use crate::{
    bsp::device_driver::common::MMIODerefWrapper,
    driver, exception, memory,
    memory::mmu::Physical,
    synchronization,
    synchronization::{IRQSafeNullLock, InitStateLock},
};
use register::{mmio::*, register_structs};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

register_structs! {
    #[allow(non_snake_case)]
    WORegisterBlock {
        (0x00 => _reserved1),
        (0x10 => ENABLE_1: WriteOnly<u32>),
        (0x14 => ENABLE_2: WriteOnly<u32>),
        (0x24 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    RORegisterBlock {
        (0x00 => _reserved1),
        (0x04 => PENDING_1: ReadOnly<u32>),
        (0x08 => PENDING_2: ReadOnly<u32>),
        (0x0c => @END),
    }
}

/// Abstraction for the WriteOnly parts of the associated MMIO registers.
type WriteOnlyRegisters = MMIODerefWrapper<WORegisterBlock>;

/// Abstraction for the ReadOnly parts of the associated MMIO registers.
type ReadOnlyRegisters = MMIODerefWrapper<RORegisterBlock>;

type HandlerTable =
    [Option<exception::asynchronous::IRQDescriptor>; InterruptController::NUM_PERIPHERAL_IRQS];

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Representation of the peripheral interrupt controller.
pub struct PeripheralIC {
    phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>,

    /// Access to write registers is guarded with a lock.
    wo_registers: IRQSafeNullLock<WriteOnlyRegisters>,

    /// Register read access is unguarded.
    ro_registers: InitStateLock<ReadOnlyRegisters>,

    /// Stores registered IRQ handlers. Writable only during kernel init. RO afterwards.
    handler_table: InitStateLock<HandlerTable>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl PeripheralIC {
    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide correct MMIO descriptors.
    pub const unsafe fn new(phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>) -> Self {
        let addr = phys_mmio_descriptor.start_addr().into_usize();

        Self {
            phys_mmio_descriptor,
            wo_registers: IRQSafeNullLock::new(WriteOnlyRegisters::new(addr)),
            ro_registers: InitStateLock::new(ReadOnlyRegisters::new(addr)),
            handler_table: InitStateLock::new([None; InterruptController::NUM_PERIPHERAL_IRQS]),
        }
    }

    /// Query the list of pending IRQs.
    fn pending_irqs(&self) -> PendingIRQs {
        let mut r = &self.ro_registers;
        r.read(|regs| {
            let pending_mask: u64 =
                (u64::from(regs.PENDING_2.get()) << 32) | u64::from(regs.PENDING_1.get());

            PendingIRQs::new(pending_mask)
        })
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
use synchronization::interface::{Mutex, ReadWriteEx};

impl driver::interface::DeviceDriver for PeripheralIC {
    fn compatible(&self) -> &'static str {
        "BCM Peripheral Interrupt Controller"
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        let virt_addr =
            memory::mmu::kernel_map_mmio(self.compatible(), &self.phys_mmio_descriptor)?
                .into_usize();

        let mut r = &self.wo_registers;
        r.lock(|regs| *regs = WriteOnlyRegisters::new(virt_addr));

        let mut r = &self.ro_registers;
        r.write(|regs| *regs = ReadOnlyRegisters::new(virt_addr));

        Ok(())
    }
}

impl exception::asynchronous::interface::IRQManager for PeripheralIC {
    type IRQNumberType = PeripheralIRQ;

    fn register_handler(
        &self,
        irq: Self::IRQNumberType,
        descriptor: exception::asynchronous::IRQDescriptor,
    ) -> Result<(), &'static str> {
        let mut r = &self.handler_table;
        r.write(|table| {
            let irq_number = irq.get();

            if table[irq_number].is_some() {
                return Err("IRQ handler already registered");
            }

            table[irq_number] = Some(descriptor);

            Ok(())
        })
    }

    fn enable(&self, irq: Self::IRQNumberType) {
        let mut r = &self.wo_registers;
        r.lock(|regs| {
            let enable_reg = if irq.get() <= 31 {
                &regs.ENABLE_1
            } else {
                &regs.ENABLE_2
            };

            let enable_bit: u32 = 1 << (irq.get() % 32);

            // Writing a 1 to a bit will set the corresponding IRQ enable bit. All other IRQ enable
            // bits are unaffected. So we don't need read and OR'ing here.
            enable_reg.set(enable_bit);
        });
    }

    fn handle_pending_irqs<'irq_context>(
        &'irq_context self,
        _ic: &exception::asynchronous::IRQContext<'irq_context>,
    ) {
        let mut r = &self.handler_table;
        r.read(|table| {
            for irq_number in self.pending_irqs() {
                match table[irq_number] {
                    None => panic!("No handler registered for IRQ {}", irq_number),
                    Some(descriptor) => {
                        // Call the IRQ handler. Panics on failure.
                        descriptor.handler.handle().expect("Error handling IRQ");
                    }
                }
            }
        })
    }

    fn print_handler(&self) {
        use crate::info;

        info!("      Peripheral handler:");

        let mut r = &self.handler_table;
        r.read(|table| {
            for (i, opt) in table.iter().enumerate() {
                if let Some(handler) = opt {
                    info!("            {: >3}. {}", i, handler.name);
                }
            }
        });
    }
}
