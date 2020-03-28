# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2019-2020 Andre Richter <andre.o.richter@gmail.com>

require 'expect'

TIMEOUT_SECS = 3

# Verify sending and receiving works as expected.
class TxRxHandshake
    def name
        'Transmit and Receive handshake'
    end

    def run(qemu_out, qemu_in)
        qemu_in.write_nonblock('ABC')
        raise('TX/RX test failed') if qemu_out.expect('OK1234', TIMEOUT_SECS).nil?
    end
end

# Check for correct TX statistics implementation. Depends on test 1 being run first.
class TxStatistics
    def name
        'Transmit statistics'
    end

    def run(qemu_out, _qemu_in)
        raise('chars_written reported wrong') if qemu_out.expect('6', TIMEOUT_SECS).nil?
    end
end

# Check for correct RX statistics implementation. Depends on test 1 being run first.
class RxStatistics
    def name
        'Receive statistics'
    end

    def run(qemu_out, _qemu_in)
        raise('chars_read reported wrong') if qemu_out.expect('3', TIMEOUT_SECS).nil?
    end
end

##--------------------------------------------------------------------------------------------------
## Test registration
##--------------------------------------------------------------------------------------------------
def subtest_collection
    [TxRxHandshake.new, TxStatistics.new, RxStatistics.new]
end
