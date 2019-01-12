#
# MIT License
#
# Copyright (c) 2019 Andre Richter <andre.o.richter@gmail.com>
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#

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
