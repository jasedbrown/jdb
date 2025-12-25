#![allow(dead_code)]
use anyhow::Result;
use libc::{user, user_fpregs_struct, user_regs_struct};
use memoffset::offset_of;
use nix::sys::ptrace::{getregset, read_user, regset, setregset, write_user};
use nix::unistd::Pid;

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::process::register_info::{
    Location, Register, RegisterFormat, RegisterInfo, RegisterType, RegisterValue, UserField,
    registers_info,
};

static REGISTERS_MAP: LazyLock<HashMap<Register, RegisterInfo>> = LazyLock::new(|| {
    let mut regs = HashMap::new();

    registers_info().iter().for_each(|r| {
        regs.insert(r.register, r.clone());
    });

    regs
});

fn expect_register_info(register: &Register) -> &RegisterInfo {
    REGISTERS_MAP
        .get(register)
        .unwrap_or_else(|| panic!("unknown register: {register:?}"))
}

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

    pub fn read(&self, register: &Register) -> RegisterValue {
        let info = expect_register_info(register);
        match info.loc {
            // Offsets are computed relative to the `user` struct; we store only the
            // component struct, so adjust by the field offset.
            Location::Regs(_) => {
                let start = info.offset - offset_of!(user, regs);
                value_from_bytes(struct_as_bytes(&self.user_gp), start, info)
            }
            Location::Fpu(_) | Location::FpuArray(_, _) => {
                let start = info.offset - offset_of!(user, i387);
                value_from_bytes(struct_as_bytes(&self.user_fp), start, info)
            }
            Location::UserArray(UserField::UDebugReg, idx) => {
                // Debug registers are stored separately; use the cached array.
                let start = idx * info.size;
                value_from_bytes(slice_as_bytes(&self.debug_regs), start, info)
            }
        }
    }

    pub fn write(&mut self, register: Register, value: RegisterValue) -> Result<()> {
        // TODO: there's a lot of incomplete work here ...
        // including a problem of writing a reg value less than u64 :shrug:
        // will fix when I hit it ... just really need to move forward for now

        let info = expect_register_info(&register);

        // apparently PTRACE_POKEUSER does not work on the x87 area on x86
        // (according to the Sy Brand book), so write all the x87 registers at once.
        if matches!(info.register_type, RegisterType::FloatingPoint) {
            let fpregs = self.user_fp;
            // TODO: actually set the value into the struct
            setregset::<regset::NT_PRFPREG>(self.pid, fpregs)?;
        } else {
            // clears out the bottom 3 bits (! is bitwise NOT), effectively round
            // down to a multiple of 8.
            let aligned_offset = info.offset & !0b111;
            // TODO: i have no idea if this is correct?
            write_user(self.pid, aligned_offset as _, value.try_into()?)?;
        }

        Ok(())
    }
}

fn struct_as_bytes<T>(value: &T) -> &[u8] {
    let len = std::mem::size_of::<T>();
    // SAFETY: Only reinterpreting the provided reference as bytes.
    unsafe { std::slice::from_raw_parts((value as *const T).cast::<u8>(), len) }
}

fn slice_as_bytes<T>(slice: &[T]) -> &[u8] {
    let len = std::mem::size_of_val(slice);
    // SAFETY: `T` is plain data; we only read the byte view.
    unsafe { std::slice::from_raw_parts(slice.as_ptr().cast::<u8>(), len) }
}

fn value_from_bytes(bytes: &[u8], start: usize, info: &RegisterInfo) -> RegisterValue {
    let end = start + info.size;
    let slice = &bytes[start..end];

    match info.format {
        RegisterFormat::Uint8 => RegisterValue::Uint8(slice[0]),
        RegisterFormat::Uint16 => {
            let mut buf = [0u8; 2];
            buf.copy_from_slice(slice);
            RegisterValue::Uint16(u16::from_le_bytes(buf))
        }
        RegisterFormat::Uint32 => {
            let mut buf = [0u8; 4];
            buf.copy_from_slice(slice);
            RegisterValue::Uint32(u32::from_le_bytes(buf))
        }
        RegisterFormat::Uint64 => {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(slice);
            RegisterValue::Uint64(u64::from_le_bytes(buf))
        }
        RegisterFormat::Int8 => RegisterValue::Int8(slice[0] as i8),
        RegisterFormat::Int16 => {
            let mut buf = [0u8; 2];
            buf.copy_from_slice(slice);
            RegisterValue::Int16(i16::from_le_bytes(buf))
        }
        RegisterFormat::Int32 => {
            let mut buf = [0u8; 4];
            buf.copy_from_slice(slice);
            RegisterValue::Int32(i32::from_le_bytes(buf))
        }
        RegisterFormat::Int64 => {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(slice);
            RegisterValue::Int64(i64::from_le_bytes(buf))
        }
        RegisterFormat::Float => {
            let mut buf = [0u8; 4];
            buf.copy_from_slice(slice);
            RegisterValue::Float(f32::from_le_bytes(buf))
        }
        RegisterFormat::Double => {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(slice);
            RegisterValue::Double(f64::from_le_bytes(buf))
        }
        RegisterFormat::LongDouble => {
            let mut buf = [0u8; 10];
            buf.copy_from_slice(&slice[..10]);
            RegisterValue::LongDouble(buf)
        }
        RegisterFormat::Byte64 => {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&slice[..8]);
            RegisterValue::Byte64(buf)
        }
        RegisterFormat::Byte128 => {
            let mut buf = [0u8; 16];
            buf.copy_from_slice(&slice[..16]);
            RegisterValue::Byte128(buf)
        }
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
