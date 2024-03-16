use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum SysCalls {
    Exit(Expressions),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Construct {
    Program(Vec<Statement>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Statement {
    Let(String, Expressions),
    SystemCall(SysCalls),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum _BinaryOperator {
    _Add,
    _Sub,
    _Mul,
    _Div,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Expressions {
    U32(u32),
    _F64(f64),
    _Boolean(bool),
    Alias(String),
    _BinaryOp(_BinaryOperator, Box<Expressions>, Box<Expressions>), // Example of a binary operation expression
}
