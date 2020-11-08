#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

require_relative 'miniterm'
require 'ruby-progressbar'
require_relative 'minipush/progressbar_patch'
require 'timeout'

class ProtocolError < StandardError; end

# The main class
class MiniPush < MiniTerm
    def initialize(serial_name, binary_image_path)
        super(serial_name)

        @name_short = 'MP'
        @binary_image_path = binary_image_path
        @binary_size = nil
        @binary_image = nil
    end

    private

    # The three characters signaling the request token are expected to arrive as the last three
    # characters _at the end_ of a character stream (e.g. after a header print from Miniload).
    def wait_for_binary_request
        Timeout.timeout(7) do
            loop do
                received = @target_serial.readpartial(4096)

                raise ConnectionError if received.nil?

                if received.chars.last(3) == ["\u{3}", "\u{3}", "\u{3}"]
                    # Print the last chunk minus the request token.
                    print received[0..-4]
                    return
                end

                print received
            end
        end
    end

    def load_binary
        @binary_size = File.size(@binary_image_path)
        @binary_image = File.binread(@binary_image_path)
    end

    def send_size
        @target_serial.print([@binary_size].pack('L<'))
        raise ProtocolError if @target_serial.read(2) != 'OK'
    end

    def send_binary
        pb = ProgressBar.create(
            total: @binary_size,
            format: "[#{@name_short}] ⏩ Pushing %k KiB %b🦀%i %p%% %r KiB/s %a",
            rate_scale: ->(rate) { rate / 1024 },
            length: 92
        )

        # Send in 512 byte chunks.
        while pb.progress < pb.total
            part = @binary_image.slice(pb.progress, 512)
            pb.progress += @target_serial.write(part)
        end
    end

    # When the serial is still powered.
    def handle_protocol_error
        connetion_reset

        puts
        puts "[#{@name_short}] ⚡ " \
             "#{'Protocol Error: Remove and insert the USB serial again'.light_red}"
        sleep(1) while serial_connected?
    end

    public

    def run
        open_serial
        wait_for_binary_request
        load_binary
        send_size
        send_binary
        terminal
    rescue ConnectionError, EOFError, Errno::EIO
        handle_reconnect
        retry
    rescue ProtocolError, Timeout::Error
        handle_protocol_error
        retry
    rescue StandardError => e
        handle_unexpected(e)
    ensure
        connetion_reset
        puts
        puts "[#{@name_short}] Bye 👋"
    end
end

puts 'Minipush 1.0'.cyan
puts

# CTRL + C handler. Only here to suppress Ruby's default exception print.
trap('INT') do
    # The `ensure` block from `MiniPush::run` will run after exit, restoring console state.
    exit
end

MiniPush.new(ARGV[0], ARGV[1]).run
