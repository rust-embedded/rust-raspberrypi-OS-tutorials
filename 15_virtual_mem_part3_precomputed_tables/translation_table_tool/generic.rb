# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

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

    def to_hex_underscore(with_leading_zeros: false)
        fmt = with_leading_zeros ? '%016x' : '%x'
        value = format(fmt, self).to_s.reverse.scan(/.{4}|.+/).join('_').reverse

        format('0x%s', value)
    end
end

# An array where each value is the start address of a Page.
class PageArray < Array
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
end

# A container that describes a one- or many-page virt-to-phys mapping.
class MappingDescriptor
    @max_descriptor_name_length = 0

    class << self
        attr_accessor :max_descriptor_name_length
    end

    attr_reader :name, :virt_pages, :phys_pages, :attributes

    def initialize(name, virt_pages, phys_pages, attributes)
        @name = name
        @virt_pages = virt_pages
        @phys_pages = phys_pages
        @attributes = attributes
    end

    def to_s
        name = @name.ljust(self.class.max_descriptor_name_length)
        virt_start = @virt_pages.first.to_hex_underscore(with_leading_zeros: true)
        size = ((@virt_pages.size * 65_536) / 1024).to_s.rjust(3)

        "#{name} | #{virt_start} | #{size} KiB"
    end

    def self.print_divider
        print '             '
        print '-' * max_descriptor_name_length
        puts '----------------------------------'
    end

    def self.print_header
        print_divider
        print '             '
        print 'Section'.center(max_descriptor_name_length)
        print '   '
        print 'Start Virt Addr'.center(21)
        print '   '
        print 'Size'.center(7)
        puts
        print_divider
    end
end

def kernel_patch_tables(kernel_binary)
    print 'Patching'.rjust(12).green.bold
    print ' Kernel table struct at physical '
    puts BSP.phys_table_struct_start_addr.to_hex_underscore

    File.binwrite(kernel_binary, TRANSLATION_TABLES.to_binary,
                  BSP.table_struct_offset_in_kernel_elf)
end

def kernel_patch_base_addr(kernel_binary)
    print 'Patching'.rjust(12).green.bold
    print ' Value of kernel table physical base address ('
    print TRANSLATION_TABLES.phys_tables_base_addr.to_hex_underscore
    print ') at physical '
    puts BSP.phys_tables_base_addr.to_hex_underscore

    File.binwrite(kernel_binary, TRANSLATION_TABLES.phys_tables_base_addr_binary,
                  BSP.phys_tables_base_addr_offset_in_kernel_elf)
end
