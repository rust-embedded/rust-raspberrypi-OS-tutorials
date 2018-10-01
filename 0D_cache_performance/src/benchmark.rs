use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::{barrier, regs::*};

/// We assume that addr is cacheline aligned
pub fn batch_modify(addr: u64) -> u32 {
    const CACHELINE_SIZE_BYTES: u64 = 64; // TODO: retrieve this from a system register
    const NUM_CACHELINES_TOUCHED: u64 = 5;
    const BYTES_PER_U64_REG: usize = 8;
    const NUM_BENCH_ITERATIONS: u64 = 100_000;

    const NUM_BYTES_TOUCHED: u64 = CACHELINE_SIZE_BYTES * NUM_CACHELINES_TOUCHED;

    let t1 = CNTPCT_EL0.get();

    compiler_fence(Ordering::SeqCst);

    let mut data_ptr: *mut u64;
    let mut temp: u64;
    for _ in 0..NUM_BENCH_ITERATIONS {
        for i in (addr..(addr + NUM_BYTES_TOUCHED)).step_by(BYTES_PER_U64_REG) {
            data_ptr = i as *mut u64;

            unsafe {
                temp =  core::ptr::read_volatile(data_ptr);
                core::ptr::write_volatile(data_ptr, temp + 1);
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
