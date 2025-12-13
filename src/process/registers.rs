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
///
/// this is a glorified wrapper around the `user` struct, but deconstructed
/// to the pieces we need.
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

    pub fn get_value(&self, register: &Register) -> RegisterValue {
        match register {
            // 64-bit values
            Register::RAX => RegisterValue::Uint(self.user_gp.rax),
            Register::RDX => RegisterValue::Uint(self.user_gp.rdx),
            Register::RCX => RegisterValue::Uint(self.user_gp.rcx),
            Register::RBX => RegisterValue::Uint(self.user_gp.rbx),
            Register::RSI => RegisterValue::Uint(self.user_gp.rsi),
            Register::RDI => RegisterValue::Uint(self.user_gp.rdi),
            Register::RBP => RegisterValue::Uint(self.user_gp.rbp),
            Register::RSP => RegisterValue::Uint(self.user_gp.rsp),
            Register::R8 => RegisterValue::Uint(self.user_gp.r8),
            Register::R9 => RegisterValue::Uint(self.user_gp.r9),
            Register::R10 => RegisterValue::Uint(self.user_gp.r10),
            Register::R11 => RegisterValue::Uint(self.user_gp.r11),
            Register::R12 => RegisterValue::Uint(self.user_gp.r12),
            Register::R13 => RegisterValue::Uint(self.user_gp.r13),
            Register::R14 => RegisterValue::Uint(self.user_gp.r14),
            Register::R15 => RegisterValue::Uint(self.user_gp.r15),
            Register::RIP => RegisterValue::Uint(self.user_gp.rip),
            Register::EFLAGS => RegisterValue::Uint(self.user_gp.eflags),
            Register::CS => RegisterValue::Uint(self.user_gp.cs),
            Register::FS => RegisterValue::Uint(self.user_gp.fs),
            Register::SS => RegisterValue::Uint(self.user_gp.ss),
            Register::DS => RegisterValue::Uint(self.user_gp.ds),
            Register::ES => RegisterValue::Uint(self.user_gp.es),
            Register::ORIGRAX => RegisterValue::Uint(self.user_gp.orig_rax),

            // 32-bit registers
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

    // read out the debug registers
    let mut debug_regs = [0; 8];
    let base_regs_offset = offset_of!(user, u_debugreg);
    for (i, e) in debug_regs.iter_mut().enumerate() {
        // TODO: don't hardcode the offset
        let offset = base_regs_offset + (i * 8);
        let reg = read_user(pid, offset as _).unwrap();
        *e = reg as u64;
    }

    Ok(RegisterSnapshot::new(pid, gp_reg, fp_reg, debug_regs))
}
