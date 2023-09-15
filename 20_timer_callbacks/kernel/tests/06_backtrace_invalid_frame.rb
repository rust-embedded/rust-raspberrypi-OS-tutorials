# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

require 'console_io_test'

# Test detection of invalid frame pointers.
class InvalidFramePointerTest < SubtestBase
    def name
        'Detect invalid frame pointer'
    end

    def run(qemu_out, _qemu_in)
        expect_or_raise(qemu_out,
                        /Encountered invalid frame pointer \(.*\) during backtrace/)
    end
end

## -------------------------------------------------------------------------------------------------
## Test registration
## -------------------------------------------------------------------------------------------------
def subtest_collection
    [InvalidFramePointerTest.new]
end
