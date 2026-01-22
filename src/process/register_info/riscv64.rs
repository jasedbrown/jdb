//! Centralized declaration of all supported CPU registers for riscv64.

use crate::process::register_info::{RegisterFormat, RegisterInfo, RegisterType, RegisterWidth};

/// Registers for risc-v 64.
///
/// The calling convention follows standard save/restore semantics:
/// caller saves temporaries (t-registers), callee saves s-registers.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum Register {
    // riscv64 has 32 general purpose 64-bit registers
    X0,  // (zero), hardwired to 0
    X1,  // (RA) return address
    X2,  // (SP) stack pointer
    X3,  // (GP) global pointer
    X4,  // (TP) thread pointer
    X5,  // (T0) temporary (caller-saved)
    X6,  // (T2) temporary (caller-saved)
    X7,  // (T1) temporary (caller-saved)
    X8,  // (SO/FP) saved register/frame po[inter]
    X9,  // (S1) saved register
    X10, // (AO) function arguments / return values
    X11, // (A1) function arguments / return values
    X12, // (A2) function arguments
    X13, // (A3) function arguments
    X14, // (A4) function arguments
    X15, // (A5) function arguments
    X16, // (A6) function arguments
    X17, // (A7) function arguments
    X18, // (S2) saved registers (callee-saved)
    X19, // (S3) saved registers (callee-saved)
    X20, // (S4) saved registers (callee-saved)
    X21, // (S5) saved registers (callee-saved)
    X22, // (S6) saved registers (callee-saved)
    X23, // (S7) saved registers (callee-saved)
    X24, // (S8) saved registers (callee-saved)
    X25, // (S9) saved registers (callee-saved)
    X26, // (S10) saved registers (callee-saved)
    X27, // (S11) saved registers (callee-saved)
    X28, // (T3) temporaries (caller-saved)
    X29, // (T4) temporaries (caller-saved)
    X30, // (T5) temporaries (caller-saved)
    X31, // (T6) temporaries (caller-saved)

    // floating-point registers
    F0,  // (FT0) FP temporaries
    F1,  // (FT1) FP temporaries
    F2,  // (FT2) FP temporaries
    F3,  // (FT3) FP temporaries
    F4,  // (FT4) FP temporaries
    F5,  // (FT5) FP temporaries
    F6,  // (FT6) FP temporaries
    F7,  // (FT7) FP temporaries
    F8,  // (FS0) FP saved register
    F9,  // (FS1) FP saved register
    F10, // (FA0) FP arguments/returned values
    F11, // (FA1) FP arguments/returned values
    F12, // (FA2) FP arguments
    F13, // (FA3) FP arguments
    F14, // (FA4) FP arguments
    F15, // (FA5) FP arguments
    F16, // (FA6) FP arguments
    F17, // (FA7) FP arguments
    F18, // (FS2) FP saved arguments
    F19, // (FS3) FP saved arguments
    F20, // (FS4) FP saved arguments
    F21, // (FS5) FP saved arguments
    F22, // (FS6) FP saved arguments
    F23, // (FS7) FP saved arguments
    F24, // (FS8) FP saved arguments
    F25, // (FS9) FP saved arguments
    F26, // (FS10) FP saved arguments
    F27, // (FS11) FP saved arguments
    F28, // (FT8) FP temporaries
    F29, // (FT9) FP temporaries
    F30, // (FT10) FP temporaries
    F31, // (FT11) FP temporaries
}

/// Declarative metadata describing how to locate and format a register.
#[derive(Clone, Debug)]
struct RegisterDecl {
    pub register: Register,
    pub name: &'static str,
    pub dwarf: i32,
    pub width: RegisterWidth,
    pub reg_type: RegisterType,
    pub format: RegisterFormat,
}

impl RegisterDecl {
    /// Derive the struct offset for a given register.
    ///
    /// slightly janky, assumes all decl widths are the same (which they are
    /// for general purpose and floating point regs), and that the DWARF ID
    /// is an incremental value from 0-31 within the target struct (which is also
    /// true for the _currently supported_ riscv structs/registers)
    fn offset(&self) -> usize {
        (self.dwarf % 32) as usize * self.width.bytes()
    }
}

