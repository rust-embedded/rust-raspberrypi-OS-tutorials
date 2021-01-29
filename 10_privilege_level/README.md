# Tutorial 10 - Privilege Level

## tl;dr

- In early boot code, we transition from the `Hypervisor` privilege level (`EL2` in AArch64) to the
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
ARMv8-A] before you continue. It gives a concise overview about the topic.

[Programmer’s Guide forARMv8-A]: http://infocenter.arm.com/help/topic/com.arm.doc.den0024a/DEN0024A_v8_architecture_PG.pdf

## Scope of this tutorial

By default, the Rpi will always start executing in `EL2`. Since we are writing a traditional
`Kernel`, we have to transition into the more appropriate `EL1`.

## Checking for EL2 in the entrypoint

First of all, we need to ensure that we actually execute in `EL2` before we can call respective code
to transition to `EL1`:

```rust
#[no_mangle]
pub unsafe fn _start() -> ! {
    // Expect the boot core to start in EL2.
    if (bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id())
        && (CurrentEL.get() == CurrentEL::EL::EL2.value)
    {
        el2_to_el1_transition()
    } else {
        // If not core0, infinitely wait for events.
        cpu::wait_forever()
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

[Counter-timer Hypervisor Control register]:  https://docs.rs/cortex-a/5.1.2/src/cortex_a/regs/cnthctl_el2.rs.html

```rust
// Enable timer counter registers for EL1.
CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

// No offset for reading the counters.
CNTVOFF_EL2.set(0);
```

Next, we configure the [Hypervisor Configuration Register] such that `EL1` runs in `AArch64` mode,
and not in `AArch32`, which would also be possible.

[Hypervisor Configuration Register]: https://docs.rs/cortex-a/5.1.2/src/cortex_a/regs/hcr_el2.rs.html

```rust
// Set EL1 execution state to AArch64.
HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
```

## Returning from an exception that never happened

There is actually only one way to transition from a higher EL to a lower EL, which is by way of
executing the [ERET] instruction.

[ERET]: https://docs.rs/cortex-a/5.1.2/src/cortex_a/asm.rs.html#87-96

This instruction will copy the contents of the [Saved Program Status Register - EL2] to `Current
Program Status Register - EL1` and jump to the instruction address that is stored in the [Exception
Link Register - EL2].

This is basically the reverse of what is happening when an exception is taken. You'll learn about it
in an upcoming tutorial.

[Saved Program Status Register - EL2]: https://docs.rs/cortex-a/5.1.2/src/cortex_a/regs/spsr_el2.rs.html
[Exception Link Register - EL2]: https://docs.rs/cortex-a/5.1.2/src/cortex_a/regs/elr_el2.rs.html

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

0000000000080000 <_start>:
   80000:       d53800a8        mrs     x8, mpidr_el1
   80004:       f240051f        tst     x8, #0x3
   80008:       54000081        b.ne    80018 <_start+0x18>  // b.any
   8000c:       d5384248        mrs     x8, currentel
   80010:       f100211f        cmp     x8, #0x8
   80014:       54000060        b.eq    80020 <_start+0x20>  // b.none
   80018:       d503205f        wfe
   8001c:       17ffffff        b       80018 <_start+0x18>
   80020:       aa1f03e8        mov     x8, xzr
   80024:       52800069        mov     w9, #0x3                        // #3
   80028:       d51ce109        msr     cnthctl_el2, x9
   8002c:       d51ce068        msr     cntvoff_el2, x8
   80030:       d0000008        adrp    x8, 82000 <kernel::kernel_main+0x6b4>
   80034:       52b0000a        mov     w10, #0x80000000                // #-2147483648
   80038:       528078ab        mov     w11, #0x3c5                     // #965
   8003c:       52a0010c        mov     w12, #0x80000                   // #524288
   80040:       d51c110a        msr     hcr_el2, x10
   80044:       d51c400b        msr     spsr_el2, x11
   80048:       9114c108        add     x8, x8, #0x530
   8004c:       d51c4028        msr     elr_el2, x8
   80050:       d51c410c        msr     sp_el1, x12
   80054:       d69f03e0        eret
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
[MP] ✅ Serial connected
[MP] 🔌 Please power the target now
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
[MP] ⏩ Pushing 13 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.637617] Booting on: Raspberry Pi 3
[    0.638737] Current privilege level: EL1
[    0.640645] Exception handling state:
[    0.642424]       Debug:  Masked
[    0.643986]       SError: Masked
[    0.645548]       IRQ:    Masked
[    0.647110]       FIQ:    Masked
[    0.648672] Architectural timer resolution: 52 ns
[    0.650971] Drivers loaded:
[    0.652316]       1. BCM GPIO
[    0.653748]       2. BCM PL011 UART
[    0.655440] Timer test, spinning for 1 second
[    1.657567] Echoing input now
```

