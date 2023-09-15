# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

def generate_symbols(kernel_elf, output_file)
    File.open(output_file, 'w') do |file|
        header = <<~HEREDOC
            use debug_symbol_types::Symbol;

            # [no_mangle]
            # [link_section = ".rodata.symbol_desc"]
            static KERNEL_SYMBOLS: [Symbol; #{kernel_elf.num_symbols}] = [
        HEREDOC

        file.write(header)
        kernel_elf.symbols.each do |sym|
            value = sym.header.st_value
            size = sym.header.st_size
            name = sym.name

            file.write("    Symbol::new(#{value}, #{size}, \"#{name}\"),\n")
        end
        file.write("];\n")
    end
end

def get_symbols_section_virt_addr(kernel_elf)
    kernel_elf.kernel_symbols_section_virt_addr
end

def patch_symbol_data(kernel_elf, symbols_blob_path)
    symbols_blob = File.binread(symbols_blob_path)

    raise if symbols_blob.size > kernel_elf.kernel_symbols_section_size

    File.binwrite(kernel_elf.path, File.binread(symbols_blob_path),
                  kernel_elf.kernel_symbols_section_offset_in_file)
end

def patch_num_symbols(kernel_elf)
    num_packed = [kernel_elf.num_symbols].pack('Q<*') # "Q" == uint64_t, "<" == little endian
    File.binwrite(kernel_elf.path, num_packed, kernel_elf.num_kernel_symbols_offset_in_file)
end
