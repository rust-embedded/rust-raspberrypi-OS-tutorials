#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2019-2021 Andre Richter <andre.o.richter@gmail.com>

require_relative 'boot_test'
require_relative 'console_io_test'
require_relative 'exit_code_test'

qemu_cmd = ARGV.join(' ')
binary = ARGV.last
test_name = binary.gsub(%r{.*deps/}, '').split('-')[0]

case test_name
when 'kernel8.img'
    load 'tests/boot_test_string.rb' # provides 'EXPECTED_PRINT'
    BootTest.new(qemu_cmd, EXPECTED_PRINT).run # Doesn't return

when 'libkernel'
    ExitCodeTest.new(qemu_cmd, 'Kernel library unit tests').run # Doesn't return

else
    console_test_file = "tests/#{test_name}.rb"
    test_name.concat('.rs')
    test = if File.exist?(console_test_file)
               load console_test_file # provides 'subtest_collection'
               ConsoleIOTest.new(qemu_cmd, test_name, subtest_collection)
           else
               ExitCodeTest.new(qemu_cmd, test_name)
           end

    test.run # Doesn't return
end
