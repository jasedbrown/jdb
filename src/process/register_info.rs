#![allow(dead_code)]
//! Module to generate a mapping of supported debug registers
//! to information about each field in the respective c structs.
//!
//! Part of what makes this module weird is the intersection of
//! c structs, `libc` crate, `memoffset` crate, the advice from
//! "Building a Debugger", and the rust macro system.
// use libc::{user, user_regs_struct, user_fpregs_struct};

use strum::EnumDiscriminants;

#[derive(Clone, Copy, Debug, EnumDiscriminants)]
#[strum_discriminants(name(RegisterFormat))]
pub enum RegisterValue {
    // placeholder variants ....
    Uint(u64),
    // DoubleFloat(f32),
    // LongDouble(f64),
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum RegisterType {
    /// 64-bit instructions
    GeneralPurpose,
    SubGeneralPurpose,
    FloatingPoint,
    Debug,
}

enum RegisterWidth {
    // note: variants are prefixed with 'W' as rust won't allow a digit as the first char.
    W64,
    W32,
    W16,
    W8H,
    W8L,
}

impl RegisterWidth {
    /// Register width in bits.
    const fn bits(&self) -> usize {
        match self {
            RegisterWidth::W64 => 64,
            RegisterWidth::W32 => 32,
            RegisterWidth::W16 => 16,
            RegisterWidth::W8H | RegisterWidth::W8L => 8,
        }
    }

    /// Register width in bytes.
    const fn bytes(&self) -> usize {
        self.bits() / 8
    }

