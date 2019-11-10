#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

require_relative 'helpers/tutorial_folders.rb'

def fmt_all(check = false)
    crates = tutorial_folders

    args = if check != false
               '-- --check'
           else
               ''
           end

    crates.each do |x|
        x = File.dirname(x)
        Dir.chdir(x) do
            puts "Format #{x}"
            unless system("cargo fmt #{args}")
                puts "\n\nFmt check failed!"
                exit(1) # Exit with error code
            end
        end
    end
end

if $PROGRAM_NAME == __FILE__
    # Any command line argument means --check
    fmt_all(!ARGV[0].nil?)
end
