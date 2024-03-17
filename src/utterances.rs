use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum SysCall {
    Exit(Expression),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Construct {
    Program(Vec<Statement>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Statement {
    Let(String, Expression),
    SystemCall(SysCall),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ArithmeticOperator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Literal {
    U32(u32),
    _F64(f64),
    _String(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Expression {
    BinaryOp(ArithmeticOperator, Box<Expression>, Box<Expression>),
    Alias(String),
    Literal(Literal),
    Boolean(bool),
}
