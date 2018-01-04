Tutorial 0A - PC Screen Font
============================

Drawing pixmaps is fun, but definitely there's a need to display characters as well. Basicaly fonts
are nothing more than bitmaps for each character. For this tutorial I choosed PC Screen Font format,
the same Linux Console uses.

Lfb.h, lfb.c
------------

`lfb_init()` sets up resolution, depth, and color channel order. Also queries framebuffer's address.

`lfb_print(x,y,s)` displays a string on screen.

Font.psf
--------

The font file. Use any file from /usr/share/kbd/consolefonts. Unicode table is not supported. Translating
characters to glyph index using that table (instead of one-to-one relation) is a homework for you. This font
is generated from the original IBM PC VGA 8x16 Font ROM, and includes 127 glyphs.

Makefile
--------

I've added a new object file, generated from the psf. It's a good example of how to include and reference
a binary file in C.

Main
----

Very simple. We set the resolution and display the string.
