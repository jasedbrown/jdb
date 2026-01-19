#![allow(dead_code)]
use anyhow::Result;
use libc::{user_regs_struct, user_fpsimd_struct};
use nix::unistd::Pid;
use crate::process::{Register, RegisterBackend, RegisterValue};

/// Current state of the registers for the debugged process.
#[derive(Clone, Debug)]
pub struct RegisterSnapshot {
    pid: Pid,

    // Note: the current implementation is simply wrapping the libc structs.
    // this is probably sufficient, but i could create a rust-equivalent that pairs
    // with the RegisterInfo a bit more. we shall see if this impl becomes a burden ...
    user_gp: user_regs_struct,
    user_fp: user_fpsimd_struct,
}

impl RegisterSnapshot {
    pub fn read(&self, _register: &Register) -> RegisterValue {
        todo!("impl me");
    }
}

pub fn read_all_registers(_pid: Pid) -> Result<RegisterSnapshot> {
    todo!("impl me");
}

