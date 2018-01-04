Oktatóanyag 07 - Késleltetések
==============================

Roppant fontos, hogy a megfelelő időtartamot késleltessünk, amikor alacsony szintű hardverrel bánunk.
Ebben az okatatóanyagban három megközelítést nézünk meg. Az egyik CPU órajel függő (akkor hasznos, ha
a várakozási idő óraciklusban van megadva), a másik kettő mikroszekundum (másodperc milliomod része) alapú.

Delays.h, delays.c
------------------

`wait_cycles(n)` ez nagyon faék, n-szer lefuttatjuk a `nop` (nincs utasítás) utasítást.

`wait_msec(n)` ez a megvalósítás ARM rendszer regisztereket használ (minden AArch64 CPU-n elérhető).

`wait_msec_st(n)` ez pedig BCM specifikus, ami a Rendszer Időzítő perifériát használja (nincs emulálva qemu-n).

Main
----

Különböző implementációkkal várakozunk a konzolra írások között.
