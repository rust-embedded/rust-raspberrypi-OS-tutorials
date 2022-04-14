# Tutorial 04 - Variables globales seguras

## tl;dr

* Se añade un pseudo-bloqueo.
* Esta es la primera vez que se muestra primitivas de sincronización del sistema operativo y habilita el acceso seguro a una estructura de datos global.

## Variables globales mutables en Rust

Cuando usamos la macro globalmente usabale `print!` en el [tutorial 03](../03_hacky_hello_world/README.ES.md), hicimos un poco de trampa. Llamando a la función `write_fmt()` de `core::fmt`, que toma una variable `&mut self`, esto solo funcionaba porque con cada llamada, se creaba una nueva instancia de `QEMUOutput`.

Si quisiéramos conservar algun estado, p. ej. estadísiticas acerca del número de carácteres que se han escrito, necesitamos crear una sola instancia global de `QEMUOutput` (en Rust, usando la palabra clave `static`).

Una `static QEMU_OUTPUT`, sin embargo, esto no nos permitiría llamar funciones que tomen `&mut self`. Para eso necesitaremos una `static mut`; pero llamar funciones que cambian de estado en una `static mut` es inseguro. El razonamiento del compilador de Rust para esta situación es que ya no puede evitar que múltiples núcleos/hilos cambien los datos al mismo tiempo (es una variable global, así que todos la pueden referenciar desde cualquier lugar. El inspector de préstamos o *borrow checker* no nos puede ayudar en esta situación).

La solución a este problema es hacerle un wrap a la variable global y convertirla en una primitiva de sincronización. En nuestro caso, una variante de una primitiva *MUTual EXclusion*. Se agrega `Mutex` como un rasgo (*trait*) en `synchronization.rs`, y es implementado por el `NullLock` en el mismo archivo. Para hacer que el código se acerque al propósito de la enseñanza, esto hace que dejemos afuera la lógica real dedicada a una arquitectura específica para la protección contra el acceso simultáneo, ya que no lo necesitamos mientras el kernel (núcleo) solo se ejecuta en un solo núcleo con las interrupciones desactivadas.

