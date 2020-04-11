# Tutorial 10 - Privilege Level

## tl;dr

In early boot code, we transition from the `Hypervisor` privilege level (`EL2` in AArch64) to the
`Kernel` (`EL1`) privilege level.

## Table of Contents

- [Introduction](#introduction)
- [Scope of this tutorial](#scope-of-this-tutorial)
- [Checking for EL2 in the entrypoint](#checking-for-el2-in-the-entrypoint)
- [Transition preparation](#transition-preparation)
- [Returning from an exception that never happened](#returning-from-an-exception-that-never-happened)
- [Are we stackless?](#are-we-stackless)
- [Test it](#test-it)
- [Diff to previous](#diff-to-previous)

## Introduction

Application-grade CPUs have so-called `privilege levels`, which have different purposes:

| Typically used for | AArch64 | RISC-V | x86 |
| ------------- | ------------- | ------------- | ------------- |
| Userspace applications | EL0 | U/VU | Ring 3 |
| OS Kernel | EL1 | S/VS | Ring 0 |
| Hypervisor | EL2 | HS | Ring -1 |
| Low-Level Firmware | EL3 | M | |

`EL` in AArch64 stands for `Exception Level`. If you want more information regarding the other
architectures, please have a look at the following links:
- [x86 privilege rings](https://en.wikipedia.org/wiki/Protection_ring).
- [RISC-V privilege modes](https://content.riscv.org/wp-content/uploads/2017/12/Tue0942-riscv-hypervisor-waterman.pdf).

At this point, I strongly recommend that you glimpse over `Chapter 3` of the [Programmer’s Guide for
ARMv8-A](http://infocenter.arm.com/help/topic/com.arm.doc.den0024a/DEN0024A_v8_architecture_PG.pdf)
before you continue. It gives a concise overview about the topic.

## Scope of this tutorial

If you set up your SD Card exactly like mentioned in [tutorial 06], the Rpi will always start
executing in `EL2`. Since we are writing a traditional `Kernel`, we have to transition into the more
appropriate `EL1`.

[tutorial 06]: https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/tree/master/06_drivers_gpio_uart#boot-it-from-sd-card

## Checking for EL2 in the entrypoint

First of all, we need to ensure that we actually execute in `EL2` before we can call respective code
to transition to `EL1`:

```rust
pub unsafe extern "C" fn _start() -> ! {
    // Expect the boot core to start in EL2.
    if (bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id())
        && (CurrentEL.get() == CurrentEL::EL::EL2.value)
    {
        el2_to_el1_transition()
    } else {
        // If not core0, infinitely wait for events.
        wait_forever()
    }
}
```

If this is the case, we continue with preparing the `EL2` -> `EL1` transition in
`el2_to_el1_transition()`.

## Transition preparation

Since `EL2` is more privileged than `EL1`, it has control over various processor features and can
allow or disallow `EL1` code to use them. One such example is access to timer and counter registers.
We are already using them since [tutorial 08](../08_timestamps/), so of course we want to keep them.
Therefore we set the respective flags in the [Counter-timer Hypervisor Control register] and
additionally set the virtual offset to zero so that we get the real physical value everytime:

[Counter-timer Hypervisor Control register]: https://docs.rs/cortex-a/2.4.0/src/cortex_a/regs/cnthctl_el2.rs.html

```rust
// Enable timer counter registers for EL1.
CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

// No offset for reading the counters.
CNTVOFF_EL2.set(0);
```

Next, we configure the [Hypervisor Configuration Register] such that `EL1` should actually run in
`AArch64` mode, and not in `AArch32`, which would also be possible.

[Hypervisor Configuration Register]: https://docs.rs/cortex-a/2.4.0/src/cortex_a/regs/hcr_el2.rs.html

```rust
// Set EL1 execution state to AArch64.
HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
```

## Returning from an exception that never happened

There is actually only one way to transition from a higher EL to a lower EL, which is by way of
executing the [ERET] instruction.

[ERET]: https://docs.rs/cortex-a/2.4.0/src/cortex_a/asm.rs.html#49-62

This instruction will copy the contents of the [Saved Program Status Register - EL2] to `Current
Program Status Register - EL1` and jump to the instruction address that is stored in the [Exception
Link Register - EL2].

This is basically the reverse of what is happening when an exception is taken. You'll learn about it
in an upcoming tutorial.

[Saved Program Status Register - EL2]: https://docs.rs/cortex-a/2.4.0/src/cortex_a/regs/spsr_el2.rs.html
[Exception Link Register - EL2]: https://docs.rs/cortex-a/2.4.0/src/cortex_a/regs/elr_el2.rs.html

```rust
// Set up a simulated exception return.
//
// First, fake a saved program status where all interrupts were masked and SP_EL1 was used as a
// stack pointer.
SPSR_EL2.write(
    SPSR_EL2::D::Masked
        + SPSR_EL2::A::Masked
        + SPSR_EL2::I::Masked
        + SPSR_EL2::F::Masked
        + SPSR_EL2::M::EL1h,
);

// Second, let the link register point to runtime_init().
ELR_EL2.set(runtime_init::runtime_init as *const () as u64);
```

As you can see, we are populating `ELR_EL2` with the address of the [runtime_init()] function that
we earlier used to call directly from the entrypoint.

Finally, we set the stack pointer for `SP_EL1` and call `ERET`:

[runtime_init()]: src/runtime_init.rs

```rust
// Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it.
SP_EL1.set(bsp::cpu::BOOT_CORE_STACK_START);

// Use `eret` to "return" to EL1. This results in execution of runtime_init() in EL1.
asm::eret()
```

## Are we stackless?

We just wrote a big inline rust function, `el2_to_el1_transition()`, that is executed in a context
where we do not have a stack yet. We should double-check the generated machine code:

```console
$ make objdump
[...]
Disassembly of section .text:

0000000000080000 _start:
   80000:       mrs     x8, MPIDR_EL1
   80004:       tst     x8, #0x3
   80008:       b.ne    #0x10 <_start+0x18>
   8000c:       mrs     x8, CurrentEL
   80010:       cmp     w8, #0x8
   80014:       b.eq    #0xc <_start+0x20>
   80018:       wfe
   8001c:       b       #-0x4 <_start+0x18>
   80020:       mov     x8, xzr
   80024:       mov     w9, #0x3
   80028:       msr     CNTHCTL_EL2, x9
   8002c:       msr     CNTVOFF_EL2, x8
   80030:       adrp    x8, #0x0
   80034:       mov     w10, #-0x80000000
   80038:       mov     w11, #0x3c5
   8003c:       mov     w12, #0x80000
   80040:       msr     HCR_EL2, x10
   80044:       msr     SPSR_EL2, x11
   80048:       add     x8, x8, #0xda0
   8004c:       msr     ELR_EL2, x8
   80050:       msr     SP_EL1, x12
   80054:       eret
```

Looks good! Thanks zero-overhead abstractions in the [cortex-a] crate! :heart_eyes:

[cortex-a]: https://github.com/rust-embedded/cortex-a

## Test it

In `main.rs`, we additionally inspect if the mask bits in `SPSR_EL2` made it to `EL1` as well:

```console
$ make chainboot
[...]
Minipush 1.0

[MP] ⏳ Waiting for /dev/ttyUSB0
[MP] ✅ Connected
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
[MP] ⏩ Pushing 15 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.703812] Booting on: Raspberry Pi 3
[    0.704900] Current privilege level: EL1
[    0.706811] Exception handling state:
[    0.708592]       Debug:  Masked
[    0.710156]       SError: Masked
[    0.711719]       IRQ:    Masked
[    0.713283]       FIQ:    Masked
[    0.714848] Architectural timer resolution: 52 ns
[    0.717149] Drivers loaded:
[    0.718496]       1. BCM GPIO
[    0.719929]       2. BCM PL011 UART
[    0.721623] Timer test, spinning for 1 second
[    1.723753] Echoing input now
```

## Diff to previous
```diff

diff -uNr 09_hw_debug_JTAG/src/_arch/aarch64/cpu.rs 10_privilege_level/src/_arch/aarch64/cpu.rs
--- 09_hw_debug_JTAG/src/_arch/aarch64/cpu.rs
+++ 10_privilege_level/src/_arch/aarch64/cpu.rs
@@ -21,18 +21,59 @@
 #[naked]
 #[no_mangle]
 pub unsafe extern "C" fn _start() -> ! {
-    use crate::runtime_init;
-
     // Expect the boot core to start in EL2.
-    if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
-        SP.set(bsp::cpu::BOOT_CORE_STACK_START);
-        runtime_init::runtime_init()
+    if (bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id())
+        && (CurrentEL.get() == CurrentEL::EL::EL2.value)
+    {
+        el2_to_el1_transition()
     } else {
         // If not core0, infinitely wait for events.
         wait_forever()
     }
 }

+/// Transition from EL2 to EL1.
+///
+/// # Safety
+///
+/// - The HW state of EL1 must be prepared in a sound way.
+/// - Exception return from EL2 must must continue execution in EL1 with
+///   `runtime_init::runtime_init()`.
+#[inline(always)]
+unsafe fn el2_to_el1_transition() -> ! {
+    use crate::runtime_init;
+
+    // Enable timer counter registers for EL1.
+    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
+
+    // No offset for reading the counters.
+    CNTVOFF_EL2.set(0);
+
+    // Set EL1 execution state to AArch64.
+    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
+
+    // Set up a simulated exception return.
+    //
+    // First, fake a saved program status where all interrupts were masked and SP_EL1 was used as a
+    // stack pointer.
+    SPSR_EL2.write(
+        SPSR_EL2::D::Masked
+            + SPSR_EL2::A::Masked
+            + SPSR_EL2::I::Masked
+            + SPSR_EL2::F::Masked
+            + SPSR_EL2::M::EL1h,
+    );
+
+    // Second, let the link register point to runtime_init().
+    ELR_EL2.set(runtime_init::runtime_init as *const () as u64);
+
+    // Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it.
+    SP_EL1.set(bsp::cpu::BOOT_CORE_STACK_START);
+
+    // Use `eret` to "return" to EL1. This results in execution of runtime_init() in EL1.
+    asm::eret()
+}
+
 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------

diff -uNr 09_hw_debug_JTAG/src/_arch/aarch64/exception/asynchronous.rs 10_privilege_level/src/_arch/aarch64/exception/asynchronous.rs
--- 09_hw_debug_JTAG/src/_arch/aarch64/exception/asynchronous.rs
+++ 10_privilege_level/src/_arch/aarch64/exception/asynchronous.rs
@@ -0,0 +1,71 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Architectural asynchronous exception handling.
+
+use cortex_a::regs::*;
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+trait DaifField {
+    fn daif_field() -> register::Field<u32, DAIF::Register>;
+}
+
+struct Debug;
+struct SError;
+struct IRQ;
+struct FIQ;
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+impl DaifField for Debug {
+    fn daif_field() -> register::Field<u32, DAIF::Register> {
+        DAIF::D
+    }
+}
+
+impl DaifField for SError {
+    fn daif_field() -> register::Field<u32, DAIF::Register> {
+        DAIF::A
+    }
+}
+
+impl DaifField for IRQ {
+    fn daif_field() -> register::Field<u32, DAIF::Register> {
+        DAIF::I
+    }
+}
+
+impl DaifField for FIQ {
+    fn daif_field() -> register::Field<u32, DAIF::Register> {
+        DAIF::F
+    }
+}
+
+fn is_masked<T: DaifField>() -> bool {
+    DAIF.is_set(T::daif_field())
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Print the AArch64 exceptions status.
+#[rustfmt::skip]
+pub fn print_state() {
+    use crate::info;
+
+    let to_mask_str = |x| -> _ {
+        if x { "Masked" } else { "Unmasked" }
+    };
+
+    info!("      Debug:  {}", to_mask_str(is_masked::<Debug>()));
+    info!("      SError: {}", to_mask_str(is_masked::<SError>()));
+    info!("      IRQ:    {}", to_mask_str(is_masked::<IRQ>()));
+    info!("      FIQ:    {}", to_mask_str(is_masked::<FIQ>()));
+}

diff -uNr 09_hw_debug_JTAG/src/_arch/aarch64/exception.rs 10_privilege_level/src/_arch/aarch64/exception.rs
--- 09_hw_debug_JTAG/src/_arch/aarch64/exception.rs
+++ 10_privilege_level/src/_arch/aarch64/exception.rs
@@ -0,0 +1,23 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Architectural synchronous and asynchronous exception handling.
+
+use cortex_a::regs::*;
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+use crate::exception::PrivilegeLevel;
+
+/// The processing element's current privilege level.
+pub fn current_privilege_level() -> (PrivilegeLevel, &'static str) {
+    let el = CurrentEL.read_as_enum(CurrentEL::EL);
+    match el {
+        Some(CurrentEL::EL::Value::EL2) => (PrivilegeLevel::Hypervisor, "EL2"),
+        Some(CurrentEL::EL::Value::EL1) => (PrivilegeLevel::Kernel, "EL1"),
+        Some(CurrentEL::EL::Value::EL0) => (PrivilegeLevel::User, "EL0"),
+        _ => (PrivilegeLevel::Unknown, "Unknown"),
+    }
+}

diff -uNr 09_hw_debug_JTAG/src/exception/asynchronous.rs 10_privilege_level/src/exception/asynchronous.rs
--- 09_hw_debug_JTAG/src/exception/asynchronous.rs
+++ 10_privilege_level/src/exception/asynchronous.rs
@@ -0,0 +1,10 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Asynchronous exception handling.
+
+#[cfg(target_arch = "aarch64")]
+#[path = "../_arch/aarch64/exception/asynchronous.rs"]
+mod arch_exception_async;
+pub use arch_exception_async::*;

diff -uNr 09_hw_debug_JTAG/src/exception.rs 10_privilege_level/src/exception.rs
--- 09_hw_debug_JTAG/src/exception.rs
+++ 10_privilege_level/src/exception.rs
@@ -0,0 +1,26 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Synchronous and asynchronous exception handling.
+
+#[cfg(target_arch = "aarch64")]
+#[path = "_arch/aarch64/exception.rs"]
+mod arch_exception;
+pub use arch_exception::*;
+
+pub mod asynchronous;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Kernel privilege levels.
+#[allow(missing_docs)]
+#[derive(PartialEq)]
+pub enum PrivilegeLevel {
+    User,
+    Kernel,
+    Hypervisor,
+    Unknown,
+}

diff -uNr 09_hw_debug_JTAG/src/main.rs 10_privilege_level/src/main.rs
--- 09_hw_debug_JTAG/src/main.rs
+++ 10_privilege_level/src/main.rs
@@ -116,6 +116,7 @@
 mod console;
 mod cpu;
 mod driver;
+mod exception;
 mod memory;
 mod panic_wait;
 mod print;
@@ -146,12 +147,19 @@

 /// The main function running after the early init.
 fn kernel_main() -> ! {
+    use console::interface::All;
     use core::time::Duration;
     use driver::interface::DriverManager;
     use time::interface::TimeManager;

     info!("Booting on: {}", bsp::board_name());

+    let (_, privilege_level) = exception::current_privilege_level();
+    info!("Current privilege level: {}", privilege_level);
+
+    info!("Exception handling state:");
+    exception::asynchronous::print_state();
+
     info!(
         "Architectural timer resolution: {} ns",
         time::time_manager().resolution().as_nanos()
@@ -166,11 +174,12 @@
         info!("      {}. {}", i + 1, driver.compatible());
     }

-    // Test a failing timer case.
-    time::time_manager().spin_for(Duration::from_nanos(1));
+    info!("Timer test, spinning for 1 second");
+    time::time_manager().spin_for(Duration::from_secs(1));

+    info!("Echoing input now");
     loop {
-        info!("Spinning for 1 second");
-        time::time_manager().spin_for(Duration::from_secs(1));
+        let c = bsp::console::console().read_char();
+        bsp::console::console().write_char(c);
     }
 }

```
