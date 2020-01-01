#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

require_relative 'helpers/tutorial_folders.rb'

def clippy_all
    crates = tutorial_folders

    crates.each do |x|
        x = File.dirname(x)

        Dir.chdir(x) do
            puts "Clippy: #{x}"
            unless system('make clippy')
                puts "\n\nClippy failed!"
                exit(1) # Exit with error code
            end
        end
    end
end

clippy_all if $PROGRAM_NAME == __FILE__
