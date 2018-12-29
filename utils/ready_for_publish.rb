#!/usr/bin/env ruby

require_relative 'clean_all'
require_relative 'clippy_all'
require_relative 'fmt_all'
require_relative 'make_all'
require_relative 'make_panic_test'
require_relative 'sanity_checks'

clean_all()
clippy_all()

clean_all()
fmt_all()
make_all()
make_panic_test()
sanity_checks()
