#![allow(dead_code)]
use crate::process::{Register, RegisterValue};
use anyhow::Result;
use libc::{user_fpsimd_struct, user_regs_struct};
use nix::sys::ptrace::{getregset, read_user, regset, setregset, write_user};
use nix::unistd::Pid;

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
    fn new(pid: Pid, gp_regs: user_regs_struct, fp_regs: user_fpsimd_struct) -> Self {
        Self {
            pid,
            user_gp: gp_regs,
            user_fp: fp_regs,
        }
    }

    pub fn read(&self, _register: &Register) -> RegisterValue {
        todo!("impl me");
    }

    #[allow(dead_code)]
    pub fn write(&mut self, _register: Register, _value: RegisterValue) -> Result<()> {
        todo!("impl me");
    }
}

pub fn read_all_registers(pid: Pid) -> Result<RegisterSnapshot> {
    let gp_reg = getregset::<regset::NT_PRSTATUS>(pid).unwrap();
    let fp_reg = getregset::<regset::NT_PRFPREG>(pid).unwrap();

    Ok(RegisterSnapshot::new(pid, gp_reg, fp_reg))
}
