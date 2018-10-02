use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::{barrier, regs::*};

/// We assume that addr is cacheline aligned
pub fn batch_modify(addr: u64) -> u32 {
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

    ((t2 - t1) * 1000 / u64::from(CNTFRQ_EL0.get())) as u32
}
