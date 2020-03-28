# Tutorial 08 - Timestamps

## tl;dr

We add abstractions for the architectural timer, implement it for `aarch64` and use it to annotate
prints with timestamps; A `warn!()` macro is added.

## Test it

Check it out via chainboot (added in previous tutorial):
```console
$ make chainboot
[...]
Minipush 1.0

[MP] ⏳ Waiting for /dev/ttyUSB0
[MP] ✅ Connected
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
[MP] ⏩ Pushing 12 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.586140] Booting on: Raspberry Pi 3
[    0.587227] Architectural timer resolution: 52 ns
[    0.589530] Drivers loaded:
[    0.590876]       1. BCM GPIO
[    0.592309]       2. BCM PL011 UART
[W   0.594005] Spin duration smaller than architecturally supported, skipping
[    0.597392] Spinning for 1 second
[    1.599001] Spinning for 1 second
[    2.599872] Spinning for 1 second

```

## Diff to previous
