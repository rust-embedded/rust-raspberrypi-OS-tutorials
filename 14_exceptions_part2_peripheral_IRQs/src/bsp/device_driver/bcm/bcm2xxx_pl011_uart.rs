// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! PL011 UART driver.
//!
//! # FIFO fill level IRQ hack
//!
//! For learning purposes, we want the UART to raise an IRQ on _every_ received character.
//! Unfortunately, this rather common mode of operation is not supported by the PL011 when operating
//! in FIFO mode. It is only possible to set a fill level fraction on which the IRQ is triggered.
//! The lowest fill level is 1/8.
//!
//! On the RPi3, the RX FIFO is 16 chars deep, so the IRQ would trigger after 2 chars have been
//! received. On the RPi4, the FIFO seems to be 32 chars deep, because experiments showed that the
//! RX IRQ triggers after receiving 4 chars.
//!
//! Fortunately, the PL011 has a test mode which allows to push characters into the FIFOs. We make
//! use of this testing facilities to employ a little hack that pushes (fill-level - 1) chars into
//! the RX FIFO by default. This way, we get an IRQ for the first received char that arrives from
//! external.
//!
//! To make things even more complicated, QEMU is not honoring the fill-level dependent IRQ
//! generation. Instead, QEMU creates an IRQ on every received char.
//!
//! We use conditional compilation to differentiate between the three modes of operation (RPi3,
//! RPI4, QEMU) respectively.

use crate::{
    bsp, console, cpu, driver, exception, synchronization, synchronization::IRQSafeNullLock,
};
use core::{fmt, ops};
use register::{mmio::*, register_bitfields, register_structs};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

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

    /// Interrupt FIFO Level Select Register
    IFLS [
        /// Receive interrupt FIFO level select. The trigger points for the receive interrupt are as
        /// follows.
        RXIFLSEL OFFSET(3) NUMBITS(5) [
            OneEigth = 0b000,
            OneQuarter = 0b001,
            OneHalf = 0b010,
            ThreeQuarters = 0b011,
            SevenEights = 0b100
        ]
    ],

    /// Interrupt Mask Set Clear Register
    IMSC [
        /// Receive interrupt mask. A read returns the current mask for the UARTRXINTR interrupt. On
        /// a write of 1, the mask of the interrupt is set. A write of 0 clears the mask.
        RXIM OFFSET(4) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Interrupt Clear Register
    ICR [
        /// Meta field for all pending interrupts
        ALL OFFSET(0) NUMBITS(11) []
    ],

    /// Test Control Register
    ITCR [
        /// Test FIFO enable. When this bit it 1, a write to the Test Data Register, UART_DR writes
        /// data into the receive FIFO, and reads from the UART_DR register reads data out of the
        /// transmit FIFO. When this bit is 0, data cannot be read directly from the transmit FIFO
        /// or written directly to the receive FIFO (normal operation).
        ITCR1 OFFSET(1) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ]
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => DR: ReadWrite<u32>),
        (0x04 => _reserved1),
        (0x18 => FR: ReadOnly<u32, FR::Register>),
        (0x1c => _reserved2),
        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
        (0x2c => LCRH: WriteOnly<u32, LCRH::Register>),
        (0x30 => CR: WriteOnly<u32, CR::Register>),
        (0x34 => IFLS: ReadWrite<u32, IFLS::Register>),
        (0x38 => IMSC: ReadWrite<u32, IMSC::Register>),
        (0x3C => _reserved3),
        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
        (0x48 => _reserved4),
        (0x80 => ITCR: ReadWrite<u32, ITCR::Register>),
        (0x84 => _reserved5),
        (0x8c => TDR: ReadWrite<u32>),
        (0x90 => @END),
    }
}

pub struct PL011UartInner {
    base_addr: usize,
    chars_written: usize,
    chars_read: usize,
}

// Export the inner struct so that BSPs can use it for the panic handler.
pub use PL011UartInner as PanicUart;

/// Representation of the UART.
pub struct PL011Uart {
    inner: IRQSafeNullLock<PL011UartInner>,
    irq_number: bsp::device_driver::IRQNumber,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

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
    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide the correct `base_addr`.
    pub const unsafe fn new(base_addr: usize) -> Self {
        Self {
            base_addr,
            chars_written: 0,
            chars_read: 0,
        }
    }

    /// Set up baud rate and characteristics.
    ///
    /// Results in 8N1 and 230400 baud (if the clk has been previously set to 48 MHz by the
    /// firmware).
    pub fn init(&mut self) {
        // Turn it off temporarily.
        self.CR.set(0);

        self.ICR.write(ICR::ALL::CLEAR);
        self.IBRD.write(IBRD::IBRD.val(13));
        self.FBRD.write(FBRD::FBRD.val(2));
        self.LCRH
            .write(LCRH::WLEN::EightBit + LCRH::FEN::FifosEnabled); // 8N1 + Fifo on
        self.CR
            .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);

        // Trigger the RX interrupt at 1/8 of the FIFO fill level (this is the lowest possible) and
        // enable RX interrupts.
        self.IFLS.write(IFLS::RXIFLSEL::OneEigth);
        self.IMSC.write(IMSC::RXIM::Enabled);

