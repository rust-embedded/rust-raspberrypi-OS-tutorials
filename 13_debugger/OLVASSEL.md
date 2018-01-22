Oktatóanyag 13 - Debugger
=========================

Zuzassunk egy nagyot, rakjunk mindjárt egy interaktív debuggert a kivételkezelőbe! :-) Most hogy már van printf(),
nem lesz olyan vészes.

```sh
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial stdio
Synchronous: Breakpoint instruction
> x 
0007FFF0: 13 60 09 00  00 00 00 00  24 10 20 3F  00 00 00 00  .`......$. ?....
> i x30 x30+64 
00080804: D2800000      movz      x0, #0x0
00080808: 94003D1C      bl        0x8FC78
0008080C: 94003ECF      bl        0x90348
00080810: D69F03E0      eret      
00080814: D503201F        27 x nop
>
```

Dbg.h, dbg.c
------------

Egy nagyon minimális és egyszerű debugger (~300 C sor).

`breakpoint` újonnan definiált C kulcsszó. Ahová beillesztjük a kódba, ott meghívódik a debugger

`dbg_decodeexc()` hasonló a 11-es oktatóanyagbeli exc_handler-hez, dekódolja a kivételt kiváltó okot és kiírja

`dbg_getline()` na ja, mégegy hiányzó függvény. Szükségünk van egy módra, amivel a felhasználó szerkesztheti
a parancssort és ami sztringként visszaadja amit begépelt, mikor <kbd>Enter</kbd>-t üt. Szokásunkhoz híven minimál

`dbg_getoffs()` ez a funkció a parancs paramétereit értelmezi. Elfogad hexa és decimális számot "regiszter+/-offszet"
formátumban

`dbg_main()` a debugger fő ciklusa.

Disasm.h
--------

Mivel kicsi (~72k), kivételesen könnyű integrálni, mégis minden ARMv8.2-es utasítást ismer, ezért a választásom
az [Universal Disassembler](https://github.com/bztsrc/udisasm)-re esett ehhez az oktatóanyaghoz. Ha nem szeretnél
disassemblert belefordítani a debuggeredbe, akkor a dbg.c fájl elején állítsd a DISASSEMBLER define-t 0-ra.

Start
-----

A `_vector` táblánk kicsit máshogy fest. Először is el kell mentenünk a regiszterek értékét a `dbg_saveregs` hívással,
majd kiírjuk a kivétel okát és meghívjuk a mini-debuggerünk fő ciklusát.

Main
----

Leteszteljük az új `breakpoint` kulcsszavunkat C-ben.
