# Tutorial 03 - Hacky Hello World

## tl;dr

* Se añade la macro global `print!()` para habilitar la "depuración basada en printf" ("printf debugging") lo más pronto posible.
* Para mantener una duración razonable en este tutorial, las funciones de impresión por el momento "abusan" una propiedad de QEMU que nos permite hacer uso del `UART` de la Raspberry sin haberla configurado apropiadamente.
* El uso del hardware real de `UART` se habilitará paso por paso en los siguientes tutoriales.

## Adiciones notables

* `src/console.rs` introduce una interfaz con `Trait`s para comandos de consola.
* `src/bsp/raspberrypi/console.rs` implementa la interfaz para que QEMU pueda crear una emulación de UART.
* El *panic handler* (manejador de pánico) hace uso de la nueva macro `print!()` para mostrar mensajes de error del usuario.
* Hay un nuevo objetivo en el Makefile: `make test`, destinado a la automatización de pruebas. Este comando inicia el kernel (núcleo) compilado en `QEMU`, y busca una cadena de  texto específica en la salida que ha sido producida por el kernel (núcleo).
  * En este tutorial, se buscará la cadena `Stopping here`, que es creada por la macro `panic!()` al final del archivo `main.rs`.

## Pruébalo

QEMU ya no está siendo ejecutado en modo ensamblador. Desde ahora en adelante mostrará la salida de la `consola`.

```console
$ make qemu
[...]
Hello from Rust!

Kernel panic: Stopping here.
```

### Diccionario

* *Hacky:* Solución torpe o poco elegante para un problema.

* *Debugging:* Proceso para identificar y corregir errores de programación.
  
  * *printf debugging:* Usado para describir el trabajo de depuración (*debugging*) poniendo comandos que dan una salida en consola, como el de "printf", en diferentes lugares del programa; observando la información y tratando de deducir qué está mal en el programa basándose en la información que nos dan nuestros comandos.

