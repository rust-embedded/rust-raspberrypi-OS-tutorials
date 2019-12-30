#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

require_relative 'helpers/copyrighted'
require_relative 'helpers/tutorial_folders.rb'

def patched?
    crates = tutorial_folders

    crates.each do |f|
        unless File.readlines(f).grep(/patch.crates-io/).empty?
            puts "#{fb} contains patch.crates-io!"
            exit(1)
        end
    end
end

def check_old_copyrights
    sources = Dir.glob('**/*.{S,rs,rb}') + Dir.glob('**/Makefile')

    sources.delete_if do |x|
        # if x is not in the index, treat this as an error
        !system("git ls-files --error-unmatch #{x}", %i[out err] => File::NULL)
    end

    sources.sort.each do |f|
        puts "Checking for copyright: #{f}"
        exit(1) unless copyrighted?(f)
    end
end

def sanity_checks
    patched?
    check_old_copyrights
end

sanity_checks if $PROGRAM_NAME == __FILE__
