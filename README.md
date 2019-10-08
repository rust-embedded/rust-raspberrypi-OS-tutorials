# Bare-metal and Operating System development tutorials in Rust on the Raspberry Pi 3

## Notice

**This is a work-in-progress rewrite started on September 2019. I will first add
code and minimal READMEs, and later write accompanying full-fledged tutorial
text.**

- Check out the `make doc` command to browse the code with HTML goodness.
- Note that the branch is subject to frequent force pushing. If updates happened
  since you last visited, make sure to clone a clean copy to be safe.

_Cheers,
Andre_

## Prerequisites

Before you can start, you'll need a suitable Rust toolchain.
Please browse to the [rustup components history] and note the date of the most recent
build that shows `clippy` as `present`.

[rustup components history]: https://rust-lang.github.io/rustup-components-history/


Then, proceed to install this nightly using your noted date:
```bash
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly-YOUR_DATE_HERE
# For example:
# curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly-2019-09-05

rustup component add rust-src llvm-tools-preview clippy
cargo install cargo-xbuild cargo-binutils
```

Additionally, a Micro SD card with [firmware
files](https://github.com/raspberrypi/firmware/tree/master/boot) on a FAT
filesystem is needed.

I recommend to get a [Micro SD card USB
adapter](http://media.kingston.com/images/products/prodReader-FCR-MRG2-img.jpg)
(many manufacturers ship SD cards with such an adapter), so that you can connect
the card to any desktop computer just like an USB stick, no special card reader
interface required (although many laptops have those these days).

You can create an MBR partitioning scheme on the SD card with an LBA FAT32 (type
0x0C) partition, format it and copy `bootcode.bin`, `start.elf` and `fixup.dat`
onto it. **Delete all other files or booting might not work**. Alternatively,
you can download a raspbian image, `dd` it to the SD card, mount it and delete
the unnecessary .img files. Whichever you prefer. What's important, you'll
create `kernel8.img` with these tutorials which must be copied to the root
directory on the SD card, and no other `.img` files should exists there.

I'd also recommend to get an [USB serial debug
cable](https://www.adafruit.com/product/954). You connect it to the GPIO pins
14/15.

![UART wiring diagram](doc/wiring.png)

Then, run `screen` on your desktop computer like

```bash
sudo screen /dev/ttyUSB0 115200
```

Exit screen again by pressing <kbd>ctrl-a</kbd> <kbd>ctrl-d</kbd>

## License

Licensed under the MIT license ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT).
