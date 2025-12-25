#![allow(dead_code)]
//! Centralized declaration of all supported CPU registers for x86_64.
//!
//! Implementation note: I originally followed the macro-styling of the
//! "Building a Debugging" book (chapter 5), but it got unweildy, fast.
//! Thus, I switched to having an AI agent generate this file. This data
//! is constant, and the value of an easy-to-read if verbose file is much
//! higher than a bunch of super fucking complicated macros ... :shrug:

use std::{sync::LazyLock, u8};

use anyhow::{anyhow, Result};
use strum::EnumDiscriminants;

/// Strongly typed representation of register values in their native sizes.
#[derive(Clone, Copy, Debug, EnumDiscriminants)]
#[strum_discriminants(name(RegisterFormat))]
pub enum RegisterValue {
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),

    // TODO: are these signed interger values used in any register value?
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),

    Float(f32),
    Double(f64),

    /// On x86, most C compilers implement long double as the 80-bit extended precision type
    /// (generally stored as 12 or 16 bytes to maintain data structure alignment). [0]
    ///
    /// Typically just for the `st0` - `st7` registers. These are the older
    /// x87 registers. There are 8 of them, 80-bits each.
    ///
    /// [0] https://en.wikipedia.org/wiki/Long_double
    LongDouble([u8; 10]),
    Byte64([u8; 8]),
    Byte128([u8; 16]),
}

// WIP implementation, not sure i like this, at all
impl TryFrom<RegisterValue> for i64 {
    type Error = anyhow::Error;

    fn try_from(reg_value: RegisterValue) -> Result<Self, Self::Error> {
        use RegisterValue::*;
        let val = match reg_value {
            Uint8(v) => v as i64,
            Uint16(v) => v as i64,
            Uint32(v) => v as i64,
            Uint64(v) => v as i64,
            Int8(v) => v as i64,
            Int16(v) => v as i64,
            Int32(v) => v as i64,
            Int64(v) => v,

            Float(_) | LongDouble(_) | Double(_) => {
                return Err(anyhow!("Cannot convert floating point value to c_long"));
            }

            Byte64(_) | Byte128(_) => {
                return Err(anyhow!("WTF, idk ..."));
            }
        };

        Ok(val)
    }
}

/// Broad grouping for registers, used for display and filtering.
#[derive(Clone, Copy, Debug, Hash)]
pub enum RegisterType {
    /// 64-bit instructions
    GeneralPurpose,
    SubGeneralPurpose,
    FloatingPoint,
    Debug,
}

/// Canonical width for a register or subregister.
///
/// Note: variants are prefixed with 'W' as rust won't allow a digit as the first char.
#[derive(Clone, Copy, Debug)]
pub enum RegisterWidth {
    W128,
    /// `long_double` widths of 80 bits (used in st0..st7 registers)
    W80,
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
            RegisterWidth::W128 => 128,
            RegisterWidth::W80 => 80,
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

    /// Offset within the parent storage for subregisters.
    const fn sub_offset(&self) -> usize {
        match self {
            RegisterWidth::W8H => 1,
            RegisterWidth::W8L
            | RegisterWidth::W16
            | RegisterWidth::W32
            | RegisterWidth::W64
            | RegisterWidth::W80
            | RegisterWidth::W128 => 0,
        }
    }
}

/// All registers supported by the debugger for x86_64.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum Register {
    // 64-bit registers
    RAX,
    RDX,
    RCX,
    RBX,
    RSI,
    RDI,
    RBP,
    RSP,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    RIP,
    EFLAGS,
    CS,
    FS,
    GS,
    SS,
    DS,
    ES,
    ORIGRAX,

    // 32-bit subregisters
    EAX,
    EDX,
    ECX,
    EBX,
    ESI,
    EDI,
    EBP,
    ESP,
    R8D,
    R9D,
    R10D,
    R11D,
    R12D,
    R13D,
    R14D,
    R15D,

    // 16-bit subregisters
    AX,
    DX,
    CX,
    SI,
    DI,
    BP,
    SP,
    R8W,
    R9W,
    R10W,
    R11W,
    R12W,
    R13W,
    R14W,
    R15W,

    // 8-bit high subregisters
    AH,
    DH,
    CH,
    BH,

    // 8-bit low subregisters
    AL,
    DL,
    CL,
    BL,
    SIL,
    DIL,
    BPL,
    SPL,
    R8B,
    R9B,
    R10B,
    R11B,
    R12B,
    R13B,
    R14B,
    R15B,

    // Floating point
    FCW,
    FSW,
    FTW,
    FOP,
    FRIP,
    FRDP,
    MXCSR,
    MXCSR_MASK,
    ST0,
    ST1,
    ST2,
    ST3,
    ST4,
    ST5,
    ST6,
    ST7,
    MM0,
    MM1,
    MM2,
    MM3,
    MM4,
    MM5,
    MM6,
    MM7,
    XMM0,
    XMM1,
    XMM2,
    XMM3,
    XMM4,
    XMM5,
    XMM6,
    XMM7,
    XMM8,
    XMM9,
    XMM10,
    XMM11,
    XMM12,
    XMM13,
    XMM14,
    XMM15,

    // Debug registers
    DR0,
    DR1,
    DR2,
    DR3,
    DR4,
    DR5,
    DR6,
    DR7,
}

