#![allow(dead_code)]
//! Module to generate a mapping of supported debug registers
//! to information about each field in the respective c structs.
//!
//! Part of what makes this module weird is the intersection of
//! c structs, `libc` crate, `memoffset` crate, the advice from
//! "Building a Debugger", and the rust macro system.
// use anyhow::Result;
// use libc::{user, user_regs_struct, user_fpregs_struct};

use strum::EnumDiscriminants;

#[derive(Clone, Copy, Debug, EnumDiscriminants, Hash)]
#[strum_discriminants(name(RegisterFormat))]
pub enum RegisterValue {
    // placeholder variants ....
    Uint(u32),
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
    /// Register width in bytes.
    const fn width(&self) -> usize {
        match self {
            RegisterWidth::W64 => 64,
            RegisterWidth::W32 => 32,
            RegisterWidth::W16 => 16,
            RegisterWidth::W8H | RegisterWidth::W8L => 8,
        }
    }

    /// Register offset from the beginning of the register.
    /// Really only applicaable to the 8-bit low registers
    const fn offset(&self) -> usize {
        match self {
            RegisterWidth::W8L => 1,
            RegisterWidth::W8H | RegisterWidth::W16 | RegisterWidth::W32 | RegisterWidth::W64 => 0,
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
            (RAX, rax, 0, GeneralPurpose, W64);
            (RDX, rdx, 1, GeneralPurpose, W64);
            (RCX, rcx, 2, GeneralPurpose, W64);
            (RSI, rsi, 4, GeneralPurpose, W64);
            (RDI, rdi, 5, GeneralPurpose, W64);
            (RBP, rbp, 6, GeneralPurpose, W64);
            (RSP, rsp, 7, GeneralPurpose, W64);
            (R8, r8, 8, GeneralPurpose, W64);
            (R9, r9, 9, GeneralPurpose, W64);
            (R10, r10, 10, GeneralPurpose, W64);
            (R11, r11, 11, GeneralPurpose, W64);
            (R12, r12, 12, GeneralPurpose, W64);
            (R13, r13, 13, GeneralPurpose, W64);
            (R14, r14, 14, GeneralPurpose, W64);
            (R15, r15, 15, GeneralPurpose, W64);
            (RIP, rip, 16, GeneralPurpose, W64);
            (EFLAGS, eflags, 49, GeneralPurpose, W64);
            (CS, cs, 51, GeneralPurpose, W64);
            (FS, fs, 54, GeneralPurpose, W64);
            (GS, gs, 55, GeneralPurpose, W64);
            (SS, ss, 52, GeneralPurpose, W64);
            (DS, ds, 53, GeneralPurpose, W64);
            (ES, es, 50, GeneralPurpose, W64);

            // ptrace exposes this as the way to get the ID of a syscall.
            // it has no dwarf id.
            (ORIGRAX, orig_rax, -1, GeneralPurpose, W64);

            // 32-bit subregisters. no dwarf IDs
            (EAX, rax, -1, SubGeneralPurpose, W32);
            (EDX, rdx, -1, SubGeneralPurpose, W32);
            (ECX, rcx, -1, SubGeneralPurpose, W32);
            (EBX, rbx, -1, SubGeneralPurpose, W32);
            (ESI, rsi, -1, SubGeneralPurpose, W32);
            (EDI, rdi, -1, SubGeneralPurpose, W32);
            (EBP, rbp, -1, SubGeneralPurpose, W32);
            (ESP, rsp, -1, SubGeneralPurpose, W32);
            (R8D, r8, -1, SubGeneralPurpose, W32);
            (R9D, r9, -1, SubGeneralPurpose, W32);
            (R10D, r10, -1, SubGeneralPurpose, W32);
            (R11D, r11, -1, SubGeneralPurpose, W32);
            (R12D, r12, -1, SubGeneralPurpose, W32);
            (R13D, r13, -1, SubGeneralPurpose, W32);
            (R14D, r14, -1, SubGeneralPurpose, W32);
            (R15D, r15, -1, SubGeneralPurpose, W32);

            // 16-bit subregisters. no dwarf IDs
            (AX, rax, -1, SubGeneralPurpose, W16);
            (DX, rdx, -1, SubGeneralPurpose, W16);
            (CX, rcx, -1, SubGeneralPurpose, W16);
            (SI, rsi, -1, SubGeneralPurpose, W16);
            (DI, rdi, -1, SubGeneralPurpose, W16);
            (BP, rbp, -1, SubGeneralPurpose, W16);
            (SP, rsp, -1, SubGeneralPurpose, W16);
            (R8W, r8, -1, SubGeneralPurpose, W16);
            (R9W, r9, -1, SubGeneralPurpose, W16);
            (R10W, r10, -1, SubGeneralPurpose, W16);
            (R11W, r11, -1, SubGeneralPurpose, W16);
            (R12W, r12, -1, SubGeneralPurpose, W16);
            (R13W, r13, -1, SubGeneralPurpose, W16);
            (R14W, r14, -1, SubGeneralPurpose, W16);
            (R15W, r15, -1, SubGeneralPurpose, W16);

            // 8-bit high subregisters. no dwarf IDs
            (AH, rax, -1, SubGeneralPurpose, W8H);
            (DH, rdx, -1, SubGeneralPurpose, W8H);
            (CH, rcx, -1, SubGeneralPurpose, W8H);
            (BH, rbx, -1, SubGeneralPurpose, W8H);

            // 8-bit low subregisters. no dwarf IDs
            (AL, rax, -1, SubGeneralPurpose, W8L);
            (DL, rdx, -1, SubGeneralPurpose, W8L);
            (CL, rcx, -1, SubGeneralPurpose, W8L);
            (BL, rbx, -1, SubGeneralPurpose, W8L);
            (SIL, rsi, -1, SubGeneralPurpose, W8L);
            (DIL, rdi, -1, SubGeneralPurpose, W8L);
            (BPL, rbp, -1, SubGeneralPurpose, W8L);
            (SPL, rsp, -1, SubGeneralPurpose, W8L);
            (R8B, r8, -1, SubGeneralPurpose, W8L);
            (R9B, r9, -1, SubGeneralPurpose, W8L);
            (R10B, r10, -1, SubGeneralPurpose, W8L);
            (R11B, r11, -1, SubGeneralPurpose, W8L);
            (R12B, r12, -1, SubGeneralPurpose, W8L);
            (R13B, r13, -1, SubGeneralPurpose, W8L);
            (R14B, r14, -1, SubGeneralPurpose, W8L);
            (R15B, r15, -1, SubGeneralPurpose, W8L);
        }
    };
}

