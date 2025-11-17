//! Module to generate a mapping of supported debug registers
//! to information about each field in the respective c structs.
//!
//! Aprt of what makes this module weird is the intersection of
//! c structs, `libc` crate, `memoffset` crate, the advice from
//! "Building a Debugger", and the rust macro system.
// use anyhow::Result;
// use libc::{user, user_regs_struct, user_fpregs_struct};
// use nix::sys::ptrace::{getregset, read_user, regset};

use strum::EnumDiscriminants;

#[derive(Clone, Copy, Debug, EnumDiscriminants)]
#[allow(dead_code)]
pub enum RegisterValue {
    // placeholder variants ....
    Uint(u32),
    DoubleFloat(f32),
    LongDouble(f64),
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
enum RegisterType {
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
            // (EnumVariant, struct_field, dwarf_regno)
            (R15, r15, 15);
            (R14, r14, 14);
            (R13, r13, 13);
        }
    };
}

/// Definte the standalone enum names.
/// Central enum of supported registers.
///
/// currently only supporting x86_64 and what i find in /usr/include/sys/user.h
/// as i think that's all that's exposed in `nix::ptrace`
macro_rules! DEFINE_ENUM {
    ( $( ($register:ident, $field:ident, $dwarf:expr); )* ) => {
        #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
        pub enum Register {
            $( $register, )*
        }
    };
}

// pub struct RegisterInfo {
//     register: Register,
//     name: String,
//     dwarf_id: i32,
//     size: usize, // ???
//     /// Offset into the `user` struct
//     offset: usize,
//     register_type: RegisterType,
//     format: RegisterValueDiscriminants,
// }

macro_rules! DEFINE_INFO {
    ( $( ($register:ident, $field:ident, $dwarf:expr); )* ) => {
        #[derive(Clone, Debug, Hash)]
        pub struct RegisterInfo {
            pub register: Register,
            pub name: &'static str,
            pub dwarf: u16,
            pub offset: usize,
        }

        #[allow(dead_code)]
        pub const REGISTERS_INFO: &[RegisterInfo] = &[
            $(
                RegisterInfo {
                    register: Register::$register,
                    name: stringify!($field),
                    dwarf: $dwarf,
                    offset: memoffset::offset_of!(libc::user_regs_struct, $field),
                },
            )*
        ];
    };
}

REGISTER_LIST!(DEFINE_ENUM);
REGISTER_LIST!(DEFINE_INFO);
