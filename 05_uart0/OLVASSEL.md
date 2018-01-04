Oktatóanyag 05 - UART0, PL011
=============================

Ebben az okatatóanyagban ugyanazt csináljuk, mint a 4-esben, de most a szériaszámot az UART0-ra küldjük ki.
Emiatt ez a példa könnyen használható qemu-val is:

```sh
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial stdio
My serial number is: 0000000000000000
```

Uart.h, uart.c
--------------

Mielőtt a frekvenciaosztót megadhatnánk, be kell állítanunk egy fix órajelet a PL011 csipben. Ezt levelesládán
keresztül tehetjük meg, ugyanazon a property csatornán keresztül, amit már korábban is használtunk. Ettől eltekintve
ez az interfész teljesen azonos az UART1-ével.

Main
----

Lekérjük az alaplap egyedi szériaszámát, majd kiírjuk a soros konzolra.
