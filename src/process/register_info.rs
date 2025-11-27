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
#[strum_discriminants(name(RegisterValueType))]
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
    SubGeneralPupose,
    FloatingPoint,
    Debug,
}

/// This is your single source of truth.
/// You edit *only* this list when you add/remove registers.
macro_rules! REGISTER_LIST {
    ($macro:ident) => {
        $macro! {
            // (EnumVariant, struct_field, dwarf_regno, reg_type, bit-width)
            // 64-bit registers
            (RAX, rax, 0, GeneralPurpose, 64);
            (RDX, rdx, 1, GeneralPurpose, 64);
            (RCX, rcx, 2, GeneralPurpose, 64);
            (RSI, rsi, 4, GeneralPurpose, 64);
            (RDI, rdi, 5, GeneralPurpose, 64);
            (RBP, rbp, 6, GeneralPurpose, 64);
            (RSP, rsp, 7, GeneralPurpose, 64);
            (R8, r8, 8, GeneralPurpose, 64);
            (R9, r9, 9, GeneralPurpose, 64);
            (R10, r10, 10, GeneralPurpose, 64);
            (R11, r11, 11, GeneralPurpose, 64);
            (R12, r12, 12, GeneralPurpose, 64);
            (R13, r13, 13, GeneralPurpose, 64);
            (R14, r14, 14, GeneralPurpose, 64);
            (R15, r15, 15, GeneralPurpose, 64);
            (RIP, rip, 16, GeneralPurpose, 64);
            (EFLAGS, eflags, 49, GeneralPurpose, 64);
            (CS, cs, 51, GeneralPurpose, 64);
            (FS, fs, 54, GeneralPurpose, 64);            
            (GS, gs, 55, GeneralPurpose, 64);
            (SS, ss, 52, GeneralPurpose, 64);            
            (DS, ds, 53, GeneralPurpose, 64);
            (ES, es, 50, GeneralPurpose, 64);

            // ptrace exposes this as the way to get the ID of a syscall.
            // it has no dwarf id.
            (ORIG_RAX, orig_rax, -1, GeneralPurpose, 64);

            // 32-bit subregisters. no dwarf IDs
            (EAX, eax, -1, SubGeneralPupose, 32);
        }
    };
}

/// Definte the standalone enum names.
/// Central enum of supported registers.
///
/// currently only supporting x86_64 and what i find in /usr/include/sys/user.h
/// as i think that's all that's exposed in `nix::ptrace`
macro_rules! DEFINE_ENUM {
    ( $( ($register:ident, $field:ident, $dwarf:expr, $reg_type:ident, $size:expr); )* ) => {
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
//     size: usize,                          // ??? (64 bit, 32, 16, 8)
//     /// Offset into the `user` struct
//     offset: usize,                        // fix me
//     format: RegisterValueDiscriminants,  /// ????
// }

macro_rules! DEFINE_INFO {
    ( $( ($register:ident, $field:ident, $dwarf:expr, $reg_type:ident, $size:expr); )* ) => {
        #[derive(Clone, Debug, Hash)]
        pub struct RegisterInfo {
            pub register: Register,
            pub name: &'static str,
            pub dwarf: u16,
            pub offset: usize,
            pub size: usize,
            pub register_type: RegisterType,
        }

        pub const REGISTERS_INFO: &[RegisterInfo] = &[
            $(
                RegisterInfo {
                    register: Register::$register,
                    name: stringify!($field),
                    dwarf: $dwarf,
                    offset: memoffset::offset_of!(libc::user_regs_struct, $field),
                    size: $size / 8,
                    register_type: RegisterType::$reg_type,
                },
            )*
        ];
    };
}

REGISTER_LIST!(DEFINE_ENUM);
REGISTER_LIST!(DEFINE_INFO);
