use anyhow::{Error, anyhow};

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
