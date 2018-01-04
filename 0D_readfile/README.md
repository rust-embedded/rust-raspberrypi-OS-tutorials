Tutorial 0D - Read File
=======================

We learned how to read and parse the root directory. In this tutorial we'll get one file from the
root directory, and walk through the cluster chain to load it entirely into memory.

```sh
$ qemu-system-aarch64 -M raspi3 -drive file=test.dd,if=sd,format=raw -serial stdio
        ... output removed for clearity ...
FAT File LICENC~1BRO starts at cluster: 00000192
FAT Bytes per Sector: 00000200
FAT Sectors per Cluster: 00000004
FAT Number of FAT: 00000002
FAT Sectors per FAT: 00000014
FAT Reserved Sectors Count: 00000004
FAT First data sector: 00000054
        ... output removed for clearity ...
00085020: 43 6F 70 79  72 69 67 68  74 20 28 63  29 20 32 30  Copyright (c) 20
00085030: 30 36 2C 20  42 72 6F 61  64 63 6F 6D  20 43 6F 72  06, Broadcom Cor
00085040: 70 6F 72 61  74 69 6F 6E  2E 0A 43 6F  70 79 72 69  poration..Copyri
00085050: 67 68 74 20  28 63 29 20  32 30 31 35  2C 20 52 61  ght (c) 2015, Ra
00085060: 73 70 62 65  72 72 79 20  50 69 20 28  54 72 61 64  spberry Pi (Trad
00085070: 69 6E 67 29  20 4C 74 64  0A 41 6C 6C  20 72 69 67  ing) Ltd.All rig
        ... output removed for clearity ...
```

Fat.h, fat.c
------------

This is also easy and pretty well documented. We locate the directory entry for our file, and get
the starting cluster number. Then we load each cluster into memory as we walk the cluster chain.

`fat_getpartition()` load and check the boot record of the first MBR partition.

`fat_getcluster(fn)` return the starting cluster for the given filename.

`fat_readfile(clu)` reads a file into memory, returns pointer to the first byte.

Main
----

Once we initialize EMMC to read sectors, we load the boot record of the first partition. If the BPB
describes a valid FAT partition, we find the starting cluster for the file 'LICENCE.broadcom'. If that
not found, we'll look for 'kernel8.img'. If any of these found, we load it and dump it's first 512 bytes.
