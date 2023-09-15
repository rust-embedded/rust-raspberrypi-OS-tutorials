# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2019-2023 Andre Richter <andre.o.richter@gmail.com>

require 'English'
require_relative 'test'
require 'io/wait'

# A test that only inspects the exit code of the QEMU binary.
class ExitCodeTest < Test
    MAX_WAIT_SECS = 5

    def initialize(qemu_cmd, test_name)
        super()

        @qemu_cmd = qemu_cmd

        @test_name = test_name
        @test_description = nil
        @test_output = []
        @test_error = nil
    end

    private

    # override
    def setup
        @qemu_serial = IO.popen(@qemu_cmd)
    end

    # override
    # Convert the recorded output to an array of lines, and extract the test description.
    def finish
        @test_output = @test_output.join.split("\n")
        @test_description = @test_output.shift
    end

    # override
    def run_concrete_test
        Timeout.timeout(MAX_WAIT_SECS) do
            @test_output << @qemu_serial.read_nonblock(1024) while @qemu_serial.wait_readable
        end
    rescue EOFError
        @qemu_serial.close
        @test_error = $CHILD_STATUS.to_i.zero? ? false : 'QEMU exit status != 0'
    rescue Timeout::Error
        @test_error = 'Timed out waiting for test'
    rescue StandardError => e
        @test_error = e.inspect
    end
end
