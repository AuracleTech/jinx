use std::collections::HashMap;

use crate::utterances::{
    ArithmeticOperator, Construct, Expression, Literal, Statement, SysCall, Term,
};

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

#[derive(Debug)]
struct Var {
    stack_location: u32,
}

pub struct Transpiler {
    output: String,
    vars: HashMap<String, Var>,

    stack_len: usize,
    max_stack_size: usize,
}

impl Transpiler {
    pub fn new() -> Self {
        let max_stack_size = 100; // TODO: make this configurable
        Self {
            output: String::new(),
            vars: HashMap::new(),
            stack_len: 0,
            max_stack_size,
        }
    }

    pub fn instructions(&mut self, instructions: Vec<Instructions>) {
        for instruction in instructions {
            match instruction {
                Instructions::Push(register) => {
                    self.stack_len += 1;

                    if self.stack_len > self.max_stack_size {
                        panic!("stack overflow");
                    }

                    self.output += &format!("\tpush {}\n", register);
                }
                Instructions::Pop(register) => {
                    if self.stack_len == 0 {
                        panic!("stack underflow");
                    }

                    self.stack_len -= 1;
                    self.output += &format!("\tpop {}\n", register);
                }
                Instructions::Mov(register, value) => {
                    self.output += &format!("\tmov {}, {}\n", register, value);
                }
                Instructions::Syscall => {
                    self.instructions(vec![
                        Instructions::Mov(Registers::Rax.to_string(), 60),
                        Instructions::Pop(Registers::Rdi.to_string()),
                    ]);
                    self.output += "\tsyscall\n";
                }
                Instructions::Add(left, right) => {
                    self.output += &format!("\tadd {}, {}\n", left, right);
                }
            }
        }

        self.output += "\n";
    }

    pub fn construct(&mut self, construct: Construct) -> String {
        self.output += &format!(
            "# {} / {}\n\n",
            chrono::Local::now().format("%H:%M:%S"),
            chrono::Local::now().format("%e %b %Y"),
        );

        match construct {
            Construct::Program(statements) => {
                self.output += "global _start\n_start:\n";
                for statement in statements {
                    self.statement(statement);
                }
            }
        }

        self.output.to_string()
    }

    fn expr(&mut self, expression: Expression) {
        match expression {
            Expression::Term(term) => match term {
                Term::Alias(alias) => {
                    if let Some(var) = self.vars.get(&alias) {
                        self.instructions(vec![Instructions::Push(format!(
                            "QWORD [{}+{}]",
                            Registers::Rsp,
                            (self.stack_len - var.stack_location as usize - 1) * 8
                        ))]);
                    } else {
                        panic!("undeclared alias '{}'", alias);
                    }
                }
                Term::Literal(literal) => self.literal(literal),
            },
            Expression::BinaryOp(operator, left, right) => {
                self.expr(*left);
                self.expr(*right);

                self.instructions(vec![Instructions::Pop(Registers::Rax.to_string())]);
                self.instructions(vec![Instructions::Pop(Registers::Rbx.to_string())]);

                match operator {
                    ArithmeticOperator::Add => {
                        self.instructions(vec![
                            Instructions::Add(
                                Registers::Rax.to_string(),
                                Registers::Rbx.to_string(),
                            ),
                            Instructions::Push(Registers::Rax.to_string()),
                        ]);
                    }
                    _ => panic!("unimplemented operator {:?}", operator),
                }
            }
        }
    }

    fn literal(&mut self, literal: Literal) {
        match literal {
            Literal::U32(value) => {
                self.instructions(vec![
                    Instructions::Mov(Registers::Rax.to_string(), value),
                    Instructions::Push(Registers::Rax.to_string()),
                ]);
            }
            _ => unimplemented!(),
        }
    }

    fn statement(&mut self, statement: Statement) {
        match statement {
            Statement::Let(alias, expression) => {
                if let Some(var) = self.vars.get(&alias) {
                    panic!("alias '{}' already declared {:?}", alias, var);
                }

                let stack_location = self.stack_len as u32;
                self.vars.insert(alias.to_owned(), Var { stack_location });
                self.expr(expression);
            }
            Statement::SystemCall(syscall) => match syscall {
                SysCall::Exit(expression) => {
                    self.expr(expression);
                    self.instructions(vec![Instructions::Syscall]);
                }
            },
        }
    }
}