El `NullLock` se enfoca en mostrar el concepto principal de Rust de la [mutabilidad interior](https://doc.rust-lang.org/std/cell/index.html). Asegúrate de leerlo. También recomiendo este artículo acerca del [modelo mental preciso para las referencias de tipos en Rust.](https://docs.rs/dtolnay/0.0.6/dtolnay/macro._02__reference_types.html)

Si necesitas comparar el `NullLock` a una implementación real de mutex, puedes revisar las implementaciones en el [*spin crate*](https://github.com/mvdnes/spin-rs) o el [*parking lot crate*](https://github.com/Amanieu/parking_lot).

## Pruébalo

```textile
$ make qemu
[...]

[0] Hello from Rust!
[1] Chars written: 22
[2] Stopping here.
```

## Diferencias con el archivo anterior

```diff

diff -uNr 03_hacky_hello_world/Cargo.toml 04_safe_globals/Cargo.toml
--- 03_hacky_hello_world/Cargo.toml
+++ 04_safe_globals/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.3.0"
+version = "0.4.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2021"


diff -uNr 03_hacky_hello_world/src/bsp/raspberrypi/console.rs 04_safe_globals/src/bsp/raspberrypi/console.rs
--- 03_hacky_hello_world/src/bsp/raspberrypi/console.rs
+++ 04_safe_globals/src/bsp/raspberrypi/console.rs
@@ -4,7 +4,7 @@

 //! BSP console facilities.

-use crate::console;
+use crate::{console, synchronization, synchronization::NullLock};
 use core::fmt;

 //--------------------------------------------------------------------------------------------------
@@ -12,25 +12,64 @@
 //--------------------------------------------------------------------------------------------------

 /// A mystical, magical device for generating QEMU output out of the void.
-struct QEMUOutput;
+///
+/// The mutex protected part.
+struct QEMUOutputInner {
+    chars_written: usize,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// The main struct.
+pub struct QEMUOutput {
+    inner: NullLock<QEMUOutputInner>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();

 //--------------------------------------------------------------------------------------------------
 // Private Code
 //--------------------------------------------------------------------------------------------------

+impl QEMUOutputInner {
+    const fn new() -> QEMUOutputInner {
+        QEMUOutputInner { chars_written: 0 }
+    }
+
+    /// Send a character.
+    fn write_char(&mut self, c: char) {
+        unsafe {
+            core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
+        }
+
+        self.chars_written += 1;
+    }
+}
+
 /// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
 /// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
 /// we get `write_fmt()` automatically.
 ///
+/// The function takes an `&mut self`, so it must be implemented for the inner struct.
+///
 /// See [`src/print.rs`].
 ///
 /// [`src/print.rs`]: ../../print/index.html
-impl fmt::Write for QEMUOutput {
+impl fmt::Write for QEMUOutputInner {
     fn write_str(&mut self, s: &str) -> fmt::Result {
         for c in s.chars() {
-            unsafe {
-                core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
+            // Convert newline to carrige return + newline.
+            if c == '\n' {
+                self.write_char('\r')
             }
+
+            self.write_char(c);
         }

         Ok(())
@@ -41,7 +80,37 @@
 // Public Code
 //--------------------------------------------------------------------------------------------------

+impl QEMUOutput {
+    /// Create a new instance.
+    pub const fn new() -> QEMUOutput {
+        QEMUOutput {
+            inner: NullLock::new(QEMUOutputInner::new()),
+        }
+    }
+}
+
 /// Return a reference to the console.
-pub fn console() -> impl console::interface::Write {
-    QEMUOutput {}
+pub fn console() -> &'static impl console::interface::All {
+    &QEMU_OUTPUT
+}
+
+//------------------------------------------------------------------------------
+// OS Interface Code
+//------------------------------------------------------------------------------
+use synchronization::interface::Mutex;
+
+/// Passthrough of `args` to the `core::fmt::Write` implementation, but guarded by a Mutex to
+/// serialize access.
+impl console::interface::Write for QEMUOutput {
+    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
+        // Fully qualified syntax for the call to `core::fmt::Write::write:fmt()` to increase
+        // readability.
+        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
+    }
+}
+
+impl console::interface::Statistics for QEMUOutput {
+    fn chars_written(&self) -> usize {
+        self.inner.lock(|inner| inner.chars_written)
+    }
 }

diff -uNr 03_hacky_hello_world/src/console.rs 04_safe_globals/src/console.rs
--- 03_hacky_hello_world/src/console.rs
+++ 04_safe_globals/src/console.rs
@@ -10,10 +10,22 @@

 /// Console interfaces.
 pub mod interface {
+    use core::fmt;
+
     /// Console write functions.
-    ///
-    /// `core::fmt::Write` is exactly what we need for now. Re-export it here because
-    /// implementing `console::Write` gives a better hint to the reader about the
-    /// intention.
-    pub use core::fmt::Write;
+    pub trait Write {
+        /// Write a Rust format string.
+        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
+    }
+
+    /// Console statistics.
+    pub trait Statistics {
+        /// Return the number of characters written.
+        fn chars_written(&self) -> usize {
+            0
+        }
+    }
+
+    /// Trait alias for a full-fledged console.
+    pub trait All = Write + Statistics;
 }

diff -uNr 03_hacky_hello_world/src/main.rs 04_safe_globals/src/main.rs
--- 03_hacky_hello_world/src/main.rs
+++ 04_safe_globals/src/main.rs
@@ -106,6 +106,7 @@

 #![feature(format_args_nl)]
 #![feature(panic_info_message)]
+#![feature(trait_alias)]
 #![no_main]
 #![no_std]

@@ -114,6 +115,7 @@
 mod cpu;
 mod panic_wait;
 mod print;
+mod synchronization;

 /// Early init code.
 ///
@@ -121,7 +123,15 @@
 ///
 /// - Only a single core must be active and running this function.
 unsafe fn kernel_init() -> ! {
-    println!("Hello from Rust!");
+    use console::interface::Statistics;

-    panic!("Stopping here.")
+    println!("[0] Hello from Rust!");
+
+    println!(
+        "[1] Chars written: {}",
+        bsp::console::console().chars_written()
+    );
+
+    println!("[2] Stopping here.");
+    cpu::wait_forever()
 }

diff -uNr 03_hacky_hello_world/src/synchronization.rs 04_safe_globals/src/synchronization.rs
--- 03_hacky_hello_world/src/synchronization.rs
+++ 04_safe_globals/src/synchronization.rs
@@ -0,0 +1,77 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>
+
+//! Synchronization primitives.
+//!
+//! # Resources
+//!
+//!   - <https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html>
+//!   - <https://stackoverflow.com/questions/59428096/understanding-the-send-trait>
+//!   - <https://doc.rust-lang.org/std/cell/index.html>
+
+use core::cell::UnsafeCell;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Synchronization interfaces.
+pub mod interface {
+
+    /// Any object implementing this trait guarantees exclusive access to the data wrapped within
+    /// the Mutex for the duration of the provided closure.
+    pub trait Mutex {
+        /// The type of the data that is wrapped by this mutex.
+        type Data;
+
+        /// Locks the mutex and grants the closure temporary mutable access to the wrapped data.
+        fn lock<R>(&self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
+    }
+}
+
+/// A pseudo-lock for teaching purposes.
+///
+/// In contrast to a real Mutex implementation, does not protect against concurrent access from
+/// other cores to the contained data. This part is preserved for later lessons.
+///
+/// The lock will only be used as long as it is safe to do so, i.e. as long as the kernel is
+/// executing single-threaded, aka only running on a single core with interrupts disabled.
+pub struct NullLock<T>
+where
+    T: ?Sized,
+{
+    data: UnsafeCell<T>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+unsafe impl<T> Send for NullLock<T> where T: ?Sized + Send {}
+unsafe impl<T> Sync for NullLock<T> where T: ?Sized + Send {}
+
+impl<T> NullLock<T> {
+    /// Create an instance.
+    pub const fn new(data: T) -> Self {
+        Self {
+            data: UnsafeCell::new(data),
+        }
+    }
+}
+
+//------------------------------------------------------------------------------
+// OS Interface Code
+//------------------------------------------------------------------------------
+
+impl<T> interface::Mutex for NullLock<T> {
+    type Data = T;
+
+    fn lock<R>(&self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
+        // In a real lock, there would be code encapsulating this line that ensures that this
+        // mutable reference will ever only be given out once at a time.
+        let data = unsafe { &mut *self.data.get() };
+
+        f(data)
+    }
+}

```
