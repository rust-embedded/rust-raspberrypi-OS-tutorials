#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

require 'rubygems'
require 'bundler/setup'
require 'colorize'
require 'elftools'

require_relative 'kernel_elf'
require_relative 'cmds'

KERNEL_SYMBOLS_SECTION = '.kernel_symbols'
NUM_KERNEL_SYMBOLS = 'NUM_KERNEL_SYMBOLS'

cmd = ARGV[0]

kernel_elf_path = ARGV[1]
kernel_elf = KernelELF.new(kernel_elf_path, KERNEL_SYMBOLS_SECTION, NUM_KERNEL_SYMBOLS)

case cmd
when '--gen_symbols'
    output_file = ARGV[2]

    print 'Generating'.rjust(12).green.bold
    puts ' Symbols source file'

    generate_symbols(kernel_elf, output_file)
when '--get_symbols_section_virt_addr'
    addr = get_symbols_section_virt_addr(kernel_elf)

    puts "0x#{addr.to_s(16)}"
when '--patch_data'
    symbols_blob_path = ARGV[2]
    num_symbols = kernel_elf.num_symbols

    print 'Patching'.rjust(12).green.bold
    puts " Symbols blob and number of symbols (#{num_symbols}) into ELF"

    patch_symbol_data(kernel_elf, symbols_blob_path)
    patch_num_symbols(kernel_elf)
else
    raise
end
