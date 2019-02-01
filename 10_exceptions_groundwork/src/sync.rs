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

use core::cell::UnsafeCell;

pub struct NullLock<T> {
    data: UnsafeCell<T>,
}

// Since we are instantiating this struct as a static variable, which could
// potentially be shared between different threads, we need to tell the compiler
// that sharing of this struct is safe by marking it with the Sync trait.
//
// At this point in time, we can do so without worrying, because the kernel
// anyways runs on a single core and interrupts are disabled. In short, multiple
// threads don't exist yet in our code.
//
// Literature:
// https://doc.rust-lang.org/beta/nomicon/send-and-sync.html
// https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html
unsafe impl<T> Sync for NullLock<T> {}

impl<T> NullLock<T> {
    pub const fn new(data: T) -> NullLock<T> {
        NullLock {
            data: UnsafeCell::new(data),
        }
    }
}

impl<T> NullLock<T> {
    pub fn lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        // In a real lock, there would be code around this line that ensures
        // that this mutable reference will ever only be given out to one thread
        // at a time.
        f(unsafe { &mut *self.data.get() })
    }
}
