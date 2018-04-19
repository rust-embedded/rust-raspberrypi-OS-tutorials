#!/usr/bin/env ruby

crates = Dir["**/Cargo.toml"]

crates.each do |x|
  x = File.dirname(x)

  puts "\n\n" + x.to_s + "\n\n"
  Dir.chdir(x) do
    `make`
  end
end
