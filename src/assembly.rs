use std::fmt;

pub enum Instructions {
    Push(Registers),
    Pop(Registers),
    Mov(Registers, u32),
    Syscall,
}

#[derive(Debug)]
pub enum Registers {
    Rax,
    Rdi,
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Registers::Rax => write!(f, "rax"),
            Registers::Rdi => write!(f, "rdi"),
        }
    }
}
