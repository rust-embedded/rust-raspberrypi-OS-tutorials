/*
 * MIT License
 *
 * Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use super::MMIO_BASE;
use core::ops;
use register::{
    mmio::{ReadOnly, WriteOnly},
    register_bitfields,
};

register_bitfields! {
    u32,

    STATUS [
        FULL  OFFSET(31) NUMBITS(1) [],
        EMPTY OFFSET(30) NUMBITS(1) []
    ]
}

const VIDEOCORE_MBOX: u32 = MMIO_BASE + 0xB880;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    READ: ReadOnly<u32>,                     // 0x00
    __reserved_0: [u32; 5],                  // 0x04
    STATUS: ReadOnly<u32, STATUS::Register>, // 0x18
    __reserved_1: u32,                       // 0x1C
    WRITE: WriteOnly<u32>,                   // 0x20
}

// Custom errors
pub enum MboxError {
    ResponseError,
    UnknownError,
}
pub type Result<T> = ::core::result::Result<T, MboxError>;

// Channels
pub mod channel {
    pub const PROP: u32 = 8;
}

// Tags
pub mod tag {
    pub const SETCLKRATE: u32 = 0x38002;
    pub const LAST: u32 = 0;
}

// Clocks
pub mod clock {
    pub const UART: u32 = 0x0_0000_0002;
}

// Responses
mod response {
    pub const SUCCESS: u32 = 0x8000_0000;
    pub const ERROR: u32 = 0x8000_0001; // error parsing request buffer (partial response)
}

pub const REQUEST: u32 = 0;

// Public interface to the mailbox
#[repr(C)]
#[repr(align(16))]
pub struct Mbox {
    // The address for buffer needs to be 16-byte aligned so that the
    // Videcore can handle it properly.
    pub buffer: [u32; 36],
}

/// Deref to RegisterBlock
///
/// Allows writing
/// ```
/// self.STATUS.read()
/// ```
/// instead of something along the lines of
/// ```
/// unsafe { (*Mbox::ptr()).STATUS.read() }
/// ```
impl ops::Deref for Mbox {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl Mbox {
    pub fn new() -> Mbox {
        Mbox { buffer: [0; 36] }
    }

    /// Returns a pointer to the register block
    fn ptr() -> *const RegisterBlock {
        VIDEOCORE_MBOX as *const _
    }

    /// Make a mailbox call. Returns Err(MboxError) on failure, Ok(()) success
    pub fn call(&self, channel: u32) -> Result<()> {
        // wait until we can write to the mailbox
        loop {
            if !self.STATUS.is_set(STATUS::FULL) {
                break;
            }

            unsafe { asm!("nop" :::: "volatile") };
        }

        let buf_ptr = self.buffer.as_ptr() as u32;

        // write the address of our message to the mailbox with channel identifier
        self.WRITE.set((buf_ptr & !0xF) | (channel & 0xF));

        // now wait for the response
        loop {
            // is there a response?
            loop {
                if !self.STATUS.is_set(STATUS::EMPTY) {
                    break;
                }

                unsafe { asm!("nop" :::: "volatile") };
            }

            let resp: u32 = self.READ.get();

            // is it a response to our message?
            if ((resp & 0xF) == channel) && ((resp & !0xF) == buf_ptr) {
                // is it a valid successful response?
                return match self.buffer[1] {
                    response::SUCCESS => Ok(()),
                    response::ERROR => Err(MboxError::ResponseError),
                    _ => Err(MboxError::UnknownError),
                };
            }
        }
    }
}
