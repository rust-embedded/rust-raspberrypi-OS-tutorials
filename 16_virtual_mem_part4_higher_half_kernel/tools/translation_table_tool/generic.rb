# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

module Granule64KiB
    SIZE = 64 * 1024
    SHIFT = Math.log2(SIZE).to_i
end

module Granule512MiB
    SIZE = 512 * 1024 * 1024
    SHIFT = Math.log2(SIZE).to_i
    MASK = SIZE - 1
end

# Monkey-patch Integer with some helper functions.
class Integer
    def power_of_two?
        self[0].zero?
    end

    def aligned?(alignment)
        raise unless alignment.power_of_two?

        (self & (alignment - 1)).zero?
    end

    def align_up(alignment)
        raise unless alignment.power_of_two?

        (self + alignment - 1) & ~(alignment - 1)
    end

    def to_hex_underscore(with_leading_zeros: false)
        fmt = with_leading_zeros ? '%016x' : '%x'
        value = format(fmt, self).to_s.reverse.scan(/.{4}|.+/).join('_').reverse

        format('0x%s', value)
    end
end

# An array where each value is the start address of a Page.
class MemoryRegion < Array
    def initialize(start_addr, size, granule_size)
        raise unless start_addr.aligned?(granule_size)
        raise unless size.positive?
        raise unless (size % granule_size).zero?

        num_pages = size / granule_size
        super(num_pages) do |i|
            (i * granule_size) + start_addr
        end
    end
end

# Collection of memory attributes.
class AttributeFields
    attr_reader :mem_attributes, :acc_perms, :execute_never

    def initialize(mem_attributes, acc_perms, execute_never)
        @mem_attributes = mem_attributes
        @acc_perms = acc_perms
        @execute_never = execute_never
    end

    def to_s
        x = case @mem_attributes
            when :CacheableDRAM
                'C'
            else
                '?'
            end

        y = case @acc_perms
            when :ReadWrite
                'RW'
            when :ReadOnly
                'RO'
            else
                '??'
            end

        z = @execute_never ? 'XN' : 'X '

        "#{x} #{y} #{z}"
    end
end

# A container that describes a virt-to-phys region mapping.
class MappingDescriptor
    @max_section_name_length = 'Sections'.length

    class << self
        attr_accessor :max_section_name_length

        def update_max_section_name_length(length)
            @max_section_name_length = [@max_section_name_length, length].max
        end
    end

    attr_reader :name, :virt_region, :phys_region, :attributes

    def initialize(name, virt_region, phys_region, attributes)
        @name = name
        @virt_region = virt_region
        @phys_region = phys_region
        @attributes = attributes
    end

    def to_s
        name = @name.ljust(self.class.max_section_name_length)
        virt_start = @virt_region.first.to_hex_underscore(with_leading_zeros: true)
        phys_start = @phys_region.first.to_hex_underscore(with_leading_zeros: true)
        size = ((@virt_region.size * 65_536) / 1024).to_s.rjust(3)

        "#{name} | #{virt_start} | #{phys_start} | #{size} KiB | #{@attributes}"
    end

    def self.print_divider
        print '             '
        print '-' * max_section_name_length
        puts '--------------------------------------------------------------------'
    end

    def self.print_header
        print_divider
        print '             '
        print 'Sections'.center(max_section_name_length)
        print '   '
        print 'Virt Start Addr'.center(21)
        print '   '
        print 'Phys Start Addr'.center(21)
        print '   '
        print 'Size'.center(7)
        print '   '
        print 'Attr'.center(7)
        puts
        print_divider
    end
end

def kernel_map_binary
    mapping_descriptors = KERNEL_ELF.generate_mapping_descriptors

    # Generate_mapping_descriptors updates the header being printed with this call. So it must come
    # afterwards.
    MappingDescriptor.print_header

    mapping_descriptors.each do |i|
        print 'Generating'.rjust(12).green.bold
        print ' '
        puts i

        TRANSLATION_TABLES.map_at(i.virt_region, i.phys_region, i.attributes)
    end

    MappingDescriptor.print_divider
end

def kernel_patch_tables(kernel_elf_path)
    print 'Patching'.rjust(12).green.bold
    print ' Kernel table struct at ELF file offset '
    puts BSP.kernel_tables_offset_in_file.to_hex_underscore

    File.binwrite(kernel_elf_path, TRANSLATION_TABLES.to_binary, BSP.kernel_tables_offset_in_file)
end

def kernel_patch_base_addr(kernel_elf_path)
    print 'Patching'.rjust(12).green.bold
    print ' Kernel tables physical base address start argument to value '
    print TRANSLATION_TABLES.phys_tables_base_addr.to_hex_underscore
    print ' at ELF file offset '
    puts BSP.phys_kernel_tables_base_addr_offset_in_file.to_hex_underscore

    File.binwrite(kernel_elf_path, TRANSLATION_TABLES.phys_tables_base_addr_binary,
                  BSP.phys_kernel_tables_base_addr_offset_in_file)
end
