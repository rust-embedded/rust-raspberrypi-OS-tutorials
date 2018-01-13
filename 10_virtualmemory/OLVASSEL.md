Oktatóanyag 10 - Virtuális Memória
==================================

Elérkeztünk a legegyszerűbb és egyben legbonyolultabb oktatóanyagunkhoz. Egyszerű, mert nem csinálunk mást, mint
feltöltünk egy tömböt, aztán beállítunk pár regisztert. A nehézség abban rejlik, hogy mi kerüljön a tömbbe.

Feltételezem kellő ismerettel rendelkezel az AMD64-es lapozásáról. Ha nem, akkor erősen javaslom, hogy szánj rá
időt mielőtt folytatnád. Az ARMv8 MMU-ja sokkal de sokkal bonyolultabb és összetettebb, mint az AMD64-é. Határozottan
nem jó ötlet ezzel kezdeni.

Az AMD64 címfordítása roppant egyszerű, egy táblacím regisztere van, a memóriát 4k-s lapokra osztja, 4 szinten,
és csak egyféle luk van definilálva a címtérben. ARMv8 ennél sokkal combosabb. Beállíthatot a lapkeret méretét,
a lapozási szintek számát, összefűzheted a lapfordító táblázatokat egy adott szinten, de még a luk méretét is
beállíthatod. Képtelenség mindezt egy oktatóanyaggal lefedni. Ezért úgy döntöttem, inkább úgy állítom be az ARMv8
MMU-ját, hogy minnél jobban hasonlítson az AMD64-éhez. Ez a következőket jelenti: 4k-s lapméretet fogunk használni,
2M-es blokkmérettel és 512G-s címterülettel (3 szint), ahol a 4. szintet két regiszter helyettesíti. Fogd fel
úgy, hogy amíg AMD64-n a 4. szintű laptábla címe a CR3-ban van megadva, addig ARMv8-on van egyszer a TTBR0 regiszter,
ami ennek a táblának az első elemét tartalmazza, valamint van a TTBR1 regiszter, ami meg az utolsó, 512. elemét,
ezért nincs szükség 4. szintű laptáblára. Minden köztes cím (amit a 2.-511. elemek fordítanának) a lukra esik, más
szavakkal azok nem kanonikus memória címek.


A lapfordító táblázat egyébként ugyanúgy néz ki: 64 bites elemeket tartalmaz, amikben van egy fizikai cím és néhány
attribútum bit, minden egyes szinten. De ARMv8-on sokkal több lehetőséged van. Külön beállíthatod a gyorsítótárat
(cachability), megosztást (shareability) és a hozzáférést is. Ezen kívül van még egy memória típus tömböt tartalmazó
regiszter, aminek az elemeire indexelnek a lapcím fordító tábla bejegyzéseinek bitjei.

A következőképp fogjuk leképezni a virtuális memóriát: az alacsony címeket egy-az-egyben megfeleltetéssel 2M
blokkonként, kivéve az első blokkot, amit 4k-nként. A magas címekre pedig, -2M-nél leképezzük az UART0 MMIO-ját.

Mmu.h, mmu.c
------------

`mmu_init()` inicializálja a memória címfordító egységet (Memory Management Unit)

Start
-----

Ezúttal hozzáférést kell biztosítanunk a rendszer kontroll regiszterhez is (system control register).

Link.ld
-------

Most szükség van arra, hogy az adatterületet és a programunk vége cimkét laphatárra igazítsuk.

Main
----

Beállítjuk a címfordítást, majd kiírunk a konzolra mind egy-az-egyben lapozást, mind magas című lapozást használva.