    /// Offset from the start of the underlying storage, used for subregisters.
    const fn offset(&self) -> usize {
        match self {
            RegisterWidth::W8H => 1,
            RegisterWidth::W8L | RegisterWidth::W16 | RegisterWidth::W32 | RegisterWidth::W64 => 0,
        }
    }
}

/// This is your single source of truth.
/// You edit *only* this list when you add/remove registers.
macro_rules! REGISTER_LIST {
    ($macro:ident) => {
        $macro! {
            // (EnumVariant, struct_field, dwarf_regno, reg_type, bit-width)
            // 64-bit registers
            (RAX, Regs(rax), 0, GeneralPurpose, W64);
            (RDX, Regs(rdx), 1, GeneralPurpose, W64);
            (RCX, Regs(rcx), 2, GeneralPurpose, W64);
            (RBX, Regs(rbx), 3, GeneralPurpose, W64);
            (RSI, Regs(rsi), 4, GeneralPurpose, W64);
            (RDI, Regs(rdi), 5, GeneralPurpose, W64);
            (RBP, Regs(rbp), 6, GeneralPurpose, W64);
            (RSP, Regs(rsp), 7, GeneralPurpose, W64);
            (R8, Regs(r8), 8, GeneralPurpose, W64);
            (R9, Regs(r9), 9, GeneralPurpose, W64);
            (R10, Regs(r10), 10, GeneralPurpose, W64);
            (R11, Regs(r11), 11, GeneralPurpose, W64);
            (R12, Regs(r12), 12, GeneralPurpose, W64);
            (R13, Regs(r13), 13, GeneralPurpose, W64);
            (R14, Regs(r14), 14, GeneralPurpose, W64);
            (R15, Regs(r15), 15, GeneralPurpose, W64);
            (RIP, Regs(rip), 16, GeneralPurpose, W64);
            (EFLAGS, Regs(eflags), 49, GeneralPurpose, W64);
            (CS, Regs(cs), 51, GeneralPurpose, W64);
            (FS, Regs(fs), 54, GeneralPurpose, W64);
            (GS, Regs(gs), 55, GeneralPurpose, W64);
            (SS, Regs(ss), 52, GeneralPurpose, W64);
            (DS, Regs(ds), 53, GeneralPurpose, W64);
            (ES, Regs(es), 50, GeneralPurpose, W64);

            // ptrace exposes this as the way to get the ID of a syscall.
            // it has no dwarf id.
            (ORIGRAX, Regs(orig_rax), -1, GeneralPurpose, W64);

            // 32-bit subregisters. no dwarf IDs
            (EAX, Regs(rax), -1, SubGeneralPurpose, W32);
            (EDX, Regs(rdx), -1, SubGeneralPurpose, W32);
            (ECX, Regs(rcx), -1, SubGeneralPurpose, W32);
            (EBX, Regs(rbx), -1, SubGeneralPurpose, W32);
            (ESI, Regs(rsi), -1, SubGeneralPurpose, W32);
            (EDI, Regs(rdi), -1, SubGeneralPurpose, W32);
            (EBP, Regs(rbp), -1, SubGeneralPurpose, W32);
            (ESP, Regs(rsp), -1, SubGeneralPurpose, W32);
            (R8D, Regs(r8), -1, SubGeneralPurpose, W32);
            (R9D, Regs(r9), -1, SubGeneralPurpose, W32);
            (R10D, Regs(r10), -1, SubGeneralPurpose, W32);
            (R11D, Regs(r11), -1, SubGeneralPurpose, W32);
            (R12D, Regs(r12), -1, SubGeneralPurpose, W32);
            (R13D, Regs(r13), -1, SubGeneralPurpose, W32);
            (R14D, Regs(r14), -1, SubGeneralPurpose, W32);
            (R15D, Regs(r15), -1, SubGeneralPurpose, W32);

            // 16-bit subregisters. no dwarf IDs
            (AX, Regs(rax), -1, SubGeneralPurpose, W16);
            (DX, Regs(rdx), -1, SubGeneralPurpose, W16);
            (CX, Regs(rcx), -1, SubGeneralPurpose, W16);
            (SI, Regs(rsi), -1, SubGeneralPurpose, W16);
            (DI, Regs(rdi), -1, SubGeneralPurpose, W16);
            (BP, Regs(rbp), -1, SubGeneralPurpose, W16);
            (SP, Regs(rsp), -1, SubGeneralPurpose, W16);
            (R8W, Regs(r8), -1, SubGeneralPurpose, W16);
            (R9W, Regs(r9), -1, SubGeneralPurpose, W16);
            (R10W, Regs(r10), -1, SubGeneralPurpose, W16);
            (R11W, Regs(r11), -1, SubGeneralPurpose, W16);
            (R12W, Regs(r12), -1, SubGeneralPurpose, W16);
            (R13W, Regs(r13), -1, SubGeneralPurpose, W16);
            (R14W, Regs(r14), -1, SubGeneralPurpose, W16);
            (R15W, Regs(r15), -1, SubGeneralPurpose, W16);

            // 8-bit high subregisters. no dwarf IDs
            (AH, Regs(rax), -1, SubGeneralPurpose, W8H);
            (DH, Regs(rdx), -1, SubGeneralPurpose, W8H);
            (CH, Regs(rcx), -1, SubGeneralPurpose, W8H);
            (BH, Regs(rbx), -1, SubGeneralPurpose, W8H);

            // 8-bit low subregisters. no dwarf IDs
            (AL, Regs(rax), -1, SubGeneralPurpose, W8L);
            (DL, Regs(rdx), -1, SubGeneralPurpose, W8L);
            (CL, Regs(rcx), -1, SubGeneralPurpose, W8L);
            (BL, Regs(rbx), -1, SubGeneralPurpose, W8L);
            (SIL, Regs(rsi), -1, SubGeneralPurpose, W8L);
            (DIL, Regs(rdi), -1, SubGeneralPurpose, W8L);
            (BPL, Regs(rbp), -1, SubGeneralPurpose, W8L);
            (SPL, Regs(rsp), -1, SubGeneralPurpose, W8L);
            (R8B, Regs(r8), -1, SubGeneralPurpose, W8L);
            (R9B, Regs(r9), -1, SubGeneralPurpose, W8L);
            (R10B, Regs(r10), -1, SubGeneralPurpose, W8L);
            (R11B, Regs(r11), -1, SubGeneralPurpose, W8L);
            (R12B, Regs(r12), -1, SubGeneralPurpose, W8L);
            (R13B, Regs(r13), -1, SubGeneralPurpose, W8L);
            (R14B, Regs(r14), -1, SubGeneralPurpose, W8L);
            (R15B, Regs(r15), -1, SubGeneralPurpose, W8L);

            // define the floating-point registers


            // define the debug registers - overload the "struct field"
            // with position in the debug reg array
            (DR0, UserArray(u_debugreg, 0), -1, Debug, W64);
            (DR1, UserArray(u_debugreg, 1), -1, Debug, W64);
            (DR2, UserArray(u_debugreg, 2), -1, Debug, W64);
            (DR3, UserArray(u_debugreg, 3), -1, Debug, W64);
            (DR4, UserArray(u_debugreg, 4), -1, Debug, W64);
            (DR5, UserArray(u_debugreg, 5), -1, Debug, W64);
            (DR6, UserArray(u_debugreg, 6), -1, Debug, W64);
            (DR7, UserArray(u_debugreg, 7), -1, Debug, W64);
        }
    };
}

/// Definte the standalone enum names.
/// Central enum of supported registers.
///
/// currently only supporting x86_64 and what i find in /usr/include/sys/user.h
/// as i think that's all that's exposed in `nix::ptrace`
macro_rules! DEFINE_ENUM {
    ( $( ($register:ident, $location_kind:ident($($location_args:tt)*), $dwarf:expr, $reg_type:ident, $width:expr); )* ) => {
        #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
        #[allow(clippy::upper_case_acronyms)]
        pub enum Register {
            $( $register, )*
        }
    };
}

#[derive(Clone, Debug)]
pub struct RegisterInfo {
    pub register: Register,
    /// The actual name of the register, as appears in the `user` family of structs.
    // TODO: can this be a field/const function on `Register`?
    pub name: &'static str,
    pub dwarf_id: i32,
    /// The byte offset into the `user` struct of this register.
    /// Primarily used for `read_user()` and `write_user()`.
    pub offset: usize,
    /// Size in bytes of the register's value.
    pub size: usize,
    pub register_type: RegisterType,
    pub format: RegisterFormat,
}

macro_rules! DEFINE_INFO {
    ( $( ($register:ident, $location_kind:ident($($location_args:tt)*), $dwarf:expr, $reg_type:ident, $width:ident); )* ) => {
        pub const REGISTERS_INFO: &[RegisterInfo] = &[
            $(
                RegisterInfo {
                    register: Register::$register,
                    name: DEFINE_INFO!(@name $location_kind($($location_args)*)),
                    dwarf_id: $dwarf,
                    offset: DEFINE_INFO!(@field_offset $location_kind($($location_args)*), $width),
                    size: RegisterWidth::$width.bytes(),
                    register_type: RegisterType::$reg_type,
                    format: RegisterFormat::Uint,
                },
            )*
        ];
    };

    (@name Regs($field:ident)) => {
        stringify!($field)
    };
    (@name RegsArray($field:ident, $index:expr)) => {
        concat!(stringify!($field), "[", stringify!($index), "]")
    };
    (@name Fpu($field:ident)) => {
        stringify!($field)
    };
    (@name FpuArray($field:ident, $index:expr)) => {
        concat!(stringify!($field), "[", stringify!($index), "]")
    };
    (@name User($field:ident)) => {
        stringify!($field)
    };
    (@name UserArray($field:ident, $index:expr)) => {
        concat!(stringify!($field), "[", stringify!($index), "]")
    };

    // Helper rules to select the correct struct type based on Register storage
    (@field_offset Regs($field:ident), $width:ident) => {
        memoffset::offset_of!(libc::user, regs) + memoffset::offset_of!(libc::user_regs_struct, $field)
    };
    (@field_offset RegsArray($field:ident, $index:expr), $width:ident) => {
        memoffset::offset_of!(libc::user, regs)
            + memoffset::offset_of!(libc::user_regs_struct, $field)
            + ($index * RegisterWidth::$width.bytes())
    };
    (@field_offset Fpu($field:ident), $width:ident) => {
        memoffset::offset_of!(libc::user, i387) + memoffset::offset_of!(libc::user_fpregs_struct, $field)
    };
    (@field_offset FpuArray($field:ident, $index:expr), $width:ident) => {
        memoffset::offset_of!(libc::user, i387)
            + memoffset::offset_of!(libc::user_fpregs_struct, $field)
            + ($index * RegisterWidth::$width.bytes())
    };
    (@field_offset User($field:ident), $width:ident) => {
        memoffset::offset_of!(libc::user, $field)
    };
    (@field_offset UserArray($field:ident, $index:expr), $width:ident) => {
        memoffset::offset_of!(libc::user, $field) + ($index * RegisterWidth::$width.bytes())
    };
}

REGISTER_LIST!(DEFINE_ENUM);
REGISTER_LIST!(DEFINE_INFO);
