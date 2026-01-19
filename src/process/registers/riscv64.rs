#![allow(dead_code)]
use crate::process::{Register, RegisterValue};
use anyhow::Result;
use libc::user_regs_struct;
use nix::unistd::Pid;

/// Current state of the registers for the debugged process.
#[derive(Clone, Debug)]
pub struct RegisterSnapshot {
    pid: Pid,

    // Note: the current implementation is simply wrapping the libc structs.
    // this is probably sufficient, but i could create a rust-equivalent that pairs
    // with the RegisterInfo a bit more. we shall see if this impl becomes a burden ...
    user_gp: user_regs_struct,
    // NOTE: leaving the floating point registers out for now. The linux riscv64
    // headers (/usr/riscv64-linux-gnu/include/asm/ptrace.h) defines __riscv_d_ext_state /
    // __riscv_f_ext_state / __riscv_q_ext_state, but I'm not sure how to get those from `nix`.
    // I think they need to be added to libc first (there's some similar "signal"-related structs),
    // and then figure out how to plumb it into nix::ptrace.
}

impl RegisterSnapshot {
    pub fn read(&self, _register: &Register) -> RegisterValue {
        todo!("impl me");
    }
}

pub fn read_all_registers(_pid: Pid) -> Result<RegisterSnapshot> {
    todo!("impl me");
}
