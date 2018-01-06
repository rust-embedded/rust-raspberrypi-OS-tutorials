Tutorial 0E - Initial RamDisk
=============================

Many OS uses initial ramdisk to load files into memory during boot. I felt the need for such
a tutorial as most hobby OS developer's never learned how to do this properly.

First of all, we're not going to reinvent the wheel a come up with a new format and an awful
image creator tool. We're going to use the POSIX standard `tar` utility to create our initrd. It's format
is really simple, first comes an 512 bytes long header with file meta information, followed by the
file contents padded with zeros to round up to multiple of 512 bytes. This repeats for every file in the archive.
If you want a compressed initrd, you can use for example the [tinf](https://bitbucket.org/jibsen/tinf) library to
deflate. The uncompressed buffer can be parsed by the method described here.

Second, about loading it into memory, we have several options:

### Load a file on our own
You can use the `fat_readfile()` from the previous tutorial. In that case your initrd's address
will be returned by the function.

### Ask the GPU to do so
You can use `config.txt` to tell start.elf to load the initrd for you. With this you won't need
any SD card reader or FAT parser at all, resulting in a much smaller kernel. As for the
[config.txt](https://www.raspberrypi.org/documentation/configuration/config-txt/boot.md),
you have two options:

`initramfs (filename) followkernel` - this will load (filename) after your kernel. You can access it at the label
*&_end* defined by our linker script.

`initramfs (filename) (address)` - load (filename) into a specified location. You can access it at *(address)*.

### Statically link
This is not very practical because you have to build your kernel every time you want to change the initrd. But
it is the simplest method, and to keep this tutorial simple we'll use this. You can access the initrd by the label
*_binary_initrd_tar_start*.

Makefile
--------
I've added a tar.o to the usual Makefile. This rule will dinamically create a tar file and convert it into an
object file.

Initrd.h, initrd.c
------------------

`initrd_list(buf)` list the contents of a tar archive in the buffer.

Main
----

We initialize console and the pass the initrd buffer to lister.
