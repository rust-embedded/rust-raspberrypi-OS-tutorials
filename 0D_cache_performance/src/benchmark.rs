use super::uart;
use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::{barrier, regs::*};

/// We assume that addr is cacheline aligned
fn batch_modify_time(addr: u64) -> Option<u64> {
    const CACHELINE_SIZE_BYTES: usize = 64; // TODO: retrieve this from a system register
    const NUM_CACHELINES_TOUCHED: usize = 5;
    const NUM_BENCH_ITERATIONS: usize = 20_000;

    const NUM_BYTES_TOUCHED: usize = CACHELINE_SIZE_BYTES * NUM_CACHELINES_TOUCHED;

    let mem = unsafe { core::slice::from_raw_parts_mut(addr as *mut u64, NUM_BYTES_TOUCHED) };

    // Benchmark starts here
    let t1 = CNTPCT_EL0.get();

    compiler_fence(Ordering::SeqCst);

    let mut temp: u64;
    for _ in 0..NUM_BENCH_ITERATIONS {
        for qword in mem.iter_mut() {
            unsafe {
                temp = core::ptr::read_volatile(qword);
                core::ptr::write_volatile(qword, temp + 1);
            }
        }
    }

    // Insert a barrier to ensure that the last memory operation has finished
    // before we retrieve the elapsed time with the subsequent counter read. Not
    // needed at all given the sample size, but let's be a bit pedantic here for
    // education purposes. For measuring single-instructions, this would be
    // needed.
    unsafe { barrier::dsb(barrier::SY) };

    let t2 = CNTPCT_EL0.get();
    let frq = u64::from(CNTFRQ_EL0.get());

    ((t2 - t1) * 1000).checked_div(frq)
}

pub fn run(uart: &uart::Uart) {
    const SIZE_2MIB: u64 = 2 * 1024 * 1024;
    const ERROR_STRING: &str = "Something went wrong!";

    // Start of the __SECOND__ virtual 2 MiB block (counting starts at zero).
    // NON-cacheable DRAM memory.
    let non_cacheable_addr: u64 = SIZE_2MIB;

    // Start of the __THIRD__ virtual 2 MiB block.
    // Cacheable DRAM memory
    let cacheable_addr: u64 = 2 * SIZE_2MIB;

    uart.puts("Benchmarking non-cacheable DRAM modifications at virtual 0x");
    uart.hex(non_cacheable_addr as u32);
    uart.puts(", physical 0x");
    uart.hex(2 * SIZE_2MIB as u32);
    uart.puts(":\n");

    let result_nc = match batch_modify_time(non_cacheable_addr) {
        Some(t) => {
            uart.dec(t as u32);
            uart.puts(" miliseconds.\n\n");
            t
        }
        None => {
            uart.puts(ERROR_STRING);
            return;
        }
    };

    uart.puts("Benchmarking cacheable DRAM modifications at virtual 0x");
    uart.hex(cacheable_addr as u32);
    uart.puts(", physical 0x");
    uart.hex(2 * SIZE_2MIB as u32);
    uart.puts(":\n");

    let result_c = match batch_modify_time(cacheable_addr) {
        Some(t) => {
            uart.dec(t as u32);
            uart.puts(" miliseconds.\n\n");
            t
        }
        None => {
            uart.puts(ERROR_STRING);
            return;
        }
    };

    if let Some(t) = (result_nc - result_c).checked_div(result_c) {
        uart.puts("With caching, the function is ");
        uart.dec((t * 100) as u32);
        uart.puts("% faster!\n");
    }
}
