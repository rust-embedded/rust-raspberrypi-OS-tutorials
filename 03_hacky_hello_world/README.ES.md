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

Please check [the english version](README.md#diff-to-previous), which is kept up-to-date.
