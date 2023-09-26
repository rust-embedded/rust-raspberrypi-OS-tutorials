# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

require 'console_io_test'

# Verify that panic produces a backtrace.
class PanicBacktraceTest < SubtestBase
    def name
        'Panic produces backtrace'
    end

    def run(qemu_out, _qemu_in)
        expect_or_raise(qemu_out, 'Kernel panic!')
        expect_or_raise(qemu_out, 'Backtrace:')
    end
end

# Verify backtrace correctness.
class BacktraceCorrectnessTest < SubtestBase
    def name
        'Backtrace is correct'
    end

    def run(qemu_out, _qemu_in)
        expect_or_raise(qemu_out, '| core::panicking::panic')
        expect_or_raise(qemu_out, '| _05_backtrace_sanity::nested')
        expect_or_raise(qemu_out, '| kernel_init')
    end
end

## -------------------------------------------------------------------------------------------------
## Test registration
## -------------------------------------------------------------------------------------------------
def subtest_collection
    [PanicBacktraceTest.new, BacktraceCorrectnessTest.new]
end
