// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! PL011 UART driver.

use crate::{arch, arch::sync::NullLock, interface};
use core::{fmt, ops};
use register::{mmio::*, register_bitfields};

// PL011 UART registers.
//
// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,

    /// Flag Register
    FR [
        /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// Line Control Register, UARTLCR_ LCRH.
        ///
        /// If the FIFO is disabled, this bit is set when the transmit holding register is empty. If
        /// the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty. This bit does
        /// not indicate if there is data in the transmit shift register.
        TXFE OFFSET(7) NUMBITS(1) [],

        /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the
        /// UARTLCR_ LCRH Register.
        ///
        /// If the FIFO is disabled, this bit is set when the transmit holding register is full. If
        /// the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
        TXFF OFFSET(5) NUMBITS(1) [],

        /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// UARTLCR_H Register.
        ///
        /// If the FIFO is disabled, this bit is set when the receive holding register is empty. If
        /// the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
        RXFE OFFSET(4) NUMBITS(1) []
    ],

    /// Integer Baud rate divisor
    IBRD [
        /// Integer Baud rate divisor
        IBRD OFFSET(0) NUMBITS(16) []
    ],

    /// Fractional Baud rate divisor
    FBRD [
        /// Fractional Baud rate divisor
        FBRD OFFSET(0) NUMBITS(6) []
    ],

    /// Line Control register
    LCRH [
        /// Word length. These bits indicate the number of data bits transmitted or received in a
        /// frame.
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ],

        /// Enable FIFOs:
        ///
        /// 0 = FIFOs are disabled (character mode) that is, the FIFOs become 1-byte-deep holding
        /// registers
        ///
        /// 1 = transmit and receive FIFO buffers are enabled (FIFO mode).
        FEN  OFFSET(4) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1
        ]
    ],

    /// Control Register
    CR [
        /// Receive enable. If this bit is set to 1, the receive section of the UART is enabled.
        /// Data reception occurs for UART signals. When the UART is disabled in the middle of
        /// reception, it completes the current character before stopping.
        RXE    OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// Transmit enable. If this bit is set to 1, the transmit section of the UART is enabled.
        /// Data transmission occurs for UART signals. When the UART is disabled in the middle of
        /// transmission, it completes the current character before stopping.
        TXE    OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// UART enable
        UARTEN OFFSET(0) NUMBITS(1) [
            /// If the UART is disabled in the middle of transmission or reception, it completes the
            /// current character before stopping.
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Interrupt Clear Register
    ICR [
        /// Meta field for all pending interrupts
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    DR: ReadWrite<u32>,                   // 0x00
    __reserved_0: [u32; 5],               // 0x04
    FR: ReadOnly<u32, FR::Register>,      // 0x18
    __reserved_1: [u32; 2],               // 0x1c
    IBRD: WriteOnly<u32, IBRD::Register>, // 0x24
    FBRD: WriteOnly<u32, FBRD::Register>, // 0x28
    LCRH: WriteOnly<u32, LCRH::Register>, // 0x2C
    CR: WriteOnly<u32, CR::Register>,     // 0x30
    __reserved_2: [u32; 4],               // 0x34
    ICR: WriteOnly<u32, ICR::Register>,   // 0x44
}

/// The driver's mutex protected part.
struct PL011UartInner {
    base_addr: usize,
    chars_written: usize,
}

/// Deref to RegisterBlock.
///
/// Allows writing
/// ```
/// self.DR.read()
/// ```
/// instead of something along the lines of
/// ```
/// unsafe { (*PL011UartInner::ptr()).DR.read() }
/// ```
impl ops::Deref for PL011UartInner {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl PL011UartInner {
    const fn new(base_addr: usize) -> PL011UartInner {
        PL011UartInner {
            base_addr,
            chars_written: 0,
        }
    }

    /// Return a pointer to the register block.
    fn ptr(&self) -> *const RegisterBlock {
        self.base_addr as *const _
    }

    /// Send a character.
    fn write_char(&mut self, c: char) {
        // Wait until we can send.
        loop {
            if !self.FR.is_set(FR::TXFF) {
                break;
            }

            arch::nop();
        }

        // Write the character to the buffer.
        self.DR.set(c as u32);
    }
}

/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
/// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
/// we get `write_fmt()` automatically.
///
/// The function takes an `&mut self`, so it must be implemented for the inner struct.
///
/// See [`src/print.rs`].
///
/// [`src/print.rs`]: ../../print/index.html
impl fmt::Write for PL011UartInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            // Convert newline to carrige return + newline.
            if c == '\n' {
                self.write_char('\r')
            }

            self.write_char(c);
        }

        self.chars_written += s.len();

        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// BSP-public
//--------------------------------------------------------------------------------------------------

/// The driver's main struct.
pub struct PL011Uart {
    inner: NullLock<PL011UartInner>,
}

impl PL011Uart {
    /// # Safety
    ///
    /// The user must ensure to provide the correct `base_addr`.
    pub const unsafe fn new(base_addr: usize) -> PL011Uart {
        PL011Uart {
            inner: NullLock::new(PL011UartInner::new(base_addr)),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// OS interface implementations
//--------------------------------------------------------------------------------------------------
use interface::sync::Mutex;

impl interface::driver::DeviceDriver for PL011Uart {
    fn compatible(&self) -> &str {
        "PL011Uart"
    }

    /// Set up baud rate and characteristics
    ///
    /// Results in 8N1 and 115200 baud (if the clk has been previously set to 4 MHz by the
    /// firmware).
    fn init(&self) -> interface::driver::Result {
        let mut r = &self.inner;
        r.lock(|inner| {
            // Turn it off temporarily.
            inner.CR.set(0);

            inner.ICR.write(ICR::ALL::CLEAR);
            inner.IBRD.write(IBRD::IBRD.val(26)); // Results in 115200 baud for UART Clk of 48 MHz.
            inner.FBRD.write(FBRD::FBRD.val(3));
            inner
                .LCRH
                .write(LCRH::WLEN::EightBit + LCRH::FEN::FifosEnabled); // 8N1 + Fifo on
            inner
                .CR
                .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
        });

        Ok(())
    }
}

impl interface::console::Write for PL011Uart {
    /// Passthrough of `args` to the `core::fmt::Write` implementation, but guarded by a Mutex to
    /// serialize access.
    fn write_char(&self, c: char) {
        let mut r = &self.inner;
        r.lock(|inner| inner.write_char(c));
    }

    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        // Fully qualified syntax for the call to `core::fmt::Write::write:fmt()` to increase
        // readability.
        let mut r = &self.inner;
        r.lock(|inner| fmt::Write::write_fmt(inner, args))
    }

    fn flush(&self) {
        let mut r = &self.inner;
        // Spin until the TX FIFO empty flag is set.
        r.lock(|inner| loop {
            if inner.FR.is_set(FR::TXFE) {
                break;
            }

            arch::nop();
        });
    }
}

impl interface::console::Read for PL011Uart {
    fn read_char(&self) -> char {
        let mut r = &self.inner;
        r.lock(|inner| {
            // Wait until buffer is filled.
            loop {
                if !inner.FR.is_set(FR::RXFE) {
                    break;
                }

                arch::nop();
            }

            // Read one character.
            let mut ret = inner.DR.get() as u8 as char;

            // Convert carrige return to newline.
            if ret == '\r' {
                ret = '\n'
            }

            ret
        })
    }

    fn clear(&self) {
        let mut r = &self.inner;
        r.lock(|inner| loop {
            // Read from the RX FIFO until the empty bit is '1'.
            if !inner.FR.is_set(FR::RXFE) {
                inner.DR.get();
            } else {
                break;
            }
        })
    }
}

impl interface::console::Statistics for PL011Uart {
    fn chars_written(&self) -> usize {
        let mut r = &self.inner;
        r.lock(|inner| inner.chars_written)
    }
}
