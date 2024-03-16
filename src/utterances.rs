#[cfg(target_os = "linux")]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum SysCalls {
    Exit(Expressions),
}

#[derive(Debug)]
pub enum Construct {
    Program(Vec<Statement>),
}

#[derive(Debug)]
pub enum Statement {
    Let(String, Expressions),
    #[cfg(target_os = "linux")]
    SystemCall(SysCalls),
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub enum Expressions {
    U32(u32),
    F64(f64),
    Boolean(bool),
    Alias(String),
    BinaryOp(BinaryOperator, Box<Expressions>, Box<Expressions>), // Example of a binary operation expression
}