/// Physical storage location for a register within the `user` structures.
#[derive(Copy, Clone, Debug)]
pub enum Location {
    Regs(RegsField),
    Fpu(FpuField),
    FpuArray(FpuArrayField, usize),
    UserArray(UserField, usize),
}

/// Field inside `libc::user_regs_struct` that holds a given register.
#[derive(Copy, Clone, Debug)]
pub enum RegsField {
    Rax,
    Rdx,
    Rcx,
    Rbx,
    Rsi,
    Rdi,
    Rbp,
    Rsp,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    Rip,
    Eflags,
    Cs,
    Fs,
    Gs,
    Ss,
    Ds,
    Es,
    OrigRax,
}

impl RegsField {
    const fn offset(self) -> usize {
        match self {
            RegsField::Rax => memoffset::offset_of!(libc::user_regs_struct, rax),
            RegsField::Rdx => memoffset::offset_of!(libc::user_regs_struct, rdx),
            RegsField::Rcx => memoffset::offset_of!(libc::user_regs_struct, rcx),
            RegsField::Rbx => memoffset::offset_of!(libc::user_regs_struct, rbx),
            RegsField::Rsi => memoffset::offset_of!(libc::user_regs_struct, rsi),
            RegsField::Rdi => memoffset::offset_of!(libc::user_regs_struct, rdi),
            RegsField::Rbp => memoffset::offset_of!(libc::user_regs_struct, rbp),
            RegsField::Rsp => memoffset::offset_of!(libc::user_regs_struct, rsp),
            RegsField::R8 => memoffset::offset_of!(libc::user_regs_struct, r8),
            RegsField::R9 => memoffset::offset_of!(libc::user_regs_struct, r9),
            RegsField::R10 => memoffset::offset_of!(libc::user_regs_struct, r10),
            RegsField::R11 => memoffset::offset_of!(libc::user_regs_struct, r11),
            RegsField::R12 => memoffset::offset_of!(libc::user_regs_struct, r12),
            RegsField::R13 => memoffset::offset_of!(libc::user_regs_struct, r13),
            RegsField::R14 => memoffset::offset_of!(libc::user_regs_struct, r14),
            RegsField::R15 => memoffset::offset_of!(libc::user_regs_struct, r15),
            RegsField::Rip => memoffset::offset_of!(libc::user_regs_struct, rip),
            RegsField::Eflags => memoffset::offset_of!(libc::user_regs_struct, eflags),
            RegsField::Cs => memoffset::offset_of!(libc::user_regs_struct, cs),
            RegsField::Fs => memoffset::offset_of!(libc::user_regs_struct, fs),
            RegsField::Gs => memoffset::offset_of!(libc::user_regs_struct, gs),
            RegsField::Ss => memoffset::offset_of!(libc::user_regs_struct, ss),
            RegsField::Ds => memoffset::offset_of!(libc::user_regs_struct, ds),
            RegsField::Es => memoffset::offset_of!(libc::user_regs_struct, es),
            RegsField::OrigRax => memoffset::offset_of!(libc::user_regs_struct, orig_rax),
        }
    }
}

/// Scalar fields within `libc::user_fpregs_struct`.
#[derive(Copy, Clone, Debug)]
pub enum FpuField {
    Cwd,
    Swd,
    Ftw,
    Fop,
    Rip,
    Rdp,
    Mxcsr,
    MxcrMask,
}

