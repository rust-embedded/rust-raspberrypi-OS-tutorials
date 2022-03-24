# Antes de comenzar

El texto a continuación es una copia 1:1 de la documentación que 
puede ser encontrada al principio del archivo del código fuente 
del núcleo (kernel) en cada tutorial. Esta describe la estructura 
general del código fuente, e intenta transmitir la filosofía detrás
de cada enfoque. Por favor leélo para familiarizarte
con lo que te vas a encontrar durante los tutoriales. Te ayudará a navegar el código de una mejor manera y a entender las diferencias y agregados entre los diferentes tutoriales.

Por favor, nota también que el siguiente texto va a referenciar
los archivos del código fuente (p. e.j. `**/memory.rs`) o funciones que
no van a existir aún en los primeros tutoriales. Estos archivos serán agregados 
a medida que el tutorial avance.

¡Diviértanse!

# La estructura del código y la arquitectura

El código está dividido en diferentes módulos donde cada uno representa un
subsistema típico del `kernel (núcleo)`. Los módulos de más alto nivel de los subsistemas se encuentran directamente en la carpeta `src`.
Por ejemplo, `src/memory.rs` contiene el código que está relacionado
con el manejo de memoria.

## Visibilidad del código de arquitectura del procesador

Algunos de los subsistemas del `núcleo (kernel)` dependen del código de bajo nivel (low-level) dedicado a la arquitectura del procesador.
Por cada arquitectura de procesador que está soportada, existe una subcarpeta en `src/_arch`, por ejemplo, `src/_arch/aarch64`.

La carpeta de arquitecturas refleja los módulos del subsistema establecidos en `src`. Por ejemplo, el código de arquitectura que pertenece al subsistema MMU del `núcleo(kernel)` (`src/memory/mmu.rs`) irá dentro de (`src/_arch/aarch64/memory/mmu.rs`).
Este archivo puede ser cargado como un módulo en `src/memory/mmu.rs` usando el `path attribute` (atributo de ruta). Usualmente, el nombre del módulo elegido es el nombre del módulo genérico con el prefijo de `arch_`

Por ejemplo, esta es la parte superior de `src/memory/mmu.rs`:

```
#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/memory/mmu.rs"]
mod arch_mmu;
```

En muchas ocasiones, los elementos de `arch_module` serán reexportados públicamente por el módulo principal.
De esta manera, cada módulo específico de la arquitectura puede proporcionar su implementación de un elemento, mientras que el *invocante* no debe de preocuparse por la arquitectura que se ha compilado condicionalmente.

## Código BSP

`BSP` significa Board Support Package (Paquete de Soporte de la Placa).
El código `BSP` está dentro de `src/bsp.rs` y contiene las definiciones y funciones de la placa base específica elegida. 
Entre estas cosas se encuentran diferentes elementos como el mapa de memoria de la placa o instancias de controladores para dispositivos que se presentan en la placa elegida.

Justo como el código de la arquitectura del procesador, la estructura del módulo del código `BSP` trata de reflejar los módulos del subsistema del `núcleo (kernel)`, pero no ocurre una reexportación esta vez. Eso significa que lo que sea que se esté proporcionando debe ser llamado empezando por el *namespace* (espacio de nombres) de `bsp`, p. ej. `bsp::driver::driver_manager()`.

## La interfaz del núcleo (kernel)

El `arch` y el `bsp` contienen código que se compilará condicionalmente dependiendo del procesador y placa actual para la que se compila el núcleo (kernel).
Por ejemplo, el hardware de control de interrupciones de la `Raspberry Pi 3` y la  `Raspberry Pi 4` es diferente, pero nosotros queremos que el resto del código del kernel funcione correctamente con cualquiera de los dos sin mucha complicación.

Para poder dar una limpia abstracción entre `arch`, `bsp` y código genérico del núcleo, los rasgos de `interface` se proporcionan *siempre y cuando tenga sentido*. Son definidos en su módulo de subsistema correspondiente y ayuda a reforzar el patrón de programar con respecto a una interfaz, sin importar la implementación concreta.

Por ejemplo, habrá una *IRQ handling interface* (interfaz de manejo de interrupciones) común, el cual los dos diferentes controladores de ambas `Raspberry` implementarán, y solo exportarán la interfaz común al resto del `núcleo (kernel)`.

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
  
  - Código común que es independiente de la arquitectura del procesador de destino y las características de la placa (`BSP`).
    - Ejemplo: Una función para poner a cero un trozo de memoria.
  - Las interfaces para el subsistema de la memoria que son implementados por código de `arch` o `BSP`.
    - Ejemplo: Una interfaz `MMU` que define prototipos de función de `MMU`.

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

1. El punto de entrada del núcleo (kernel) es la función `cpu::boot::arch_boot::_start()`.
   - Está implementado en `src/_arch/__arch_name__/cpu/boot.s`.
