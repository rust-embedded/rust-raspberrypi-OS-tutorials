Oktatóanyag 0B - Szektor Beolvasás
==================================

Ezidáig minden adatot (kép, font) hozzálinkeltünk a kernelhez. Itt az ideje, hogy adatot olvassunk be
az SD kártyáról. Ebben az oktatóanyagban egy igazi meghajtót implementálunk a szektor beolvasásához.

Sd.h, sd.c
------------

Nos, jó lenne, ha lenne levelesláda szektorok olvasására és írására, de nincs. Ezért nekünk kell direktben
beszélni az EMMC-vel, ami trükkös és unalmas feladat. Különböző kártyákkal is törődnünk kell. De végül,
rendelkezésre áll két funkció.

`sd_init()` inicializálja az EMMC-t SD kártya olvasáshoz.

`sd_readblock(lba,buffer,num)` beolvas num blokkot (szektort) az SD kártyáról a buffer-be lba-tól kezdve.

Main
----

A blokkot a bss szegmens utánra töltjük be, majd kidumpoljuk a konzolra. A beolvasás funkció részletes
üzeneteket ír ki arról, hogy épp mit kommunikál az EMMC vezérlővel.
