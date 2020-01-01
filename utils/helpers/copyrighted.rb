# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

def copyrighted?(file)
    copyright_years = []

    File.readlines(file).grep(/.*Copyright.*/).each do |x|
        copyright_years << x.scan(/\d\d\d\d/).map!(&:to_i)
    end

    copyright_years = copyright_years.flatten.uniq

    unless copyright_years.include?(Time.now.year)
        puts "\tHeader:   " + copyright_years.inspect
        return false
    end

    true
end