impl FpuField {
    const fn offset(self) -> usize {
        match self {
            FpuField::Cwd => memoffset::offset_of!(libc::user_fpregs_struct, cwd),
            FpuField::Swd => memoffset::offset_of!(libc::user_fpregs_struct, swd),
            FpuField::Ftw => memoffset::offset_of!(libc::user_fpregs_struct, ftw),
            FpuField::Fop => memoffset::offset_of!(libc::user_fpregs_struct, fop),
            FpuField::Rip => memoffset::offset_of!(libc::user_fpregs_struct, rip),
            FpuField::Rdp => memoffset::offset_of!(libc::user_fpregs_struct, rdp),
            FpuField::Mxcsr => memoffset::offset_of!(libc::user_fpregs_struct, mxcsr),
            FpuField::MxcrMask => memoffset::offset_of!(libc::user_fpregs_struct, mxcr_mask),
        }
    }
}

/// Array fields inside `libc::user_fpregs_struct`.
#[derive(Copy, Clone, Debug)]
pub enum FpuArrayField {
    St,
    Xmm,
}

impl FpuArrayField {
    const fn offset(self) -> usize {
        match self {
            // MMX (mm0..mm7) use the same space as x87 registers (st)
            FpuArrayField::St => memoffset::offset_of!(libc::user_fpregs_struct, st_space),
            FpuArrayField::Xmm => memoffset::offset_of!(libc::user_fpregs_struct, xmm_space),
        }
    }

    /// Size, in bytes, of a single slot in the backing array.
    /// x87/MMX and XMM entries each occupy 16 bytes.
    const fn stride(self) -> usize {
        match self {
            FpuArrayField::St | FpuArrayField::Xmm => 16,
        }
    }
}

/// Miscellaneous fields within `libc::user`.
#[derive(Copy, Clone, Debug)]
pub enum UserField {
    UDebugReg,
}

impl UserField {
    const fn offset(self) -> usize {
        match self {
            UserField::UDebugReg => memoffset::offset_of!(libc::user, u_debugreg),
        }
    }
}

impl Location {
    const fn offset(self, width: RegisterWidth) -> usize {
        let base = match self {
            Location::Regs(field) => memoffset::offset_of!(libc::user, regs) + field.offset(),
            Location::Fpu(field) => memoffset::offset_of!(libc::user, i387) + field.offset(),
            Location::FpuArray(field, index) => {
                memoffset::offset_of!(libc::user, i387) + field.offset() + (index * field.stride())
            }
            Location::UserArray(field, index) => field.offset() + (index * width.bytes()),
        };

        base + width.sub_offset()
    }
}

/// Declarative metadata describing how to locate and format a register.
#[derive(Copy, Clone, Debug)]
pub struct RegisterDecl {
    pub register: Register,
    pub name: &'static str,
    pub dwarf: i32,
    pub width: RegisterWidth,
    pub reg_type: RegisterType,
    pub loc: Location,
    pub format: RegisterFormat,
}

/// Fully derived register information, including computed offsets and sizes.
#[derive(Clone, Debug)]
pub struct RegisterInfo {
    pub register: Register,
    /// The actual name of the register, as appears in the `user` family of structs.
    pub name: &'static str,
    pub dwarf_id: i32,
    /// The byte offset into the `user` struct of this register.
    /// Primarily used for `read_user()` and `write_user()`.
    pub offset: usize,
    /// Size in bytes of the register's value.
    pub size: usize,
    pub width: RegisterWidth,
    pub register_type: RegisterType,
    pub format: RegisterFormat,
    pub loc: Location,
}

impl From<&RegisterDecl> for RegisterInfo {
    fn from(decl: &RegisterDecl) -> Self {
        Self {
            register: decl.register,
            name: decl.name,
            dwarf_id: decl.dwarf,
            offset: decl.loc.offset(decl.width),
            size: decl.width.bytes(),
            width: decl.width,
            register_type: decl.reg_type,
            format: decl.format,
            loc: decl.loc,
        }
    }
}

