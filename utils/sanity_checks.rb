#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT
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
  error = false

  sources = Dir.glob('**/*.{S,rs,rb}') + Dir.glob('**/Makefile')

  sources.delete_if do |x|
    !system("git ls-files --error-unmatch #{x}", %i[out err] => File::NULL)
  end

  sources.sort.reverse_each do |f|
    puts "Checking for copyright: #{f}"
    error = true unless copyrighted?(f, false)
  end

  exit(1) if error
end

def sanity_checks
  patched?
  check_old_copyrights
end

sanity_checks if $PROGRAM_NAME == __FILE__
