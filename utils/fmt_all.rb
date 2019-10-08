#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

require_relative 'helpers/tutorial_folders.rb'

def fmt_all
  crates = tutorial_folders

  crates.each do |x|
    x = File.dirname(x)

    Dir.chdir(x) do
      puts "Formatting #{x}"
      system('cargo fmt')
    end
  end
end

fmt_all if $PROGRAM_NAME == __FILE__
