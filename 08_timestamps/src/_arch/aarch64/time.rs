// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Architectural timer primitives.

use crate::{time, warn};
use core::time::Duration;
use cortex_a::regs::*;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

const NS_PER_S: u64 = 1_000_000_000;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// ARMv8 Generic Timer.
pub struct GenericTimer;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static TIME_MANAGER: GenericTimer = GenericTimer;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the time manager.
pub fn time_manager() -> &'static impl time::interface::TimeManager {
    &TIME_MANAGER
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

impl time::interface::TimeManager for GenericTimer {
    fn resolution(&self) -> Duration {
        Duration::from_nanos(NS_PER_S / (CNTFRQ_EL0.get() as u64))
    }

    fn uptime(&self) -> Duration {
        let frq: u64 = CNTFRQ_EL0.get() as u64;
        let current_count: u64 = CNTPCT_EL0.get() * NS_PER_S;

        Duration::from_nanos(current_count / frq)
    }

    fn spin_for(&self, duration: Duration) {
        // Instantly return on zero.
        if duration.as_nanos() == 0 {
            return;
        }

        // Calculate the register compare value.
        let frq = CNTFRQ_EL0.get() as u64;
        let x = match frq.checked_mul(duration.as_nanos() as u64) {
            None => {
                warn!("Spin duration too long, skipping");
                return;
            }
            Some(val) => val,
        };
        let tval = x / NS_PER_S;

        // Check if it is within supported bounds.
        let warn: Option<&str> = if tval == 0 {
            Some("smaller")
        } else if tval > u32::max_value().into() {
            Some("bigger")
        } else {
            None
        };

        if let Some(w) = warn {
            warn!(
                "Spin duration {} than architecturally supported, skipping",
                w
            );
            return;
        }

        // Set the compare value register.
        CNTP_TVAL_EL0.set(tval as u32);

        // Kick off the counting.                       // Disable timer interrupt.
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::SET);

        // ISTATUS will be '1' when cval ticks have passed. Busy-check it.
        while !CNTP_CTL_EL0.matches_all(CNTP_CTL_EL0::ISTATUS::SET) {}

        // Disable counting again.
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::CLEAR);
    }
}
