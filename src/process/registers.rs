#![allow(dead_code)]
use anyhow::Result;
use libc::{user, user_fpregs_struct, user_regs_struct};
use memoffset::offset_of;
use nix::sys::ptrace::{getregset, read_user, regset};
use nix::unistd::Pid;

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::process::register_info::{REGISTERS_INFO, Register, RegisterInfo, RegisterValue};

// ugh, phf will not work as the keys must be of a limited set of types.
// Just use lazy/once_cell to bulid a fucking map
static REGISTERS_MAP: LazyLock<HashMap<Register, RegisterInfo>> = LazyLock::new(|| {
    let mut regs = HashMap::new();

    REGISTERS_INFO.iter().for_each(|r| {
        regs.insert(r.register, r.clone());
    });

    regs
});

/// Current state of the registers for the debugged process.
#[derive(Clone, Debug)]
pub struct RegisterSnapshot {
    pid: Pid,

    // Note: the current implementation is simply wrapping the libc structs.
    // this is probably sufficient, but i could create a rust-equivalent that pairs
    // with the RegisterInfo a bit more. we shall see if this impl becomes a burden ...
    user_gp: user_regs_struct,
    user_fp: user_fpregs_struct,
    debug_regs: [u64; 8],
}

impl RegisterSnapshot {
    fn new(
        pid: Pid,
        gp_regs: user_regs_struct,
        fp_regs: user_fpregs_struct,
        debug_regs: [u64; 8],
    ) -> Self {
        Self {
            pid,
            user_gp: gp_regs,
            user_fp: fp_regs,
            debug_regs,
        }
    }

    pub fn get_value(&self, register: Register) -> RegisterValue {
        // TODO: actually implement me
        match register {
            Register::R15 => RegisterValue::Uint(15),
            _ => RegisterValue::Uint(0),
        }
    }

    // This is called infrequently enough that just re-reading all the registers
    // and returning a new instance is not a problem.
    pub fn set_value(
        &mut self,
        _register: Register,
        _value: RegisterValue,
    ) -> Result<RegisterSnapshot> {
        // update register on CPU - ptrace::write_user

        // update self - or just re-read_all_registers()
        read_all_registers(self.pid)
    }
}

pub fn read_all_registers(pid: Pid) -> Result<RegisterSnapshot> {
    let gp_reg = getregset::<regset::NT_PRSTATUS>(pid).unwrap();
    let fp_reg = getregset::<regset::NT_PRFPREG>(pid).unwrap();

    let mut debug_regs = [0; 8];
    // read out the debug registers
    for i in 0..debug_regs.len() {
        let base_regs_offset = offset_of!(user, u_debugreg);
        // TODO: don't hardcode the offset
        let offset = base_regs_offset + (i * 8);
        let reg = read_user(pid, offset as _).unwrap();
        debug_regs[i] = reg as u64;
    }

    Ok(RegisterSnapshot::new(pid, gp_reg, fp_reg, debug_regs))
}
