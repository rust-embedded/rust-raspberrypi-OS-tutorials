Tutorial 12 - Printf
====================

Before we can improve our exception handler, we are going to need some functions very well known from the C library.
Since we are programming bare metal, we don't have libc, therefore we have to implement printf() on our own.

```sh
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial stdio
Hello World!
This is character 'A', a hex number: 7FFF and in decimal: 32767
Padding test: '00007FFF', '    -123'
```

Sprintf.h, sprintf.c
--------------------

The interesting part. We heavily rely on our compiler's features to handle variable length argument list. As usual
in these tutorials, it's not a fully featured, but rather a bare minimum implementation. Supports '%s', '%c',
'%d' and '%x'. Padding is limited, only right alignment with leading zeros for hex and spaces for decimal.

`sprintf(dst, fmt, ...)` same as printf, but stores result in a string

`vsprintf(dst, fmt, va)` a variant that receives an argument list parameter instead of a variable length list of arguments.


Uart.h, uart.c
-------------

`printf(fmt, ...)` the good old C library function. Uses the sprintf function above and then outputs the string
in the same way as uart_puts() did. Since we have '%x', uart_hex() became unnecessary, therefore removed.

Start
-----

Although we are not going to use floats and doubles, gcc built-ins might. So we have to enable the FPU
coprocessor to avoid "undefined instruction" exceptions. Also, in a lack of a proper exception handler,
we have a dummy `exc_handler` stub this time.

Main
----

We test our printf implementation.
