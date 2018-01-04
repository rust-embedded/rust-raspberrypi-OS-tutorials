Oktatóanyag 0C - Könyvtárak
===========================

Most hogy már tudunk szektort beolvasni, ideje értelmezni a fájlrendszert. Ez az oktatóanyag megtanítja,
hogy hogyan listázzuk ki egy FAT16 vagy FAT32 partíció gyökérkönyvtárát.

```sh
$ qemu-system-aarch64 -M raspi3 -drive file=test.dd,if=sd,format=raw -serial stdio
        ... kimenet törölve az átláthatóság miatt ...
MBR disk identifier: 12345678
FAT partition starts at: 00000008
        ... kimenet törölve az átláthatóság miatt ...
FAT type: FAT16
FAT number of root diretory entries: 00000200
FAT root directory LBA: 00000034
        ... kimenet törölve az átláthatóság miatt ...
Attrib Cluster  Size     Name
...L.. 00000000 00000000 EFI System 
....D. 00000003 00000000 FOLDER     
.....A 00000171 0000C448 BOOTCODEBIN
.....A 0000018A 000019B3 FIXUP   DAT
.....A 0000018E 00001B10 KERNEL8 IMG
.....A 00000192 000005D6 LICENC~1BRO
.....A 00000193 002B2424 START   ELF
```

Fat.h, fat.c
------------

Ez elég egyszerű és jól dokumentált művelet. Beolvassuk az MBR-t, megkeressük a partíciónkat, majd
betöltjük az első szektorát (Volume Boot Record). Ebben van az ún. BIOS Parameter Block, ami leírja
a FAT fájlrendszert.

`fat_getpartition()` betölti és értelmezi a partíció első szektorát.

`fat_listdirectory()` kilistázza a fájlrendszer gyökérkönyvtárának tartalmát.

Main
----

Miután inicializájuk az EMMC-t szektorok olvasásához, beolvassuk az első szektort a partícióról. Ha a BPB
egy érvényes FAT fájlrendszert ír le, akkor kilistázzuk a fájlrendszer gyükérkönyvtárát.
