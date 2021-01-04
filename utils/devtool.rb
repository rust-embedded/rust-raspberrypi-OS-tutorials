#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

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

        Dir.chdir(@folder) { system('make clean') }
    end

    def update
        puts 'Updating '.light_blue + @folder

        Dir.chdir(@folder) { system('cargo update') }
    end

    def clippy(bsp)
        puts "Clippy #{@folder} - BSP: #{bsp}".light_blue

        Dir.chdir(@folder) { exit(1) unless system("BSP=#{bsp} make clippy") }
    end

    def fmt_cargo_rust(args)
        print 'Rust cargo fmt '.light_blue
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
        Dir.exist?("#{@folder}/tests")
    end
end

# Forks commands to all applicable receivers
class DevTool
    def initialize
        @user_has_supplied_crates = false
        @bsp = bsp_from_env || SUPPORTED_BSPS.first

        cl = user_supplied_crate_list || Dir['*/Cargo.toml'].sort
        @crates = cl.map { |c| TutorialCrate.new(c.delete_suffix('/Cargo.toml')) }
    end

    def clean
        @crates.each(&:clean)
    end

    def update
        @crates.each(&:update)
    end

    def clippy(bsp = nil)
        bsp ||= @bsp

        @crates.each do |c|
            c.clippy(bsp)
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

    def fmt
        fmt_cargo_rust(check: false)
        puts
        fmt_prettier(check: false)
    end

    def fmt_check
        fmt_cargo_rust(check: true)
        puts
        fmt_prettier(check: true)
    end

    def make(bsp = nil)
        bsp ||= @bsp

        @crates.each do |c|
            c.make(bsp)
            puts
            puts
        end
    end

    def make_xtra
        return if @user_has_supplied_crates

        puts 'Make Xtra stuff'.light_blue
        system('cd 07_uart_chainloader && bash update.sh')
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
        puts 'Misspell'.light_blue
        exit(1) unless system(".vendor/misspell -error #{tracked_files.join(' ')}")
    end

    def rubocop
        puts 'Rubocop'.light_blue
        exit(1) unless system('bundle exec rubocop')
    end

    def ready_for_publish
        clean
        fmt
        misspell
        rubocop
        clippy('rpi4')
        clippy('rpi3')
        copyright
        diff

        clean
        make('rpi4')
        make('rpi3')
        make_xtra
        test_unit
        test_integration
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

    SUPPORTED_BSPS = %w[rpi3 rpi4].freeze

    def bsp_from_env
        bsp = ENV['BSP']

        return bsp if SUPPORTED_BSPS.include?(bsp)

        nil
    end

    def fmt_cargo_rust(check: false)
        args = '-- --check' if check

        @crates.each { |c| c.fmt_cargo_rust(args) }
    end

    def fmt_prettier(check: false)
        args = if check
                   '--check'
               else
                   '--write'
               end

        args += if @user_has_supplied_crates
                    " #{@crates.map(&:folder).join(' ')}"
                else
                    ' .'
                end

        puts 'Prettier:'.light_blue
        exit(1) unless system("./node_modules/.bin/prettier #{args}")
    end

    def user_supplied_crate_list
        folders = ARGV.drop(1)

        return nil if folders.empty?

        crates = folders.map { |d| "#{d}/Cargo.toml" }.sort
        crates.each do |c|
            unless File.exist?(c)
                puts "Crate not found: #{c}"
                exit(1)
            end
        end

        @user_has_supplied_crates = true
        crates
    end

    def tutorials
        @crates.select(&:tutorial?)
    end

    def tracked_files
        crate_list = @crates.map(&:folder).join(' ') if @user_has_supplied_crates

        `git ls-files #{crate_list}`.split("\n") # crates_list == nil means all files
    end

    def diff_pair(original, update, padding)
        # Only diff adjacent tutorials. This checks the numbers of the tutorial folders.
        return unless original[0..1].to_i + 1 == update[0..1].to_i

        puts 'Diffing '.light_blue + original.ljust(padding) + " -> #{update}"
        system("bash utils/diff_tut_folders.bash #{original} #{update}")
    end

    def copyright_source_files
        extensions = ['.S', '.rs', '.rb']

        # NOTE: The selection result is the return value of the function.
        tracked_files.select do |f|
            next unless File.exist?(f)
            next if f.include?('build.rs')

            f.include?('Makefile') ||
                f.include?('Dockerfile') ||
                extensions.include?(File.extname(f))
        end
    end
end

##--------------------------------------------------------------------------------------------------
## Execution starts here
##--------------------------------------------------------------------------------------------------
tool = DevTool.new
cmd = ARGV[0]
commands = tool.public_methods(false).sort

if commands.include?(cmd&.to_sym)
    tool.public_send(cmd)
else
    puts "Usage: ./#{__FILE__.split('/').last} COMMAND [optional list of folders]"
    puts
    puts 'Commands:'
    commands.each { |m| puts "  #{m}" }
end
