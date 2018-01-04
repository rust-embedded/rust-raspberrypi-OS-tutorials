AArch64 Kereszt-Fordító
=======================

Mielőtt nekiugranál az oktatóanyagoknak, szükséged lesz néhány szerszámra. Nevezetesen egy fordítóra, ami
képes AArch64-re fordítani, és a hozzá kapcsolódó programokra.

Build renszer
-------------

A fordítás levezénylésére a GNU make-t fogjuk használni. Ezt nem kell kereszt-fordítani, mivel csak az asztali gépen
fogjuk futtatni, nem a céleszközön. Azért választottam ezt az oktatóanyagokhoz, mert a GNU make-re a fordító
lefordításához is szükséged lesz, szóval ígyis-úgyis kelleni fog.

Források letöltése és kicsomagolása
------------------------------------

Legelőször is, töltsd le a binutils és gcc forrásait. Ebben a példában az íráskor legfrissebb verziókat használtam.
Javaslom, hogy nézd meg a szervereket, és a legfrissebbre módosítsd a fájlneveket.

```sh
wget http://ftpmirror.gnu.org/binutils/binutils-2.29.tar.gz
wget http://ftpmirror.gnu.org/gcc/gcc-7.2.0/gcc-7.2.0.tar.gz
wget http://ftpmirror.gnu.org/mpfr/mpfr-3.1.6.tar.gz
wget http://ftpmirror.gnu.org/gmp/gmp-6.1.2.tar.bz2
wget http://ftpmirror.gnu.org/mpc/mpc-1.0.3.tar.gz
wget ftp://gcc.gnu.org/pub/gcc/infrastructure/isl-0.18.tar.bz2
wget ftp://gcc.gnu.org/pub/gcc/infrastructure/cloog-0.18.1.tar.gz
```

Miután végezett a letöltés, csomagold ki a tömörített fájlokat:

```sh
for i in *.tar.gz; do tar -xzf $i; done
for i in *.tar.bz2; do tar -xjf $i; done
```

Szükség lesz még néhány symlink-re mielőtt a fordtást elkezdhetnénk, hozzuk létre ezeket:

```sh
cd binutils-*
ln -s ../isl-* isl
cd ..
cd gcc-*
ln -s ../mpfr-* mpfr
ln -s ../gmp-* gmp
ln -s ../mpc-* mpc
ln -s ../cloog-* cloog
cd ..
```

Források lefordítása
--------------------

Oké, két csomagot kell fordítanunk. Az egyik a *binutils*, ami tartalmazza a linker-t, assembler-t és még pár
hasznos parancsot.

```sh
cd binutils-*
configure --prefix=/usr/local/cross-compiler --target=aarch64-elf \
--enable-shared --enable-threads=posix --enable-libmpx --with-system-zlib --with-isl --enable-__cxa_atexit \
--disable-libunwind-exceptions --enable-clocale=gnu --disable-libstdcxx-pch --disable-libssp --enable-plugin \
--disable-linker-build-id --enable-lto --enable-install-libiberty --with-linker-hash-style=gnu --with-gnu-ld\
--enable-gnu-indirect-function --disable-multilib --disable-werror --enable-checking=release --enable-default-pie \
--enable-default-ssp --enable-gnu-unique-object
make -j4
make install
```

Az első paraméter megmondja a configure szkriptnek, hogy a `/usr/local/cross-compiler` mappába telepítsen. A második
megadja a célarchítektúrát, amire a most fordítandó eszközök fordítanak majd. A maradék paramétek ki és bekapcsolgat
bizonyos funkciókat, ne foglalkozz velük. Elég annyit tudni, hogy ezek egy beágyazott fordítóhoz vannak optimalizálva.

A második csomag, természetesen maga a *gcc* fordító.

```sh
cd gcc-*
configure --prefix=/usr/local/cross-compiler --target=aarch64-elf --enable-languages=c \
--enable-shared --enable-threads=posix --enable-libmpx --with-system-zlib --with-isl --enable-__cxa_atexit \
--disable-libunwind-exceptions --enable-clocale=gnu --disable-libstdcxx-pch --disable-libssp --enable-plugin \
--disable-linker-build-id --enable-lto --enable-install-libiberty --with-linker-hash-style=gnu --with-gnu-ld\
--enable-gnu-indirect-function --disable-multilib --disable-werror --enable-checking=release --enable-default-pie \
--enable-default-ssp --enable-gnu-unique-object
make -j4
make install
```

Itt ugyanúgy megadjuk a könyvtárat és a célarchitektúrát, mint az előbb. Megadjuk azt is, hogy csak C fordítót kérünk,
mivel a gcc rengeteg nyelvet ismer, amire nem lesz szükség. Ez jelentősen lecsökkenti a fordítási időt. A fennmaradó
kapcsolók ugyan azok, mint a binutils esetében.

Ha végzett, nézd meg a `bin` mappát a `/usr/local/cross-compiler` könyvtárban. Egy rakás futtatható programot kell
ott találnod. Meggyőződésem, hogy ezt a mappát hozzá akarod adni a PATH-odhoz is.

```sh
$ ls /usr/local/cross-compiler/bin
aarch64-elf-addr2line  aarch64-elf-elfedit    aarch64-elf-gcc-ranlib  aarch64-elf-ld       aarch64-elf-ranlib
aarch64-elf-ar         aarch64-elf-gcc        aarch64-elf-gcov        aarch64-elf-ld.bfd   aarch64-elf-readelf
aarch64-elf-as         aarch64-elf-gcc-7.2.0  aarch64-elf-gcov-dump   aarch64-elf-nm       aarch64-elf-size
aarch64-elf-c++filt    aarch64-elf-gcc-ar     aarch64-elf-gcov-tool   aarch64-elf-objcopy  aarch64-elf-strings
aarch64-elf-cpp        aarch64-elf-gcc-nm     aarch64-elf-gprof       aarch64-elf-objdump  aarch64-elf-strip
```

Amik ezek közül számunkra érdekesek:
 - aarch64-elf-as - az assembler
 - aarch64-elf-gcc - a C fordító
 - aarch64-elf-ld - a linker
 - aarch64-elf-objcopy - az ELF futtathatók IMG-re való konvertálásához kell
 - aarch64-elf-objdump - futtathatók disassemblálásához (debuggolásnál)
 - aarch64-elf-readelf - hasznos eszköz a futtathatókban lévő szekciók és szegmensek listázáshoz (debuggolásnál)

Ha mind az öt fenti futtahatót látod, és hibaüzenet nélkül le is futnak, gratulálok!
Minden eszköz a rendelkezésedre áll, ami ehhez az oktatóanyaghoz kelleni fog.
