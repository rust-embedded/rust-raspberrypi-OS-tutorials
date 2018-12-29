#!/usr/bin/env ruby

require 'fileutils'

def clean_all
  crates = Dir["**/Cargo.toml"].sort!

  crates.each do |x|
    next if x.include?('raspi3_boot')

    x = File.dirname(x)
    Dir.chdir(x) do
      system('make clean') or exit(1)
    end
  end

  FileUtils.rm_rf('xbuild_sysroot')
end

if __FILE__ == $0
  clean_all()
end
