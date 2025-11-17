use anyhow::Result;
use libc::{user, user_fpregs_struct, user_regs_struct};
use memoffset::offset_of;
use nix::sys::ptrace::{getregset, read_user, regset};
use nix::unistd::Pid;
use strum::{AsRefStr, EnumDiscriminants};

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::process::register_info::{REGISTERS_INFO, Register, RegisterInfo};

// ugh, phf will not work as the keys must be of a limited set of types.
// Just use lazy/once_cell to bulid a fucking map
static REGISTERS_MAP: LazyLock<HashMap<Register, RegisterInfo>> = LazyLock::new(|| {
    let mut regs = HashMap::new();

    REGISTERS_INFO.iter().for_each(|r| {
        regs.insert(r.register, r.clone());
    });

    regs
});

// rename me
pub struct RegisterSnapshot {
    // layout ideas
    // 1. field per register
    // 2. wrapper around the backing c structs
    user_gp: user_regs_struct,
    user_fp: user_fpregs_struct,
    debug_regs: [u64; 8],
    // 3.
}

impl RegisterSnapshot {
    // pub fn new(_gp_regs: user_regs_struct, _fp_regs: user_fpregs_struct, _debug_regs: [u64; 8]) -> Self {
    //     Self {}
    // }

    // pub fn get_value(&self, register: Register) -> RegisterValue {

    // }

    // pub fn set_value(&mut self, register: Register, value: RegisterValue) -> Result<()> {
    //     // update register on CPU - ptrace::write_user

    //     // update self - or just re-read_all_registers()
    // }
}

// pub fn read_all_registers(pid: Pid) -> Result<RegisterSnapshot> {
//     let gp_reg = getregset::<regset::NT_PRSTATUS>(pid).unwrap();
//     let fp_reg = getregset::<regset::NT_PRFPREG>(pid).unwrap();

//     let mut debug_regs = [0; 8];
//     // read out the debug registers
//     for i in 0..7 {
//         let base_regs_offset = offset_of!(user, u_debugreg);
//         // TODO: don't hardcode the offset
//         let offset = base_regs_offset + (i * 8);
//         let reg = read_user(pid, offset as _).unwrap();
//         debug_regs[i] = reg as u64;
//     }

//     Ok(RegisterSnapshot::new(gp_reg, fp_reg, debug_regs))
// }
