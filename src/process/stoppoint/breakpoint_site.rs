use std::sync::atomic::{AtomicI32, Ordering};

use crate::process::stoppoint::{StoppointId, StoppointState, VirtualAddress};

// Simple global ID generator; relaxed ordering is sufficient for a monotonic counter.
static NEXT_ID: AtomicI32 = AtomicI32::new(1);

fn next_id() -> StoppointId {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    StoppointId { id }
}

/// A software breakpoint.
#[derive(Clone, Debug)]
pub struct BreakpointSite {
    id: StoppointId,
    //process: Process ???
    address: VirtualAddress,

    state: StoppointState,
}

impl BreakpointSite {
    pub fn new(address: VirtualAddress) -> Self {
        Self {
            id: next_id(),
            address,
            state: StoppointState::Disabled,
        }
    }

    pub fn id(&self) -> StoppointId {
        self.id
    }

    pub fn enable(&mut self) {
        self.state = StoppointState::Enabled;
    }

    pub fn disable(&mut self) {
        self.state = StoppointState::Disabled
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self.state, StoppointState::Enabled)
    }

    pub fn address(&self) -> VirtualAddress {
        self.address
    }

    pub fn at_address(&self, address: VirtualAddress) -> bool {
        self.address == address
    }

    pub fn in_range(&self, low: VirtualAddress, high: VirtualAddress) -> bool {
        low <= self.address && high > self.address
    }
}
