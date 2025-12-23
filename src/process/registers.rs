#![allow(dead_code)]
use anyhow::Result;
use libc::{user, user_fpregs_struct, user_regs_struct};
use memoffset::offset_of;
use nix::sys::ptrace::{getregset, read_user, regset};
use nix::unistd::Pid;

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::process::register_info::{Register, RegisterInfo, RegisterValue, registers_info};

static REGISTERS_MAP: LazyLock<HashMap<Register, RegisterInfo>> = LazyLock::new(|| {
    let mut regs = HashMap::new();

    registers_info().iter().for_each(|r| {
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
        // helpers for floating-point layouts
        fn take_long_double(st_space: &[u32; 32], idx: usize) -> RegisterValue {
            let mut bytes = [0u8; 16];
            let base = idx * 4;
            for i in 0..4 {
                let word = st_space[base + i].to_le_bytes();
                bytes[(i * 4)..(i * 4 + 4)].copy_from_slice(&word);
            }
            let mut ten = [0u8; 10];
            ten.copy_from_slice(&bytes[..10]);
            RegisterValue::LongDouble(ten)
        }

        fn take_xmm(xmm_space: &[u32; 64], idx: usize) -> RegisterValue {
            let mut bytes = [0u8; 16];
            let base = idx * 4;
            for i in 0..4 {
                let word = xmm_space[base + i].to_le_bytes();
                bytes[(i * 4)..(i * 4 + 4)].copy_from_slice(&word);
            }
            RegisterValue::Byte128(bytes)
        }

        match register {
            // 64-bit values
            Register::RAX => RegisterValue::Uint64(self.user_gp.rax),
            Register::RDX => RegisterValue::Uint64(self.user_gp.rdx),
            Register::RCX => RegisterValue::Uint64(self.user_gp.rcx),
            Register::RBX => RegisterValue::Uint64(self.user_gp.rbx),
            Register::RSI => RegisterValue::Uint64(self.user_gp.rsi),
            Register::RDI => RegisterValue::Uint64(self.user_gp.rdi),
            Register::RBP => RegisterValue::Uint64(self.user_gp.rbp),
            Register::RSP => RegisterValue::Uint64(self.user_gp.rsp),
            Register::R8 => RegisterValue::Uint64(self.user_gp.r8),
            Register::R9 => RegisterValue::Uint64(self.user_gp.r9),
            Register::R10 => RegisterValue::Uint64(self.user_gp.r10),
            Register::R11 => RegisterValue::Uint64(self.user_gp.r11),
            Register::R12 => RegisterValue::Uint64(self.user_gp.r12),
            Register::R13 => RegisterValue::Uint64(self.user_gp.r13),
            Register::R14 => RegisterValue::Uint64(self.user_gp.r14),
            Register::R15 => RegisterValue::Uint64(self.user_gp.r15),
            Register::RIP => RegisterValue::Uint64(self.user_gp.rip),
            Register::EFLAGS => RegisterValue::Uint64(self.user_gp.eflags),
            Register::CS => RegisterValue::Uint64(self.user_gp.cs),
            Register::FS => RegisterValue::Uint64(self.user_gp.fs),
            Register::GS => RegisterValue::Uint64(self.user_gp.gs),
            Register::SS => RegisterValue::Uint64(self.user_gp.ss),
            Register::DS => RegisterValue::Uint64(self.user_gp.ds),
            Register::ES => RegisterValue::Uint64(self.user_gp.es),
            Register::ORIGRAX => RegisterValue::Uint64(self.user_gp.orig_rax),

            // 32-bit registers
            Register::EAX => RegisterValue::Uint64(self.user_gp.rax as u32 as u64),
            Register::EDX => RegisterValue::Uint64(self.user_gp.rdx as u32 as u64),
            Register::ECX => RegisterValue::Uint64(self.user_gp.rcx as u32 as u64),
            Register::EBX => RegisterValue::Uint64(self.user_gp.rbx as u32 as u64),
            Register::ESI => RegisterValue::Uint64(self.user_gp.rsi as u32 as u64),
            Register::EDI => RegisterValue::Uint64(self.user_gp.rdi as u32 as u64),
            Register::EBP => RegisterValue::Uint64(self.user_gp.rbp as u32 as u64),
            Register::ESP => RegisterValue::Uint64(self.user_gp.rsp as u32 as u64),
            Register::R8D => RegisterValue::Uint64(self.user_gp.r8 as u32 as u64),
            Register::R9D => RegisterValue::Uint64(self.user_gp.r9 as u32 as u64),
            Register::R10D => RegisterValue::Uint64(self.user_gp.r10 as u32 as u64),
            Register::R11D => RegisterValue::Uint64(self.user_gp.r11 as u32 as u64),
            Register::R12D => RegisterValue::Uint64(self.user_gp.r12 as u32 as u64),
            Register::R13D => RegisterValue::Uint64(self.user_gp.r13 as u32 as u64),
            Register::R14D => RegisterValue::Uint64(self.user_gp.r14 as u32 as u64),
            Register::R15D => RegisterValue::Uint64(self.user_gp.r15 as u32 as u64),

            // 16-bit registers
            Register::AX => RegisterValue::Uint64(self.user_gp.rax as u16 as u64),
            Register::DX => RegisterValue::Uint64(self.user_gp.rdx as u16 as u64),
            Register::CX => RegisterValue::Uint64(self.user_gp.rcx as u16 as u64),
            Register::SI => RegisterValue::Uint64(self.user_gp.rsi as u16 as u64),
            Register::DI => RegisterValue::Uint64(self.user_gp.rdi as u16 as u64),
            Register::BP => RegisterValue::Uint64(self.user_gp.rbp as u16 as u64),
            Register::SP => RegisterValue::Uint64(self.user_gp.rsp as u16 as u64),
            Register::R8W => RegisterValue::Uint64(self.user_gp.r8 as u16 as u64),
            Register::R9W => RegisterValue::Uint64(self.user_gp.r9 as u16 as u64),
            Register::R10W => RegisterValue::Uint64(self.user_gp.r10 as u16 as u64),
            Register::R11W => RegisterValue::Uint64(self.user_gp.r11 as u16 as u64),
            Register::R12W => RegisterValue::Uint64(self.user_gp.r12 as u16 as u64),
            Register::R13W => RegisterValue::Uint64(self.user_gp.r13 as u16 as u64),
            Register::R14W => RegisterValue::Uint64(self.user_gp.r14 as u16 as u64),
            Register::R15W => RegisterValue::Uint64(self.user_gp.r15 as u16 as u64),

            // 8-bit high
            Register::AH => RegisterValue::Uint64((self.user_gp.rax >> 8) & 0xff),
            Register::DH => RegisterValue::Uint64((self.user_gp.rdx >> 8) & 0xff),
            Register::CH => RegisterValue::Uint64((self.user_gp.rcx >> 8) & 0xff),
            Register::BH => RegisterValue::Uint64((self.user_gp.rbx >> 8) & 0xff),

            // 8-bit low
            Register::AL => RegisterValue::Uint64(self.user_gp.rax & 0xff),
            Register::DL => RegisterValue::Uint64(self.user_gp.rdx & 0xff),
            Register::CL => RegisterValue::Uint64(self.user_gp.rcx & 0xff),
            Register::BL => RegisterValue::Uint64(self.user_gp.rbx & 0xff),
            Register::SIL => RegisterValue::Uint64(self.user_gp.rsi & 0xff),
            Register::DIL => RegisterValue::Uint64(self.user_gp.rdi & 0xff),
            Register::BPL => RegisterValue::Uint64(self.user_gp.rbp & 0xff),
            Register::SPL => RegisterValue::Uint64(self.user_gp.rsp & 0xff),
            Register::R8B => RegisterValue::Uint64(self.user_gp.r8 & 0xff),
            Register::R9B => RegisterValue::Uint64(self.user_gp.r9 & 0xff),
            Register::R10B => RegisterValue::Uint64(self.user_gp.r10 & 0xff),
            Register::R11B => RegisterValue::Uint64(self.user_gp.r11 & 0xff),
            Register::R12B => RegisterValue::Uint64(self.user_gp.r12 & 0xff),
            Register::R13B => RegisterValue::Uint64(self.user_gp.r13 & 0xff),
            Register::R14B => RegisterValue::Uint64(self.user_gp.r14 & 0xff),
            Register::R15B => RegisterValue::Uint64(self.user_gp.r15 & 0xff),

            // Floating point control/status
            Register::FCW => RegisterValue::Uint64(self.user_fp.cwd as u64),
            Register::FSW => RegisterValue::Uint64(self.user_fp.swd as u64),
            Register::FTW => RegisterValue::Uint64(self.user_fp.ftw as u64),
            Register::FOP => RegisterValue::Uint64(self.user_fp.fop as u64),
            Register::FIP => RegisterValue::Uint64(self.user_fp.rip),
            Register::FDP => RegisterValue::Uint64(self.user_fp.rdp),
            Register::MXCSR => RegisterValue::Uint64(self.user_fp.mxcsr as u64),
            Register::MXCSR_MASK => RegisterValue::Uint64(self.user_fp.mxcr_mask as u64),

            // x87 stack (80-bit, stored in 16 bytes)
            Register::ST0 => take_long_double(&self.user_fp.st_space, 0),
            Register::ST1 => take_long_double(&self.user_fp.st_space, 1),
            Register::ST2 => take_long_double(&self.user_fp.st_space, 2),
            Register::ST3 => take_long_double(&self.user_fp.st_space, 3),
            Register::ST4 => take_long_double(&self.user_fp.st_space, 4),
            Register::ST5 => take_long_double(&self.user_fp.st_space, 5),
            Register::ST6 => take_long_double(&self.user_fp.st_space, 6),
            Register::ST7 => take_long_double(&self.user_fp.st_space, 7),

            // XMM registers
            Register::XMM0 => take_xmm(&self.user_fp.xmm_space, 0),
            Register::XMM1 => take_xmm(&self.user_fp.xmm_space, 1),
            Register::XMM2 => take_xmm(&self.user_fp.xmm_space, 2),
            Register::XMM3 => take_xmm(&self.user_fp.xmm_space, 3),
            Register::XMM4 => take_xmm(&self.user_fp.xmm_space, 4),
            Register::XMM5 => take_xmm(&self.user_fp.xmm_space, 5),
            Register::XMM6 => take_xmm(&self.user_fp.xmm_space, 6),
            Register::XMM7 => take_xmm(&self.user_fp.xmm_space, 7),
            Register::XMM8 => take_xmm(&self.user_fp.xmm_space, 8),
            Register::XMM9 => take_xmm(&self.user_fp.xmm_space, 9),
            Register::XMM10 => take_xmm(&self.user_fp.xmm_space, 10),
            Register::XMM11 => take_xmm(&self.user_fp.xmm_space, 11),
            Register::XMM12 => take_xmm(&self.user_fp.xmm_space, 12),
            Register::XMM13 => take_xmm(&self.user_fp.xmm_space, 13),
            Register::XMM14 => take_xmm(&self.user_fp.xmm_space, 14),
            Register::XMM15 => take_xmm(&self.user_fp.xmm_space, 15),

            // Debug registers
            Register::DR0 => RegisterValue::Uint64(self.debug_regs[0]),
            Register::DR1 => RegisterValue::Uint64(self.debug_regs[1]),
            Register::DR2 => RegisterValue::Uint64(self.debug_regs[2]),
            Register::DR3 => RegisterValue::Uint64(self.debug_regs[3]),
            Register::DR4 => RegisterValue::Uint64(self.debug_regs[4]),
            Register::DR5 => RegisterValue::Uint64(self.debug_regs[5]),
            Register::DR6 => RegisterValue::Uint64(self.debug_regs[6]),
            Register::DR7 => RegisterValue::Uint64(self.debug_regs[7]),
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
