use anyhow::Result;
use nix::unistd::Pid;

pub trait RegisterBackend {
    fn read_all_registers(pid: Pid) -> Result<RegisterSnapshot>;
}

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::{ArchRegisterBackend, RegisterSnapshot};

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "riscv64")]
mod riscv64;
