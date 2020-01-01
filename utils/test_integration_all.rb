#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

require 'fileutils'
require_relative 'helpers/tutorial_folders.rb'

def run_tests
    Dir['tests/*.rs'].sort.each do |int_test|
        int_test = int_test.delete_prefix!('tests/').delete_suffix('.rs')
        exit(1) unless system("TEST=#{int_test} make test")
    end
end

def test_integration_all
    crates = tutorial_folders(false, true)

    crates.each do |x|
        tut = File.dirname(x)
        Dir.chdir(tut) do
            puts "\n\n" + tut.to_s + "\n\n"
            run_tests
        end
    end
end

test_integration_all if $PROGRAM_NAME == __FILE__
