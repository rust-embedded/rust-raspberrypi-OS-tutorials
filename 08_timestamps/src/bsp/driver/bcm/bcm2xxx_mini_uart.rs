// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! Mini UART driver.

use crate::{arch, arch::sync::NullLock, interface};
use core::{fmt, ops};
use register::{mmio::*, register_bitfields};

// Mini UART registers.
//
// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,

    /// Auxiliary enables
    AUX_ENABLES [
        /// If set the mini UART is enabled. The UART will immediately start receiving data,
        /// especially if the UART1_RX line is low.
        ///
        /// If clear the mini UART is disabled. That also disables any mini UART register access
        MINI_UART_ENABLE OFFSET(0) NUMBITS(1) []
    ],

    /// Mini Uart Interrupt Identify
    AUX_MU_IIR [
        /// Writing with bit 1 set will clear the receive FIFO
        /// Writing with bit 2 set will clear the transmit FIFO
        FIFO_CLEAR OFFSET(1) NUMBITS(2) [
            Rx = 0b01,
            Tx = 0b10,
            All = 0b11
        ]
    ],

    /// Mini Uart Line Control
    AUX_MU_LCR [
        /// Mode the UART works in
        DATA_SIZE OFFSET(0) NUMBITS(2) [
            SevenBit = 0b00,
            EightBit = 0b11
        ]
    ],

    /// Mini Uart Line Status
    AUX_MU_LSR [
        /// This bit is set if the transmit FIFO is empty and the transmitter is idle. (Finished
        /// shifting out the last bit).
        TX_IDLE    OFFSET(6) NUMBITS(1) [],

        /// This bit is set if the transmit FIFO can accept at least one byte.
        TX_EMPTY   OFFSET(5) NUMBITS(1) [],

        /// This bit is set if the receive FIFO holds at least 1 symbol.
        DATA_READY OFFSET(0) NUMBITS(1) []
    ],

    /// Mini Uart Extra Control
    AUX_MU_CNTL [
        /// If this bit is set the mini UART transmitter is enabled.
        /// If this bit is clear the mini UART transmitter is disabled.
        TX_EN OFFSET(1) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// If this bit is set the mini UART receiver is enabled.
        /// If this bit is clear the mini UART receiver is disabled.
        RX_EN OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Mini Uart Baudrate
    AUX_MU_BAUD [
        /// Mini UART baudrate counter
        RATE OFFSET(0) NUMBITS(16) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    __reserved_0: u32,                                  // 0x00
    AUX_ENABLES: ReadWrite<u32, AUX_ENABLES::Register>, // 0x04
    __reserved_1: [u32; 14],                            // 0x08
    AUX_MU_IO: ReadWrite<u32>,                          // 0x40 - Mini Uart I/O Data
    AUX_MU_IER: WriteOnly<u32>,                         // 0x44 - Mini Uart Interrupt Enable
    AUX_MU_IIR: WriteOnly<u32, AUX_MU_IIR::Register>,   // 0x48
    AUX_MU_LCR: WriteOnly<u32, AUX_MU_LCR::Register>,   // 0x4C
    AUX_MU_MCR: WriteOnly<u32>,                         // 0x50
    AUX_MU_LSR: ReadOnly<u32, AUX_MU_LSR::Register>,    // 0x54
    __reserved_2: [u32; 2],                             // 0x58
    AUX_MU_CNTL: WriteOnly<u32, AUX_MU_CNTL::Register>, // 0x60
    __reserved_3: u32,                                  // 0x64
    AUX_MU_BAUD: WriteOnly<u32, AUX_MU_BAUD::Register>, // 0x68
}

/// The driver's mutex protected part.
struct MiniUartInner {
    base_addr: usize,
    chars_written: usize,
}

/// Deref to RegisterBlock.
///
/// Allows writing
/// ```
/// self.MU_IER.read()
/// ```
/// instead of something along the lines of
/// ```
/// unsafe { (*MiniUart::ptr()).MU_IER.read() }
/// ```
impl ops::Deref for MiniUartInner {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl MiniUartInner {
    const fn new(base_addr: usize) -> MiniUartInner {
        MiniUartInner {
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
            if self.AUX_MU_LSR.is_set(AUX_MU_LSR::TX_EMPTY) {
                break;
            }

            arch::nop();
        }

        // Write the character to the buffer.
        self.AUX_MU_IO.set(c as u32);
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
impl fmt::Write for MiniUartInner {
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
pub struct MiniUart {
    inner: NullLock<MiniUartInner>,
}

impl MiniUart {
    /// # Safety
    ///
    /// The user must ensure to provide the correct `base_addr`.
    pub const unsafe fn new(base_addr: usize) -> MiniUart {
        MiniUart {
            inner: NullLock::new(MiniUartInner::new(base_addr)),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// OS interface implementations
//--------------------------------------------------------------------------------------------------
use interface::sync::Mutex;

impl interface::driver::DeviceDriver for MiniUart {
    fn compatible(&self) -> &str {
        "MiniUart"
    }

    /// Set up baud rate and characteristics (115200 8N1).
    fn init(&self) -> interface::driver::Result {
        let mut r = &self.inner;
        r.lock(|inner| {
            // Enable register access to the MiniUart
            inner.AUX_ENABLES.modify(AUX_ENABLES::MINI_UART_ENABLE::SET);
            inner.AUX_MU_IER.set(0); // disable RX and TX interrupts
            inner.AUX_MU_CNTL.set(0); // disable send and receive
            inner.AUX_MU_LCR.write(AUX_MU_LCR::DATA_SIZE::EightBit);
            inner.AUX_MU_BAUD.write(AUX_MU_BAUD::RATE.val(270)); // 115200 baud
            inner.AUX_MU_MCR.set(0); // set "ready to send" high

            // Enable receive and send.
            inner
                .AUX_MU_CNTL
                .write(AUX_MU_CNTL::RX_EN::Enabled + AUX_MU_CNTL::TX_EN::Enabled);

            // Clear FIFOs before using the device.
            inner.AUX_MU_IIR.write(AUX_MU_IIR::FIFO_CLEAR::All);
        });

        Ok(())
    }
}

impl interface::console::Write for MiniUart {
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
        r.lock(|inner| loop {
            if inner.AUX_MU_LSR.is_set(AUX_MU_LSR::TX_IDLE) {
                break;
            }
        });
    }
}

impl interface::console::Read for MiniUart {
    fn read_char(&self) -> char {
        let mut r = &self.inner;
        r.lock(|inner| {
            // Wait until buffer is filled.
            loop {
                if inner.AUX_MU_LSR.is_set(AUX_MU_LSR::DATA_READY) {
                    break;
                }

                arch::nop();
            }

            // Read one character.
            let mut ret = inner.AUX_MU_IO.get() as u8 as char;

            // Convert carrige return to newline.
            if ret == '\r' {
                ret = '\n'
            }

            ret
        })
    }

    fn clear(&self) {
        let mut r = &self.inner;
        r.lock(|inner| {
            inner.AUX_MU_IIR.write(AUX_MU_IIR::FIFO_CLEAR::All);
        })
    }
}

impl interface::console::Statistics for MiniUart {
    fn chars_written(&self) -> usize {
        let mut r = &self.inner;
        r.lock(|inner| inner.chars_written)
    }
}
