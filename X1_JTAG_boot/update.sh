#!/usr/bin/env bash

BSP=rpi4 make
cp kernel8.img jtag_boot_rpi4.img
make
cp kernel8.img jtag_boot_rpi3.img
rm kernel8.img
