Oktatóanyag 04 - Levelesládák
=============================

Mielőtt nekiugranánk az UART0-ának, szükségünk lesz a levelesládára. Ezért ebben az oktatóanyagban bemutatom a
mailbox interfészt. Arra használjuk, hogy lekérdezzük az alaplap egyedi sorszámát, majd kiírjuk azt.
NOTE: qemu nem irányítja át alapból az UART1-et a terminálra, csak az UART0-át!

Uart.h, uart.c
--------------

`uart_hex(d)` kiír egy bináris értéket hexadecimális formátumban.

Mbox.h, mbox.c
--------------

A levelesláda interfésze. Először értékekkel feltöltjük az `mbox` tömböt, aztán meghívjuk a `mbox_call(ch)`-t,
hogy értesüljön róla a GPU, megadva közben a levelesláda csatornáját.
Ebben a példában a [property csatornát](https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface) 
használtam, aminek az üzenete a következőképp néz ki:
 0. üzenet teljes hossza bájtban, (x+1)*4
 1. MBOX_REQUEST mágikus szám, kérés típusú üzenetet jelent
 2-x. parancsok
 x+1. MBOX_TAG_LAST mágikus szám, nincs további parancs jelölése

Ahol minden egyes parancs szerkezete a következő:
 n+0. parancs azonosító
 n+1. adatterület mérete bájtban
 n+2. nulla
 n+3. opcionális adatterület

Main
----

Lekérjük az alaplap egyedi szériaszámát, majd kiírjuk a soros konzolra.
