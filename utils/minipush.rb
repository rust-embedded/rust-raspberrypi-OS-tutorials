#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

require 'rubygems'
require 'bundler/setup'
require 'io/console'
require 'colorize'
require 'ruby-progressbar'
require 'serialport'
require 'timeout'
require_relative 'minipush/progressbar_patch'

class ConnectionError < StandardError; end
class ProtocolError < StandardError; end

# The main class
class MiniPush
    def initialize(serial_name, binary_image_path)
        @target_serial_name = serial_name
        @target_serial = nil
        @binary_image_path = binary_image_path
        @binary_size = nil
        @binary_image = nil
        @host_console = IO.console
    end

    private

    def serial_connected?
        File.exist?(@target_serial_name)
    end

    def wait_for_serial
        loop do
            break if serial_connected?

            print "\r[MP] ⏳ Waiting for #{@target_serial_name}"
            sleep(1)
        end
    end

    def open_serial
        wait_for_serial

        @target_serial = SerialPort.new(@target_serial_name, 230_400, 8, 1, SerialPort::NONE)

        # Ensure all output is immediately flushed to the device.
        @target_serial.sync = true
    rescue Errno::EACCES => e
        puts
        puts '[MP] 🚫 ' + e.message + " - Maybe try with 'sudo'"
        exit
    else
        puts
        puts '[MP] ✅ Connected'
    end

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
            format: '[MP] ⏩ Pushing %k KiB %b🦀%i %p%% %r KiB/s %a',
            rate_scale: ->(rate) { rate / 1024 },
            length: 92
        )

        # Send in 512 byte chunks.
        while pb.progress < pb.total
            part = @binary_image.slice(pb.progress, 512)
            pb.progress += @target_serial.write(part)
        end
    end

    def terminal
        @host_console.raw!

        Thread.abort_on_exception = true
        Thread.report_on_exception = false

        # Receive from target and print on host console.
        target_to_host = Thread.new do
            loop do
                char = @target_serial.getc

                raise ConnectionError if char.nil?

                # onlcr
                @host_console.putc("\r") if char == "\n"
                @host_console.putc(char)
            end
        end

        # Transmit host console input to target.
        loop do
            c = @host_console.getc

            # CTRL + C in raw mode was pressed
            if c == "\u{3}"
                target_to_host.kill
                break
            end

            @target_serial.putc(c)
        end
    end

    def connetion_reset
        @target_serial&.close
        @target_serial = nil
        @host_console.cooked!
    end

    # When the serial lost power or was removed during R/W operation.
    def handle_reconnect
        connetion_reset

        puts
        puts '[MP] ⚡ ' + 'Connection Error: Reinsert the USB serial again'.light_red
    end

    # When the serial is still powered.
    def handle_protocol_error
        connetion_reset

        puts
        puts '[MP] ⚡ ' + 'Protocol Error: Remove and insert the USB serial again'.light_red
        sleep(1) while serial_connected?
    end

    def handle_unexpected(error)
        connetion_reset

        puts
        puts '[MP] ⚡ ' + "Unexpected Error: #{error.inspect}".light_red
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
        puts '[MP] Bye 👋'
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
