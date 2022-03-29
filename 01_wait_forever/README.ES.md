# Tutorial 01 - Esperar para siempre

## tl;dr

* Se configura la estructura que tendrá el proyecto.

* Se ejecuta una pequeño código hecho en ensamblador que tiene como función detener todos los núcleos del procesador que están ejecutando el kernel.

## Compilar

* El archivo `Makefile` selecciona:
  
  * `doc`: Genera la documentación.
  
  * `qemu`: Ejecutar el kernel en QEMU.
  
  * `clippy`
  
  * `clean`
  
  * `readelf`: Inspeccionar la salida de `ELF`.
  
  * `objdump`: Inspecciona el ensamblador. 
  
  * `nm`: Inspecciona los símbolos.

## Código a revisar

* El script enlazador específico de `BSP` llamado `link.ld`.
  
  * Carga la dirección en `0x8_0000`.
  
  * Solo la sección `.text`.

* `main.rs`: [Atributos internos](https://doc.rust-lang.org/reference/attributes.html) importantes:
  
  * `#![no_std]`, `#![no_main]`.

* `boot.s`: La función de ensamblador `__start()` que inicia `wfe` (Wait For Event / Esperar Por Un Evento), detiene todos los núcleos del procesador que están ejecutando `_start()`. 

* Tenemos que definir una función `#[panic_handler]` para que el compilador no nos cause problemas.
  
  * Hazla una `unimplemented!()` porque se eliminará ya que no está siendo usada.

## Pruébalo

Dentro de la carpeta del proyecto, ejecuta a QEMU y mira el núcleo del procesador hilado en `wfe`:

```
$ make qemu
[...]
IN:
0x00080000:  d503205f  wfe
0x00080004:  17ffffff  b        #0x80000
```
