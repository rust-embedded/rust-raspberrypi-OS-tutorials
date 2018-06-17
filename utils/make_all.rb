#!/usr/bin/env ruby

crates = Dir["**/Cargo.toml"].sort!

crates.each do |x|
  next if x.include?('raspi3_glue')

  x = File.dirname(x)
  puts "\n\n" + x.to_s + "\n\n"
  Dir.chdir(x) do
    unless system('make')
      puts "\n\nBuild failed!"
      exit(1) # Exit with error code
    end
  end
end
