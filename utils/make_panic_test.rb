#!/usr/bin/env ruby

def make_panic_test
  crates = Dir["**/Cargo.toml"].sort!

  crates.each do |x|
    next if x.include?('raspi3_boot')

    x = File.dirname(x)
    puts "\n" + x.to_s + ':'
    Dir.chdir(x) do
      system('make nm | grep panic_fmt')
    end
  end
end

if __FILE__ == $0
  make_panic_test()
end
