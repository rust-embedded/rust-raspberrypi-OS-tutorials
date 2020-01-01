#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

require 'fileutils'
require_relative 'helpers/tutorial_folders.rb'

def test_unit_all
    crates = tutorial_folders(false, true)

    crates.each do |x|
        x = File.dirname(x)
        Dir.chdir(x) do
            puts "\n\n" + x.to_s + "\n\n"
            exit(1) unless system('TEST=unit make test')
        end
    end
end

test_unit_all if $PROGRAM_NAME == __FILE__
