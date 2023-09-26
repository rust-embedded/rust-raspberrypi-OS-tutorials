#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2019-2023 Andre Richter <andre.o.richter@gmail.com>

file_dir = File.dirname(__FILE__)
$LOAD_PATH.unshift(file_dir) unless $LOAD_PATH.include?(file_dir)

require 'boot_test'
require 'console_io_test'
require 'exit_code_test'

qemu_cmd = ARGV.join(' ')
binary = ARGV.last
test_name = binary.gsub(%r{.*deps/}, '').split('-')[0]

# Check if virtual manifest (tutorial 12 or later) or not
path_prefix = File.exist?('kernel/Cargo.toml') ? 'kernel/' : ''

case test_name
when 'kernel8.img'
    load "#{path_prefix}tests/boot_test_string.rb" # provides 'EXPECTED_PRINT'
    BootTest.new(qemu_cmd, EXPECTED_PRINT).run # Doesn't return

when 'libkernel'
    ExitCodeTest.new(qemu_cmd, 'Kernel library unit tests').run # Doesn't return

else
    console_test_file = "#{path_prefix}tests/#{test_name}.rb"
    test_name.concat('.rs')
    test = if File.exist?(console_test_file)
               load console_test_file # provides 'subtest_collection'
               ConsoleIOTest.new(qemu_cmd, test_name, subtest_collection)
           else
               ExitCodeTest.new(qemu_cmd, test_name)
           end

    test.run # Doesn't return
end
