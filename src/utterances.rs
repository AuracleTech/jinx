use logos::{Logos, Skip};
use serde::{Deserialize, Serialize};

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Kind {
    #[regex(r"//.*", |_| Skip, priority = 3)]
    CommentLine,
    #[regex(r"/\*[^*]*\*/", |_| Skip, priority = 3)]
    CommentBlock,

    #[token("let")]
    KeywordLet,

    #[token("syscall")]
    SystemCall,

    #[token("(")]
    ParenthesisOpen,
    #[token(")")]
    ParenthesisClose,

    #[token("{")]
    BracketOpen,
    #[token("}")]
    BracketClose,

    #[regex(r"[a-z][a-zA-Z0-9_]*")]
    AliasSnakeCase,
    #[regex(r"[0-9]+")]
    Number,

    #[token("=")]
    Assign,
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,

    #[token(";")]
    SemiColon,
}

type Alias = String;

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
    _Boolean(bool),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SysCall {
    Exit(Expression),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Construct {
    Program(Vec<Statement>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Statement {
    Let(Alias, Expression),
    SystemCall(SysCall),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Expression {
    BinaryOp(ArithmeticOperator, Box<Expression>, Box<Expression>),
    Term(Term),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Term {
    Alias(Alias),
    Literal(Literal),
}
