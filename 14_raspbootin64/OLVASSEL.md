Oktatóanyag 14 - Raspbootin64
=============================

Mivel folyton iragatni az SD kártyát strapás, és nem tesz jót a kártyának, ezért egy olyan kernel8.img-t készítünk,
ami a soros vonalról fogja betölteni az igazi kernel8.img-t.

Ez az oktatóanyag a jól ismert [raspbootin](https://github.com/mrvn/raspbootin) átírása 64 bitre.
A betöltőprogram egyik felét adja csak, a kernel fogadót, ami az RPi-n fut. A másik fél, a PC-n futó küldő,
megtalálható az eredeti forrásban [raspbootcom](https://github.com/mrvn/raspbootin/blob/master/raspbootcom/raspbootcom.cc) néven.
Ha Windowsos gépekről is szeretnél kernelt küldeni, akkor javaslom inkább a John Cronin féle átiratot, a
[raspbootin-server](https://github.com/jncronin/rpi-boot/blob/master/raspbootin-server.c)-t, ami natív Win32 API-t használ.

Hogy az új kernelt ugyanoda tölthessük be, el kell mozdítanunk a kódunkat az útból. Ezt chain loading-nak hívják, amikor
az első kód ugyanarra a címre tölti be a második kódot, ezért az utóbbi azt hiszi, a firmware töltötte be.
Hogy ezt megvalósítsuk, egy alacsonyabb címre linkeljük a kódot, és mivel a GPU ettől függetlenül a 0x80000-ra tölt be,
nekünk kell a módosított címre másolnunk magunkat. Fontos, hogy ezalatt csak relatív címzést használhatunk. Amikor
végeztünk, a 0x80000-as címen lévő memóriának használaton kívülinek kell lennie.  Ezt a következő paranccsal ellenőrizheted:

```sh
$ aarch64-elf-readelf -s kernel8.elf | grep __bss_end
    21: 000000000007ffc0     0 NOTYPE  GLOBAL DEFAULT    4 __bss_end
```

Ajánlott a kódunkat minimalizálni, mivel úgyis figyelmen kívül hagyja az újonnan betöltendő kód. Ezért kivettem az
`uart_puts()` eljárást, így a teljes méret 1024 bájt alá csökkent.

Start
-----

Hozzáadtam egy ciklust, ami átmásolja a kódunkat arra a címre, ahová vártuk, hogy betöltődjön.

Linker
------

Másik címre linkelünk ebben a példában. Hasonlóan a bss méret kiszámításához, ugyanúgy meghatározzuk a kód
méretét is, amit másolnunk kell.

Main
----

Kiírjuk, hogy 'RBIN64', majd beolvassuk az új kernelt a soros vonalról, és átadjuk rá a vezérlést.
