#!/usr/bin/env ruby
#
# MIT License
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
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

require_relative 'copyrighted'

def patched?
  crates = Dir['**/Cargo.toml'].sort!

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
    error = true unless copyrighted?(f, false)
  end

  exit(1) if error
end

def sanity_checks
  patched?
  check_old_copyrights
end

sanity_checks if $PROGRAM_NAME == __FILE__
