#!/usr/bin/env ruby

def clippy_all
  crates = Dir["*/Cargo.toml"].sort!
  crates.delete_if { |x| x.include?('bareminimum') }

  crates.each do |x|
    x = File.dirname(x)

    Dir.chdir(x) do
      system('make clippy')
    end
  end
end

if __FILE__ == $0
  clippy_all()
end
