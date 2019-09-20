# Tutorial 0E - Cache Performance

Now that we finally have virtual memory capabilities available, we also have
fine grained control over `cacheability`. You've caught a glimpse already in the
last tutorial, where we used page table entries to reference the `MAIR_EL1`
register to indicate the cacheability of a page or block.

Unfortunately, for the user it is often hard to grasp the advantage of caching
in early stages of OS or bare-metal software development. This tutorial is a
short interlude that tries to give you a feeling of what caching can do for
performance.

## Benchmark

Let's write a tiny, arbitrary micro-benchmark to showcase the performance of
operating with data on the same DRAM with caching enabled and disabled.

### mmu.rs

Therefore, we will map the same physical memory via two different virtual
addresses. We set up our pagetables such that the virtual address `0x400000`
points to the physical DRAM at `0x200000`, and we configure it as
`non-cacheable` in the page tables.

There is also an identity mapped block, which starts at virtual `0x200000` and
points at physical `0x200000`. This time, the block is configured as cacheable.

### benchmark.rs

We write a little function that iteratively reads memory of five times the size
of a `cacheline`, in steps of 8 bytes, aka one processor register at a time. We
read the value, add 1, and write it back. This whole process is repeated
`20_000` times.

The benchmark function is called twice. Once for the cacheable and once for the
non-cacheable virtual addresses. Remember that both virtual addresses point to
the _same_ physical DRAM, so the difference in time that we will see will
showcase how much faster it is to operate on DRAM with caching enabled.

## Output

On my Raspberry, I get the following results:

```console
ferris@box:~$ make raspboot

[0] UART is live!
[1] Press a key to continue booting... Greetings fellow Rustacean!
[2] MMU online.
Benchmarking non-cacheable DRAM modifications at virtual 0x0000000000400000, physical 0x0000000000200000:
1040 miliseconds.

Benchmarking cacheable DRAM modifications at virtual 0x0000000000200000, physical 0x0000000000200000:
53 miliseconds.

With caching, the function is 1800% faster!
```

Impressive, isn't it?
