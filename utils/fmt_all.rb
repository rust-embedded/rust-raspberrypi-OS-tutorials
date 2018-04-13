#!/usr/bin/env ruby

crates = Dir["**/Cargo.toml"]

crates.each do |x|
  x = File.dirname(x)

  Dir.chdir(x) do
    `cargo fmt`
  end
end
