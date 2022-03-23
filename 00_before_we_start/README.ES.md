# Antes de comenzar

El texto a continuación es una copia 1:1 de la documentación que 
puede ser encontrada al principio del archivo del código fuente 
del núcleo (kernel) en cada tutorial. Esta describe la estructura 
general del código fuente, e intenta transmitir la filosofía detrás
de cada respectivo acercamiento. Por favor leélo para familiarizarte
con lo que te vas a encontrar durante los tutoriales. Te ayudará a navegar el código de una mejor manera y entender las diferencias y adiciones entre los diferentes tutoriales.

Por favor también nota que el siguiente texto va a referenciar
los archivos del código fuente (e.j. `**/memory.rs`) o funciones que
no van a existir aún en los primeros tutoriales. Estos archivos serán agregados 
a medida que el tutorial avance.

¡Diviértanse!

# La estructura del código y la arquitectura

El código está dividido en diferentes módulos, cada uno representa un
subsistema típico del `kernel (núcleo)`. Los módulos de más alto nivel de los subsistemas se encuentran directamente en la carpeta `src`.
Por ejemplo, `src/memory.rs` contiene el código que está relacionado
con el manejo de memoria.

## Visibilidad del código de arquitectura del procesador

Algunos de los subsistemas del `núcleo (kernel)` dependen del código de nivel-bajo (low-level) que tiene como objetivo la arquitectura del procesador.
Por cada arquitectura de procesador que está soportada, existe una subcarpeta en `src/_arch`, por ejemplo, `src/_arch/aarch64`.

La carpeta de arquitecturas refleja los módulos del subsistema establecidos en `src`. Por ejemplo, el código de arquitectura que pertenece al subsistema MMU del `núcleo(kernel)` (`src/memory/mmu.rs`) irá dentro de (`src/_arch/aarch64/memory/mmu.rs`).
El último archivo cargado como un módulo en `src/memory/mmu.rs` usando el `path attribute` (atributo de ruta). Usualmente, el nombre del módulo elegido es el nombre del módulo genérico con el prefijo de `arch_`

Por ejemplo, esta es la parte superior de `src/memory/mmu.rs`:

```
#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/memory/mmu.rs"]
mod arch_mmu;
```

En muchas ocasiones, los elementos de `arch_module` serán reexportados públicamente por el módulo principal.
De esta manera, cada módulo específico de la arquitectura puede proporcionar su implementación de un elemento, mientras que el *caller* no debe de preocuparse por la arquitectura que se ha compilado condicionalmente.

## Código BSP

`BSP` significa Board Support Package (Paquete de Soporte de la Placa).
El código `BSP` está dentro de `src/bsp.rs` y contiene las definiciones y funciones de la placa base específica a la que se tendrá como objetivo. 
Entre estas cosas se encuentran diferentes elementos como el mapa de memoria de la placa o instancias de controladores para dispositivos que se presentan en la placa respectiva.

Justo como el código de la arquitectura del procesador, la estructura del módulo del código `BSP` trata de reflejar los módulos del subsistema del `núcleo (kernel)`, pero no ocurre una reexportación esta vez. Eso significa que lo que sea que se esté proporcionando debe ser llamado empezando por el *namespace* de `bsp`, e.j. `bsp::driver::driver_manager()`.

## La interfaz del núcleo (kernel)

El `arch` y el `bsp` contienen código que se compilará condicionalmente dependiendo del objetivo y placa actual para la que el núcleo (kernel) es compilado.
Por ejemplo, el hardware de `interrupt controller` de la `Raspberry Pi 3`
y la  `Raspberry Pi 4` es diferente, pero nosotros queremos que el resto del código del kernel funcione correctamente con cualquiera de los dos sin mucha complicación.

Para poder dar una limpia abstracción entre `arch`, `bsp` y `generic kernel code`, los rasgos de `interface` se proporcionan *siempre y cuando tenga sentido*. Son definidos en su respectivo módulo de subsistema y ayuda a reforzar el idioma de *program to an interface* *(programa a una interface)*, no a una implementación. 

Por ejemplo, habrá una *IRQ handling interface* común, el cual los dos diferentes `drivers` de `interrupt controller` de ambas `Raspberry` implementarán, y solo exportarán la interface del resto del `núcleo (kernel)`.

```
        +-------------------+
        | Interface (Trait) |
        |                   |
        +--+-------------+--+
           ^             ^
           |             |
           |             |
+----------+--+       +--+----------+
| kernel code |       |  bsp code   |
|             |       |  arch code  |
+-------------+       +-------------+
```

# Resumen

Para un subsistema lógico del `núcleo (kernel)`, el código correspondiente puede ser distribuido sobre diferentes localizaciones físicas. Aquí un ejemplo para el subsistema de memoria:

- `src/memory.rs` y `src/memory/**/*`
  
  - Código común que es independiente de la arquitectura del procesador de destino y las características de `BSP`.
    - Ejemplo: Una función a un pedazo cero de memoria.
  - Las interfaces para el subsistema de la memoria que son implementados por código `arch` o `BSP`.
    - Ejemplo: Una interface `MMU` que define prototipos de función `MMU`.

- `src/bsp/__board_name__/memory.rs` y `src/bsp/__board_name__/memory/**/*`
  
  - Código específico de `BSP`.
  - Ejemplo: El mapa de memoria de la placa (direcciones físicas de DRAM y dispositivos MMIO).

- `src/_arch/__arch_name__/memory.rs` y `src/_arch/__arch_name__/memory/**/*`
  
  - El código específico de la arquitectura del procesador.
  - Ejemplo: Implementación de la interfaz `MMU` para la arquitectura `__arch_name__`.

Desde una perspectiva de *namespace*, el código del subsistema de **memoria** vive en:

- `crate::memory::*`
- `crate::bsp::memory::*`

# Flujo de Boot / Boot flow

1. La punto de entrada del núcleo (kernel) es la función `cpu::boot::arch_boot::_start()`.
   - Está implementada en `src/_arch/__arch_name__/cpu/boot.s`.
