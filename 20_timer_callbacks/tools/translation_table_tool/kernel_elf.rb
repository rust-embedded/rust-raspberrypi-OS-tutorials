# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

# KernelELF
class KernelELF
    SECTION_FLAG_ALLOC = 2

    def initialize(kernel_elf_path)
        @elf = ELFTools::ELFFile.new(File.open(kernel_elf_path))
        @symtab_section = @elf.section_by_name('.symtab')
    end

    def machine
        @elf.machine.to_sym
    end

    def symbol_value(symbol_name)
        @symtab_section.symbol_by_name(symbol_name).header.st_value
    end

    def segment_containing_virt_addr(virt_addr)
        @elf.each_segments do |segment|
            return segment if segment.vma_in?(virt_addr)
        end
    end

    def virt_to_phys(virt_addr)
        segment = segment_containing_virt_addr(virt_addr)
        translation_offset = segment.header.p_vaddr - segment.header.p_paddr

        virt_addr - translation_offset
    end

    def virt_addr_to_file_offset(virt_addr)
        segment = segment_containing_virt_addr(virt_addr)
        segment.vma_to_offset(virt_addr)
    end

    def sections_in_segment(segment)
        head = segment.mem_head
        tail = segment.mem_tail

        sections = @elf.each_sections.select do |section|
            file_offset = section.header.sh_addr
            flags = section.header.sh_flags

            file_offset >= head && file_offset < tail && (flags & SECTION_FLAG_ALLOC != 0)
        end

        sections.map(&:name).join(' ')
    end

    def select_load_segments
        @elf.each_segments.select do |segment|
            segment.instance_of?(ELFTools::Segments::LoadSegment)
        end
    end

    def segment_get_acc_perms(segment)
        if segment.readable? && segment.writable?
            :ReadWrite
        elsif segment.readable?
            :ReadOnly
        else
            :Invalid
        end
    end

    def update_max_section_name_length(descriptors)
        MappingDescriptor.update_max_section_name_length(descriptors.map { |i| i.name.size }.max)
    end

    def generate_mapping_descriptors
        descriptors = select_load_segments.map do |segment|
            # Assume each segment is page aligned.
            size = segment.mem_size.align_up(BSP.kernel_granule::SIZE)
            virt_start_addr = segment.header.p_vaddr
            phys_start_addr = segment.header.p_paddr
            acc_perms = segment_get_acc_perms(segment)
            execute_never = !segment.executable?
            section_names = sections_in_segment(segment)

            virt_region = MemoryRegion.new(virt_start_addr, size, BSP.kernel_granule::SIZE)
            phys_region = MemoryRegion.new(phys_start_addr, size, BSP.kernel_granule::SIZE)
            attributes = AttributeFields.new(:CacheableDRAM, acc_perms, execute_never)

            MappingDescriptor.new(section_names, virt_region, phys_region, attributes)
        end

        update_max_section_name_length(descriptors)
        descriptors
    end
end