impl From<&RegisterDecl> for RegisterInfo {
    fn from(decl: &RegisterDecl) -> Self {
        Self {
            register: decl.register,
            name: decl.name,
            dwarf_id: decl.dwarf,
            offset: decl.offset(),
            size: decl.width.bytes(),
            width: decl.width,
            register_type: decl.reg_type,
            format: decl.format,
        }
    }
}

const REGISTER_DECLS: &[RegisterDecl] = &[
    // 64-bit registers
    RegisterDecl {
        register: Register::X0,
        name: "x0",
        dwarf: 0,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X1,
        name: "x1",
        dwarf: 1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X2,
        name: "x2",
        dwarf: 2,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X3,
        name: "x3",
        dwarf: 3,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X4,
        name: "x4",
        dwarf: 4,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X5,
        name: "x5",
        dwarf: 5,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X6,
        name: "x6",
        dwarf: 6,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X7,
        name: "x7",
        dwarf: 7,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X8,
        name: "x8",
        dwarf: 8,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X9,
        name: "x9",
        dwarf: 9,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X10,
        name: "x10",
        dwarf: 10,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X11,
        name: "x11",
        dwarf: 11,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X12,
        name: "x12",
        dwarf: 12,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X13,
        name: "x13",
        dwarf: 13,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X14,
        name: "x14",
        dwarf: 14,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X15,
        name: "x15",
        dwarf: 15,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X16,
        name: "x16",
        dwarf: 16,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X17,
        name: "x17",
        dwarf: 17,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X18,
        name: "x18",
        dwarf: 18,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X19,
        name: "x19",
        dwarf: 19,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X20,
        name: "x20",
        dwarf: 20,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X21,
        name: "x21",
        dwarf: 21,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X22,
        name: "x22",
        dwarf: 22,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X23,
        name: "x23",
        dwarf: 23,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X24,
        name: "x24",
        dwarf: 24,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X25,
        name: "x25",
        dwarf: 25,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X26,
        name: "x26",
        dwarf: 26,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X27,
        name: "x27",
        dwarf: 27,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X28,
        name: "x28",
        dwarf: 28,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X29,
        name: "x29",
        dwarf: 29,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X30,
        name: "x30",
        dwarf: 30,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    RegisterDecl {
        register: Register::X31,
        name: "x31",
        dwarf: 31,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
    },
    // floating-point registers
    RegisterDecl {
        register: Register::F0,
        name: "f0",
        dwarf: 0,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F1,
        name: "f1",
        dwarf: 1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F2,
        name: "f2",
        dwarf: 2,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F3,
        name: "f3",
        dwarf: 3,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F4,
        name: "f4",
        dwarf: 4,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F5,
        name: "f5",
        dwarf: 5,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F6,
        name: "f6",
        dwarf: 6,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F7,
        name: "f7",
        dwarf: 7,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F8,
        name: "f8",
        dwarf: 8,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F9,
        name: "f9",
        dwarf: 9,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F10,
        name: "f10",
        dwarf: 10,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F11,
        name: "f11",
        dwarf: 11,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F12,
        name: "f12",
        dwarf: 12,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F13,
        name: "f13",
        dwarf: 13,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F14,
        name: "f14",
        dwarf: 14,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F15,
        name: "f15",
        dwarf: 15,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F16,
        name: "f16",
        dwarf: 16,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F17,
        name: "f17",
        dwarf: 17,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F18,
        name: "f18",
        dwarf: 18,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F19,
        name: "f19",
        dwarf: 19,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F20,
        name: "f20",
        dwarf: 20,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F21,
        name: "f21",
        dwarf: 21,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F22,
        name: "f22",
        dwarf: 22,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F23,
        name: "f23",
        dwarf: 23,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F24,
        name: "f24",
        dwarf: 24,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F25,
        name: "f25",
        dwarf: 25,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F26,
        name: "f26",
        dwarf: 26,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F27,
        name: "f27",
        dwarf: 27,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F28,
        name: "f28",
        dwarf: 28,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F29,
        name: "f29",
        dwarf: 29,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F30,
        name: "f30",
        dwarf: 30,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
    RegisterDecl {
        register: Register::F31,
        name: "f31",
        dwarf: 31,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Double,
    },
];

pub fn registers_info_iter() -> impl Iterator<Item = RegisterInfo> {
    REGISTER_DECLS.iter().map(RegisterInfo::from)
}
