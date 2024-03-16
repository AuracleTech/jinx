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
pub enum _BinaryOperator {
    _Add,
    _Sub,
    _Mul,
    _Div,
}

#[derive(Debug)]
pub enum Expressions {
    U32(u32),
    _F64(f64),
    _Boolean(bool),
    Alias(String),
    _BinaryOp(_BinaryOperator, Box<Expressions>, Box<Expressions>), // Example of a binary operation expression
}