pub const REGISTER_DECLS: &[RegisterDecl] = &[
    // 64-bit registers
    RegisterDecl {
        register: Register::RAX,
        name: "rax",
        dwarf: 0,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Rax),
    },
    RegisterDecl {
        register: Register::RDX,
        name: "rdx",
        dwarf: 1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Rdx),
    },
    RegisterDecl {
        register: Register::RCX,
        name: "rcx",
        dwarf: 2,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Rcx),
    },
    RegisterDecl {
        register: Register::RBX,
        name: "rbx",
        dwarf: 3,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Rbx),
    },
    RegisterDecl {
        register: Register::RSI,
        name: "rsi",
        dwarf: 4,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Rsi),
    },
    RegisterDecl {
        register: Register::RDI,
        name: "rdi",
        dwarf: 5,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Rdi),
    },
    RegisterDecl {
        register: Register::RBP,
        name: "rbp",
        dwarf: 6,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Rbp),
    },
    RegisterDecl {
        register: Register::RSP,
        name: "rsp",
        dwarf: 7,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Rsp),
    },
    RegisterDecl {
        register: Register::R8,
        name: "r8",
        dwarf: 8,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::R8),
    },
    RegisterDecl {
        register: Register::R9,
        name: "r9",
        dwarf: 9,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::R9),
    },
    RegisterDecl {
        register: Register::R10,
        name: "r10",
        dwarf: 10,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::R10),
    },
    RegisterDecl {
        register: Register::R11,
        name: "r11",
        dwarf: 11,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::R11),
    },
    RegisterDecl {
        register: Register::R12,
        name: "r12",
        dwarf: 12,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::R12),
    },
    RegisterDecl {
        register: Register::R13,
        name: "r13",
        dwarf: 13,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::R13),
    },
    RegisterDecl {
        register: Register::R14,
        name: "r14",
        dwarf: 14,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::R14),
    },
    RegisterDecl {
        register: Register::R15,
        name: "r15",
        dwarf: 15,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::R15),
    },
    RegisterDecl {
        register: Register::RIP,
        name: "rip",
        dwarf: 16,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Rip),
    },
    RegisterDecl {
        register: Register::EFLAGS,
        name: "eflags",
        dwarf: 49,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Eflags),
    },
    RegisterDecl {
        register: Register::CS,
        name: "cs",
        dwarf: 51,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Cs),
    },
    RegisterDecl {
        register: Register::FS,
        name: "fs",
        dwarf: 54,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Fs),
    },
    RegisterDecl {
        register: Register::GS,
        name: "gs",
        dwarf: 55,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Gs),
    },
    RegisterDecl {
        register: Register::SS,
        name: "ss",
        dwarf: 52,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Ss),
    },
    RegisterDecl {
        register: Register::DS,
        name: "ds",
        dwarf: 53,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Ds),
    },
    RegisterDecl {
        register: Register::ES,
        name: "es",
        dwarf: 50,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::Es),
    },
    RegisterDecl {
        register: Register::ORIGRAX,
        name: "orig_rax",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::GeneralPurpose,
        format: RegisterFormat::Uint64,
        loc: Location::Regs(RegsField::OrigRax),
    },
    // 32-bit subregisters. no dwarf IDs
    RegisterDecl {
        register: Register::EAX,
        name: "eax",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::Rax),
    },
    RegisterDecl {
        register: Register::EDX,
        name: "edx",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::Rdx),
    },
    RegisterDecl {
        register: Register::ECX,
        name: "ecx",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::Rcx),
    },
    RegisterDecl {
        register: Register::EBX,
        name: "ebx",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::Rbx),
    },
    RegisterDecl {
        register: Register::ESI,
        name: "esi",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::Rsi),
    },
    RegisterDecl {
        register: Register::EDI,
        name: "edi",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::Rdi),
    },
    RegisterDecl {
        register: Register::EBP,
        name: "ebp",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::Rbp),
    },
    RegisterDecl {
        register: Register::ESP,
        name: "esp",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::Rsp),
    },
    RegisterDecl {
        register: Register::R8D,
        name: "r8d",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::R8),
    },
    RegisterDecl {
        register: Register::R9D,
        name: "r9d",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::R9),
    },
    RegisterDecl {
        register: Register::R10D,
        name: "r10d",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::R10),
    },
    RegisterDecl {
        register: Register::R11D,
        name: "r11d",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::R11),
    },
    RegisterDecl {
        register: Register::R12D,
        name: "r12d",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::R12),
    },
    RegisterDecl {
        register: Register::R13D,
        name: "r13d",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::R13),
    },
    RegisterDecl {
        register: Register::R14D,
        name: "r14d",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::R14),
    },
    RegisterDecl {
        register: Register::R15D,
        name: "r15d",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint32,
        loc: Location::Regs(RegsField::R15),
    },
    // 16-bit subregisters. no dwarf IDs
    RegisterDecl {
        register: Register::AX,
        name: "ax",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::Rax),
    },
    RegisterDecl {
        register: Register::DX,
        name: "dx",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::Rdx),
    },
    RegisterDecl {
        register: Register::CX,
        name: "cx",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::Rcx),
    },
    RegisterDecl {
        register: Register::SI,
        name: "si",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::Rsi),
    },
    RegisterDecl {
        register: Register::DI,
        name: "di",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::Rdi),
    },
    RegisterDecl {
        register: Register::BP,
        name: "bp",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::Rbp),
    },
    RegisterDecl {
        register: Register::SP,
        name: "sp",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::Rsp),
    },
    RegisterDecl {
        register: Register::R8W,
        name: "r8w",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::R8),
    },
    RegisterDecl {
        register: Register::R9W,
        name: "r9w",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::R9),
    },
    RegisterDecl {
        register: Register::R10W,
        name: "r10w",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::R10),
    },
    RegisterDecl {
        register: Register::R11W,
        name: "r11w",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::R11),
    },
    RegisterDecl {
        register: Register::R12W,
        name: "r12w",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::R12),
    },
    RegisterDecl {
        register: Register::R13W,
        name: "r13w",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::R13),
    },
    RegisterDecl {
        register: Register::R14W,
        name: "r14w",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::R14),
    },
    RegisterDecl {
        register: Register::R15W,
        name: "r15w",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint16,
        loc: Location::Regs(RegsField::R15),
    },
    // 8-bit high subregisters. no dwarf IDs
    RegisterDecl {
        register: Register::AH,
        name: "ah",
        dwarf: -1,
        width: RegisterWidth::W8H,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::Rax),
    },
    RegisterDecl {
        register: Register::DH,
        name: "dh",
        dwarf: -1,
        width: RegisterWidth::W8H,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::Rdx),
    },
    RegisterDecl {
        register: Register::CH,
        name: "ch",
        dwarf: -1,
        width: RegisterWidth::W8H,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::Rcx),
    },
    RegisterDecl {
        register: Register::BH,
        name: "bh",
        dwarf: -1,
        width: RegisterWidth::W8H,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::Rbx),
    },
    // 8-bit low subregisters. no dwarf IDs
    RegisterDecl {
        register: Register::AL,
        name: "al",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::Rax),
    },
    RegisterDecl {
        register: Register::DL,
        name: "dl",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::Rdx),
    },
    RegisterDecl {
        register: Register::CL,
        name: "cl",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::Rcx),
    },
    RegisterDecl {
        register: Register::BL,
        name: "bl",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::Rbx),
    },
    RegisterDecl {
        register: Register::SIL,
        name: "sil",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::Rsi),
    },
    RegisterDecl {
        register: Register::DIL,
        name: "dil",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::Rdi),
    },
    RegisterDecl {
        register: Register::BPL,
        name: "bpl",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::Rbp),
    },
    RegisterDecl {
        register: Register::SPL,
        name: "spl",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::Rsp),
    },
    RegisterDecl {
        register: Register::R8B,
        name: "r8b",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::R8),
    },
    RegisterDecl {
        register: Register::R9B,
        name: "r9b",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::R9),
    },
    RegisterDecl {
        register: Register::R10B,
        name: "r10b",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::R10),
    },
    RegisterDecl {
        register: Register::R11B,
        name: "r11b",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::R11),
    },
    RegisterDecl {
        register: Register::R12B,
        name: "r12b",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::R12),
    },
    RegisterDecl {
        register: Register::R13B,
        name: "r13b",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::R13),
    },
    RegisterDecl {
        register: Register::R14B,
        name: "r14b",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::R14),
    },
    RegisterDecl {
        register: Register::R15B,
        name: "r15b",
        dwarf: -1,
        width: RegisterWidth::W8L,
        reg_type: RegisterType::SubGeneralPurpose,
        format: RegisterFormat::Uint8,
        loc: Location::Regs(RegsField::R15),
    },
    // Floating point control registers
    RegisterDecl {
        register: Register::FCW,
        name: "cwd",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Uint16,
        loc: Location::Fpu(FpuField::Cwd),
    },
    RegisterDecl {
        register: Register::FSW,
        name: "swd",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Uint16,
        loc: Location::Fpu(FpuField::Swd),
    },
    RegisterDecl {
        register: Register::FTW,
        name: "ftw",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Uint16,
        loc: Location::Fpu(FpuField::Ftw),
    },
    RegisterDecl {
        register: Register::FOP,
        name: "fop",
        dwarf: -1,
        width: RegisterWidth::W16,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Uint16,
        loc: Location::Fpu(FpuField::Fop),
    },
    RegisterDecl {
        register: Register::FRIP,
        name: "rip",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Uint16,
        loc: Location::Fpu(FpuField::Rip),
    },
    RegisterDecl {
        register: Register::FRDP,
        name: "rdp",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Uint16,
        loc: Location::Fpu(FpuField::Rdp),
    },
    RegisterDecl {
        register: Register::MXCSR,
        name: "mxcsr",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Uint16,
        loc: Location::Fpu(FpuField::Mxcsr),
    },
    RegisterDecl {
        register: Register::MXCSR_MASK,
        name: "mxcr_mask",
        dwarf: -1,
        width: RegisterWidth::W32,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Uint16,
        loc: Location::Fpu(FpuField::MxcrMask),
    },
    // x87 stack
    RegisterDecl {
        register: Register::ST0,
        name: "st0",
        dwarf: -1,
        width: RegisterWidth::W80,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::LongDouble,
        loc: Location::FpuArray(FpuArrayField::St, 0),
    },
    RegisterDecl {
        register: Register::ST1,
        name: "st1",
        dwarf: -1,
        width: RegisterWidth::W80,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::LongDouble,
        loc: Location::FpuArray(FpuArrayField::St, 1),
    },
    RegisterDecl {
        register: Register::ST2,
        name: "st2",
        dwarf: -1,
        width: RegisterWidth::W80,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::LongDouble,
        loc: Location::FpuArray(FpuArrayField::St, 2),
    },
    RegisterDecl {
        register: Register::ST3,
        name: "st3",
        dwarf: -1,
        width: RegisterWidth::W80,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::LongDouble,
        loc: Location::FpuArray(FpuArrayField::St, 3),
    },
    RegisterDecl {
        register: Register::ST4,
        name: "st4",
        dwarf: -1,
        width: RegisterWidth::W80,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::LongDouble,
        loc: Location::FpuArray(FpuArrayField::St, 4),
    },
    RegisterDecl {
        register: Register::ST5,
        name: "st5",
        dwarf: -1,
        width: RegisterWidth::W80,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::LongDouble,
        loc: Location::FpuArray(FpuArrayField::St, 5),
    },
    RegisterDecl {
        register: Register::ST6,
        name: "st6",
        dwarf: -1,
        width: RegisterWidth::W80,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::LongDouble,
        loc: Location::FpuArray(FpuArrayField::St, 6),
    },
    RegisterDecl {
        register: Register::ST7,
        name: "st7",
        dwarf: -1,
        width: RegisterWidth::W80,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::LongDouble,
        loc: Location::FpuArray(FpuArrayField::St, 7),
    },
    // MMX registers
    RegisterDecl {
        register: Register::MM0,
        name: "mm0",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte64,
        loc: Location::FpuArray(FpuArrayField::St, 0),
    },
    RegisterDecl {
        register: Register::MM1,
        name: "mm1",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte64,
        loc: Location::FpuArray(FpuArrayField::St, 1),
    },
    RegisterDecl {
        register: Register::MM2,
        name: "mm2",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte64,
        loc: Location::FpuArray(FpuArrayField::St, 2),
    },
    RegisterDecl {
        register: Register::MM3,
        name: "mm3",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte64,
        loc: Location::FpuArray(FpuArrayField::St, 3),
    },
    RegisterDecl {
        register: Register::MM4,
        name: "mm4",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte64,
        loc: Location::FpuArray(FpuArrayField::St, 4),
    },
    RegisterDecl {
        register: Register::MM5,
        name: "mm5",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte64,
        loc: Location::FpuArray(FpuArrayField::St, 5),
    },
    RegisterDecl {
        register: Register::MM6,
        name: "mm6",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte64,
        loc: Location::FpuArray(FpuArrayField::St, 6),
    },
    RegisterDecl {
        register: Register::MM7,
        name: "mm7",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte64,
        loc: Location::FpuArray(FpuArrayField::St, 7),
    },
    // XMM registers
    RegisterDecl {
        register: Register::XMM0,
        name: "xmm0",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 0),
    },
    RegisterDecl {
        register: Register::XMM1,
        name: "xmm1",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 1),
    },
    RegisterDecl {
        register: Register::XMM2,
        name: "xmm2",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 2),
    },
    RegisterDecl {
        register: Register::XMM3,
        name: "xmm3",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 3),
    },
    RegisterDecl {
        register: Register::XMM4,
        name: "xmm4",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 4),
    },
    RegisterDecl {
        register: Register::XMM5,
        name: "xmm5",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 5),
    },
    RegisterDecl {
        register: Register::XMM6,
        name: "xmm6",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 6),
    },
    RegisterDecl {
        register: Register::XMM7,
        name: "xmm7",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 7),
    },
    RegisterDecl {
        register: Register::XMM8,
        name: "xmm8",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 8),
    },
    RegisterDecl {
        register: Register::XMM9,
        name: "xmm9",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 9),
    },
    RegisterDecl {
        register: Register::XMM10,
        name: "xmm10",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 10),
    },
    RegisterDecl {
        register: Register::XMM11,
        name: "xmm11",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 11),
    },
    RegisterDecl {
        register: Register::XMM12,
        name: "xmm12",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 12),
    },
    RegisterDecl {
        register: Register::XMM13,
        name: "xmm13",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 13),
    },
    RegisterDecl {
        register: Register::XMM14,
        name: "xmm14",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 14),
    },
    RegisterDecl {
        register: Register::XMM15,
        name: "xmm15",
        dwarf: -1,
        width: RegisterWidth::W128,
        reg_type: RegisterType::FloatingPoint,
        format: RegisterFormat::Byte128,
        loc: Location::FpuArray(FpuArrayField::Xmm, 15),
    },
    // Debug registers
    RegisterDecl {
        register: Register::DR0,
        name: "u_debugreg[0]",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::Debug,
        format: RegisterFormat::Uint64,
        loc: Location::UserArray(UserField::UDebugReg, 0),
    },
    RegisterDecl {
        register: Register::DR1,
        name: "u_debugreg[1]",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::Debug,
        format: RegisterFormat::Uint64,
        loc: Location::UserArray(UserField::UDebugReg, 1),
    },
    RegisterDecl {
        register: Register::DR2,
        name: "u_debugreg[2]",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::Debug,
        format: RegisterFormat::Uint64,
        loc: Location::UserArray(UserField::UDebugReg, 2),
    },
    RegisterDecl {
        register: Register::DR3,
        name: "u_debugreg[3]",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::Debug,
        format: RegisterFormat::Uint64,
        loc: Location::UserArray(UserField::UDebugReg, 3),
    },
    RegisterDecl {
        register: Register::DR4,
        name: "u_debugreg[4]",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::Debug,
        format: RegisterFormat::Uint64,
        loc: Location::UserArray(UserField::UDebugReg, 4),
    },
    RegisterDecl {
        register: Register::DR5,
        name: "u_debugreg[5]",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::Debug,
        format: RegisterFormat::Uint64,
        loc: Location::UserArray(UserField::UDebugReg, 5),
    },
    RegisterDecl {
        register: Register::DR6,
        name: "u_debugreg[6]",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::Debug,
        format: RegisterFormat::Uint64,
        loc: Location::UserArray(UserField::UDebugReg, 6),
    },
    RegisterDecl {
        register: Register::DR7,
        name: "u_debugreg[7]",
        dwarf: -1,
        width: RegisterWidth::W64,
        reg_type: RegisterType::Debug,
        format: RegisterFormat::Uint64,
        loc: Location::UserArray(UserField::UDebugReg, 7),
    },
];

pub static REGISTERS_INFO: LazyLock<Vec<RegisterInfo>> =
    LazyLock::new(|| REGISTER_DECLS.iter().map(RegisterInfo::from).collect());

pub fn registers_info() -> &'static [RegisterInfo] {
    REGISTERS_INFO.as_slice()
}
