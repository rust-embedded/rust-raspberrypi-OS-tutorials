# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

# Bitfield manipulation.
class BitField
    def initialize
        @value = 0
    end

    def self.attr_bitfield(name, offset, num_bits)
        define_method("#{name}=") do |bits|
            mask = (2**num_bits) - 1

            raise "Input out of range: #{name} = 0x#{bits.to_s(16)}" if (bits & ~mask).positive?

            # Clear bitfield
            @value &= ~(mask << offset)

            # Set it
            @value |= (bits << offset)
        end
    end

    def to_i
        @value
    end

    def size_in_byte
        8
    end
end

# An array class that knows its memory location.
class CArray < Array
    attr_reader :phys_start_addr

    def initialize(phys_start_addr, size, &block)
        @phys_start_addr = phys_start_addr

        super(size, &block)
    end

    def size_in_byte
        inject(0) { |sum, n| sum + n.size_in_byte }
    end
end

#---------------------------------------------------------------------------------------------------
# Arch::
#---------------------------------------------------------------------------------------------------
module Arch
#---------------------------------------------------------------------------------------------------
# Arch::ARMv8
#---------------------------------------------------------------------------------------------------
module ARMv8
# ARMv8 Table Descriptor.
class Stage1TableDescriptor < BitField
    module NextLevelTableAddr
        OFFSET = 16
        NUMBITS = 32
    end

    module Type
        OFFSET = 1
        NUMBITS = 1

        BLOCK = 0
        TABLE = 1
    end

    module Valid
        OFFSET = 0
        NUMBITS = 1

        FALSE = 0
        TRUE = 1
    end

    attr_bitfield(:__next_level_table_addr, NextLevelTableAddr::OFFSET, NextLevelTableAddr::NUMBITS)
    attr_bitfield(:type, Type::OFFSET, Type::NUMBITS)
    attr_bitfield(:valid, Valid::OFFSET, Valid::NUMBITS)

    def next_level_table_addr=(addr)
        addr >>= Granule64KiB::SHIFT

        self.__next_level_table_addr = addr
    end

    private :__next_level_table_addr=
end

# ARMv8 level 3 page descriptor.
class Stage1PageDescriptor < BitField
    module UXN
        OFFSET = 54
        NUMBITS = 1

        FALSE = 0
        TRUE = 1
    end

    module PXN
        OFFSET = 53
        NUMBITS = 1

        FALSE = 0
        TRUE = 1
    end

    module OutputAddr
        OFFSET = 16
        NUMBITS = 32
    end

    module AF
        OFFSET = 10
        NUMBITS = 1

        FALSE = 0
        TRUE = 1
    end

    module SH
        OFFSET = 8
        NUMBITS = 2

        INNER_SHAREABLE = 0b11
    end

    module AP
        OFFSET = 6
        NUMBITS = 2

        RW_EL1 = 0b00
        RO_EL1 = 0b10
    end

    module AttrIndx
        OFFSET = 2
        NUMBITS = 3
    end

    module Type
        OFFSET = 1
        NUMBITS = 1

        RESERVED_INVALID = 0
        PAGE = 1
    end

    module Valid
        OFFSET = 0
        NUMBITS = 1

        FALSE = 0
        TRUE = 1
    end

    attr_bitfield(:uxn, UXN::OFFSET, UXN::NUMBITS)
    attr_bitfield(:pxn, PXN::OFFSET, PXN::NUMBITS)
    attr_bitfield(:__output_addr, OutputAddr::OFFSET, OutputAddr::NUMBITS)
    attr_bitfield(:af, AF::OFFSET, AF::NUMBITS)
    attr_bitfield(:sh, SH::OFFSET, SH::NUMBITS)
    attr_bitfield(:ap, AP::OFFSET, AP::NUMBITS)
    attr_bitfield(:attr_indx, AttrIndx::OFFSET, AttrIndx::NUMBITS)
    attr_bitfield(:type, Type::OFFSET, Type::NUMBITS)
    attr_bitfield(:valid, Valid::OFFSET, Valid::NUMBITS)

    def output_addr=(addr)
        addr >>= Granule64KiB::SHIFT

        self.__output_addr = addr
    end

    private :__output_addr=
end

