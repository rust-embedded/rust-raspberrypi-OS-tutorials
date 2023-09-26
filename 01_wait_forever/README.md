# Tutorial 01 - Wait Forever

## tl;dr

- The project skeleton is set up.
- A small piece of assembly code runs that just halts all CPU cores executing the kernel code.

## Building

- `Makefile` targets:
    - `doc`: Generate documentation.
    - `qemu`: Run the `kernel` in QEMU
    - `clippy`
    - `clean`
    - `readelf`: Inspect the `ELF` output.
    - `objdump`: Inspect the assembly.
    - `nm`: Inspect the symbols.

## Code to look at

- `BSP`-specific `kernel.ld` linker script.
    - Load address at `0x8_0000`
    - Only `.text` section.
- `main.rs`: Important [inner attributes]:
    - `#![no_std]`, `#![no_main]`
- `boot.s`: Assembly `_start()` function that executes `wfe` (Wait For Event), halting all cores
  that are executing `_start()`.
- We (have to) define a `#[panic_handler]` function to make the compiler happy.
    - Make it `unimplemented!()` because it will be stripped out since it is not used.

[inner attributes]: https://doc.rust-lang.org/reference/attributes.html

### Test it

In the project folder, invoke QEMU and observe the CPU core spinning on `wfe`:

```console
$ make qemu
[...]
IN:
0x00080000:  d503205f  wfe
0x00080004:  17ffffff  b        #0x80000
```
