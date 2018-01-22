Oktatóanyag 12 - Printf
=======================

Mielőtt kibővítenénk a kivételkezelőnket, szükségünk lesz néhány jól ismert C függvényre. Mivel alacsony szinten
programozunk, nem támaszkodhatunk a libc-re, ezért nekünk kell egy saját printf() implementációt megvalósítanunk.

```sh
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial stdio
Hello World!
This is character 'A', a hex number: 7FFF and in decimal: 32767
Padding test: '00007FFF', '    -123'
```

Sprintf.h, sprintf.c
--------------------

Az érdekes rész. Nagyban ráhagyatkozunk a fordítónk beépített funkcióira, hogy a változó elemszámú paramétereket
lekezeljük. Amint ebben az oktatóanyag sorozatban megszokhattunk, nem a tökéletes kompatibilitásra, hanem a
szükséges minimum implementációra törekszünk. Ezért csak a '%s', '%c', '%d' és '%x' opciókat támogatjuk. Az
igazítás is limitált, csak jobbra lehet igazítani, hexa számokat nullákkal, decimálisakat szóközzel.

`sprintf(dst, fmt, ...)` ugyanaz, mint a printf, csak az eredményt egy sztringbe rakja

`vsprintf(dst, fmt, va)` olyan változat, ami paraméterlistát vár változó számú paraméter helyett.


Uart.h, uart.c
-------------

`printf(fmt, ...)` a jó öreg C függvény. A fenti sprintf-et hívja, majd az eredményt ugyanúgy írja ki, mint
ahogy azt az uart_puts() tette. Mivel most már van '%x', az uart_hex() feleslegessé vált, ezért kivettem.

Start
-----

Bár mi nem fogunk floatokat és doubleöket használni, a beépített gcc funkciók lehet, hogy használnak. Ezért
engedélyeznünk kell az FPU koprocesszort, hogy ne keletkezzenek "ismeretlen utasítás" kivételek. Továbbá mivel
ebben a példában nincs rendes kivételkezőnk, ezért az `exc_handler` csak egy függvénycsonk ebben a fájlban.

Main
----

Leteszteljük a printf megvalósításunkat.
