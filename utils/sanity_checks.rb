#!/usr/bin/env ruby

def sanity_checks
  crates = Dir["**/Cargo.toml"].sort!

  crates.each do |x|
    if File.readlines(x).grep(/patch.crates-io/).size > 0
      puts "#{x} contains patch.crates-io!"
      exit(1)
    end
  end
end

if __FILE__ == $0
  sanity_checks()
end
