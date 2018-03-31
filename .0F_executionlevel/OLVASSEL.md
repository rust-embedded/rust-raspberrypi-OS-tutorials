Oktatóanyag 0F - Futási szintek
===============================

Mielőtt rátérhetnénk a virtuális memóriára, beszélnünk kell a futási szintekről. Minden szintnek saját
lapfordító tára van, emiatt életbevágó, hogy tudjuk, melyik szinten futunk éppen. Ezért ebben az oktatóanyagban
megbizonyosodunk róla, hogy rendszerfelügyeleti szinten (supervisor) azaz EL1-en vagyunk-e. Qemu alatt a gép
indulhat egyből EL1-en, az igazi Raspberry Pi vason azonban általában virtualizációs szinten (hypervisor) azaz EL2-n
ébredünk. Qemu alatt a szintváltást a "-d int" kapcsolóval debuggolhatjuk.

```sh
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial stdio -d int
Exception return from AArch64 EL2 to AArch64 EL1 PC 0x8004c
Current EL is: 00000001
```

Start
-----

Hozzáadtam egy pár Assembly sort, ami átállítja a futási szintet, ha nem rendszerfelügyeleti szinten lennénk.
De mielőtt ezt megtehetnénk, hozzáférést kell biztosítani a számláló regiszterekhez (counter, amit a wait_msec()
használ), valamint megmondjuk a CPU-nak, hogy AArch64 módban fut az EL1. Végezetül egy kivételkezelőből való
visszatérést hazudunk, hogy ténylegesen szintet váltsunk.

FIGYELEM: a teljesség kedvéért hozzáadtam az EL3-at is az [Issue #6](https://github.com/bztsrc/raspi3-tutorial/issues/6)
miatt, bár normális körülmények között a Raspberry EL2-n futtatja a kernel8.img-t. Bizonyos config.txt parancsokkal elérhető,
hogy EL3-on induljon (köszönet [@btauro](https://github.com/btauro)-nak az infóért).

Main
----

Lekérjük az aktuális futási szintet, és kiírjuk a soros konzolra.
