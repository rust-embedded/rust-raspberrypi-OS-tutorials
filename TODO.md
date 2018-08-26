# TODO

## Add debugging targets to Makefile

sudo apt installl gdb-multiarch

qemu-gdb:
       $(DOCKER_CMD) -p 1234:1234 $(UTILS_CONTAINER) $(QEMU_CMD) -s -S

gdb-multiarch kernel8.img -ex "target remote :1234"
set architecture aarch64
symbols-file ??

combine this with rust-gdb?

## Find a way to easily switch betwenn release build and local development

```toml
[patch.crates-io]
cortex-a = { path = "../../cortex-a" }
register = { path = "../../register-rs" }
```
