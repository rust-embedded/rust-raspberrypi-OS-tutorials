# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

require 'rubygems'
require 'bundler/setup'
require 'colorize'

def copyright_check_files(source_files)
    source_files.sort.each do |f|
        puts 'Checking for copyright: '.light_blue + f.to_s

        years = copyright_years(f)
        unless years.include?(Time.now.year)
            puts "\tOnly found years: #{years}".red
            return false
        end
    end

    true
end

def copyright_years(file)
    years = []
    File.readlines(file).grep(/.*Copyright.*/).each do |x|
        years << x.scan(/\d\d\d\d/).map!(&:to_i)
    end

    years.flatten
end
