#!/usr/bin/env bash

cd ../05_drivers_gpio_uart
BSP=rpi4 make
cp kernel8.img ../06_uart_chainloader/demo_payload_rpi4.img
make
cp kernel8.img ../06_uart_chainloader/demo_payload_rpi3.img
rm kernel8.img
