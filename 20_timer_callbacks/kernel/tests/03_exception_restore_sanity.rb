# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

require 'console_io_test'

# Verify that exception restore works.
class ExceptionRestoreTest < SubtestBase
    def name
        'Exception restore'
    end

    def run(qemu_out, _qemu_in)
        expect_or_raise(qemu_out, 'Back from system call!')
    end
end

## -------------------------------------------------------------------------------------------------
## Test registration
## -------------------------------------------------------------------------------------------------
def subtest_collection
    [ExceptionRestoreTest.new]
end