# Translation table representing the structure defined in translation_table.rs.
class TranslationTable
    module MAIR
        NORMAL = 1
    end

    def initialize
        do_sanity_checks

        num_lvl2_tables = BSP.kernel_virt_addr_space_size >> Granule512MiB::SHIFT

        @lvl3 = new_lvl3(num_lvl2_tables, BSP.phys_addr_of_kernel_tables)

        @lvl2_phys_start_addr = @lvl3.phys_start_addr + @lvl3.size_in_byte
        @lvl2 = new_lvl2(num_lvl2_tables, @lvl2_phys_start_addr)

        populate_lvl2_entries
    end

    def map_at(virt_region, phys_region, attributes)
        return if virt_region.empty?

        raise if virt_region.size != phys_region.size
        raise if phys_region.last > BSP.phys_addr_space_end_page

        virt_region.zip(phys_region).each do |virt_page, phys_page|
            desc = page_descriptor_from(virt_page)
            set_lvl3_entry(desc, phys_page, attributes)
        end
    end

    def to_binary
        data = @lvl3.flatten.map(&:to_i) + @lvl2.map(&:to_i)
        data.pack('Q<*') # "Q" == uint64_t, "<" == little endian
    end

    def phys_tables_base_addr_binary
        [@lvl2_phys_start_addr].pack('Q<*') # "Q" == uint64_t, "<" == little endian
    end

    def phys_tables_base_addr
        @lvl2_phys_start_addr
    end

    private

    def do_sanity_checks
        raise unless BSP.kernel_granule::SIZE == Granule64KiB::SIZE
        raise unless (BSP.kernel_virt_addr_space_size % Granule512MiB::SIZE).zero?
    end

    def new_lvl3(num_lvl2_tables, start_addr)
        CArray.new(start_addr, num_lvl2_tables) do
            temp = CArray.new(start_addr, 8192) do
                Stage1PageDescriptor.new
            end
            start_addr += temp.size_in_byte

            temp
        end
    end

    def new_lvl2(num_lvl2_tables, start_addr)
        CArray.new(start_addr, num_lvl2_tables) do
            Stage1TableDescriptor.new
        end
    end

    def populate_lvl2_entries
        @lvl2.each_with_index do |descriptor, i|
            descriptor.next_level_table_addr = @lvl3[i].phys_start_addr
            descriptor.type = Stage1TableDescriptor::Type::TABLE
            descriptor.valid = Stage1TableDescriptor::Valid::TRUE
        end
    end

    def lvl2_lvl3_index_from(addr)
        addr -= BSP.kernel_virt_start_addr

        lvl2_index = addr >> Granule512MiB::SHIFT
        lvl3_index = (addr & Granule512MiB::MASK) >> Granule64KiB::SHIFT

        raise unless lvl2_index < @lvl2.size

        [lvl2_index, lvl3_index]
    end

    def page_descriptor_from(virt_addr)
        lvl2_index, lvl3_index = lvl2_lvl3_index_from(virt_addr)

        @lvl3[lvl2_index][lvl3_index]
    end

    # rubocop:disable Metrics/MethodLength
    def set_attributes(desc, attributes)
        case attributes.mem_attributes
        when :CacheableDRAM
            desc.sh = Stage1PageDescriptor::SH::INNER_SHAREABLE
            desc.attr_indx = MAIR::NORMAL
        else
            raise 'Invalid input'
        end

        desc.ap = case attributes.acc_perms
                  when :ReadOnly
                      Stage1PageDescriptor::AP::RO_EL1
                  when :ReadWrite
                      Stage1PageDescriptor::AP::RW_EL1
                  else
                      raise 'Invalid input'

                  end

        desc.pxn = if attributes.execute_never
                       Stage1PageDescriptor::PXN::TRUE
                   else
                       Stage1PageDescriptor::PXN::FALSE
                   end

        desc.uxn = Stage1PageDescriptor::UXN::TRUE
    end
    # rubocop:enable Metrics/MethodLength

    def set_lvl3_entry(desc, output_addr, attributes)
        desc.output_addr = output_addr
        desc.af = Stage1PageDescriptor::AF::TRUE
        desc.type = Stage1PageDescriptor::Type::PAGE
        desc.valid = Stage1PageDescriptor::Valid::TRUE

        set_attributes(desc, attributes)
    end
end
end
end
