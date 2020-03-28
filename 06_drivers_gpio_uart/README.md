# Tutorial 06 - Drivers: GPIO and UART

## tl;dr

Now that we enabled safe globals in the previous tutorial, the infrastructure is laid for adding the
first real device drivers. We throw out the magic QEMU console and use a real UART now. Like serious
embedded hackers do!

## Notable additions

- For the first time, we will be able to run the code on the real hardware.
  - Therefore, building is now differentiated between the **RPi 3** and the **RPi4**.
  - By default, all `Makefile` targets will build for the **RPi 3**.
  - In order to build for the the **RPi4**, prepend `BSP=rpi4` to each target. For example:
    - `BSP=rpi4 make`
    - `BSP=rpi4 make doc`
  - Unfortunately, QEMU does not yet support the **RPi4**, so `BSP=rpi4 make qemu` won't work.
- A `driver::interface::DeviceDriver` trait is added for abstracting `BSP` driver implementations
  from kernel code.
- Drivers are stored in `src/bsp/device_driver`, and can be reused between `BSP`s.
    - We introduce the `GPIO` driver, which pinmuxes the RPi's PL011 UART.
    - Most importantly, the `PL011Uart` driver: It implements the `console::interface::*` traits and
      is from now on used as the main system console output.
- `BSP`s now contain a memory map in `src/bsp/memory.rs`. In the specific case, they contain the
  Raspberry's `MMIO` addresses which are used to instantiate the respectivedevice drivers.
- We also modify the `panic!` handler, so that it does not anymore rely on `println!`, which uses
  the globally-shared instance of the `UART` that might be locked when an error is encountered (for
  now this can't happen due to the `NullLock`, but with a real lock it becomes an issue).
    - Instead, it creates a new UART driver instance, re-initializes the device and uses that one to
      print. This increases the chances that the system is able to print a final important message
      before it suspends itself.

## Boot it from SD card

Some steps for preparing the SD card differ between RPi3 and RPi4, so be careful.

### Common for both

1. Make a single `FAT32` partition named `boot`.
2. On the card, generate a file named `config.txt` with the following contents:

```txt
init_uart_clock=48000000
```
### Pi 3

3. Copy the following files from the [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot) onto the SD card:
    - [bootcode.bin](https://github.com/raspberrypi/firmware/raw/master/boot/bootcode.bin)
    - [fixup.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup.dat)
    - [start.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start.elf)
4. Run `make` and copy the [kernel8.img](kernel8.img) onto the SD card.

### Pi 4

3. Copy the following files from the [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot) onto the SD card:
    - [fixup4.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup4.dat)
    - [start4.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start4.elf)
    - [bcm2711-rpi-4-b.dtb](https://github.com/raspberrypi/firmware/raw/master/boot/bcm2711-rpi-4-b.dtb)
4. Run `BSP=rpi4 make` and copy the [kernel8.img](kernel8.img) onto the SD card.

_**Note**: Should it not work on your RPi4, try renaming `start4.elf` to `start.elf` (without the 4)
on the SD card._

### Common again

5. Insert the SD card into the RPi and connect the USB serial to your host PC.
    - Wiring diagram at [top-level README](../README.md#usb-serial).
6. Run `screen` (you might need to install it first):

```console
sudo screen /dev/ttyUSB0 230400
```

7. Hit <kbd>Enter</kbd> to kick off the kernel boot process. Observe the output:

```console
[0] Booting on: Raspberry Pi 3
[1] Drivers loaded:
      1. BCM GPIO
      2. BCM PL011 UART
[2] Chars written: 93
[3] Echoing input now
```

8. Exit screen by pressing <kbd>ctrl-a</kbd> <kbd>ctrl-d</kbd> or disconnecting the USB serial.

## Diff to previous
