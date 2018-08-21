#!/usr/bin/env ruby

require 'fileutils'

crates = Dir["**/Cargo.toml"].sort!

crates.each do |x|
  next if x.include?('raspi3_boot')

  x = File.dirname(x)
  puts "\n\n" + x.to_s + "\n\n"
  Dir.chdir(x) do
    unless system('make clean')
      puts "\n\nBuild failed!"
      exit(1) # Exit with error code
    end
  end
end

FileUtils.rm_rf('xbuild_sysroot')
