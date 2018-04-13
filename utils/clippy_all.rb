#!/usr/bin/env ruby

crates = Dir["*/Cargo.toml"]
crates.delete_if { |x| x.include?('bareminimum') }

crates.each do |x|
  x = File.dirname(x)

  Dir.chdir(x) do
    `make clippy`
  end
end
