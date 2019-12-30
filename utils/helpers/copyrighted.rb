# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

def checkin_years(file)
    checkin_years = `git --no-pager log \
                     --reverse --pretty=format:'%ad' --date=format:'%Y' #{file}`

    checkin_years.split(/\n/).map!(&:to_i)
end

def parse_checkin_years(file)
    checkin_years = checkin_years(file)
    checkin_years << Time.now.year

    checkin_years.minmax
end

def min_max_seen?(copyright_lines, checkin_min, checkin_max)
    min_seen = false
    max_seen = false
    copyright_lines.each do |x|
        x.scan(/\d\d\d\d/).map!(&:to_i).each do |y|
            min_seen = true if y == checkin_min
            max_seen = true if y == checkin_max
        end
    end

    [min_seen, max_seen]
end

def print_on_err(file, copyright_lines, checkin_min, checkin_max)
    puts file + ': '
    puts "\tHeader:   " + copyright_lines.inspect
    puts "\tMin year: " + checkin_min.to_s
    puts "\tMax year: " + checkin_max.to_s
end

def copyrighted?(file)
    checkin_min, checkin_max = parse_checkin_years(file)
    copyright_lines = File.readlines(file).grep(/.*Copyright.*/)
    min_seen, max_seen = min_max_seen?(copyright_lines, checkin_min, checkin_max)

    unless min_seen && max_seen
        print_on_err(file, copyright_lines, checkin_min, checkin_max)
        return false
    end

    true
end
