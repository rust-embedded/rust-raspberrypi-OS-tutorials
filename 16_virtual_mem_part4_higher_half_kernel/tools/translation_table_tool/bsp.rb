# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

# Raspberry Pi 3 + 4
class RaspberryPi
    attr_reader :kernel_granule, :kernel_virt_addr_space_size, :kernel_virt_start_addr

    MEMORY_SRC = File.read('kernel/src/bsp/raspberrypi/memory.rs').split("\n")

    def initialize
        @kernel_granule = Granule64KiB

        @kernel_virt_addr_space_size = KERNEL_ELF.symbol_value('__kernel_virt_addr_space_size')
        @kernel_virt_start_addr = KERNEL_ELF.symbol_value('__kernel_virt_start_addr')

        @virt_addr_of_kernel_tables = KERNEL_ELF.symbol_value('KERNEL_TABLES')
        @virt_addr_of_phys_kernel_tables_base_addr = KERNEL_ELF.symbol_value(
            'PHYS_KERNEL_TABLES_BASE_ADDR'
        )
    end

    def phys_addr_of_kernel_tables
        KERNEL_ELF.virt_to_phys(@virt_addr_of_kernel_tables)
    end

    def kernel_tables_offset_in_file
        KERNEL_ELF.virt_addr_to_file_offset(@virt_addr_of_kernel_tables)
    end

    def phys_kernel_tables_base_addr_offset_in_file
        KERNEL_ELF.virt_addr_to_file_offset(@virt_addr_of_phys_kernel_tables_base_addr)
    end

    def phys_addr_space_end_page
        x = MEMORY_SRC.grep(/pub const END/)
        x = case BSP_TYPE
            when :rpi3
                x[0]
            when :rpi4
                x[1]
            else
                raise
            end

        x.scan(/\d+/).join.to_i(16)
    end
end
