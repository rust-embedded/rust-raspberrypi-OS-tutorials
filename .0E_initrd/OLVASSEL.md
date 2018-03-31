Oktatóanyag 0E - Kezdeti memória lemez
======================================

Sok OS használ kezdeti memória lemezt (initrd) hogy fájlokat töltsön be induláskor. Szükségét éreztem egy
ilyen oktatóanyagnak, mivel a legtöbb hobby OS fejlesztőnek fingja sincs, hogy kell ezt rendesen csinálni.

Először is, nem fogjuk újra feltalálni a spanyol viaszt és kiagyalni egy új formátumot, amihez aztán egy
szörnyű kreátor programot heggesztünk. Helyette a POSIX szabvány `tar` archíválót és a `cpio` parancsot
fogjuk használni. Előbbi egyértelműbb, utúbbit a Linux is használja.

A tar formátuma pofonegyszerű, először jön egy 512 bájtos fejléc a fájl adataival, majd ezt követi a fájl
tartalma nullákkal kiegészítve, hogy a hossza 512-vel osztható legyen. Ez ismétlődik minden egyes fájlra az archívban.

A cpio nagyon hasonló, csak változó hosszúságú fejléccel operál, és nincs a fájl tartalma kiegészítve.

Ha szeretnél tömörített initrd-t, akkor javaslom például a [tinf](https://bitbucket.org/jibsen/tinf) könyvtárat
a kicsomagoláshoz. A kitömörített adatot az itt ismertetett módszerrel olvashatod.

Másodszor, a betöltéshez több lehetőségünk is van:

### Betöltjük a fájlt saját magunk
Ehhez használhatod a `fat_readfile()` funkciót az előző oktatóanyagból. Ebben az esetben az initrd címét visszaadja
a függvény.

### Megkéred a GPU-t hogy töltse be neked
Aztán használhatod a `config.txt`-t hogy utasítsd a start.elf-et az initrd betöltésére. Ez azért jó, mert ehhez
nem kell SD kártya olvasó és FAT értelmező, így a kerneled jóval kissebb lesz. Ami a
[config.txt](https://www.raspberrypi.org/documentation/configuration/config-txt/boot.md) parancsait illeti,
két lehetőséged is van:

`ramfsfile=(fájlnév)` - ez betölti a (fájlnév) nevű fájlt mindjárt a kerneled után. Az initrd-d kezdőcíme
ekkor a *&_end* cimke lesz, amit a linker szkriptben definiáltunk.

`initramfs (fájlnév) (cím)` - a megadott címre tölti be a (fájlnév) nevű fájlt. A kezdőcíme ekkor *(cím)* lesz.

### Statikus linkelés
Nem túl praktikus, mivel mindig újra kell fordítani a kernelt, ha változtatni akarsz az initrd tartalmán. Azonban
ez a legegyszerűbb módszer, és hogy átlátható legyen az oktatóanyag, ezért ezt választottam. Az initrd-d
kezdőcímét ez esetben a *_binary_ramdisk_start* cimke szolgáltatja.

Makefile
--------
Hozzáadtam egy rd.o-t létrehozó szabályt, ami object fájllá konvertálja az archívot. Ezen kívül lett két
új szabály:

`make tar` ami *tar* formátumban hozza létre az archívumot

`make cpio` ami pedig *cpio hpodc* formátumot használ.

Ezért amikor fordítani szeretnél, vagy a `make tar all`, vagy a `make cpio all` parancsot kell használnod.

Initrd.h, initrd.c
------------------

`initrd_list(buf)` kilistázza a bufferben lévő archív tartalmát. A formátumát automatikusan detektálja.

Main
----

Inicializáljuk a konzolt, majd átadjuk az initrd kezdőcímét a listázónak.
