Oktatóanyag 01 - A legszükségesebbek
====================================

Rendben, nem fogunk semmi érdekeset csinálni, csak kipróbáljuk a környezetet. A keletkezett kernel8.img-nek
be kell tudnia bootolni a Raspberry Pi-n, ahol végtelen ciklusban várakoztatja a CPU magokat. Ezt ellenőrizheted a
következő paranccsal:

```sh
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -d in_asm
        ... kimenet törölve az átláthatóság miatt, utolsó sor: ...
0x0000000000080004:  17ffffff      b #-0x4 (addr 0x80000)
```

Start
-----

Amikor a vezérlés átadódik a kernel8.img-nek, a környezet még nem alkalmas C kód futtatására. Ezért mindenképp
szükség van egy kis Assembly bevezetőre. Mivel ez az első oktatóanyag nagyon egyszerű, nem is lesz más, nincs
még C kódunk.

Ne feledkezzünk meg róla, hogy 4 CPU magunk van. Most mind ugyanazt a végtelen ciklust hajtja végre.


Makefile
--------

A Makefile-unk végtelenü legyszerű. Lefordítjuk a start.S-t, ez az egyetlen forrás fájlunk. Aztán a szerkesztési
fázisban a linker.ld szrkript segítségével linkeljük. Végezetül pedig a keletkező elf futtahatót nyers programfájllá
kovertáljuk.

Linker szkript
--------------

Nem túl meglepő módon ez is egyszerű. Be kell állítanunk a bázis címet, ahová a kernel8.img töltődik, és mindent
ide rakunk, mivel csak egy szekciónk van.
