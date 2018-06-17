#!/usr/bin/env ruby

crates = Dir["**/Cargo.toml"]

crates.each do |x|
  next if x.include?('raspi3_glue')

  x = File.dirname(x)
  puts "\n\n" + x.to_s + "\n\n"
  Dir.chdir(x) do
    `make`
  end
end
