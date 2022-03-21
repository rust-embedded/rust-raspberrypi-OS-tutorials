# Tutoriales de desarrollo de Sistemas Operativos en Rust con la Raspberry Pi

![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/BSP-RPi3/badge.svg) ![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/BSP-RPi4/badge.svg) ![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/Unit-Tests/badge.svg) ![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/Integration-Tests/badge.svg) ![](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue)

<br/>

<img src="doc/header.jpg" height="372"> <img src="doc/minipush_demo_frontpage.gif" height="372">

## ℹ️ Introducción

Esto es una serie de tutoriales para los desarrolladores aficionados a los Sistemas Operativos (OS) 
que se están adentrando a la nueva arquitectura ARM de 64 bits [ARMv8-A
architecture]. Los tutoriales darán una guía paso a paso en cómo escribir un Sistema Operativo 
[monolitico] desde cero.
Estos tutoriales cubren la implementación común de diferentes tareas de Sistemas Operativos, como 
escribir en una serial console, configurar la memoria virtual y manejar excepciones de hardware (HW). 
Todo mientras usamos la seguridad y velocidad que `Rust` nos proporciona.

¡Divértanse!

_Atentamente, <br>Andre ([@andre-richter])_

P.S.: Las versiones chinas :cn: de los tutoriales fueron iniciadas por [@colachg] y [@readlnh].
Las puedes encontrar como [`README.CN.md`](README.CN.md) en sus respectivas carpetas. Por el
momento están un poco desactualizadas.

La traducción de este [documento](README.ES.md) :mexico: :es: fue creada y enviada por [@zanezhub].
De igual manera se traducirán los tutoriales que sean proporcionados por este repositorio.

[ARMv8-A architecture]: https://developer.arm.com/products/architecture/cpu-architecture/a-profile/docs
[monolitico]: https://en.wikipedia.org/wiki/Monolithic_kernel
[@andre-richter]: https://github.com/andre-richter
[@colachg]: https://github.com/colachg
[@readlnh]: https://github.com/readlnh
[@zanezhub]: https://github.com/zanezhub

## 📑 Estructura

