use std::fmt;

use crate::utterances::{
    ArithmeticOperator, Construct, Expression, Literal, Statement, SysCall, Term,
};

const SPACE_SIZE: usize = 2;

impl fmt::Display for Construct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CONSTRUCT\n")?;
        match self {
            Construct::Program(statements) => {
                for statement in statements {
                    write!(f, "{:indent$}{}\n", "", statement, indent = SPACE_SIZE)?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "STATEMENT ")?;
        match self {
            Statement::Let(alias, expression) => write!(f, "LET {} ASSIGN {};", alias, expression),
            Statement::SystemCall(sys_call) => write!(f, "{}", sys_call),
        }
    }
}

impl fmt::Display for SysCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SYSCALL ")?;
        match self {
            SysCall::Exit(expression) => write!(f, "EXIT {}", expression),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::BinaryOp(op, exp1, exp2) => write!(f, "({} {} {})", exp1, op, exp2),
            Expression::Term(term) => write!(f, "{}", term),
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Alias(alias) => write!(f, "{}", alias),
            Term::Literal(literal) => write!(f, "{}", literal),
        }
    }
}

impl fmt::Display for ArithmeticOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithmeticOperator::Add => write!(f, "+"),
            ArithmeticOperator::Sub => write!(f, "-"),
            ArithmeticOperator::Mul => write!(f, "*"),
            ArithmeticOperator::Div => write!(f, "/"),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::U32(val) => write!(f, "{}", val),
            Literal::_F64(val) => write!(f, "{}", val),
            Literal::_String(val) => write!(f, "{}", val),
            Literal::_Boolean(val) => write!(f, "{}", val),
        }
    }
}
