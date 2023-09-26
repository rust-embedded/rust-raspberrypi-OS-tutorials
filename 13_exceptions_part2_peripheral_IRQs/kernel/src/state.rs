// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>

//! State information about the kernel itself.

use core::sync::atomic::{AtomicU8, Ordering};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// Different stages in the kernel execution.
#[derive(Copy, Clone, Eq, PartialEq)]
enum State {
    /// The kernel starts booting in this state.
    Init,

    /// The kernel transitions to this state when jumping to `kernel_main()` (at the end of
    /// `kernel_init()`, after all init calls are done).
    SingleCoreMain,

    /// The kernel transitions to this state when it boots the secondary cores, aka switches
    /// exectution mode to symmetric multiprocessing (SMP).
    MultiCoreMain,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Maintains the kernel state and state transitions.
pub struct StateManager(AtomicU8);

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static STATE_MANAGER: StateManager = StateManager::new();

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the global StateManager.
pub fn state_manager() -> &'static StateManager {
    &STATE_MANAGER
}

impl StateManager {
    const INIT: u8 = 0;
    const SINGLE_CORE_MAIN: u8 = 1;
    const MULTI_CORE_MAIN: u8 = 2;

    /// Create a new instance.
    pub const fn new() -> Self {
        Self(AtomicU8::new(Self::INIT))
    }

    /// Return the current state.
    fn state(&self) -> State {
        let state = self.0.load(Ordering::Acquire);

        match state {
            Self::INIT => State::Init,
            Self::SINGLE_CORE_MAIN => State::SingleCoreMain,
            Self::MULTI_CORE_MAIN => State::MultiCoreMain,
            _ => panic!("Invalid KERNEL_STATE"),
        }
    }

    /// Return if the kernel is init state.
    pub fn is_init(&self) -> bool {
        self.state() == State::Init
    }

    /// Transition from Init to SingleCoreMain.
    pub fn transition_to_single_core_main(&self) {
        if self
            .0
            .compare_exchange(
                Self::INIT,
                Self::SINGLE_CORE_MAIN,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_err()
        {
            panic!("transition_to_single_core_main() called while state != Init");
        }
    }
}
