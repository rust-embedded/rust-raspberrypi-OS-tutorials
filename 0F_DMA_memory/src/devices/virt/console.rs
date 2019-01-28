/*
 * MIT License
 *
 * Copyright (c) 2019 Andre Richter <andre.o.richter@gmail.com>
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

use crate::devices::hw;
use core::fmt;

/// A trait that must be implemented by devices that are candidates for the
/// global console.
#[allow(unused_variables)]
pub trait ConsoleOps: Drop {
    fn putc(&self, c: char) {}
    fn puts(&self, string: &str) {}
    fn getc(&self) -> char {
        ' '
    }
    fn flush(&self) {}
}

/// A dummy console that just ignores its inputs.
pub struct NullConsole;
impl Drop for NullConsole {
    fn drop(&mut self) {}
}
impl ConsoleOps for NullConsole {}

/// Possible outputs which the console can store.
pub enum Output {
    None(NullConsole),
    MiniUart(hw::MiniUart),
    PL011Uart(hw::PL011Uart),
}

impl From<hw::MiniUart> for Output {
    fn from(instance: hw::MiniUart) -> Self {
        Output::MiniUart(instance)
    }
}

impl From<hw::PL011Uart> for Output {
    fn from(instance: hw::PL011Uart) -> Self {
        Output::PL011Uart(instance)
    }
}

pub struct Console {
    output: Output,
}

impl Console {
    pub const fn new() -> Console {
        Console {
            output: Output::None(NullConsole {}),
        }
    }

    #[inline(always)]
    fn current_ptr(&self) -> &dyn ConsoleOps {
        match &self.output {
            Output::None(i) => i,
            Output::MiniUart(i) => i,
            Output::PL011Uart(i) => i,
        }
    }

    /// Overwrite the current output. The old output will go out of scope and
    /// it's Drop function will be called.
    pub fn replace_with(&mut self, x: Output) {
        self.current_ptr().flush();

        self.output = x;
    }

    /// A command prompt. Currently does nothing.
    pub fn command_prompt(&self) -> ! {
        self.puts("\n$> ");

        let mut input;
        loop {
            input = self.getc();

            if input == '\n' {
                self.puts("\n$> ")
            } else {
                self.putc(input);
            }
        }
    }
}

impl Drop for Console {
    fn drop(&mut self) {}
}

/// Dispatch the respective function to the currently stored output device.
impl ConsoleOps for Console {
    fn putc(&self, c: char) {
        self.current_ptr().putc(c);
    }

    fn puts(&self, string: &str) {
        self.current_ptr().puts(string);
    }

    fn getc(&self) -> char {
        self.current_ptr().getc()
    }

    fn flush(&self) {
        self.current_ptr().flush()
    }
}

/// Implementing this trait enables usage of the format_args! macros, which in
/// turn are used to implement the kernel's print! and println! macros.
///
/// See src/macros.rs.
impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.current_ptr().puts(s);

        Ok(())
    }
}
