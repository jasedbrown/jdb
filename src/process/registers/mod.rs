use anyhow::Result;
use nix::unistd::Pid;

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::{read_all_registers, RegisterSnapshot};

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::{read_all_registers, RegisterSnapshot};

#[cfg(target_arch = "riscv64")]
mod riscv64;