/// Definte the standalone enum names.
/// Central enum of supported registers.
///
/// currently only supporting x86_64 and what i find in /usr/include/sys/user.h
/// as i think that's all that's exposed in `nix::ptrace`
macro_rules! DEFINE_ENUM {
    ( $( ($register:ident, $field:ident, $dwarf:expr, $reg_type:ident, $width:expr); )* ) => {
        #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
        #[allow(clippy::upper_case_acronyms)]
        pub enum Register {
            $( $register, )*
        }
    };
}

// pub struct RegisterInfo {
//     register: Register,
//     name: String,
//     dwarf_id: i32,
//     register_type: RegisterType,
//     size: usize
//     /// Offset into the `user` struct
//     offset: usize,
//     format: RegisterValueDiscriminants,  /// ????
// }

macro_rules! DEFINE_INFO {
    ( $( ($register:ident, $field:ident, $dwarf:expr, $reg_type:ident, $width:ident); )* ) => {
        #[derive(Clone, Debug, Hash)]
        pub struct RegisterInfo {
            pub register: Register,
            pub name: &'static str,
            pub dwarf_id: i32,
            pub offset: usize,
            pub size: usize,
            pub register_type: RegisterType,
        }

        pub const REGISTERS_INFO: &[RegisterInfo] = &[
            $(
                RegisterInfo {
                    register: Register::$register,
                    name: stringify!($field),
                    dwarf_id: $dwarf,
                    offset: memoffset::offset_of!(libc::user_regs_struct, $field) + RegisterWidth::$width.offset(),
                    size: RegisterWidth::$width.width(),
                    register_type: RegisterType::$reg_type,
                },
            )*
        ];
    };
}

REGISTER_LIST!(DEFINE_ENUM);
REGISTER_LIST!(DEFINE_INFO);
