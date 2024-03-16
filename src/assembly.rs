use std::fmt;

pub enum Instructions {
    Push(String),
    Pop(String),
    Mov(String, u32),
    Syscall,
}

#[derive(Debug)]
pub enum Registers {
    Rsp,

    Rax,
    Rdi,
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Registers::Rsp => write!(f, "rsp"),

            Registers::Rax => write!(f, "rax"),
            Registers::Rdi => write!(f, "rdi"),
        }
    }
}
