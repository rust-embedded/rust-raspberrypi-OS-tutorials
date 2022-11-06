# Tutorial 02 - Inicialización del `runtime`

## tl;dr

* Extendimos la funcionalidad de `boot.s` para que sea capaz de llamar código Rust por primera vez. Antes de que el cambio a Rust ocurra, se realizan algunos trabajos de inicialización del `runtime` (soporte para ejecución de código).
* El código Rust que es llamado solo pausa la ejecución con una llamada a `panic!()`.
* Ejecuta `make qemu` de nuevo para que puedas ver el código adicional en acción.

## Adiciones importantes

* Adiciones importantes al script `kernel.ld`:

  * Nuevas secciones: `.rodata`, `.got`, `.data`, `.bss`.

  * Un lugar totalmente dedicado a enlazar argumentos de tiempo de arranque (boot-time) que necesitan estar listos cuando se llame a `_start()`.

* `_start()` en `_arch/__arch_name__/cpu/boot.s`:

  1. Para todos los núcleos expecto el núcleo 0.

  2. Inicializa la [`DRAM`](https://es.wikipedia.org/wiki/DRAM) poniendo a cero la sección [`.bss`](https://en.wikipedia.org/wiki/.bss).

  3. Configura el `stack pointer` (puntero a la memoria [pila](https://es.wikipedia.org/wiki/Pila_(inform%C3%A1tica))).

  4. Salta hacia la función `_start_rust()`, definida en `arch/__arch_name__/cpu/boot.rs`.

* `_start_rust()`:

  * Llama a `kernel_init()`, que llama a `panic!()`, que al final también pone al núcleo 0 en pausa.

* La librería ahora usa el crate [aarch64-cpu](https://github.com/rust-embedded/aarch64-cpu), que nos da abstracciones sin coste y envuelve las partes que hacen uso de un `unsafe` (partes con código que no es seguro y podría causar errores) cuando se trabaja directamente con los recursos del procesador.

  * Lo puedes ver en acción en `_arch/__arch_name__/cpu.rs`.

## Diferencia con el archivo anterior

Please check [the english version](README.md#diff-to-previous), which is kept up-to-date.
