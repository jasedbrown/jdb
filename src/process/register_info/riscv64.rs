
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
