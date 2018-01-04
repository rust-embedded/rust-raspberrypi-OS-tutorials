Oktatóanyag 0D - Fájl Beolvasása
================================

Megtanultuk, hogy kell beolvasni és értelmezni a gyökérkönyvtárat. Ebben az oktatóanyagban fogunk egy fájlt
a gyökérkönyvtárból, és a kluszterláncát végigjárva teljes egészében betöltjük a memóriába.

```sh
$ qemu-system-aarch64 -M raspi3 -drive file=test.dd,if=sd,format=raw -serial stdio
        ... kimenet törölve az átláthatóság miatt ...
FAT File LICENC~1BRO starts at cluster: 00000192
FAT Bytes per Sector: 00000200
FAT Sectors per Cluster: 00000004
FAT Number of FAT: 00000002
FAT Sectors per FAT: 00000014
FAT Reserved Sectors Count: 00000004
FAT First data sector: 00000054
        ... kimenet törölve az átláthatóság miatt ...
00085020: 43 6F 70 79  72 69 67 68  74 20 28 63  29 20 32 30  Copyright (c) 20
00085030: 30 36 2C 20  42 72 6F 61  64 63 6F 6D  20 43 6F 72  06, Broadcom Cor
00085040: 70 6F 72 61  74 69 6F 6E  2E 0A 43 6F  70 79 72 69  poration..Copyri
00085050: 67 68 74 20  28 63 29 20  32 30 31 35  2C 20 52 61  ght (c) 2015, Ra
00085060: 73 70 62 65  72 72 79 20  50 69 20 28  54 72 61 64  spberry Pi (Trad
00085070: 69 6E 67 29  20 4C 74 64  0A 41 6C 6C  20 72 69 67  ing) Ltd.All rig
        ... kimenet törölve az átláthatóság miatt ...
```

Fat.h, fat.c
------------

Ez is elég egyszerű és jól dokumentált. Megkeressük a fájlukhoz tartozó könyvtárbejegyzést, és kivesszük
belóle az induló kluszter számát. Ezután minden egyes klusztert betöltünk miközben végigjárjuk a láncot.

`fat_getpartition()` betölti és értelmezi a partíció első szektorát.

`fat_getcluster(fn)` visszaadja egy adott fájlnévhez tartozó kluszterlánc első tagját.

`fat_readfile(clu)` rbeolvas egy fájlt a memóriába. Visszatérési értéke egy mutató a legelső beolvasott bájtra.

Main
----

Miután inicializájuk az EMMC-t szektorok olvasásához, beolvassuk az első szektort a partícióról. Ha a BPB
egy érvényes FAT fájlrendszert ír le, akkor megkeressük a 'LICENCE.broadcom'-hoz tartozó első kluszter számát.
Ha ilyent nem találunk, akkor a 'kernel8.img'-t keresünk. Ha bármelyiket megtaláltuk, akkor betöltjük és
kidumpoljuk a fájl első 512 bájtját.
