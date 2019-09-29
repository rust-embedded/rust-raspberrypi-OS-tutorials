use cortex_a::{asm, regs::*};

#[naked]
#[no_mangle]
#[link_section = ".text.start"]
pub unsafe fn start() -> ! {
    use crate::runtime_init;

    const CORE_0: u64 = 0;
    const CORE_MASK: u64 = 0x3;
    const STACK_START: u64 = 0x80_000;

    if CORE_0 == MPIDR_EL1.get() & CORE_MASK {
        SP.set(STACK_START);
        runtime_init::init()
    } else {
        // if not core0, infinitely wait for events
        loop {
            asm::wfe();
        }
    }
}
