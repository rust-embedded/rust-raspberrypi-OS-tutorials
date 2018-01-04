Oktatóanyag 06 - Hardveres Véletlenszám Generátor
=================================================

Ez egy egyszerű okatatóanyag lesz. Lekérjük az aktuális értéket (az egyéként nem dokumentált)
hardveres véletlenszám generátorból. Ez arra használható többek között, hogy egy egyszerű, megfelelő
kockadobást szimuláljunk bármilyen játékban. Ez azért fontos, mert hardveres támogatás nélkül csak
kizárólag pszeudo-véletlen állítható elő.

Rand.h, rand.c
--------------

`rand_init()` inicializálja a hardvert.

`rand(min,max)` visszaad egy min és max közötti véletlen számot.

Main
----

Lekérjük a véletlen számot, és kiírjuk a soros konzolra.
