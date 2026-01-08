#![allow(dead_code)]
use anyhow::{Result, anyhow};
use strum::EnumDiscriminants;

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

#[cfg(target_arch = "riscv64")]
mod riscv64;

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
