Tutorial 0C - Directory
=======================

Now that we can load a sector from the storage, it is time to parse it as a file system. This
tutorial will show you how to list the root directory entries of a FAT16 or FAT32 partition.

```sh
$ qemu-system-aarch64 -M raspi3 -drive file=test.dd,if=sd,format=raw -serial stdio
        ... output removed for clearity ...
MBR disk identifier: 12345678
FAT partition starts at: 00000008
        ... output removed for clearity ...
FAT type: FAT16
FAT number of root diretory entries: 00000200
FAT root directory LBA: 00000034
        ... output removed for clearity ...
Attrib Cluster  Size     Name
...L.. 00000000 00000000 EFI System 
....D. 00000003 00000000 FOLDER     
.....A 00000171 0000C448 BOOTCODEBIN
.....A 0000018A 000019B3 FIXUP   DAT
.....A 0000018E 00001B10 KERNEL8 IMG
.....A 00000192 000005D6 LICENC~1BRO
.....A 00000193 002B2424 START   ELF
```

Fat.h, fat.c
------------

This is easy and pretty well documented. We have to read the MBR, locate our partition, and load
it's first sector (Volume Boot Record). That has the BIOS Parameter Block, which describes the FAT
file system.

`fat_getpartition()` load and check the boot record of the first MBR partition.

`fat_listdirectory()` list root directory entries on the volume.

Main
----

Once we initialize EMMC to read sectors, we load the boot record of the first partition. If the BPB
describes a valid FAT partition, we list the root directory entries in it.
