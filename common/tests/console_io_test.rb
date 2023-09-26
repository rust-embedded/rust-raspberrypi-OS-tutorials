# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2019-2023 Andre Richter <andre.o.richter@gmail.com>

require 'expect'
require 'pty'
require 'timeout'
require_relative 'test'

# Error class for when expect times out.
class ExpectTimeoutError < StandardError
    def initialize(string)
        super("Timeout while expecting string: #{string}")
    end
end

# Provide boilderplate for expecting a string and throwing an error on failure.
class SubtestBase
    TIMEOUT_SECONDS = 3

    def expect_or_raise(io, string, timeout = TIMEOUT_SECONDS)
        raise ExpectTimeoutError, string if io.expect(string, timeout).nil?
    end
end

# Monkey-patch IO so that we get access to the buffer of a previously unsuccessful expect().
class IO
    def unused_buf
        @unusedBuf
    end
end

# A wrapper class that records characters that have been received from a PTY.
class PTYLoggerWrapper
    def initialize(pty, linebreak = "\n")
        @pty = pty
        @linebreak = linebreak
        @log = []
    end

    def expect(pattern, timeout)
        result = @pty.expect(pattern, timeout)
        @log << if result.nil?
                    @pty.unused_buf
                else
                    result
                end

        result
    end

    def log
        @log.join.split(@linebreak)
    end
end

# A test doing console I/O with the QEMU binary.
class ConsoleIOTest < Test
    MAX_TIME_ALL_TESTS_SECONDS = 20

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

    # override
    def setup
        qemu_out, @qemu_in = PTY.spawn(@qemu_cmd)
        @qemu_out_wrapped = PTYLoggerWrapper.new(qemu_out)
    end

    # override
    def finish
        @test_output << ''
        @test_output << 'Console log:'
        @test_output += @qemu_out_wrapped.log.map { |line| "  #{line}" }
    end

    # override
    def run_concrete_test
        @test_error = false

        Timeout.timeout(MAX_TIME_ALL_TESTS_SECONDS) do
            @console_subtests.each_with_index do |t, i|
                run_subtest(t, i + 1, @qemu_out_wrapped, @qemu_in)
            end
        end
    rescue Errno::EIO => e
        @test_error = "#{e.inspect} - QEMU might have quit early"
    rescue Timeout::Error
        @test_error = "Overall time for tests exceeded (#{MAX_TIME_ALL_TESTS_SECONDS}s)"
    rescue StandardError => e
        @test_error = e.inspect
    end
end
