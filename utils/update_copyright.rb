#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

require 'date'

files = `git ls-files`.split("\n")
files = files.delete_if { |f| File.symlink?(f) }
files = files.join(' ')

year = Date.today.year

# Update "Copyright (c) 20..-20.."
`sed -i -- 's,\\(Copyright .c. 20..\\)-20..,\\1-#{year},g' #{files}`

# Update "Copyright (c) 20.. Name" -> "Copyright (c) 20..-20.. Name"
`sed -i -- 's,\\(Copyright .c. 20..\\) ,\\1-#{year} ,g' #{files}`
