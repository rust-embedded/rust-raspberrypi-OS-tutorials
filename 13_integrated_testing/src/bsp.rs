// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Conditional re-exporting of Board Support Packages.

mod device_driver;

#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
mod raspberrypi;

#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
pub use raspberrypi::*;

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Ensure the kernel's virtual memory layout is free of overlaps.
    #[kernel_test]
    fn virt_mem_layout_has_no_overlaps() {
        let layout = memory::mmu::virt_mem_layout().inner();

        for (i, first) in layout.iter().enumerate() {
            for second in layout.iter().skip(i + 1) {
                let first_range = first.virtual_range;
                let second_range = second.virtual_range;

                assert!(!first_range().contains(second_range().start()));
                assert!(!first_range().contains(second_range().end()));
                assert!(!second_range().contains(first_range().start()));
                assert!(!second_range().contains(first_range().end()));
            }
        }
    }
}
