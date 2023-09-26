# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2019-2023 Andre Richter <andre.o.richter@gmail.com>

require 'console_io_test'

# Verify sending and receiving works as expected.
class TxRxHandshakeTest < SubtestBase
    def name
        'Transmit and Receive handshake'
    end

    def run(qemu_out, qemu_in)
        qemu_in.write_nonblock('ABC')
        expect_or_raise(qemu_out, 'OK1234')
    end
end

# Check for correct TX statistics implementation. Depends on test 1 being run first.
class TxStatisticsTest < SubtestBase
    def name
        'Transmit statistics'
    end

    def run(qemu_out, _qemu_in)
        expect_or_raise(qemu_out, '6')
    end
end

# Check for correct RX statistics implementation. Depends on test 1 being run first.
class RxStatisticsTest < SubtestBase
    def name
        'Receive statistics'
    end

    def run(qemu_out, _qemu_in)
        expect_or_raise(qemu_out, '3')
    end
end

## -------------------------------------------------------------------------------------------------
## Test registration
## -------------------------------------------------------------------------------------------------
def subtest_collection
    [TxRxHandshakeTest.new, TxStatisticsTest.new, RxStatisticsTest.new]
end
