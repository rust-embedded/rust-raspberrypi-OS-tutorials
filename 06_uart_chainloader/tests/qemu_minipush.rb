# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

require_relative '../../utils/minipush'
require 'expect'
require 'timeout'

# Match for the last print that 'demo_payload_rpiX.img' produces.
EXPECTED_PRINT = 'Echoing input now'

# The main class
class QEMUMiniPush < MiniPush
    TIMEOUT_SECS = 3

    # override
    def initialize(qemu_cmd, binary_image_path)
        super(nil, binary_image_path)

        @qemu_cmd = qemu_cmd
    end

    private

    def quit_qemu_graceful
        Timeout.timeout(5) do
            pid = @target_serial.pid
            Process.kill('TERM', pid)
            Process.wait(pid)
        end
    end

    # override
    def open_serial
        @target_serial = IO.popen(@qemu_cmd, 'r+', err: '/dev/null')

        # Ensure all output is immediately flushed to the device.
        @target_serial.sync = true

        puts "[#{@name_short}] ✅ Serial connected"
    end

    # override
    def terminal
        result = @target_serial.expect(EXPECTED_PRINT, TIMEOUT_SECS)
        exit(1) if result.nil?

        puts result

        quit_qemu_graceful
    end

    public

    # override
    def connetion_reset; end

    # override
    def handle_reconnect(error)
        handle_unexpected(error)
    end
end

##--------------------------------------------------------------------------------------------------
## Execution starts here
##--------------------------------------------------------------------------------------------------
puts
puts 'QEMUMiniPush 1.0'.cyan
puts

# CTRL + C handler. Only here to suppress Ruby's default exception print.
trap('INT') do
    # The `ensure` block from `QEMUMiniPush::run` will run after exit, restoring console state.
    exit
end

binary_image_path = ARGV.pop
qemu_cmd = ARGV.join(' ')

QEMUMiniPush.new(qemu_cmd, binary_image_path).run