## Diff to previous
```diff

diff -uNr 09_hw_debug_JTAG/src/_arch/aarch64/cpu/boot.rs 10_privilege_level/src/_arch/aarch64/cpu/boot.rs
--- 09_hw_debug_JTAG/src/_arch/aarch64/cpu/boot.rs
+++ 10_privilege_level/src/_arch/aarch64/cpu/boot.rs
@@ -12,7 +12,55 @@
 //! crate::cpu::boot::arch_boot

 use crate::{bsp, cpu};
-use cortex_a::regs::*;
+use cortex_a::{asm, regs::*};
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+/// Transition from EL2 to EL1.
+///
+/// # Safety
+///
+/// - The HW state of EL1 must be prepared in a sound way.
+/// - Exception return from EL2 must must continue execution in EL1 with
+///   `runtime_init::runtime_init()`.
+/// - We have to hope that the compiler omits any stack pointer usage, because we are not setting up
+///   a stack for EL2.
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
+    SP_EL1.set(bsp::memory::boot_core_stack_end() as u64);
+
+    // Use `eret` to "return" to EL1. This results in execution of runtime_init() in EL1.
+    asm::eret()
+}

 //--------------------------------------------------------------------------------------------------
 // Public Code
@@ -25,15 +73,15 @@
 /// # Safety
 ///
 /// - Linker script must ensure to place this function where it is expected by the target machine.
-/// - We have to hope that the compiler omits any stack pointer usage before the stack pointer is
-///   actually set (`SP.set()`).
+/// - We have to hope that the compiler omits any stack pointer usage, because we are not setting up
+///   a stack for EL2.
 #[no_mangle]
 pub unsafe fn _start() -> ! {
-    use crate::runtime_init;
-
-    if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
-        SP.set(bsp::memory::boot_core_stack_end() as u64);
-        runtime_init::runtime_init()
+    // Expect the boot core to start in EL2.
+    if (bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id())
+        && (CurrentEL.get() == CurrentEL::EL::EL2.value)
+    {
+        el2_to_el1_transition()
     } else {
         // If not core0, infinitely wait for events.
         cpu::wait_forever()

diff -uNr 09_hw_debug_JTAG/src/_arch/aarch64/exception/asynchronous.rs 10_privilege_level/src/_arch/aarch64/exception/asynchronous.rs
--- 09_hw_debug_JTAG/src/_arch/aarch64/exception/asynchronous.rs
+++ 10_privilege_level/src/_arch/aarch64/exception/asynchronous.rs
@@ -0,0 +1,81 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Architectural asynchronous exception handling.
+//!
+//! # Orientation
+//!
+//! Since arch modules are imported into generic modules using the path attribute, the path of this
+//! file is:
+//!
+//! crate::exception::asynchronous::arch_asynchronous
+
+use cortex_a::regs::*;
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+trait DaifField {
+    fn daif_field() -> register::Field<u64, DAIF::Register>;
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
+    fn daif_field() -> register::Field<u64, DAIF::Register> {
+        DAIF::D
+    }
+}
+
+impl DaifField for SError {
+    fn daif_field() -> register::Field<u64, DAIF::Register> {
+        DAIF::A
+    }
+}
+
+impl DaifField for IRQ {
+    fn daif_field() -> register::Field<u64, DAIF::Register> {
+        DAIF::I
+    }
+}
+
+impl DaifField for FIQ {
+    fn daif_field() -> register::Field<u64, DAIF::Register> {
+        DAIF::F
+    }
+}
+
+fn is_masked<T>() -> bool
+where
+    T: DaifField,
+{
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
@@ -0,0 +1,30 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Architectural synchronous and asynchronous exception handling.
+//!
+//! # Orientation
+//!
+//! Since arch modules are imported into generic modules using the path attribute, the path of this
+//! file is:
+//!
+//! crate::exception::arch_exception
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
@@ -0,0 +1,14 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Asynchronous exception handling.
+
+#[cfg(target_arch = "aarch64")]
+#[path = "../_arch/aarch64/exception/asynchronous.rs"]
+mod arch_asynchronous;
+
+//--------------------------------------------------------------------------------------------------
+// Architectural Public Reexports
+//--------------------------------------------------------------------------------------------------
+pub use arch_asynchronous::print_state;

diff -uNr 09_hw_debug_JTAG/src/exception.rs 10_privilege_level/src/exception.rs
--- 09_hw_debug_JTAG/src/exception.rs
+++ 10_privilege_level/src/exception.rs
@@ -0,0 +1,30 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Synchronous and asynchronous exception handling.
+
+#[cfg(target_arch = "aarch64")]
+#[path = "_arch/aarch64/exception.rs"]
+mod arch_exception;
+
+pub mod asynchronous;
+
+//--------------------------------------------------------------------------------------------------
+// Architectural Public Reexports
+//--------------------------------------------------------------------------------------------------
+pub use arch_exception::current_privilege_level;
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
@@ -118,6 +118,7 @@
 mod console;
 mod cpu;
 mod driver;
+mod exception;
 mod memory;
 mod panic_wait;
 mod print;
@@ -148,12 +149,20 @@

 /// The main function running after the early init.
 fn kernel_main() -> ! {
+    use bsp::console::console;
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
@@ -168,11 +177,15 @@
         info!("      {}. {}", i + 1, driver.compatible());
     }

-    // Test a failing timer case.
-    time::time_manager().spin_for(Duration::from_nanos(1));
+    info!("Timer test, spinning for 1 second");
+    time::time_manager().spin_for(Duration::from_secs(1));
+
+    info!("Echoing input now");

+    // Discard any spurious received characters before going into echo mode.
+    console().clear_rx();
     loop {
-        info!("Spinning for 1 second");
-        time::time_manager().spin_for(Duration::from_secs(1));
+        let c = bsp::console::console().read_char();
+        bsp::console::console().write_char(c);
     }
 }

```
