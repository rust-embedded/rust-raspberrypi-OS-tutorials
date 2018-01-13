Oktatóanyag 11 - Kivételkezelők
===============================

Legutóbb egy nagyon egyszerű címfordítási táblát használtunk, de a mindennapi életben többre van szükség.
Nem könnyű vakon létrehozni ezt a táblát, ezért kivételkezelőket fogunk definiálni. Ez ki fogja dumpolni
a fontos rendszer regisztereket, hogy azonosíthassuk és megtalálhassuk a problémát a címfordítási táblánkban.

```sh
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial stdio
Synchronous: Data abort, same EL, Translation fault at level 2:
  ESR_EL1 0000000096000006 ELR_EL1 0000000000080D7C
 SPSR_EL1 00000000200003C4 FAR_EL1 FFFFFFFFFF000000
```

Az ESR_EL1 elárulja, hogy Adathozzáférési probléma (Data Abort) lépett fel a táblázat második szintjén. Az
utasítás, ami okozta, a 0x80D7C címen található, és a 0xFFFFFFFFFF000000 címet szerette volna elérni.

A megadott vektor hasonló az AMD64 IDT-jéhez, két eltéréssel: először is, nincs külön utasítás (mint az sidt),
hanem egy rendszer regiszter tárolja a címét. Másodszor, nem egy címeket tartalmazó táblázat címét adjuk meg,
hanem a konkrét kód címét. Ezért minden "bejegyzés" a táblában nagyobb, kis programrészek amikkel be lehet
állítani a paramétereket és meghívni egy közös kivételkezelőt.

AMD64-en 32 belépési pont van, minden egyes kivételtípushoz egy. AArch-on ezzel szemben csak egy van, és egy
rendszer regiszterből olvasható ki, melyik kivételtípusról van szó. Figyelembe véve, hogy minden OS úgyis csak
átad egy kivételkódot egy közös kivételkezelőnek, ezzel megkönnyíti az életünket.

Exc.c
-----

`exc_handler()` egy egyszerű kivételkezelő, ami kidumpolja a regisztereket és dekódolja az ESR_EL1-t (részben).
Aztán megállítjuk a CPU-t, mert egyelőre nincs hova visszatérni a kivételkezelőből. A Kivétel Szindróma Regiszter
(Exception Syndrome Register) részletes leírása megtalálható az ARM DDI0487B_b könyv D10.2.28 fejezetében.

Start
-----

Mielőtt rendszerfelügyeleti módra váltanánk, beállítjuk a *vbar_el2*-t. Fontos, hogy az EL1 szinten fellépő
kivételeket EL2 szinten futó kód kezeli le. Minden kezelőt megfelelően kell pozicionálni a memóriában. Qemu
nem érzékeny annyira erre, de az igazi vas igen.

`_vectors` kivételkezelők vektor táblája, kis assembly programokkal, mind az `exc_handler()` nevű C függvényt hívja.

Main
----

Beállítjuk a címfordítást, majd szándékosan egy leképezetlen címre hivatkozunk, hogy kivételt idézzünk elő.
