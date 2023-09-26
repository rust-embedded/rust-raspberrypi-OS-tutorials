#!/usr/bin/env ruby
# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>

require 'rubygems'
require 'bundler/setup'
require 'colorize'
require 'fileutils'
require_relative 'devtool/copyright'

# Actions for tutorial folders.
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

        # No output needed.
        Dir.chdir(@folder) { `make clean` }
    end

    def update
        puts "\n\n"
        puts 'Updating '.light_blue + @folder

        Dir.chdir(@folder) { system('cargo update') }
    end

    def clippy(bsp)
        puts "\n\n"
        puts "Clippy #{@folder} - BSP: #{bsp}".light_blue

        Dir.chdir(@folder) { exit(1) unless system("BSP=#{bsp} make clippy") }
    end

    def fmt_cargo_rust(args)
        Dir.chdir(@folder) { exit(1) unless system("cargo fmt #{args}") }
    end

    def make(bsp)
        puts "\n\n"
        puts "Make #{@folder} - BSP: #{bsp}".light_blue

        Dir.chdir(@folder) { exit(1) unless system("BSP=#{bsp} make") }
    end

    def test(bsp)
        return unless boot_test?

        puts "\n\n"
        puts "Test #{@folder} - BSP: #{bsp}".light_blue

        Dir.chdir(@folder) { exit(1) unless system("BSP=#{bsp} make test") }
    end

    def test_boot(bsp)
        return unless boot_test?

        puts "\n\n"
        puts "Test Boot #{@folder} - BSP: #{bsp}".light_blue

        Dir.chdir(@folder) { exit(1) unless system("BSP=#{bsp} make test_boot") }
    end

    def test_unit(bsp)
        return unless unit_integration_tests?

        puts "\n\n"
        puts "Test Unit #{@folder} - BSP: #{bsp}".light_blue

        Dir.chdir(@folder) { exit(1) unless system("BSP=#{bsp} make test_unit") }
    end

    def test_integration(bsp)
        return unless unit_integration_tests?

        puts "\n\n"
        puts "Test Integration #{@folder} - BSP: #{bsp}".light_blue

        Dir.chdir(@folder) { exit(1) unless system("BSP=#{bsp} make test_integration") }
    end

    private

    def boot_test?
        Dir.exist?("#{@folder}/tests") || Dir.exist?("#{@folder}/kernel/tests")
    end

    def unit_integration_tests?
        !Dir.glob("#{@folder}/kernel/tests/00_*.rs").empty?
    end
end

# Forks commands to all applicable receivers.
class DevTool
    def initialize
        @user_has_supplied_crates = false
        @bsp = bsp_from_env || SUPPORTED_BSPS.first

        cl = user_supplied_crate_list || Dir['*/Cargo.toml']
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

        @crates.each { |c| c.clippy(bsp) }
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

        @crates.each { |c| c.make(bsp) }
    end

    def make_xtra
        return if @user_has_supplied_crates

        puts "\n\n"
        puts 'Make Xtra stuff'.light_blue
        system('cd *_uart_chainloader && bash update.sh')
        system('cd X1_JTAG_boot && bash update.sh')
    end

    def test(bsp = nil)
        bsp ||= @bsp

        @crates.each { |c| c.test(bsp) }
    end

    def test_boot(bsp = nil)
        bsp ||= @bsp

        @crates.each { |c| c.test_boot(bsp) }
    end

    def test_unit(bsp = nil)
        bsp ||= @bsp

        @crates.each { |c| c.test_unit(bsp) }
    end

    def test_integration(bsp = nil)
        bsp ||= @bsp

        @crates.each { |c| c.test_integration(bsp) }
    end

    def copyright
        exit(1) unless copyright_check_files(copyright_source_files)
    end

    def misspell
        puts 'Misspell'.light_blue

        translations = ['README.CN.md', 'README.ES.md']
        files = tracked_files.reject { |f| translations.include?(File.basename(f)) }
        files = files.join(' ')

        exit(1) unless system(".vendor/misspell -error #{files}")
    end

    def rubocop
        puts 'Rubocop'.light_blue
        exit(1) unless system('bundle exec rubocop')
    end

    def ready_for_publish_no_rust
        clean
        fmt
        rubocop
        copyright
        diff
        misspell
        clean
    end

    def ready_for_publish
        ready_for_publish_no_rust

        make_xtra
        clippy('rpi4')
        clippy('rpi3')
        test_boot('rpi3')
        test_unit('rpi3')
        test_integration('rpi3')
        clean
    end

    private

    SUPPORTED_BSPS = %w[rpi3 rpi4].freeze

    def bsp_from_env
        bsp = ENV.fetch('BSP', nil)

        return bsp if SUPPORTED_BSPS.include?(bsp)

        nil
    end

    def fmt_cargo_rust(check: false)
        args = '-- --check' if check

        @crates.each do |c|
            print 'Rust cargo fmt '.light_blue
            print "#{args} ".light_blue unless args.nil?
            puts c.folder

            Process.fork { c.fmt_cargo_rust(args) }
        end
        Process.waitall
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

        # Skip for tutorial 11. Due to the change to virtual manifest, the diff is rather
        # unreadable.
        if original[0..1].to_i == 11
            puts 'Skipping '.light_yellow +
                 "#{original}: Too noisy due to change to virtual manifest"
            return
        end

        puts 'Diffing  '.light_blue + original.ljust(padding) + " -> #{update}"
        system("bash utils/diff_tut_folders.bash #{original} #{update}")
    end

    def copyright_source_files
        extensions = ['.S', '.rs', '.rb']

        # NOTE: The selection result is the return value of the function.
        tracked_files.select do |f|
            next unless File.exist?(f)
            next if f.include?('build.rs')
            next if f.include?('boot_test_string.rb')

            f.include?('Makefile') ||
                f.include?('Dockerfile') ||
                extensions.include?(File.extname(f))
        end
    end
end

## -------------------------------------------------------------------------------------------------
## Execution starts here
## -------------------------------------------------------------------------------------------------
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
    exit(1)
end
