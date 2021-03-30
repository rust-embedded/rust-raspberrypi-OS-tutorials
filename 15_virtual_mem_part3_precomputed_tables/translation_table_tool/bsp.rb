# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

# Raspberry Pi 3 + 4
class RaspberryPi
    attr_reader :kernel_granule, :kernel_virt_addr_space_size, :kernel_virt_start_addr

    NM_BINARY = 'aarch64-none-elf-nm'
    READELF_BINARY = 'aarch64-none-elf-readelf'
    MEMORY_SRC = File.read('src/bsp/raspberrypi/memory.rs').split("\n")

    def initialize(kernel_elf)
        @kernel_granule = Granule64KiB

        @virt_addresses = {
            boot_core_stack_start: /__boot_core_stack_start/,
            boot_core_stack_end_exclusive: /__boot_core_stack_end_exclusive/,

            rx_start: /__rx_start/,
            rx_end_exclusive: /__rx_end_exclusive/,

            rw_start: /__rw_start/,
            rw_end_exclusive: /__rw_end_exclusive/,

            table_struct_start_addr: /bsp::.*::memory::mmu::KERNEL_TABLES/,
            phys_tables_base_addr: /PHYS_KERNEL_TABLES_BASE_ADDR/
        }

        symbols = `#{NM_BINARY} --demangle #{kernel_elf}`.split("\n")
        @kernel_virt_addr_space_size = parse_from_symbols(symbols, /__kernel_virt_addr_space_size/)
        @kernel_virt_start_addr = 0
        @virt_addresses = parse_from_symbols(symbols, @virt_addresses)
        @phys_addresses = virt_to_phys(@virt_addresses)

        @descriptors = parse_descriptors
        update_max_descriptor_name_length

        @text_section_offset_in_elf = parse_text_section_offset_in_elf(kernel_elf)
    end

    def rw_end_exclusive
        @virt_addresses[:rw_end_exclusive]
    end

    def phys_table_struct_start_addr
        @phys_addresses[:table_struct_start_addr]
    end

    def table_struct_offset_in_kernel_elf
        (@virt_addresses[:table_struct_start_addr] - @virt_addresses[:rx_start]) +
            @text_section_offset_in_elf
    end

    def phys_tables_base_addr
        @phys_addresses[:phys_tables_base_addr]
    end

    def phys_tables_base_addr_offset_in_kernel_elf
        (@virt_addresses[:phys_tables_base_addr] - @virt_addresses[:rx_start]) +
            @text_section_offset_in_elf
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

    def kernel_map_binary
        MappingDescriptor.print_header

        @descriptors.each do |i|
            print 'Generating'.rjust(12).green.bold
            print ' '
            puts i.to_s

            TRANSLATION_TABLES.map_pages_at(i.virt_pages, i.phys_pages, i.attributes)
        end

        MappingDescriptor.print_divider
    end

    private

    def parse_from_symbols(symbols, input)
        case input.class.to_s
        when 'Regexp'
            symbols.grep(input).first.split.first.to_i(16)
        when 'Hash'
            input.transform_values do |val|
                symbols.grep(val).first.split.first.to_i(16)
            end
        else
            raise
        end
    end

    def parse_text_section_offset_in_elf(kernel_elf)
        `#{READELF_BINARY} --sections #{kernel_elf}`.scan(/.text.*/).first.split.last.to_i(16)
    end

    def virt_to_phys(input)
        case input.class.to_s
        when 'Integer'
            input - @kernel_virt_start_addr
        when 'Hash'
            input.transform_values do |val|
                val - @kernel_virt_start_addr
            end
        else
            raise
        end
    end

    def descriptor_ro
        name = 'Code and RO data'

        ro_size = @virt_addresses[:rx_end_exclusive] -
                  @virt_addresses[:rx_start]

        virt_ro_pages = PageArray.new(@virt_addresses[:rx_start], ro_size, @kernel_granule::SIZE)
        phys_ro_pages = PageArray.new(@phys_addresses[:rx_start], ro_size, @kernel_granule::SIZE)
        ro_attribues = AttributeFields.new(:CacheableDRAM, :ReadOnly, :X)

        MappingDescriptor.new(name, virt_ro_pages, phys_ro_pages, ro_attribues)
    end

    def descriptor_data
        name = 'Data and bss'

        data_size = @virt_addresses[:rw_end_exclusive] -
                    @virt_addresses[:rw_start]

        virt_data_pages = PageArray.new(@virt_addresses[:rw_start], data_size,
                                        @kernel_granule::SIZE)
        phys_data_pages = PageArray.new(@phys_addresses[:rw_start], data_size,
                                        @kernel_granule::SIZE)
        data_attribues = AttributeFields.new(:CacheableDRAM, :ReadWrite, :XN)

        MappingDescriptor.new(name, virt_data_pages, phys_data_pages, data_attribues)
    end

    def descriptor_boot_core_stack
        name = 'Boot-core stack'

        boot_core_stack_size = @virt_addresses[:boot_core_stack_end_exclusive] -
                               @virt_addresses[:boot_core_stack_start]

        virt_boot_core_stack_pages = PageArray.new(@virt_addresses[:boot_core_stack_start],
                                                   boot_core_stack_size, @kernel_granule::SIZE)
        phys_boot_core_stack_pages = PageArray.new(@phys_addresses[:boot_core_stack_start],
                                                   boot_core_stack_size, @kernel_granule::SIZE)
        boot_core_stack_attribues = AttributeFields.new(:CacheableDRAM, :ReadWrite, :XN)

        MappingDescriptor.new(name, virt_boot_core_stack_pages, phys_boot_core_stack_pages,
                              boot_core_stack_attribues)
    end

    def parse_descriptors
        [descriptor_ro, descriptor_data, descriptor_boot_core_stack]
    end

    def update_max_descriptor_name_length
        MappingDescriptor.max_descriptor_name_length = @descriptors.map { |i| i.name.size }.max
    end
end