        #[cfg(not(feature = "qemu-quirks"))]
        self.fill_hack_push();
    }

    /// Return a pointer to the register block.
    fn ptr(&self) -> *const RegisterBlock {
        self.base_addr as *const _
    }

    /// Send a character.
    fn write_char(&mut self, c: char) {
        // Spin while TX FIFO full is set, waiting for an empty slot.
        while self.FR.matches_all(FR::TXFF::SET) {
            cpu::nop();
        }

        // Write the character to the buffer.
        self.DR.set(c as u32);

        self.chars_written += 1;
    }

    /// Retrieve a character.
    fn read_char_converting(&mut self, blocking: bool) -> Option<char> {
        #[cfg(not(feature = "qemu-quirks"))]
        self.fill_hack_pop();

        // If blocking, spin while RX FIFO empty is set, else return None.
        while self.FR.matches_all(FR::RXFE::SET) {
            if !blocking {
                #[cfg(not(feature = "qemu-quirks"))]
                self.fill_hack_push();

                return None;
            }

            cpu::nop();
        }

        // Read one character.
        let mut ret = self.DR.get() as u8 as char;

        // Convert carrige return to newline.
        if ret == '\r' {
            ret = '\n'
        }

        // Update statistics.
        self.chars_read += 1;

        #[cfg(not(feature = "qemu-quirks"))]
        self.fill_hack_push();

        Some(ret)
    }

    /// Push characters into the receive FIFO.
    ///
    /// See top level comments why this is needed.
    #[cfg(not(feature = "qemu-quirks"))]
    fn fill_hack_push(&mut self) {
        self.ITCR.write(ITCR::ITCR1::Enabled);

        #[cfg(feature = "bsp_rpi4")]
        {
            self.TDR.set(b'X' as u32);
            self.TDR.set(b'Y' as u32);
        }
        self.TDR.set(b'Z' as u32);

        self.ITCR.write(ITCR::ITCR1::Disabled);
    }

    /// Pop characters from the receive FIFO.
    ///
    /// See top level comments why this is needed.
    #[cfg(not(feature = "qemu-quirks"))]
    fn fill_hack_pop(&mut self) {
        #[cfg(feature = "bsp_rpi4")]
        {
            self.DR.get();
            self.DR.get();
        }
        self.DR.get();
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
            self.write_char(c);
        }

        Ok(())
    }
}

impl PL011Uart {
    /// # Safety
    ///
    /// - The user must ensure to provide the correct `base_addr`.
    pub const unsafe fn new(base_addr: usize, irq_number: bsp::device_driver::IRQNumber) -> Self {
        Self {
            inner: IRQSafeNullLock::new(PL011UartInner::new(base_addr)),
            irq_number,
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
use synchronization::interface::Mutex;

impl driver::interface::DeviceDriver for PL011Uart {
    fn compatible(&self) -> &str {
        "BCM PL011 UART"
    }

    fn init(&self) -> Result<(), ()> {
        let mut r = &self.inner;
        r.lock(|inner| inner.init());

        Ok(())
    }

    fn register_and_enable_irq_handler(&'static self) -> Result<(), &'static str> {
        use bsp::exception::asynchronous::irq_manager;
        use exception::asynchronous::{interface::IRQManager, IRQDescriptor};

        let descriptor = IRQDescriptor {
            name: "BCM PL011 UART",
            handler: self,
        };

        irq_manager().register_handler(self.irq_number, descriptor)?;
        irq_manager().enable(self.irq_number);

        Ok(())
    }
}

impl console::interface::Write for PL011Uart {
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
        // Spin until TX FIFO empty is set.
        let mut r = &self.inner;
        r.lock(|inner| {
            while !inner.FR.matches_all(FR::TXFE::SET) {
                cpu::nop();
            }
        });
    }
}

impl console::interface::Read for PL011Uart {
    fn read_char(&self) -> char {
        let mut r = &self.inner;
        r.lock(|inner| inner.read_char_converting(true).unwrap())
    }

    fn clear(&self) {
        let mut r = &self.inner;
        r.lock(|inner| {
            // Read from the RX FIFO until it is indicating empty.
            while !inner.FR.matches_all(FR::RXFE::SET) {
                inner.DR.get();
            }
        })
    }
}

impl console::interface::Statistics for PL011Uart {
    fn chars_written(&self) -> usize {
        let mut r = &self.inner;
        r.lock(|inner| inner.chars_written)
    }

    fn chars_read(&self) -> usize {
        let mut r = &self.inner;
        r.lock(|inner| inner.chars_read)
    }
}

impl exception::asynchronous::interface::IRQHandler for PL011Uart {
    fn handle(&self) -> Result<(), &'static str> {
        let mut r = &self.inner;
        r.lock(|inner| {
            // Echo any received characters.
            loop {
                match inner.read_char_converting(false) {
                    None => break,
                    Some(c) => inner.write_char(c),
                }
            }
        });

        Ok(())
    }
}
