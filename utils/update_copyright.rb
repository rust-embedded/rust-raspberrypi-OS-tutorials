#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

require 'date'

files = `git ls-files`.split("\n")
files = files.delete_if { |f| File.symlink?(f) }
files = files.join(' ')

year = Date.today.year

`sed -i -- 's,\\(Copyright .* 20..\\)-20..,\\1-#{year},g' #{files}`
`sed -i -- 's,\\(Copyright .* #{year - 1}\\) ,\\1-#{year} ,g' #{files}`
