#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

require 'fileutils'

def tutorial_folders
    crates = Dir['**/Cargo.toml']

    crates.delete_if do |x|
        !/[0-9][0-9]/.match?(x[0..1])
    end

    crates.sort!
end

puts tutorial_folders if $PROGRAM_NAME == __FILE__
