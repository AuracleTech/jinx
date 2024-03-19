use std::fmt;

#[derive(Debug)]
pub enum Instructions {
    Push(String),
    Pop(String),
    Mov(String, u32),
    Add(String, String),
    Syscall,
}

#[derive(Debug)]
pub enum Registers {
    Rsp,

    Rax,
    Rbx,
    Rdi,
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Registers::Rsp => write!(f, "rsp"),

            Registers::Rax => write!(f, "rax"),
            Registers::Rbx => write!(f, "rbx"),
            Registers::Rdi => write!(f, "rdi"),
        }
    }
}
