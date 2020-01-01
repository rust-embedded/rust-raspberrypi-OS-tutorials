#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

require 'fileutils'
require_relative 'helpers/tutorial_folders.rb'

def clean_all
    crates = tutorial_folders

    crates.each do |x|
        x = File.dirname(x)
        Dir.chdir(x) do
            puts "Cleaning #{x}"
            FileUtils.rm_rf('target')
        end
    end

    FileUtils.rm_rf('xbuild_sysroot')
end

clean_all if $PROGRAM_NAME == __FILE__
