#!/usr/bin/env ruby

def fmt_all
  crates = Dir["**/Cargo.toml"].sort!

  crates.each do |x|
    x = File.dirname(x)

    Dir.chdir(x) do
      system('cargo fmt')
    end
  end
end

if __FILE__ == $0
  fmt_all()
end
