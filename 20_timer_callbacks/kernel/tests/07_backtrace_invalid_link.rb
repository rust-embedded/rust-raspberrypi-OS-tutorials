# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

require 'console_io_test'

# Test detection of invalid link.
class InvalidLinkTest < SubtestBase
    def name
        'Detect invalid link'
    end

    def run(qemu_out, _qemu_in)
        expect_or_raise(qemu_out, /Link address \(.*\) is not contained in kernel .text section/)
    end
end

## -------------------------------------------------------------------------------------------------
## Test registration
## -------------------------------------------------------------------------------------------------
def subtest_collection
    [InvalidLinkTest.new]
end
