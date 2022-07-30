#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

require_relative 'miniterm'
require 'ruby-progressbar'
require_relative 'minipush/progressbar_patch'
require 'timeout'

class ProtocolError < StandardError; end

# The main class
class MiniPush < MiniTerm
    def initialize(serial_name, payload_path)
        super(serial_name)

        @name_short = 'MP' # override
        @payload_path = payload_path
        @payload_size = nil
        @payload_data = nil
    end

    private

    # The three characters signaling the request token form the consecutive sequence "\x03\x03\x03".
    def wait_for_payload_request
        puts "[#{@name_short}] 🔌 Please power the target now"

        # Timeout for the request token starts after the first sign of life was received.
        received = @target_serial.readpartial(4096)
        Timeout.timeout(10) do
            count = 0

            loop do
                raise ProtocolError if received.nil?

                received.chars.each do |c|
                    if c == "\u{3}"
                        count += 1
                        return true if count == 3
                    else
                        # A normal character resets token counting.
                        count = 0

                        print c
                    end
                end

                received = @target_serial.readpartial(4096)
            end
        end
    end

    def load_payload
        @payload_size = File.size(@payload_path)
        @payload_data = File.binread(@payload_path)
    end

    def send_size
        @target_serial.print([@payload_size].pack('L<'))
        raise ProtocolError if @target_serial.read(2) != 'OK'
    end

    def send_payload
        pb = ProgressBar.create(
            total: @payload_size,
            format: "[#{@name_short}] ⏩ Pushing %k KiB %b🦀%i %p%% %r KiB/s %a",
            rate_scale: ->(rate) { rate / 1024 },
            length: 92,
            output: $stdout
        )

        # Send in 512 byte chunks.
        while pb.progress < pb.total
            part = @payload_data.slice(pb.progress, 512)
            pb.progress += @target_serial.write(part)
        end
    end

    # override
    def handle_reconnect(_error)
        connection_reset

        puts
        puts "[#{@name_short}] ⚡ " \
             "#{'Connection or protocol Error: '.light_red}" \
             "#{'Remove power and USB serial. Reinsert serial first, then power'.light_red}"
        sleep(1) while serial_connected?
    end

    public

    # override
    def run
        open_serial
        wait_for_payload_request
        load_payload
        send_size
        send_payload
        terminal
    rescue ConnectionError, EOFError, Errno::EIO, ProtocolError, Timeout::Error => e
        handle_reconnect(e)
        retry
    rescue StandardError => e
        handle_unexpected(e)
    ensure
        connection_reset
        puts
        puts "[#{@name_short}] Bye 👋"
    end
end

##--------------------------------------------------------------------------------------------------
## Execution starts here
##--------------------------------------------------------------------------------------------------
if __FILE__ == $PROGRAM_NAME
    puts
    puts 'Minipush 1.0'.cyan
    puts

    # CTRL + C handler. Only here to suppress Ruby's default exception print.
    trap('INT') do
        # The `ensure` block from `MiniPush::run` will run after exit, restoring console state.
        exit
    end

    MiniPush.new(ARGV[0], ARGV[1]).run
end
