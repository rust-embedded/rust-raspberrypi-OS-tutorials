Oktatóanyag 09 - Framebuffer
============================

Rendben, végre valami parasztvakítás :-) Eddig a képernyőn csak a szivárvány doboz volt. Most be fogjuk állítani a felbontását
egy csomó parancsot tartalmazó üzenettel és egyetlen egy mbox_call hívással, majd kirakunk egy képet. Teleraktam
kommenttel az lfb.c forrást (igaz, angol nyelvűek), hogy segítsenek eligazodni a parancsokban. De végeredményben
nem tesz mást, mint feltölt egy int tömböt és meghívja az mbox_call-t, igazán egyszerű. Ha gondolod, megpróbálhatsz
hozzáadni vagy elvenni parancsokat, hogy lásd, mi történik. Használhattam volna az MBOX_CH_FB (FrameBuffer csatornát)
is, de az MBOX_CH_PROP sokkal több mindent tesz lehetővé és sokkal rugalmasabb.

Fontos tudnivaló a pitch-ről: talán nem tudod, de a video képernyő rasztersorai nem feltétlenül vannak sorfolytonosan
tárolva a memóriában. Például lehetséges, hogy 800 pixelnél (800 * 4=3200 bájt helyett) 4096 bájton tárolódik minden
sor. Ezért fontos, hogy mindig a dinamikusan lekért pitch értékével számoljuk width * 4 helyett a képernyő Y
koordinátáját.

Arra is érdemes figyelni, hogy a GPU a Raspberry Pi-n nagyon combos. Létrehozhatsz például egy hatalmas virtuális
képernyőt (mondjuk 65536x768), amiből egyszerre csak 1024x768 lesz megjelenítve. Levelesláda üzenetekkel piszok
gyorsan mozgathatod ezt az ablakot, annélkül, hogy pixelbuffereket kéne másolgatni, ezáltal egy nagyon sima
szkrollozó hatást hozva létre. Ebben a példában mind a virtuális, mind a fizikai képméretet 1024x768-ra állítottam.

Lfb.h, lfb.c
------------

`lfb_init()` beállítja a felbontást, színmélységet, színcsatorna sorrendjét. Lekéri továbbá a framebuffer címét.

`lfb_showpicture()` a framebuffer-be direkt pixelek írásával megjelenít egy képet a képernyő közepén.

Homer.h
-------

A kép, Gimp-el C header formátumban lementve. Nincs tömörítve, a pixelek egymás után következnek.

Main
----

Nagyon egyszerű. Beállítjuk a felbontást, és kirajzoljuk a képet, ennyi.
