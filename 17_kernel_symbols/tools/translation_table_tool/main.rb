#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

require 'rubygems'
require 'bundler/setup'
require 'colorize'
require 'elftools'

require_relative 'generic'
require_relative 'kernel_elf'
require_relative 'bsp'
require_relative 'arch'

BSP_TYPE = ARGV[0].to_sym
kernel_elf_path = ARGV[1]

start = Time.now

KERNEL_ELF = KernelELF.new(kernel_elf_path)

BSP = case BSP_TYPE
      when :rpi3, :rpi4
          RaspberryPi.new
      else
          raise
      end

TRANSLATION_TABLES = case KERNEL_ELF.machine
                     when :AArch64
                         Arch::ARMv8::TranslationTable.new
                     else
                         raise
                     end

kernel_map_binary
kernel_patch_tables(kernel_elf_path)
kernel_patch_base_addr(kernel_elf_path)

elapsed = Time.now - start

print 'Finished'.rjust(12).green.bold
puts " in #{elapsed.round(2)}s"
