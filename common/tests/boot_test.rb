# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

require_relative 'test'
require 'timeout'

# Check for an expected string when booting the kernel in QEMU.
class BootTest < Test
    MAX_WAIT_SECS = 5

    def initialize(qemu_cmd, expected_print)
        super()

        @qemu_cmd = qemu_cmd
        @expected_print = expected_print

        @test_name = 'Boot test'
        @test_description = "Checking for the string: '#{@expected_print}'"
        @test_output = []
        @test_error = nil
    end

    private

    def expected_string_observed?(qemu_output)
        qemu_output.join.include?(@expected_print)
    end

    # Convert the recorded output to an array of lines.
    def post_process_and_add_output(qemu_output)
        @test_output += qemu_output.join.split("\n")
    end

    # override
    def setup
        @qemu_serial = IO.popen(@qemu_cmd, err: '/dev/null')
        @qemu_pid = @qemu_serial.pid
    end

    # override
    def cleanup
        Timeout.timeout(MAX_WAIT_SECS) do
            Process.kill('TERM', @qemu_pid)
            Process.wait
        end
    rescue StandardError => e
        puts 'QEMU graceful shutdown didn\'t work. Skipping it.'
        puts e
    end

    def run_concrete_test
        qemu_output = []
        Timeout.timeout(MAX_WAIT_SECS) do
            while IO.select([@qemu_serial])
                qemu_output << @qemu_serial.read_nonblock(1024)

                if expected_string_observed?(qemu_output)
                    @test_error = false
                    break
                end
            end
        end
    rescue EOFError
        @test_error = 'QEMU quit unexpectedly'
    rescue Timeout::Error
        @test_error = 'Timed out waiting for magic string'
    rescue StandardError => e
        @test_error = e.message
    ensure
        post_process_and_add_output(qemu_output)
    end
end
