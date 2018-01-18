Tutorial 09 - Framebuffer
=========================

Okay, finaly some eyecandy :-) So far the screen showed the rainbow splash. Now we're about to change the resolution with
several tags in a single mbox_call, then display a pixmap. I've put a lot of comments for each tag and
argument to help you, see lfb.c. But at the end of the day it's nothing more than filling up an int array
and call mbox_call, really simple. If you wish, you can try to remove or add more tags to the message and
see what happens. Could have used MBOX_CH_FB (FrameBuffer channel), but MBOX_CH_PROP gives us more flexibility.

Important note on pitch: maybe you don't know, but video screens does not necessairly map raster lines
continously in memory. For example it is possible that 800 pixels (800*4=3200 bytes) are stored in 4096
bytes for every line. Therefore it's important to use the queried pitch value instead of width*4 when
calculating the postition for the Y coordinate.

Also note that the GPU on the Raspberry Pi is very powerful. You can create a large virtual screen (let's say
65536x768) but display only 1024x768 pixels at once. With mailbox messages you can move that window very fast
without the need of copying pixel buffers, thus creating a smooth scrolling effect. In this tutorial both
virtual screen and physical screen is set to 1024x768.

Lfb.h, lfb.c
------------

`lfb_init()` sets up resolution, depth, and color channel order. Also queries framebuffer's address.

`lfb_showpicture()` displays a picture in the center of the screen by drawing pixels to the framebuffer.

Homer.h
-------

The pixmap, saved with the Gimp as C header file. No compression, pixels are stored one-by-one.

Main
----

Very simple. We set the resolution and display the picture, that's all.
