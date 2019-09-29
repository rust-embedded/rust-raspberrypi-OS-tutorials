use cortex_a::{asm};

#[no_mangle]
#[link_section = ".text.start"]
pub unsafe fn start() -> ! {
    loop {
        asm::wfe();
    }
}
