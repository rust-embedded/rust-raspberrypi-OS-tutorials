#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2019-2020 Andre Richter <andre.o.richter@gmail.com>

require 'English'
require 'pty'

# Test base class.
class Test
    INDENT = '         '

    def print_border(status)
        puts
        puts "#{INDENT}-------------------------------------------------------------------"
        puts status
        puts "#{INDENT}-------------------------------------------------------------------\n\n\n"
    end

    def print_error(error)
        puts
        print_border("#{INDENT}❌ Failure: #{error}: #{@test_name}")
    end

    def print_success
        print_border("#{INDENT}✅ Success: #{@test_name}")
    end

    def print_output
        puts "#{INDENT}-------------------------------------------------------------------"
        print INDENT
        print '🦀 '
        print @output.join('').gsub("\n", "\n" + INDENT)
    end

    def finish(error)
        print_output

        exit_code = if error
                        print_error(error)
                        false
                    else
                        print_success
                        true
                    end

        exit(exit_code)
    end
end

# Executes tests with console I/O.
class ConsoleTest < Test
    def initialize(binary, qemu_cmd, test_name, console_subtests)
        @binary = binary
        @qemu_cmd = qemu_cmd
        @test_name = test_name
        @console_subtests = console_subtests
        @cur_subtest = 1
        @output = ["Running #{@console_subtests.length} console-based tests\n",
                   "-------------------------------------------------------------------\n\n"]
    end

    def format_test_name(number, name)
        formatted_name = number.to_s.rjust(3) + '. ' + name
        formatted_name.ljust(63, '.')
    end

    def run_subtest(subtest, qemu_out, qemu_in)
        @output << format_test_name(@cur_subtest, subtest.name)

        subtest.run(qemu_out, qemu_in)

        @output << "[ok]\n"
        @cur_subtest += 1
    end

    def exec
        error = false

        PTY.spawn(@qemu_cmd) do |qemu_out, qemu_in|
            begin
                @console_subtests.each { |t| run_subtest(t, qemu_out, qemu_in) }
            rescue StandardError => e
                error = e.message
            end

            finish(error)
        end
    end
end

# A wrapper around the bare QEMU invocation.
class RawTest < Test
    MAX_WAIT_SECS = 5

    def initialize(binary, qemu_cmd, test_name)
        @binary = binary
        @qemu_cmd = qemu_cmd
        @test_name = test_name
        @output = []
    end

    def exec
        error = 'Timed out waiting for test'
        io = IO.popen(@qemu_cmd)

        while IO.select([io], nil, nil, MAX_WAIT_SECS)
            begin
                @output << io.read_nonblock(1024)
            rescue EOFError
                io.close
                error = $CHILD_STATUS.to_i != 0
                break
            end
        end

        finish(error)
    end
end

##--------------------------------------------------------------------------------------------------
## Script entry point
##--------------------------------------------------------------------------------------------------
binary = ARGV.last
test_name = binary.gsub(%r{.*deps/}, '').split('-')[0]
console_test_file = 'tests/' + test_name + '.rb'
qemu_cmd = ARGV.join(' ')

test_runner = if File.exist?(console_test_file)
                  load console_test_file
                  # subtest_collection is provided by console_test_file
                  ConsoleTest.new(binary, qemu_cmd, test_name, subtest_collection)
              else
                  RawTest.new(binary, qemu_cmd, test_name)
              end

test_runner.exec
