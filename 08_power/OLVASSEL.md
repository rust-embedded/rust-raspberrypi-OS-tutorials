Oktatóanyag 08 - Energiagazdálkodás
===================================

Beépített rendszerek esetén nagyon fontos az energiafogyasztás. A Raspberry Pi 3-on ezért roppant szofisztikált
interfészt találunk. Az egyes eszközöket külön-külön ki be kapcsolgathatjuk. Van egy hátulütő azonban, a GPIO
VCC lábai direktbe vannak kötve az áramforrásra, magyarán nem lehet programból vezérelni őket. Ez azt jelenti,
ha eszközöket kötsz rá, akkor neked kell megoldanod ezen eszközök kikapcsolását (mondjuk egy tranzisztorral,
amit egy másik GPIO adatláb vezérel).

Power.h, power.c
----------------

Az energiagazdálkodás az egyik periféria, amit a qemu egyáltalán nem emulál. Igazi vason szépen megy.

`power_off()` leállítja az alaplapot egy, majdnem nulla energiafogyasztási szintre.

`reset()` újraindítja a gépet. Ezt is a PMC kezeli, és mivel nincs a Raspberry-n fizikai reset gomb, roppant
hasznos tud lenni.

Main
----

Megjelenítünk egy egyszerű menüt, majd várunk a felhasználóra. A válaszától függően vagy újraindítjuk a gépet,
vagy kikapcsoljuk.
