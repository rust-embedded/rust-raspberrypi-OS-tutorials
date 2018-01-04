Oktatóanyag 03 - UART1, Auxilary mini UART
==========================================

Ezúttal a hírhedt Helló Világ példát vesszük elő. Előbb az UART1-re írjuk meg, mivel azt egyszerűbb programozni.
NOTE: qemu nem irányítja át alapból az UART1-et a terminálra, csak az UART0-át!

Gpio.h
------

Van egy új fejléc fájlunk. Ebben definiáljuk az MMIO címét, és a GPIO vezérlő szavainak címeit. Ez egy nagyon
népszerű fejléc lesz, majd minden eszközhöz kelleni fog.

Uart.h, uart.c
--------------

Egy nagyon minimális változat.

`uart_init()` inicializálja az UART csipet, és soros vonalat leképezi a GPIO lábakra.

`uart_send(c)` kiküld egy karatert a soros vonalra.

`uart_getc()` fogad egy karatert. A kocsivissza karakter (13) automatikusan újsor karakterré (10) konvertálódik.

`uart_puts(s)` kiír egy szöveget. Újsor karakternél kiküld egy kocsivissza karatert is (13 + 10).

Main
----

Először is meg kell hívni az uart inicializáló kódját. Aztán kiküldjük, "Helló Világ!". Ha beszereztél USB
soros kábelt, akkor ennek meg kell jelennie a minicom ablakában. Ezután minden, minicom-ban leütött karaktert
visszaküld és kiír. Ha nem kapcsoltad ki a helyi visszhangot (local echo), akkor ez azt jelenti, hogy minden
leütött karaktert duplán fog kiírni a minicom.

