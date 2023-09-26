# Tutorial 09 - Privilege Level

## tl;dr

- In early boot code, we transition from the `Hypervisor` privilege level (`EL2` in AArch64) to the
  `Kernel` (`EL1`) privilege level.

## Table of Contents

- [Introduction](#introduction)
- [Scope of this tutorial](#scope-of-this-tutorial)
- [Checking for EL2 in the entrypoint](#checking-for-el2-in-the-entrypoint)
- [Transition preparation](#transition-preparation)
- [Returning from an exception that never happened](#returning-from-an-exception-that-never-happened)
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

[Programmer’s Guide for ARMv8-A]: http://infocenter.arm.com/help/topic/com.arm.doc.den0024a/DEN0024A_v8_architecture_PG.pdf

## Scope of this tutorial

By default, the Raspberry will always start executing in `EL2`. Since we are writing a traditional
`Kernel`, we have to transition into the more appropriate `EL1`.

## Checking for EL2 in the entrypoint

First of all, we need to ensure that we actually execute in `EL2` before we can call respective code
to transition to `EL1`. Therefore, we add a new checkt to the top of `boot.s`, which parks the CPU
core should it not be in `EL2`.

```
// Only proceed if the core executes in EL2. Park it otherwise.
mrs	x0, CurrentEL
cmp	x0, {CONST_CURRENTEL_EL2}
b.ne	.L_parking_loop
```

Afterwards, we continue with preparing the `EL2` -> `EL1` transition by calling
`prepare_el2_to_el1_transition()` in `boot.rs`:

```rust
#[no_mangle]
pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
    prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);

    // Use `eret` to "return" to EL1. This results in execution of kernel_init() in EL1.
    asm::eret()
}
```

## Transition preparation

Since `EL2` is more privileged than `EL1`, it has control over various processor features and can
allow or disallow `EL1` code to use them. One such example is access to timer and counter registers.
We are already using them since [tutorial 07](../07_timestamps/), so of course we want to keep them.
Therefore we set the respective flags in the [Counter-timer Hypervisor Control register] and
additionally set the virtual offset to zero so that we get the real physical value everytime:

[Counter-timer Hypervisor Control register]:  https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/cnthctl_el2.rs.html

```rust
// Enable timer counter registers for EL1.
CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

// No offset for reading the counters.
CNTVOFF_EL2.set(0);
```

Next, we configure the [Hypervisor Configuration Register] such that `EL1` runs in `AArch64` mode,
and not in `AArch32`, which would also be possible.

[Hypervisor Configuration Register]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/hcr_el2.rs.html

```rust
// Set EL1 execution state to AArch64.
HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
```

## Returning from an exception that never happened

There is actually only one way to transition from a higher EL to a lower EL, which is by way of
executing the [ERET] instruction.

[ERET]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/asm.rs.html#92-101

This instruction will copy the contents of the [Saved Program Status Register - EL2] to `Current
Program Status Register - EL1` and jump to the instruction address that is stored in the [Exception
Link Register - EL2].

This is basically the reverse of what is happening when an exception is taken. You'll learn about
that in an upcoming tutorial.

[Saved Program Status Register - EL2]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/spsr_el2.rs.html
[Exception Link Register - EL2]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/elr_el2.rs.html

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

// Second, let the link register point to kernel_init().
ELR_EL2.set(crate::kernel_init as *const () as u64);

// Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it. Since there
// are no plans to ever return to EL2, just re-use the same stack.
SP_EL1.set(phys_boot_core_stack_end_exclusive_addr);
```

As you can see, we are populating `ELR_EL2` with the address of the `kernel_init()` function that we
earlier used to call directly from the entrypoint. Finally, we set the stack pointer for `SP_EL1`.

You might have noticed that the stack's address was supplied as a function argument. As you might
remember, in  `_start()` in `boot.s`, we are already setting up the stack for `EL2`. Since there
are no plans to ever return to `EL2`, we can just re-use the same stack for `EL1`, so its address is
forwarded using function arguments.

Lastly, back in `_start_rust()` a call to `ERET` is made:

```rust
#[no_mangle]
pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
    prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);

    // Use `eret` to "return" to EL1. This results in execution of kernel_init() in EL1.
    asm::eret()
}
```

## Test it

In `main.rs`, we print the `current privilege level` and additionally inspect if the mask bits in
`SPSR_EL2` made it to `EL1` as well:

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
[MP] ⏩ Pushing 14 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.162546] mingo version 0.9.0
[    0.162745] Booting on: Raspberry Pi 3
[    0.163201] Current privilege level: EL1
[    0.163677] Exception handling state:
[    0.164122]       Debug:  Masked
[    0.164511]       SError: Masked
[    0.164901]       IRQ:    Masked
[    0.165291]       FIQ:    Masked
[    0.165681] Architectural timer resolution: 52 ns
[    0.166255] Drivers loaded:
[    0.166592]       1. BCM PL011 UART
[    0.167014]       2. BCM GPIO
[    0.167371] Timer test, spinning for 1 second
[    1.167904] Echoing input now
```

## Diff to previous
```diff

diff -uNr 08_hw_debug_JTAG/Cargo.toml 09_privilege_level/Cargo.toml
--- 08_hw_debug_JTAG/Cargo.toml
+++ 09_privilege_level/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.8.0"
+version = "0.9.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2021"


diff -uNr 08_hw_debug_JTAG/src/_arch/aarch64/cpu/boot.rs 09_privilege_level/src/_arch/aarch64/cpu/boot.rs
--- 08_hw_debug_JTAG/src/_arch/aarch64/cpu/boot.rs
+++ 09_privilege_level/src/_arch/aarch64/cpu/boot.rs
@@ -11,22 +11,73 @@
 //!
 //! crate::cpu::boot::arch_boot

+use aarch64_cpu::{asm, registers::*};
 use core::arch::global_asm;
+use tock_registers::interfaces::Writeable;

 // Assembly counterpart to this file.
 global_asm!(
     include_str!("boot.s"),
+    CONST_CURRENTEL_EL2 = const 0x8,
     CONST_CORE_ID_MASK = const 0b11
 );

 //--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+/// Prepares the transition from EL2 to EL1.
+///
+/// # Safety
+///
+/// - The `bss` section is not initialized yet. The code must not use or reference it in any way.
+/// - The HW state of EL1 must be prepared in a sound way.
+#[inline(always)]
+unsafe fn prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr: u64) {
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
+    // Second, let the link register point to kernel_init().
+    ELR_EL2.set(crate::kernel_init as *const () as u64);
+
+    // Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it. Since there
+    // are no plans to ever return to EL2, just re-use the same stack.
+    SP_EL1.set(phys_boot_core_stack_end_exclusive_addr);
+}
+
+//--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------

 /// The Rust entry of the `kernel` binary.
 ///
 /// The function is called from the assembly `_start` function.
+///
+/// # Safety
+///
+/// - Exception return from EL2 must must continue execution in EL1 with `kernel_init()`.
 #[no_mangle]
-pub unsafe fn _start_rust() -> ! {
-    crate::kernel_init()
+pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
+    prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);
+
+    // Use `eret` to "return" to EL1. This results in execution of kernel_init() in EL1.
+    asm::eret()
 }

diff -uNr 08_hw_debug_JTAG/src/_arch/aarch64/cpu/boot.s 09_privilege_level/src/_arch/aarch64/cpu/boot.s
--- 08_hw_debug_JTAG/src/_arch/aarch64/cpu/boot.s
+++ 09_privilege_level/src/_arch/aarch64/cpu/boot.s
@@ -27,11 +27,16 @@
 // fn _start()
 //------------------------------------------------------------------------------
 _start:
+	// Only proceed if the core executes in EL2. Park it otherwise.
+	mrs	x0, CurrentEL
+	cmp	x0, {CONST_CURRENTEL_EL2}
+	b.ne	.L_parking_loop
+
 	// Only proceed on the boot core. Park it otherwise.
-	mrs	x0, MPIDR_EL1
-	and	x0, x0, {CONST_CORE_ID_MASK}
-	ldr	x1, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
-	cmp	x0, x1
+	mrs	x1, MPIDR_EL1
+	and	x1, x1, {CONST_CORE_ID_MASK}
+	ldr	x2, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
+	cmp	x1, x2
 	b.ne	.L_parking_loop

 	// If execution reaches here, it is the boot core.
@@ -48,7 +53,7 @@

 	// Prepare the jump to Rust code.
 .L_prepare_rust:
-	// Set the stack pointer.
+	// Set the stack pointer. This ensures that any code in EL2 that needs the stack will work.
 	ADR_REL	x0, __boot_core_stack_end_exclusive
 	mov	sp, x0

@@ -60,7 +65,7 @@
 	b.eq	.L_parking_loop
 	str	w2, [x1]

-	// Jump to Rust code.
+	// Jump to Rust code. x0 holds the function argument provided to _start_rust().
 	b	_start_rust

 	// Infinitely wait for events (aka "park the core").

diff -uNr 08_hw_debug_JTAG/src/_arch/aarch64/exception/asynchronous.rs 09_privilege_level/src/_arch/aarch64/exception/asynchronous.rs
--- 08_hw_debug_JTAG/src/_arch/aarch64/exception/asynchronous.rs
+++ 09_privilege_level/src/_arch/aarch64/exception/asynchronous.rs
@@ -0,0 +1,82 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
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
+use aarch64_cpu::registers::*;
+use tock_registers::interfaces::Readable;
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+trait DaifField {
+    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register>;
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
+    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
+        DAIF::D
+    }
+}
+
+impl DaifField for SError {
+    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
+        DAIF::A
+    }
+}
+
+impl DaifField for IRQ {
+    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
+        DAIF::I
+    }
+}
+
+impl DaifField for FIQ {
+    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
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

diff -uNr 08_hw_debug_JTAG/src/_arch/aarch64/exception.rs 09_privilege_level/src/_arch/aarch64/exception.rs
--- 08_hw_debug_JTAG/src/_arch/aarch64/exception.rs
+++ 09_privilege_level/src/_arch/aarch64/exception.rs
@@ -0,0 +1,31 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
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
+use aarch64_cpu::registers::*;
+use tock_registers::interfaces::Readable;
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

diff -uNr 08_hw_debug_JTAG/src/exception/asynchronous.rs 09_privilege_level/src/exception/asynchronous.rs
--- 08_hw_debug_JTAG/src/exception/asynchronous.rs
+++ 09_privilege_level/src/exception/asynchronous.rs
@@ -0,0 +1,14 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>
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

diff -uNr 08_hw_debug_JTAG/src/exception.rs 09_privilege_level/src/exception.rs
--- 08_hw_debug_JTAG/src/exception.rs
+++ 09_privilege_level/src/exception.rs
@@ -0,0 +1,30 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>
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
+#[derive(Eq, PartialEq)]
+pub enum PrivilegeLevel {
+    User,
+    Kernel,
+    Hypervisor,
+    Unknown,
+}

diff -uNr 08_hw_debug_JTAG/src/main.rs 09_privilege_level/src/main.rs
--- 08_hw_debug_JTAG/src/main.rs
+++ 09_privilege_level/src/main.rs
@@ -121,6 +121,7 @@
 mod console;
 mod cpu;
 mod driver;
+mod exception;
 mod panic_wait;
 mod print;
 mod synchronization;
@@ -148,6 +149,7 @@

 /// The main function running after the early init.
 fn kernel_main() -> ! {
+    use console::console;
     use core::time::Duration;

     info!(
@@ -157,6 +159,12 @@
     );
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
@@ -165,11 +173,15 @@
     info!("Drivers loaded:");
     driver::driver_manager().enumerate();

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
+        let c = console().read_char();
+        console().write_char(c);
     }
 }

diff -uNr 08_hw_debug_JTAG/tests/boot_test_string.rb 09_privilege_level/tests/boot_test_string.rb
--- 08_hw_debug_JTAG/tests/boot_test_string.rb
+++ 09_privilege_level/tests/boot_test_string.rb
@@ -1,3 +1,3 @@
 # frozen_string_literal: true

-EXPECTED_PRINT = 'Spinning for 1 second'
+EXPECTED_PRINT = 'Echoing input now'

```
