use std::collections::HashMap;
use std::sync::LazyLock;

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::{RegisterSnapshot, read_all_registers};
#[cfg(target_arch = "x86_64")]
use crate::process::register_info::{registers_info_iter, Register, RegisterInfo};

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::{RegisterSnapshot, read_all_registers};
#[cfg(target_arch = "aarch64")]
use crate::process::register_info::{registers_info_iter, Register, RegisterInfo};

#[cfg(target_arch = "riscv64")]
mod riscv64;
#[cfg(target_arch = "riscv64")]
pub use riscv64::{RegisterSnapshot, read_all_registers};
#[cfg(target_arch = "riscv64")]
use crate::process::register_info::{registers_info_iter, Register, RegisterInfo};

#[allow(dead_code)]
static REGISTERS_MAP: LazyLock<HashMap<Register, RegisterInfo>> = LazyLock::new(|| {
    let mut regs = HashMap::new();

    registers_info_iter().for_each(|r| {
        regs.insert(r.register, r.clone());
    });

    regs
});
