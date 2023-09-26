# Tutorial 01 - Esperar infinitamente

## tl;dr

* Se configura la estructura que tiene el proyecto.

* Se ejecuta una pequeño código hecho en ensamblador que tiene como función detener todos los núcleos del procesador que están ejecutando el kernel.

## Compilar

* El archivo `Makefile` permite ejecutar:

  * `doc`: Genera la documentación.

  * `qemu`: Ejecutar el kernel en QEMU.

  * `clippy`: Analiza el código y sugiere mejoras.

  * `clean`: Elimina todos los archivos generados durante la compilación, etc.

  * `readelf`: Inspecciona el archivo `ELF` de salida.

  * `objdump`: Inspecciona el ensamblador.

  * `nm`: Inspecciona los símbolos.

## Código a revisar

* El script para enlazado específico para la `BSP` llamado `kernel.ld`.

  * Carga la dirección en `0x8_0000`.

  * Solo la sección `.text`.

* `main.rs`: [Atributos internos](https://doc.rust-lang.org/reference/attributes.html) importantes:

  * `#![no_std]`, `#![no_main]`.

* `boot.s`: La función de ensamblador `_start()` que inicia `wfe` (Wait For Event / Esperar Hasta Un Evento), detiene todos los núcleos del procesador que están ejecutando `_start()`.

* Tenemos que definir una función que funcione como `#[panic_handler]` (manejador de pánico) para que el compilador no nos cause problemas.

  * Hazla `unimplemented!()` porque se eliminará ya que no está siendo usada.

## Pruébalo

Dentro de la carpeta del proyecto, ejecuta a QEMU y mira el núcleo del procesador ejecutando `wfe` en bucle:

```
$ make qemu
[...]
IN:
0x00080000:  d503205f  wfe
0x00080004:  17ffffff  b        #0x80000
```
