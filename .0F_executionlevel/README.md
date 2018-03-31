Tutorial 0F - Execution levels
==============================

Before we can go on to virtual memory, we have to talk about execution levels. Each level has it's own
memory translation tables, therefore it's cruital to know which one we are using. So in this tutorial we're
make sure of it, we are at supervisor level, EL1. Qemu may start machine at EL1, but real Raspberry Pi hardware
normally boots at hypervisor level, EL2. Under qemu use "-d int" to debug the level change.

```sh
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial stdio -d int
Exception return from AArch64 EL2 to AArch64 EL1 PC 0x8004c
Current EL is: 00000001
```

Start
-----

I've added a little bit more Assembly code for changing the execution level if we're not at supervisor level.
But before we can do that, we have to grant access for the counter registers (used by wait_msec()), and tell
the CPU we want AArch64 mode in EL1. Finally, we fake an exception return to change the level for real.

NOTE: For completeness, I've added code for EL3 too because of [Issue #6](https://github.com/bztsrc/raspi3-tutorial/issues/6),
although normally Raspberry runs kernel8.img in EL2. With some config.txt options, you can make it run in EL3 (thanks
[@btauro](https://github.com/btauro) for the info).

Main
----

We query the current execution level and then we display it on the serial console.
