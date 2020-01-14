# Tutorial 01 - Wait Forever

## tl;dr

Project skeleton is set up; Code just halts all CPU cores executing the kernel code.

- Toolchain: `cargo xbuild` tools (`xrustc`, `xclippy`) and the
  `aarch64-unknown-none-softfloat` target are used for building `AArch64`
  bare-metal code.
- `Makefile` targets:
    - `doc`: Generate documentation.
    - `qemu`: Run the `kernel` in QEMU
    - `clippy`
    - `clean`
    - `readelf`: Inspect the `ELF` output.
    - `objdump`: Inspect the assembly.
    - `nm`: Inspect the symbols.
- Code is organized into `kernel`, `arch` and `BSP` (Board Support Package)
  parts.
    - Conditional compilation includes respective `arch` and `BSP` according to
      user-supplied arguments.
- Custom `link.ld` linker script.
    - Load address at `0x80_000`
    - Only `.text` section.
- `main.rs`: Important [inner attributes]:
    - `#![no_std]`, `#![no_main]`
- Assembly `_start()` function that executes `wfe` (Wait For Event), halting all
  cores that are executing `_start()`.
- We (have to) define a `#[panic_handler]` function.
    - Just waits infinitely for a cpu event.

[inner attributes]: https://doc.rust-lang.org/reference/attributes.html

### Test it

In the project folder, invoke QEMU and observe the CPU core spinning on `wfe`:
```console
» make qemu
[...]
IN:
0x00080000:  d503205f  wfe
0x00080004:  17ffffff  b        #0x80000
```
