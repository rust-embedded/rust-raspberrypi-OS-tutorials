# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

# KernelELF
class KernelELF
    attr_reader :path

    def initialize(kernel_elf_path, kernel_symbols_section, num_kernel_symbols)
        @elf = ELFTools::ELFFile.new(File.open(kernel_elf_path))
        @symtab_section = @elf.section_by_name('.symtab')

        @path = kernel_elf_path
        fetch_values(kernel_symbols_section, num_kernel_symbols)
    end

    private

    def fetch_values(kernel_symbols_section, num_kernel_symbols)
        sym = @symtab_section.symbol_by_name(num_kernel_symbols)
        raise "Symbol \"#{num_kernel_symbols}\" not found" if sym.nil?

        @num_kernel_symbols = sym

        section = @elf.section_by_name(kernel_symbols_section)
        raise "Section \"#{kernel_symbols_section}\" not found" if section.nil?

        @kernel_symbols_section = section
    end

    def num_kernel_symbols_virt_addr
        @num_kernel_symbols.header.st_value
    end

    def segment_containing_virt_addr(virt_addr)
        @elf.each_segments do |segment|
            return segment if segment.vma_in?(virt_addr)
        end
    end

    def virt_addr_to_file_offset(virt_addr)
        segment = segment_containing_virt_addr(virt_addr)
        segment.vma_to_offset(virt_addr)
    end

    public

    def symbols
        non_zero_symbols = @symtab_section.symbols.reject { |sym| sym.header.st_size.zero? }
        non_zero_symbols.sort_by { |sym| sym.header.st_value }
    end

    def num_symbols
        symbols.size
    end

    def kernel_symbols_section_virt_addr
        @kernel_symbols_section.header.sh_addr.to_i
    end

    def kernel_symbols_section_size
        @kernel_symbols_section.header.sh_size.to_i
    end

    def kernel_symbols_section_offset_in_file
        virt_addr_to_file_offset(kernel_symbols_section_virt_addr)
    end

    def num_kernel_symbols_offset_in_file
        virt_addr_to_file_offset(num_kernel_symbols_virt_addr)
    end
end
