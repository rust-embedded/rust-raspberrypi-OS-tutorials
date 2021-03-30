#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

TARGET = ARGV[0].split('-').first.to_sym
BSP_TYPE = ARGV[1].to_sym
kernel_elf = ARGV[2]

require 'rubygems'
require 'bundler/setup'
require 'colorize'

require_relative 'generic'
require_relative 'bsp'
require_relative 'arch'

puts
puts 'Precomputing kernel translation tables and patching kernel ELF'.cyan

start = Time.now

BSP = case BSP_TYPE
      when :rpi3, :rpi4
          RaspberryPi.new(kernel_elf)
      else
          raise
      end

TRANSLATION_TABLES = case TARGET
                     when :aarch64
                         Arch::ARMv8::TranslationTable.new
                     else
                         raise
                     end

BSP.kernel_map_binary

kernel_patch_tables(kernel_elf)
kernel_patch_base_addr(kernel_elf)

elapsed = Time.now - start

print 'Finished'.rjust(12).green.bold
puts " in #{elapsed.round(2)}s"
