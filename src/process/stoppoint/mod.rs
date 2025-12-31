use anyhow::{Error, anyhow};

use crate::process::register_info::RegisterValue;

pub mod breakpoint_site;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StoppointId {
    id: i32,
}

impl TryFrom<Vec<String>> for StoppointId {
    type Error = Error;

    fn try_from(v: Vec<String>) -> Result<Self, Self::Error> {
        if v.len() != 1 {
            return Err(anyhow!("Wrong number of arguments: {:?}", v));
        }

        let s = v.first().unwrap();
        let id = s.parse::<i32>()?;
        Ok(Self { id })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct VirtualAddress {
    address: u64,
}

impl TryFrom<Vec<String>> for VirtualAddress {
    type Error = Error;

    fn try_from(v: Vec<String>) -> Result<Self, Self::Error> {
        if v.len() != 1 {
            return Err(anyhow!("Wrong number of arguments: {:?}", v));
        }

        let s = v.first().unwrap();
        let address = s.parse::<u64>()?;
        Ok(Self { address })
    }
}

impl TryFrom<RegisterValue> for VirtualAddress {
    type Error = Error;

    fn try_from(value: RegisterValue) -> Result<Self, Self::Error> {
        match value {
            RegisterValue::Uint64(address) => Ok(Self { address }),
            _ => Err(anyhow!("Cannot create virtual address from: {:?}", value)),
        }
    }
}

impl From<u64> for VirtualAddress {
    fn from(value: u64) -> Self {
        Self { address: value }
    }
}

impl From<VirtualAddress> for RegisterValue {
    fn from(address: VirtualAddress) -> RegisterValue {
        RegisterValue::Uint64(address.address)
    }
}

impl VirtualAddress {
    pub fn addr(&self) -> u64 {
        self.address
    }
}

/// This is the `int3` instruction, which causes the prcoess to break/signal.
pub const INTERRUPT_INSTRUCTION: i64 = 0xCC;

#[derive(Clone, Copy, Debug)]
pub enum StoppointState {
    Enabled,
    Disabled,
}
