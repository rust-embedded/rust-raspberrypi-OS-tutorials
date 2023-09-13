// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>

//! Timer primitives.
//!
//! # Resources
//!
//! - <https://stackoverflow.com/questions/41081240/idiomatic-callbacks-in-rust>
//! - <https://doc.rust-lang.org/stable/std/panic/fn.set_hook.html>

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/time.rs"]
mod arch_time;

use crate::{
    driver, exception,
    exception::asynchronous::IRQNumber,
    synchronization::{interface::Mutex, IRQSafeNullLock},
    warn,
};
use alloc::{boxed::Box, vec::Vec};
use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

struct Timeout {
    due_time: Duration,
    period: Option<Duration>,
    callback: TimeoutCallback,
}

struct OrderedTimeoutQueue {
    // Can be replaced with a BinaryHeap once it's new() becomes const.
    inner: Vec<Timeout>,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The callback type used by timer IRQs.
pub type TimeoutCallback = Box<dyn Fn() + Send>;

/// Provides time management functions.
pub struct TimeManager {
    queue: IRQSafeNullLock<OrderedTimeoutQueue>,
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static TIME_MANAGER: TimeManager = TimeManager::new();

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl Timeout {
    pub fn is_periodic(&self) -> bool {
        self.period.is_some()
    }

    pub fn refresh(&mut self) {
        if let Some(delay) = self.period {
            self.due_time += delay;
        }
    }
}

impl OrderedTimeoutQueue {
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push(&mut self, timeout: Timeout) {
        self.inner.push(timeout);

        // Note reverse compare order so that earliest expiring item is at end of vec. We do this so
        // that we can use Vec::pop below to retrieve the item that is next due.
        self.inner.sort_by(|a, b| b.due_time.cmp(&a.due_time));
    }

    pub fn peek_next_due_time(&self) -> Option<Duration> {
        let timeout = self.inner.last()?;

        Some(timeout.due_time)
    }

    pub fn pop(&mut self) -> Option<Timeout> {
        self.inner.pop()
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the global TimeManager.
pub fn time_manager() -> &'static TimeManager {
    &TIME_MANAGER
}

impl TimeManager {
    /// Compatibility string.
    pub const COMPATIBLE: &'static str = "ARM Architectural Timer";

    /// Create an instance.
    pub const fn new() -> Self {
        Self {
            queue: IRQSafeNullLock::new(OrderedTimeoutQueue::new()),
        }
    }

    /// The timer's resolution.
    pub fn resolution(&self) -> Duration {
        arch_time::resolution()
    }

    /// The uptime since power-on of the device.
    ///
    /// This includes time consumed by firmware and bootloaders.
    pub fn uptime(&self) -> Duration {
        arch_time::uptime()
    }

    /// Spin for a given duration.
    pub fn spin_for(&self, duration: Duration) {
        arch_time::spin_for(duration)
    }

    /// Set a timeout.
    fn set_timeout(&self, timeout: Timeout) {
        self.queue.lock(|queue| {
            queue.push(timeout);

            arch_time::set_timeout_irq(queue.peek_next_due_time().unwrap());
        });
    }

    /// Set a one-shot timeout.
    pub fn set_timeout_once(&self, delay: Duration, callback: TimeoutCallback) {
        let timeout = Timeout {
            due_time: self.uptime() + delay,
            period: None,
            callback,
        };

        self.set_timeout(timeout);
    }

    /// Set a periodic timeout.
    pub fn set_timeout_periodic(&self, delay: Duration, callback: TimeoutCallback) {
        let timeout = Timeout {
            due_time: self.uptime() + delay,
            period: Some(delay),
            callback,
        };

        self.set_timeout(timeout);
    }
}

/// Initialize the timer subsystem.
pub fn init() -> Result<(), &'static str> {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    if INIT_DONE.load(Ordering::Relaxed) {
        return Err("Init already done");
    }

    let timer_descriptor =
        driver::DeviceDriverDescriptor::new(time_manager(), None, Some(arch_time::timeout_irq()));
    driver::driver_manager().register_driver(timer_descriptor);

    INIT_DONE.store(true, Ordering::Relaxed);
    Ok(())
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

impl driver::interface::DeviceDriver for TimeManager {
    type IRQNumberType = IRQNumber;

    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    fn register_and_enable_irq_handler(
        &'static self,
        irq_number: &Self::IRQNumberType,
    ) -> Result<(), &'static str> {
        use exception::asynchronous::{irq_manager, IRQHandlerDescriptor};

        let descriptor = IRQHandlerDescriptor::new(*irq_number, Self::COMPATIBLE, self);

        irq_manager().register_handler(descriptor)?;
        irq_manager().enable(irq_number);

        Ok(())
    }
}

impl exception::asynchronous::interface::IRQHandler for TimeManager {
    fn handle(&self) -> Result<(), &'static str> {
        arch_time::conclude_timeout_irq();

        let maybe_timeout: Option<Timeout> = self.queue.lock(|queue| {
            let next_due_time = queue.peek_next_due_time()?;
            if next_due_time > self.uptime() {
                return None;
            }

            let mut timeout = queue.pop().unwrap();

            // Refresh as early as possible to prevent drift.
            if timeout.is_periodic() {
                timeout.refresh();
            }

            Some(timeout)
        });

        let timeout = match maybe_timeout {
            None => {
                warn!("Spurious timeout IRQ");
                return Ok(());
            }
            Some(t) => t,
        };

        // Important: Call the callback while not holding any lock, because the callback might
        // attempt to modify data that is protected by a lock (in particular, the timeout queue
        // itself).
        (timeout.callback)();

        self.queue.lock(|queue| {
            if timeout.is_periodic() {
                // There might be some overhead involved in the periodic path, because the timeout
                // item is first popped from the underlying Vec and then pushed back again. It could
                // be faster to keep the item in the queue and find a way to work with a reference
                // to it.
                //
                // We are not going this route on purpose, though. It allows to keep the code simple
                // and the focus on the high-level concepts.
                queue.push(timeout);
            };

            if let Some(due_time) = queue.peek_next_due_time() {
                arch_time::set_timeout_irq(due_time);
            }
        });

        Ok(())
    }
}
