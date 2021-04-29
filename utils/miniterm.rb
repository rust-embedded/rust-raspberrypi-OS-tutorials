#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

require 'rubygems'
require 'bundler/setup'
require 'io/console'
require 'colorize'
require 'serialport'

SERIAL_BAUD = 921_600

class ConnectionError < StandardError; end

# The main class
class MiniTerm
    def initialize(serial_name)
        @name_short = 'MT'
        @target_serial_name = serial_name
        @target_serial = nil
        @host_console = IO.console
    end

    private

    def serial_connected?
        File.exist?(@target_serial_name)
    end

    def wait_for_serial
        return if serial_connected?

        puts "[#{@name_short}] ⏳ Waiting for #{@target_serial_name}"
        loop do
            sleep(1)
            break if serial_connected?
        end
    end

    def open_serial
        wait_for_serial

        @target_serial = SerialPort.new(@target_serial_name, SERIAL_BAUD, 8, 1, SerialPort::NONE)

        # Ensure all output is immediately flushed to the device.
        @target_serial.sync = true
    rescue Errno::EACCES => e
        puts "[#{@name_short}] 🚫 #{e.message} - Maybe try with 'sudo'"
        exit
    else
        puts "[#{@name_short}] ✅ Serial connected"
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

                # Translate incoming newline to newline + carriage return.
                @host_console.putc("\r") if char == "\n"
                @host_console.putc(char)
            end
        end

        # Transmit host console input to target.
        loop do
            c = @host_console.getc

            # CTRL + C in raw mode was pressed.
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
    def handle_reconnect(_error)
        connetion_reset

        puts
        puts "[#{@name_short}] ⚡ #{'Connection Error: Reinsert the USB serial again'.light_red}"
    end

    def handle_unexpected(error)
        connetion_reset

        puts
        puts "[#{@name_short}] ⚡ #{"Unexpected Error: #{error.inspect}".light_red}"
    end

    public

    def run
        open_serial
        terminal
    rescue ConnectionError, EOFError, Errno::EIO => e
        handle_reconnect(e)
        retry
    rescue StandardError => e
        handle_unexpected(e)
    ensure
        connetion_reset
        puts
        puts "[#{@name_short}] Bye 👋"
    end
end

##--------------------------------------------------------------------------------------------------
## Execution starts here
##--------------------------------------------------------------------------------------------------
if __FILE__ == $PROGRAM_NAME
    puts
    puts 'Miniterm 1.0'.cyan
    puts

    # CTRL + C handler. Only here to suppress Ruby's default exception print.
    trap('INT') do
        # The `ensure` block from `MiniTerm::run` will run after exit, restoring console state.
        exit
    end

    MiniTerm.new(ARGV[0]).run
end