- Cada tutorial contienen un solo binario booteable de la `kernel`.
- Cada tutorial nuevo extiende el tutorial anterior.
- Cada tutorial tendrá un `README` y cada `README` tendrá un pequeña sección de [`tl;dr`](https://es.wikipedia.org/wiki/TL;DR) 
  en donde se dará una pequeña recapitulación de las adiciones anteriores y se mostrará el código fuente `diff` del tutorial 
  anterior para que se pueda inspeccionar los cambios/adiciones que han ocurrido.
    - Algunos tutoriales además de tener un `tl;dr` también tendrán una sección en la que se dará una explicación con lujo de detalle.
      El plan a largo plazo es que cada tutorial tenga una buena explicación en adición al `tl;dr` y al `diff`; pero por el momento los únicos tutoriales
      que gozan de una son los tutoriales en los que creo que el `tl;dr` y el `diff` no son suficientes para comprender lo que está pasando.
- El código que se escribió en este tutorial soporta y corre en la **Raspberry Pi 3** y en la **Raspberry 4**
  - Del tutorial 1 hasta el 5 son tutoriales "preparatorios", por lo que este código solo tendrá sentido ejecutarlo en [`QEMU`](https://www.qemu.org/).
  - Cuando llegues al [tutorial 5](05_drivers_gpio_uart) podr comenzar a cargar y a ejecutar el kernel en una
    Raspeberry de verdad, y observar el output en `UART`.
- Aunque la Raspberry Pi 3 y 4 son las principales tarjetas este código está escrito en un estilo modular,
  lo que permite una fácil portabilidad a otra arquitecturas de CPU o/y tarjetas.
  - Me encantaría si alguien intenta implementar este código en una arquitectura **RISC-V**.
- Para la edición recomiendo [Visual Studio Code] con [Rust Analyzer].
- En adición al texto que aparece en los tutoriales también sería recomendable de revisar 
  el comando `make doc` en cada tutorial. Este comando te deja navegar el código documentado de una manera cómoda.

### Output del comando `make doc`

![make doc](doc/make_doc.png)

[Visual Studio Code]: https://code.visualstudio.com
[Rust Analyzer]: https://rust-analyzer.github.io

## 🛠 Requesitos del sistema

Estos tutoriales están dirigidos principalmente a distribuciones de **Linux**. 
Muchas de las cosas vistas aquí también funcionan en **macOS**, pero esto solo es _experimental_.

### 🚀 La versión tl;dr

1. [Instala Docker Desktop][install_docker].
2. (**Solo para Linux**) Asegurate de que la cuenta de tu usuario están en el [docker group].
3. Prepara la `Rust` toolchain. La mayor parte será manejada en el primer uso del archivo [rust-toolchain](rust-toolchain). 
   Todo lo que nos queda hacer a nosotros es: 

   i. Si ya tienes una versión de Rust instalada:
      ```bash
      cargo install cargo-binutils rustfilt
      ```

   ii. Si necesitas instalar Rust desde cero:
      ```bash
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

      source $HOME/.cargo/env
      cargo install cargo-binutils rustfilt
      ```

4. En caso de que uses `Visual Studio Code`, recomiendo que instales la extensión [Rust Analyzer extension].
5. (**Solo para macOS**) Instala algunas `Ruby` gems.

   Ejecuta esto en la carpeta root del repositorio:

   ```bash
   bundle install --path .vendor/bundle --without development
   ```

[docker group]: https://docs.docker.com/engine/install/linux-postinstall/
[Rust Analyzer extension]: https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer

### 🧰 Más detalles: Eliminando Toolchain Hassle

Esta serie trata de enfocarse lo máximo posible en tener una experiencia amistosa con el usario.
Por lo tanto, se han dirigido muchos esfuerzos a eliminar la parte más difícil del desarrollo de
los sistemas incorporados (embedded) tanto como se pudo: `Toolchain hassle`.

Rust por sí mismo ya está ayudando mucho, porque tiene integrado el soporte para cross-compilation.
Todo lo que necesitamos para compilar desde una máquina con una arquitectura `x86` a una Raspberry Pi
con arquitectura `AArch64` será automáticamente instalado por `rustup`. Sin embargo, además de usar
el compilador de Rust, también usaremos algunas otras herrameintas, entre las cuales están:

- `QEMU` para emular nuestro kernel en nuestra máquina principal.
-  Una herramienta llamada `Minipush` para cargar el kernel en una Raspberry Pi on-demand usando `UART`.
- `OpenOCD` y `GDB` para hacer "debugging" de la máquina a instalar.

Hay muchas cosas que pueden salir mal mientras instalamos y/o compilamos las versiones correctas de cada
herramienta en tu máquina. Por ejemplo, tu distribución tal vez podría no proporcionar las versiones más
recientes que se necesiten. O tal vez te falten algunas dependencias para la compilar estas herramientas.

Esta es la razón por la cual usaremos [Docker][install_docker] en las circunstancias posibles. Te
estamos proporcionando un contenedor que tiene todas las herramientas o dependencias preinstaladas.
Si quieres saber más acerca de Docker y revisar el contenedor proporcionado, por favor revisa la carpeta
[docker](docker) del repositorio.

[install_docker]: https://docs.docker.com/get-docker/

## 📟 USB Serial Output

Ya que el desarrollo de este kernel se está ejecutando en hardware real, se recomienda que tengas
un USB serial cable para sentir la experiencia completa.

- Puedes encontrar estos cables que deberían funcionar sin ningún problema en [\[1\]] [\[2\]], pero
  hay muchos otros que pueden funcionar. Idealmente, tu cable está basado en el chip `CP2102`.
- Lo conectas a los pines `GND` y GPIO `14/15` como se muestra en la parte inferior.  
- [Tutorial 5](05_drivers_gpio_uart) es la primera vez en la que lo vas usar. Revisa las instrucciones
  en cómo preparar una tarjeta SD para bootear en tu kernel desde ahí.
- Empezando con el [tutorial 6](06_uart_chainloader), bootear kernels en tu Raspberry comienza a ser
  más fácil. En este tutorial, un `chainloader` es desarrollado, que será el último archivo que necesitarás
  copiar de manera manual a la tarjeta SD por un tiempo. Esto te permitirá cargar los kernels de los tutoriales
  durante el boot on demand usando `UART`.

![UART wiring diagram](doc/wiring.png)

[\[1\]]: https://www.amazon.de/dp/B0757FQ5CX/ref=cm_sw_r_tw_dp_U_x_ozGRDbVTJAG4Q
[\[2\]]: https://www.adafruit.com/product/954

## 🙌 Agradecimientos

La versión original de estos tutoriales empezó como un fork de los increíbles 
[tutoriales de programación en hardware en la RPi3](https://github.com/bztsrc/raspi3-tutorial) en `C`
de [Zoltan Baldaszti](https://github.com/bztsrc). ¡Gracias por darme un punto de partida!

## Licencia

Este proyecto está licenciado por cualquiera de las siguientes licencias como alguna de tus dos opciones

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)


### Contribución

A menos de que lo menciones, cualquier contribución enviada por ti para su inclusión en este trabajó,
caerá bajo la licencia de Apache-2.0, deberá tener doble licencias como se muestra en la parte superior, sin ninguna
adición de términos o condiciones.


