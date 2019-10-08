# frozen_string_literal: true

# SPDX-License-Identifier: MIT
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

def checkin_years(file)
  checkin_years = `git --no-pager log \
                   --reverse --pretty=format:'%ad' --date=format:'%Y' #{file}`

  checkin_years.split(/\n/).map!(&:to_i)
end

def copyrighted?(file, is_being_checked_in)
  checkin_years = checkin_years(file)

  checkin_years << Time.now.year if is_being_checked_in

  checkin_min = checkin_years.min
  checkin_max = checkin_years.max

  copyright_lines = File.readlines(file).grep(/.*Copyright.*/)

  min_seen = false
  max_seen = false
  copyright_lines.each do |x|
    x.scan(/\d\d\d\d/).each do |y|
      y = y.to_i

      min_seen = true if y == checkin_min
      max_seen = true if y == checkin_max
    end
  end

  unless min_seen && max_seen
    puts file + ': '
    puts "\tHeader:   " + copyright_lines.inspect
    puts "\tMin year: " + checkin_min.to_s
    puts "\tMax year: " + checkin_max.to_s

    return false
  end

  true
end
