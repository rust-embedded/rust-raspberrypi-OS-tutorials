#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

require 'fileutils'
require_relative 'helpers/tutorial_folders.rb'

def diff_all
    crates = tutorial_folders

    (0..(crates.length - 2)).each do |i|
        old = File.dirname(crates[i])
        new = File.dirname(crates[i + 1])
        puts "Diffing #{old} -> #{new}"
        system("bash utils/helpers/diff_tut_folders.bash #{old} #{new}")
    end
end

diff_all if $PROGRAM_NAME == __FILE__
