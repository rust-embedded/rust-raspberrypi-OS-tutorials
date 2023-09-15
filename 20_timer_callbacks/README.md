# Tutorial 20 - Timer Callbacks

## tl;dr

- The timer subsystem is extended so that it can be used to execute timeout callbacks in IRQ
  context.

## Note

This chapter's code will be tightly coupled to follow-up tutorials which are yet to be developed. It
is therefore expected that this chapter's code is subject to change depending upon findings that are
yet to be made.

Therefore, content for this README will be provided sometime later when all the pieces fit together.

## Diff to previous
```diff

diff -uNr 19_kernel_heap/kernel/Cargo.toml 20_timer_callbacks/kernel/Cargo.toml
--- 19_kernel_heap/kernel/Cargo.toml
+++ 20_timer_callbacks/kernel/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.19.0"
+version = "0.20.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2021"


diff -uNr 19_kernel_heap/kernel/src/_arch/aarch64/time.rs 20_timer_callbacks/kernel/src/_arch/aarch64/time.rs
--- 19_kernel_heap/kernel/src/_arch/aarch64/time.rs
+++ 20_timer_callbacks/kernel/src/_arch/aarch64/time.rs
@@ -11,14 +11,17 @@
 //!
 //! crate::time::arch_time

-use crate::warn;
+use crate::{
+    bsp::{self, exception},
+    warn,
+};
 use aarch64_cpu::{asm::barrier, registers::*};
 use core::{
     num::{NonZeroU128, NonZeroU32, NonZeroU64},
     ops::{Add, Div},
     time::Duration,
 };
-use tock_registers::interfaces::Readable;
+use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

 //--------------------------------------------------------------------------------------------------
 // Private Definitions
@@ -160,3 +163,31 @@
     // Read CNTPCT_EL0 directly to avoid the ISB that is part of [`read_cntpct`].
     while GenericTimerCounterValue(CNTPCT_EL0.get()) < counter_value_target {}
 }
+
+/// The associated IRQ number.
+pub const fn timeout_irq() -> exception::asynchronous::IRQNumber {
+    bsp::exception::asynchronous::irq_map::ARM_NS_PHYSICAL_TIMER
+}
+
+/// Program a timer IRQ to be fired after `delay` has passed.
+pub fn set_timeout_irq(due_time: Duration) {
+    let counter_value_target: GenericTimerCounterValue = match due_time.try_into() {
+        Err(msg) => {
+            warn!("set_timeout: {}. Skipping", msg);
+            return;
+        }
+        Ok(val) => val,
+    };
+
+    // Set the compare value register.
+    CNTP_CVAL_EL0.set(counter_value_target.0);
+
+    // Kick off the timer.
+    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::CLEAR);
+}
+
+/// Conclude a pending timeout IRQ.
+pub fn conclude_timeout_irq() {
+    // Disable counting. De-asserts the IRQ.
+    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::CLEAR);
+}

diff -uNr 19_kernel_heap/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/local_ic.rs 20_timer_callbacks/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/local_ic.rs
--- 19_kernel_heap/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/local_ic.rs
+++ 20_timer_callbacks/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/local_ic.rs
@@ -0,0 +1,173 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Local Interrupt Controller Driver.
+//!
+//! # Resources
+//!
+//! - <https://datasheets.raspberrypi.com/bcm2836/bcm2836-peripherals.pdf>
+
+use super::{LocalIRQ, PendingIRQs};
+use crate::{
+    bsp::device_driver::common::MMIODerefWrapper,
+    exception,
+    memory::{Address, Virtual},
+    synchronization,
+    synchronization::{IRQSafeNullLock, InitStateLock},
+};
+use alloc::vec::Vec;
+use tock_registers::{
+    interfaces::{Readable, Writeable},
+    register_structs,
+    registers::{ReadOnly, WriteOnly},
+};
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+register_structs! {
+    #[allow(non_snake_case)]
+    WORegisterBlock {
+        (0x00 => _reserved1),
+        (0x40 => CORE0_TIMER_INTERRUPT_CONTROL: WriteOnly<u32>),
+        (0x44 => @END),
+    }
+}
+
+register_structs! {
+    #[allow(non_snake_case)]
+    RORegisterBlock {
+        (0x00 => _reserved1),
+        (0x60 => CORE0_INTERRUPT_SOURCE: ReadOnly<u32>),
+        (0x64 => @END),
+    }
+}
+
+/// Abstraction for the WriteOnly parts of the associated MMIO registers.
+type WriteOnlyRegisters = MMIODerefWrapper<WORegisterBlock>;
+
+/// Abstraction for the ReadOnly parts of the associated MMIO registers.
+type ReadOnlyRegisters = MMIODerefWrapper<RORegisterBlock>;
+
+type HandlerTable = Vec<Option<exception::asynchronous::IRQHandlerDescriptor<LocalIRQ>>>;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Representation of the peripheral interrupt controller.
+pub struct LocalIC {
+    /// Access to write registers is guarded with a lock.
+    wo_registers: IRQSafeNullLock<WriteOnlyRegisters>,
+
+    /// Register read access is unguarded.
+    ro_registers: ReadOnlyRegisters,
+
+    /// Stores registered IRQ handlers. Writable only during kernel init. RO afterwards.
+    handler_table: InitStateLock<HandlerTable>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl LocalIC {
+    // See datasheet.
+    const PERIPH_IRQ_MASK: u32 = (1 << 8);
+
+    /// Create an instance.
+    ///
+    /// # Safety
+    ///
+    /// - The user must ensure to provide a correct MMIO start address.
+    pub const unsafe fn new(mmio_start_addr: Address<Virtual>) -> Self {
+        Self {
+            wo_registers: IRQSafeNullLock::new(WriteOnlyRegisters::new(mmio_start_addr)),
+            ro_registers: ReadOnlyRegisters::new(mmio_start_addr),
+            handler_table: InitStateLock::new(Vec::new()),
+        }
+    }
+
+    /// Called by the kernel to bring up the device.
+    pub fn init(&self) {
+        self.handler_table
+            .write(|table| table.resize(LocalIRQ::MAX_INCLUSIVE + 1, None));
+    }
+
+    /// Query the list of pending IRQs.
+    fn pending_irqs(&self) -> PendingIRQs {
+        // Ignore the indicator bit for a peripheral IRQ.
+        PendingIRQs::new(
+            (self.ro_registers.CORE0_INTERRUPT_SOURCE.get() & !Self::PERIPH_IRQ_MASK).into(),
+        )
+    }
+}
+
+//------------------------------------------------------------------------------
+// OS Interface Code
+//------------------------------------------------------------------------------
+use synchronization::interface::{Mutex, ReadWriteEx};
+
+impl exception::asynchronous::interface::IRQManager for LocalIC {
+    type IRQNumberType = LocalIRQ;
+
+    fn register_handler(
+        &self,
+        irq_handler_descriptor: exception::asynchronous::IRQHandlerDescriptor<Self::IRQNumberType>,
+    ) -> Result<(), &'static str> {
+        self.handler_table.write(|table| {
+            let irq_number = irq_handler_descriptor.number().get();
+
+            if table[irq_number].is_some() {
+                return Err("IRQ handler already registered");
+            }
+
+            table[irq_number] = Some(irq_handler_descriptor);
+
+            Ok(())
+        })
+    }
+
+    fn enable(&self, irq: &Self::IRQNumberType) {
+        self.wo_registers.lock(|regs| {
+            let enable_bit: u32 = 1 << (irq.get());
+
+            // Writing a 1 to a bit will set the corresponding IRQ enable bit. All other IRQ enable
+            // bits are unaffected. So we don't need read and OR'ing here.
+            regs.CORE0_TIMER_INTERRUPT_CONTROL.set(enable_bit);
+        });
+    }
+
+    fn handle_pending_irqs<'irq_context>(
+        &'irq_context self,
+        _ic: &exception::asynchronous::IRQContext<'irq_context>,
+    ) {
+        self.handler_table.read(|table| {
+            for irq_number in self.pending_irqs() {
+                match table[irq_number] {
+                    None => panic!("No handler registered for IRQ {}", irq_number),
+                    Some(descriptor) => {
+                        // Call the IRQ handler. Panics on failure.
+                        descriptor.handler().handle().expect("Error handling IRQ");
+                    }
+                }
+            }
+        })
+    }
+
+    fn print_handler(&self) {
+        use crate::info;
+
+        info!("      Local handler:");
+
+        self.handler_table.read(|table| {
+            for (i, opt) in table.iter().enumerate() {
+                if let Some(handler) = opt {
+                    info!("            {: >3}. {}", i, handler.name());
+                }
+            }
+        });
+    }
+}

diff -uNr 19_kernel_heap/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs 20_timer_callbacks/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs
--- 19_kernel_heap/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs
+++ 20_timer_callbacks/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs
@@ -4,6 +4,7 @@

 //! Interrupt Controller Driver.

+mod local_ic;
 mod peripheral_ic;

 use crate::{
@@ -40,6 +41,7 @@

 /// Representation of the Interrupt Controller.
 pub struct InterruptController {
+    local: local_ic::LocalIC,
     periph: peripheral_ic::PeripheralIC,
 }

@@ -81,7 +83,7 @@
 }

 impl InterruptController {
-    // Restrict to 3 for now. This makes future code for local_ic.rs more straight forward.
+    // Restrict to 3 for now. This makes the code for local_ic.rs more straight forward.
     const MAX_LOCAL_IRQ_NUMBER: usize = 3;
     const MAX_PERIPHERAL_IRQ_NUMBER: usize = 63;

@@ -92,8 +94,12 @@
     /// # Safety
     ///
     /// - The user must ensure to provide a correct MMIO start address.
-    pub const unsafe fn new(periph_mmio_start_addr: Address<Virtual>) -> Self {
+    pub const unsafe fn new(
+        local_mmio_start_addr: Address<Virtual>,
+        periph_mmio_start_addr: Address<Virtual>,
+    ) -> Self {
         Self {
+            local: local_ic::LocalIC::new(local_mmio_start_addr),
             periph: peripheral_ic::PeripheralIC::new(periph_mmio_start_addr),
         }
     }
@@ -111,6 +117,7 @@
     }

     unsafe fn init(&self) -> Result<(), &'static str> {
+        self.local.init();
         self.periph.init();

         Ok(())
@@ -125,7 +132,15 @@
         irq_handler_descriptor: exception::asynchronous::IRQHandlerDescriptor<Self::IRQNumberType>,
     ) -> Result<(), &'static str> {
         match irq_handler_descriptor.number() {
-            IRQNumber::Local(_) => unimplemented!("Local IRQ controller not implemented."),
+            IRQNumber::Local(lirq) => {
+                let local_descriptor = IRQHandlerDescriptor::new(
+                    lirq,
+                    irq_handler_descriptor.name(),
+                    irq_handler_descriptor.handler(),
+                );
+
+                self.local.register_handler(local_descriptor)
+            }
             IRQNumber::Peripheral(pirq) => {
                 let periph_descriptor = IRQHandlerDescriptor::new(
                     pirq,
@@ -140,7 +155,7 @@

     fn enable(&self, irq: &Self::IRQNumberType) {
         match irq {
-            IRQNumber::Local(_) => unimplemented!("Local IRQ controller not implemented."),
+            IRQNumber::Local(lirq) => self.local.enable(lirq),
             IRQNumber::Peripheral(pirq) => self.periph.enable(pirq),
         }
     }
@@ -149,11 +164,12 @@
         &'irq_context self,
         ic: &exception::asynchronous::IRQContext<'irq_context>,
     ) {
-        // It can only be a peripheral IRQ pending because enable() does not support local IRQs yet.
+        self.local.handle_pending_irqs(ic);
         self.periph.handle_pending_irqs(ic)
     }

     fn print_handler(&self) {
+        self.local.print_handler();
         self.periph.print_handler();
     }
 }

diff -uNr 19_kernel_heap/kernel/src/bsp/raspberrypi/driver.rs 20_timer_callbacks/kernel/src/bsp/raspberrypi/driver.rs
--- 19_kernel_heap/kernel/src/bsp/raspberrypi/driver.rs
+++ 20_timer_callbacks/kernel/src/bsp/raspberrypi/driver.rs
@@ -73,6 +73,12 @@
 /// This must be called only after successful init of the memory subsystem.
 #[cfg(feature = "bsp_rpi3")]
 unsafe fn instantiate_interrupt_controller() -> Result<(), &'static str> {
+    let local_mmio_descriptor = MMIODescriptor::new(mmio::LOCAL_IC_START, mmio::LOCAL_IC_SIZE);
+    let local_virt_addr = memory::mmu::kernel_map_mmio(
+        device_driver::InterruptController::COMPATIBLE,
+        &local_mmio_descriptor,
+    )?;
+
     let periph_mmio_descriptor =
         MMIODescriptor::new(mmio::PERIPHERAL_IC_START, mmio::PERIPHERAL_IC_SIZE);
     let periph_virt_addr = memory::mmu::kernel_map_mmio(
@@ -80,7 +86,10 @@
         &periph_mmio_descriptor,
     )?;

-    INTERRUPT_CONTROLLER.write(device_driver::InterruptController::new(periph_virt_addr));
+    INTERRUPT_CONTROLLER.write(device_driver::InterruptController::new(
+        local_virt_addr,
+        periph_virt_addr,
+    ));

     Ok(())
 }

diff -uNr 19_kernel_heap/kernel/src/bsp/raspberrypi/exception/asynchronous.rs 20_timer_callbacks/kernel/src/bsp/raspberrypi/exception/asynchronous.rs
--- 19_kernel_heap/kernel/src/bsp/raspberrypi/exception/asynchronous.rs
+++ 20_timer_callbacks/kernel/src/bsp/raspberrypi/exception/asynchronous.rs
@@ -13,16 +13,24 @@
 /// Export for reuse in generic asynchronous.rs.
 pub use bsp::device_driver::IRQNumber;

+/// The IRQ map.
 #[cfg(feature = "bsp_rpi3")]
-pub(in crate::bsp) mod irq_map {
-    use super::bsp::device_driver::{IRQNumber, PeripheralIRQ};
+pub mod irq_map {
+    use super::bsp::device_driver::{IRQNumber, LocalIRQ, PeripheralIRQ};

-    pub const PL011_UART: IRQNumber = IRQNumber::Peripheral(PeripheralIRQ::new(57));
+    /// The non-secure physical timer IRQ number.
+    pub const ARM_NS_PHYSICAL_TIMER: IRQNumber = IRQNumber::Local(LocalIRQ::new(1));
+
+    pub(in crate::bsp) const PL011_UART: IRQNumber = IRQNumber::Peripheral(PeripheralIRQ::new(57));
 }

+/// The IRQ map.
 #[cfg(feature = "bsp_rpi4")]
-pub(in crate::bsp) mod irq_map {
+pub mod irq_map {
     use super::bsp::device_driver::IRQNumber;

-    pub const PL011_UART: IRQNumber = IRQNumber::new(153);
+    /// The non-secure physical timer IRQ number.
+    pub const ARM_NS_PHYSICAL_TIMER: IRQNumber = IRQNumber::new(30);
+
+    pub(in crate::bsp) const PL011_UART: IRQNumber = IRQNumber::new(153);
 }

diff -uNr 19_kernel_heap/kernel/src/bsp/raspberrypi/memory.rs 20_timer_callbacks/kernel/src/bsp/raspberrypi/memory.rs
--- 19_kernel_heap/kernel/src/bsp/raspberrypi/memory.rs
+++ 20_timer_callbacks/kernel/src/bsp/raspberrypi/memory.rs
@@ -124,6 +124,9 @@
         pub const PL011_UART_START:    Address<Physical> = Address::new(0x3F20_1000);
         pub const PL011_UART_SIZE:     usize             =              0x48;

+        pub const LOCAL_IC_START:      Address<Physical> = Address::new(0x4000_0000);
+        pub const LOCAL_IC_SIZE:       usize             =              0x100;
+
         pub const END:                 Address<Physical> = Address::new(0x4001_0000);
     }


diff -uNr 19_kernel_heap/kernel/src/main.rs 20_timer_callbacks/kernel/src/main.rs
--- 19_kernel_heap/kernel/src/main.rs
+++ 20_timer_callbacks/kernel/src/main.rs
@@ -30,6 +30,11 @@
     exception::handling_init();
     memory::init();

+    // Initialize the timer subsystem.
+    if let Err(x) = time::init() {
+        panic!("Error initializing timer subsystem: {}", x);
+    }
+
     // Initialize the BSP driver subsystem.
     if let Err(x) = bsp::driver::init() {
         panic!("Error initializing BSP driver subsystem: {}", x);
@@ -52,6 +57,9 @@

 /// The main function running after the early init.
 fn kernel_main() -> ! {
+    use alloc::boxed::Box;
+    use core::time::Duration;
+
     info!("{}", libkernel::version());
     info!("Booting on: {}", bsp::board_name());

@@ -78,6 +86,11 @@
     info!("Kernel heap:");
     memory::heap_alloc::kernel_heap_allocator().print_usage();

+    time::time_manager().set_timeout_once(Duration::from_secs(5), Box::new(|| info!("Once 5")));
+    time::time_manager().set_timeout_once(Duration::from_secs(3), Box::new(|| info!("Once 2")));
+    time::time_manager()
+        .set_timeout_periodic(Duration::from_secs(1), Box::new(|| info!("Periodic 1 sec")));
+
     info!("Echoing input now");
     cpu::wait_forever();
 }

diff -uNr 19_kernel_heap/kernel/src/time.rs 20_timer_callbacks/kernel/src/time.rs
--- 19_kernel_heap/kernel/src/time.rs
+++ 20_timer_callbacks/kernel/src/time.rs
@@ -3,19 +3,54 @@
 // Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>

 //! Timer primitives.
+//!
+//! # Resources
+//!
+//! - <https://stackoverflow.com/questions/41081240/idiomatic-callbacks-in-rust>
+//! - <https://doc.rust-lang.org/stable/std/panic/fn.set_hook.html>

 #[cfg(target_arch = "aarch64")]
 #[path = "_arch/aarch64/time.rs"]
 mod arch_time;

-use core::time::Duration;
+use crate::{
+    driver, exception,
+    exception::asynchronous::IRQNumber,
+    synchronization::{interface::Mutex, IRQSafeNullLock},
+    warn,
+};
+use alloc::{boxed::Box, vec::Vec};
+use core::{
+    sync::atomic::{AtomicBool, Ordering},
+    time::Duration,
+};
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+struct Timeout {
+    due_time: Duration,
+    period: Option<Duration>,
+    callback: TimeoutCallback,
+}
+
+struct OrderedTimeoutQueue {
+    // Can be replaced with a BinaryHeap once it's new() becomes const.
+    inner: Vec<Timeout>,
+}

 //--------------------------------------------------------------------------------------------------
 // Public Definitions
 //--------------------------------------------------------------------------------------------------

+/// The callback type used by timer IRQs.
+pub type TimeoutCallback = Box<dyn Fn() + Send>;
+
 /// Provides time management functions.
-pub struct TimeManager;
+pub struct TimeManager {
+    queue: IRQSafeNullLock<OrderedTimeoutQueue>,
+}

 //--------------------------------------------------------------------------------------------------
 // Global instances
@@ -24,6 +59,46 @@
 static TIME_MANAGER: TimeManager = TimeManager::new();

 //--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+impl Timeout {
+    pub fn is_periodic(&self) -> bool {
+        self.period.is_some()
+    }
+
+    pub fn refresh(&mut self) {
+        if let Some(delay) = self.period {
+            self.due_time += delay;
+        }
+    }
+}
+
+impl OrderedTimeoutQueue {
+    pub const fn new() -> Self {
+        Self { inner: Vec::new() }
+    }
+
+    pub fn push(&mut self, timeout: Timeout) {
+        self.inner.push(timeout);
+
+        // Note reverse compare order so that earliest expiring item is at end of vec. We do this so
+        // that we can use Vec::pop below to retrieve the item that is next due.
+        self.inner.sort_by(|a, b| b.due_time.cmp(&a.due_time));
+    }
+
+    pub fn peek_next_due_time(&self) -> Option<Duration> {
+        let timeout = self.inner.last()?;
+
+        Some(timeout.due_time)
+    }
+
+    pub fn pop(&mut self) -> Option<Timeout> {
+        self.inner.pop()
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------

@@ -33,9 +108,14 @@
 }

 impl TimeManager {
+    /// Compatibility string.
+    pub const COMPATIBLE: &'static str = "ARM Architectural Timer";
+
     /// Create an instance.
     pub const fn new() -> Self {
-        Self
+        Self {
+            queue: IRQSafeNullLock::new(OrderedTimeoutQueue::new()),
+        }
     }

     /// The timer's resolution.
@@ -54,4 +134,130 @@
     pub fn spin_for(&self, duration: Duration) {
         arch_time::spin_for(duration)
     }
+
+    /// Set a timeout.
+    fn set_timeout(&self, timeout: Timeout) {
+        self.queue.lock(|queue| {
+            queue.push(timeout);
+
+            arch_time::set_timeout_irq(queue.peek_next_due_time().unwrap());
+        });
+    }
+
+    /// Set a one-shot timeout.
+    pub fn set_timeout_once(&self, delay: Duration, callback: TimeoutCallback) {
+        let timeout = Timeout {
+            due_time: self.uptime() + delay,
+            period: None,
+            callback,
+        };
+
+        self.set_timeout(timeout);
+    }
+
+    /// Set a periodic timeout.
+    pub fn set_timeout_periodic(&self, delay: Duration, callback: TimeoutCallback) {
+        let timeout = Timeout {
+            due_time: self.uptime() + delay,
+            period: Some(delay),
+            callback,
+        };
+
+        self.set_timeout(timeout);
+    }
+}
+
+/// Initialize the timer subsystem.
+pub fn init() -> Result<(), &'static str> {
+    static INIT_DONE: AtomicBool = AtomicBool::new(false);
+    if INIT_DONE.load(Ordering::Relaxed) {
+        return Err("Init already done");
+    }
+
+    let timer_descriptor =
+        driver::DeviceDriverDescriptor::new(time_manager(), None, Some(arch_time::timeout_irq()));
+    driver::driver_manager().register_driver(timer_descriptor);
+
+    INIT_DONE.store(true, Ordering::Relaxed);
+    Ok(())
+}
+
+//------------------------------------------------------------------------------
+// OS Interface Code
+//------------------------------------------------------------------------------
+
+impl driver::interface::DeviceDriver for TimeManager {
+    type IRQNumberType = IRQNumber;
+
+    fn compatible(&self) -> &'static str {
+        Self::COMPATIBLE
+    }
+
+    fn register_and_enable_irq_handler(
+        &'static self,
+        irq_number: &Self::IRQNumberType,
+    ) -> Result<(), &'static str> {
+        use exception::asynchronous::{irq_manager, IRQHandlerDescriptor};
+
+        let descriptor = IRQHandlerDescriptor::new(*irq_number, Self::COMPATIBLE, self);
+
+        irq_manager().register_handler(descriptor)?;
+        irq_manager().enable(irq_number);
+
+        Ok(())
+    }
+}
+
+impl exception::asynchronous::interface::IRQHandler for TimeManager {
+    fn handle(&self) -> Result<(), &'static str> {
+        arch_time::conclude_timeout_irq();
+
+        let maybe_timeout: Option<Timeout> = self.queue.lock(|queue| {
+            let next_due_time = queue.peek_next_due_time()?;
+            if next_due_time > self.uptime() {
+                return None;
+            }
+
+            let mut timeout = queue.pop().unwrap();
+
+            // Refresh as early as possible to prevent drift.
+            if timeout.is_periodic() {
+                timeout.refresh();
+            }
+
+            Some(timeout)
+        });
+
+        let timeout = match maybe_timeout {
+            None => {
+                warn!("Spurious timeout IRQ");
+                return Ok(());
+            }
+            Some(t) => t,
+        };
+
+        // Important: Call the callback while not holding any lock, because the callback might
+        // attempt to modify data that is protected by a lock (in particular, the timeout queue
+        // itself).
+        (timeout.callback)();
+
+        self.queue.lock(|queue| {
+            if timeout.is_periodic() {
+                // There might be some overhead involved in the periodic path, because the timeout
+                // item is first popped from the underlying Vec and then pushed back again. It could
+                // be faster to keep the item in the queue and find a way to work with a reference
+                // to it.
+                //
+                // We are not going this route on purpose, though. It allows to keep the code simple
+                // and the focus on the high-level concepts.
+                queue.push(timeout);
+            };
+
+            if let Some(due_time) = queue.peek_next_due_time() {
+                arch_time::set_timeout_irq(due_time);
+            }
+        });
+
+        Ok(())
+    }
 }

diff -uNr 19_kernel_heap/kernel/tests/boot_test_string.rb 20_timer_callbacks/kernel/tests/boot_test_string.rb
--- 19_kernel_heap/kernel/tests/boot_test_string.rb
+++ 20_timer_callbacks/kernel/tests/boot_test_string.rb
@@ -1,3 +1,3 @@
 # frozen_string_literal: true

-EXPECTED_PRINT = 'Echoing input now'
+EXPECTED_PRINT = 'Once 5'

```
