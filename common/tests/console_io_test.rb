# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2019-2021 Andre Richter <andre.o.richter@gmail.com>

require 'pty'
require_relative 'test'

# A test doing console I/O with the QEMU binary.
class ConsoleIOTest < Test
    def initialize(qemu_cmd, test_name, console_subtests)
        super()

        @qemu_cmd = qemu_cmd
        @console_subtests = console_subtests

        @test_name = test_name
        @test_description = "Running #{@console_subtests.length} console I/O tests"
        @test_output = []
        @test_error = nil
    end

    private

    def format_test_name(number, name)
        formatted_name = "#{number.to_s.rjust(3)}. #{name}"
        formatted_name.ljust(63, '.')
    end

    def run_subtest(subtest, test_id, qemu_out, qemu_in)
        @test_output << format_test_name(test_id, subtest.name)
        subtest.run(qemu_out, qemu_in)
        @test_output.last.concat('[ok]')
    end

    def run_concrete_test
        @test_error = false

        PTY.spawn(@qemu_cmd) do |qemu_out, qemu_in|
            @console_subtests.each_with_index do |t, i|
                run_subtest(t, i + 1, qemu_out, qemu_in)
            end
        rescue StandardError => e
            @test_error = e.message
        end
    end
end
