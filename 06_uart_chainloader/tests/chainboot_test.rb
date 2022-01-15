# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

require_relative '../../common/serial/minipush'
require_relative '../../common/tests/boot_test'
require 'pty'

# Match for the last print that 'demo_payload_rpiX.img' produces.
EXPECTED_PRINT = 'Echoing input now'

# Extend BootTest so that it listens on the output of a MiniPush instance, which is itself connected
# to a QEMU instance instead of a real HW.
class ChainbootTest < BootTest
    MINIPUSH = '../common/serial/minipush.rb'
    MINIPUSH_POWER_TARGET_REQUEST = 'Please power the target now'

    def initialize(qemu_cmd, payload_path)
        super(qemu_cmd, EXPECTED_PRINT)

        @test_name = 'Boot test using Minipush'

        @payload_path = payload_path
    end

    private

    # override
    def post_process_and_add_output(output)
        temp = output.join.split("\r\n")

        # Should a line have solo carriage returns, remove any overridden parts of the string.
        temp.map! { |x| x.gsub(/.*\r/, '') }

        @test_output += temp
    end

    def wait_for_minipush_power_request(mp_out)
        output = []
        Timeout.timeout(MAX_WAIT_SECS) do
            loop do
                output << mp_out.gets
                break if output.last.include?(MINIPUSH_POWER_TARGET_REQUEST)
            end
        end
    rescue Timeout::Error
        @test_error = 'Timed out waiting for power request'
    rescue StandardError => e
        @test_error = e.message
    ensure
        post_process_and_add_output(output)
    end

    # override
    def setup
        pty_main, pty_secondary = PTY.open
        mp_out, _mp_in = PTY.spawn("ruby #{MINIPUSH} #{pty_secondary.path} #{@payload_path}")

        # Wait until MiniPush asks for powering the target.
        wait_for_minipush_power_request(mp_out)

        # Now is the time to start QEMU with the chainloader binary. QEMU's virtual tty is connected
        # to the MiniPush instance spawned above, so that the two processes talk to each other.
        Process.spawn(@qemu_cmd, in: pty_main, out: pty_main)

        # The remainder of the test is done by the parent class' run_concrete_test, which listens on
        # @qemu_serial. Hence, point it to MiniPush's output.
        @qemu_serial = mp_out
    end
end

##--------------------------------------------------------------------------------------------------
## Execution starts here
##--------------------------------------------------------------------------------------------------
payload_path = ARGV.pop
qemu_cmd = ARGV.join(' ')

ChainbootTest.new(qemu_cmd, payload_path).run