* *Traits:* Un *trait* le hace saber al compilador de Rust acerca de una funcionalidad que tiene un tipo de dato particular y que puede compartir con otros tipos de datos.
  
  > NOTA: Los *traits* son similares a una característica que se le conoce comúnmente como *interfaces* en otros lenguajes, aunque con algunas diferencias.
  
  Si deseas aprender más acerca de esto, por favor lee este capítulo del libro de Rust: [Traits: Defining Shared Behavior - The Rust Programming Language](https://doc.rust-lang.org/book/ch10-02-traits.html)

## Diferencias con el archivo anterior

```diff
diff -uNr 02_runtime_init/Cargo.toml 03_hacky_hello_world/Cargo.toml
--- 02_runtime_init/Cargo.toml
+++ 03_hacky_hello_world/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.2.0"
+version = "0.3.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2021"


diff -uNr 02_runtime_init/Makefile 03_hacky_hello_world/Makefile
--- 02_runtime_init/Makefile
+++ 03_hacky_hello_world/Makefile
@@ -24,7 +24,7 @@
     KERNEL_BIN        = kernel8.img
     QEMU_BINARY       = qemu-system-aarch64
     QEMU_MACHINE_TYPE = raspi3
-    QEMU_RELEASE_ARGS = -d in_asm -display none
+    QEMU_RELEASE_ARGS = -serial stdio -display none
     OBJDUMP_BINARY    = aarch64-none-elf-objdump
     NM_BINARY         = aarch64-none-elf-nm
     READELF_BINARY    = aarch64-none-elf-readelf
@@ -35,7 +35,7 @@
     KERNEL_BIN        = kernel8.img
     QEMU_BINARY       = qemu-system-aarch64
     QEMU_MACHINE_TYPE =
-    QEMU_RELEASE_ARGS = -d in_asm -display none
+    QEMU_RELEASE_ARGS = -serial stdio -display none
     OBJDUMP_BINARY    = aarch64-none-elf-objdump
     NM_BINARY         = aarch64-none-elf-nm
     READELF_BINARY    = aarch64-none-elf-readelf
@@ -71,17 +71,20 @@
     --strip-all            \
     -O binary

-EXEC_QEMU = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+EXEC_QEMU          = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+EXEC_TEST_DISPATCH = ruby ../common/tests/dispatch.rb

 ##------------------------------------------------------------------------------
 ## Dockerization
 ##------------------------------------------------------------------------------
-DOCKER_CMD          = docker run -t --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
-DOCKER_CMD_INTERACT = $(DOCKER_CMD) -i
+DOCKER_CMD            = docker run -t --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
+DOCKER_CMD_INTERACT   = $(DOCKER_CMD) -i
+DOCKER_ARG_DIR_COMMON = -v $(shell pwd)/../common:/work/common

 # DOCKER_IMAGE defined in include file (see top of this file).
 DOCKER_QEMU  = $(DOCKER_CMD_INTERACT) $(DOCKER_IMAGE)
 DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)
+DOCKER_TEST  = $(DOCKER_CMD) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)



@@ -169,3 +172,28 @@
 ##------------------------------------------------------------------------------
 check:
     @RUSTFLAGS="$(RUSTFLAGS)" $(CHECK_CMD) --message-format=json
+
+
+
+##--------------------------------------------------------------------------------------------------
+## Testing targets
+##--------------------------------------------------------------------------------------------------
+.PHONY: test test_boot
+
+ifeq ($(QEMU_MACHINE_TYPE),) # QEMU is not supported for the board.
+
+test_boot test :
+    $(call colorecho, "\n$(QEMU_MISSING_STRING)")
+
+else # QEMU is supported.
+
+##------------------------------------------------------------------------------
+## Run boot test
+##------------------------------------------------------------------------------
+test_boot: $(KERNEL_BIN)
+    $(call colorecho, "\nBoot test - $(BSP)")
+    @$(DOCKER_TEST) $(EXEC_TEST_DISPATCH) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
+
+test: test_boot
+
+endif

diff -uNr 02_runtime_init/src/bsp/raspberrypi/console.rs 03_hacky_hello_world/src/bsp/raspberrypi/console.rs
--- 02_runtime_init/src/bsp/raspberrypi/console.rs
+++ 03_hacky_hello_world/src/bsp/raspberrypi/console.rs
@@ -0,0 +1,47 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSP console facilities.
+
+use crate::console;
+use core::fmt;
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// A mystical, magical device for generating QEMU output out of the void.
+struct QEMUOutput;
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
+/// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
+/// we get `write_fmt()` automatically.
+///
+/// See [`src/print.rs`].
+///
+/// [`src/print.rs`]: ../../print/index.html
+impl fmt::Write for QEMUOutput {
+    fn write_str(&mut self, s: &str) -> fmt::Result {
+        for c in s.chars() {
+            unsafe {
+                core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
+            }
+        }
+
+        Ok(())
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Return a reference to the console.
+pub fn console() -> impl console::interface::Write {
+    QEMUOutput {}
+}

diff -uNr 02_runtime_init/src/bsp/raspberrypi.rs 03_hacky_hello_world/src/bsp/raspberrypi.rs
--- 02_runtime_init/src/bsp/raspberrypi.rs
+++ 03_hacky_hello_world/src/bsp/raspberrypi.rs
@@ -4,4 +4,5 @@

 //! Top-level BSP file for the Raspberry Pi 3 and 4.

+pub mod console;
 pub mod cpu;

diff -uNr 02_runtime_init/src/console.rs 03_hacky_hello_world/src/console.rs
--- 02_runtime_init/src/console.rs
+++ 03_hacky_hello_world/src/console.rs
@@ -0,0 +1,19 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>
+
+//! System console.
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Console interfaces.
+pub mod interface {
+    /// Console write functions.
+    ///
+    /// `core::fmt::Write` is exactly what we need for now. Re-export it here because
+    /// implementing `console::Write` gives a better hint to the reader about the
+    /// intention.
+    pub use core::fmt::Write;
+}

diff -uNr 02_runtime_init/src/main.rs 03_hacky_hello_world/src/main.rs
--- 02_runtime_init/src/main.rs
+++ 03_hacky_hello_world/src/main.rs
@@ -104,12 +104,16 @@
 //!     - It is implemented in `src/_arch/__arch_name__/cpu/boot.s`.
 //! 2. Once finished with architectural setup, the arch code calls `kernel_init()`.

+#![feature(format_args_nl)]
+#![feature(panic_info_message)]
 #![no_main]
 #![no_std]

 mod bsp;
+mod console;
 mod cpu;
 mod panic_wait;
+mod print;

 /// Early init code.
 ///
@@ -117,5 +121,7 @@
 ///
 /// - Only a single core must be active and running this function.
 unsafe fn kernel_init() -> ! {
-    panic!()
+    println!("[0] Hello from Rust!");
+
+    panic!("Stopping here.")
 }

diff -uNr 02_runtime_init/src/panic_wait.rs 03_hacky_hello_world/src/panic_wait.rs
--- 02_runtime_init/src/panic_wait.rs
+++ 03_hacky_hello_world/src/panic_wait.rs
@@ -4,10 +4,16 @@

 //! A panic handler that infinitely waits.

-use crate::cpu;
+use crate::{cpu, println};
 use core::panic::PanicInfo;

 #[panic_handler]
-fn panic(_info: &PanicInfo) -> ! {
+fn panic(info: &PanicInfo) -> ! {
+    if let Some(args) = info.message() {
+        println!("\nKernel panic: {}", args);
+    } else {
+        println!("\nKernel panic!");
+    }
+
     cpu::wait_forever()
 }

diff -uNr 02_runtime_init/src/print.rs 03_hacky_hello_world/src/print.rs
--- 02_runtime_init/src/print.rs
+++ 03_hacky_hello_world/src/print.rs
@@ -0,0 +1,38 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>
+
+//! Printing.
+
+use crate::{bsp, console};
+use core::fmt;
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+#[doc(hidden)]
+pub fn _print(args: fmt::Arguments) {
+    use console::interface::Write;
+
+    bsp::console::console().write_fmt(args).unwrap();
+}
+
+/// Prints without a newline.
+///
+/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
+#[macro_export]
+macro_rules! print {
+    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
+}
+
+/// Prints with a newline.
+///
+/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
+#[macro_export]
+macro_rules! println {
+    () => ($crate::print!("\n"));
+    ($($arg:tt)*) => ({
+        $crate::print::_print(format_args_nl!($($arg)*));
+    })
+}

diff -uNr 02_runtime_init/tests/boot_test_string.rb 03_hacky_hello_world/tests/boot_test_string.rb
--- 02_runtime_init/tests/boot_test_string.rb
+++ 03_hacky_hello_world/tests/boot_test_string.rb
@@ -0,0 +1,3 @@
+# frozen_string_literal: true
+
+EXPECTED_PRINT = 'Stopping here'
```
