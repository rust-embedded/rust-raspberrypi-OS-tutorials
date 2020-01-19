#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

require 'rubygems'
require 'bundler/setup'
require 'colorize'
require 'fileutils'
require_relative 'devtool/copyright'

# Actions for tutorial folders
class TutorialCrate
    attr_reader :folder

    def initialize(folder)
        @folder = folder
    end

    def tutorial?
        /[0-9]/.match?(@folder[0])
    end

    def clean
        puts 'Cleaning '.light_blue + @folder

        Dir.chdir(@folder) { FileUtils.rm_rf('target') }
    end

    def clippy
        puts "Clippy #{@folder}".light_blue

        Dir.chdir(@folder) { exit(1) unless system('make clippy') }
    end

    def fmt(args)
        print 'Format '.light_blue
        print "#{args} ".light_blue unless args.nil?
        puts @folder

        Dir.chdir(@folder) { exit(1) unless system("cargo fmt #{args}") }
    end

    def make(bsp)
        puts "Make #{@folder} - BSP: #{bsp}".light_blue

        Dir.chdir(@folder) { exit(1) unless system("BSP=#{bsp} make") }
    end

    def test_unit
        return unless testable?

        puts "Unit Tests #{@folder}".light_blue

        Dir.chdir(@folder) { exit(1) unless system('TEST=unit make test') }
    end

    def test_integration
        return unless testable?

        puts "Integration Tests #{@folder}".light_blue

        Dir.chdir(@folder) do
            Dir['tests/*.rs'].sort.each do |t|
                t = t.delete_prefix('tests/').delete_suffix('.rs')
                exit(1) unless system("TEST=#{t} make test")
            end
        end
    end

    private

    def testable?
        Dir.exist?(@folder + '/tests')
    end
end

# Forks commands to all applicable receivers
class DevTool
    def initialize
        all = Dir['*/Cargo.toml'].sort

        @crates = all.map { |c| TutorialCrate.new(c.delete_suffix('/Cargo.toml')) }
    end

    def clean
        @crates.each(&:clean)
        FileUtils.rm_rf('xbuild_sysroot')
    end

    def clippy
        @crates.each do |c|
            c.clippy
            puts
            puts
        end
    end

    def diff
        tuts = tutorials.map(&:folder)
        padding = tuts.map(&:length).max

        tuts[0..-2].each_with_index do |original, i|
            update = tuts[i + 1]
            diff_pair(original, update, padding)
        end
    end

    def fmt(check = false)
        args = '-- --check' if check

        @crates.each { |c| c.fmt(args) }
    end

    def fmt_check
        fmt(true)
    end

    def make(bsp = 'rpi3')
        bsp = ARGV[1] if ARGV[1]

        @crates.each do |c|
            c.make(bsp)
            puts
            puts
        end
    end

    def make_xtra
        system('cd X1_JTAG_boot && bash update.sh')
    end

    def test_unit
        @crates.each(&:test_unit)
    end

    def test_integration
        @crates.each(&:test_integration)
    end

    def copyright
        exit(1) unless copyright_check_files(copyright_source_files)
    end

    def misspell
        exit(1) unless system("~/bin/misspell -error #{tracked_files.join(' ')}")
    end

    def rubocop
        exit(1) unless system('rubocop')
    end

    def ready_for_publish
        clean
        fmt
        misspell
        rubocop
        clippy
        copyright

        clean
        make('rpi4')
        make('rpi3')
        make_xtra
        test_unit
        test_integration
        diff
        clean
    end

    def ready_for_publish_no_rust
        clean
        misspell
        rubocop
        copyright
        diff
        clean
    end

    private

    def tutorials
        @crates.select(&:tutorial?)
    end

    def tracked_files
        `git ls-files`.split("\n")
    end

    def diff_pair(original, update, padding)
        puts 'Diffing '.light_blue + original.ljust(padding) + " -> #{update}"
        system("bash utils/diff_tut_folders.bash #{original} #{update}")
    end

    def copyright_source_files
        extensions = ['.S', '.rs', '.rb']

        files = tracked_files.select do |f|
            next unless File.exist?(f)

            f.include?('Makefile') ||
                f.include?('Dockerfile') ||
                extensions.include?(File.extname(f))
        end

        files
    end
end

##--------------------------------------------------------------------------------------------------
## Execution starts here
##--------------------------------------------------------------------------------------------------
tool = DevTool.new
cmd = ARGV[0]
commands = tool.public_methods(false).sort

if !commands.include?(cmd&.to_sym)
    puts "Usage: ./#{__FILE__} COMMAND"
    puts
    puts 'Commands:'
    commands.each { |m| puts "  #{m}" }
else
    tool.public_send(cmd)
end
