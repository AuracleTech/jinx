use logos::Lexer;

use crate::utterances::{
    ArithmeticOperator, Construct, Expression, Kind, Literal, Statement, SysCall, Term,
};

#[derive(Debug)]
enum ParserError {
    NoStatementsFound,
}

pub struct Parser<'a> {
    lexer: Lexer<'a, Kind>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a, Kind>) -> Self {
        Self { lexer }
    }

    fn expect_token(&mut self) -> Kind {
        if let Some(token) = self.lexer.next() {
            match token {
                Ok(kind) => kind,
                Err(error) => panic!("{:?}", error),
            }
        } else {
            panic!("expected token but got None");
        }
    }

    fn expect_token_kind(&mut self, expected: Kind) -> Kind {
        let kind = self.expect_token();
        if kind == expected {
            kind
        } else {
            panic!(
                "expected {:?} but got {:?} value {:?}",
                expected,
                kind,
                self.lexer.slice()
            );
        }
    }

    pub fn program(&mut self) -> Construct {
        let mut statements = Vec::new();

        while let Some(kind) = self.lexer.next() {
            statements.push(self.statement(kind.expect("expected kind but got error")));
        }

        if statements.is_empty() {
            panic!("{:?}", ParserError::NoStatementsFound);
        }

        // TODO : If there's no syscall exit, append one

        Construct::Program(statements)
    }

    fn statement(&mut self, kind: Kind) -> Statement {
        let statement = match kind {
            Kind::KeywordLet => self.assign(),
            Kind::SystemCall => self.syscall(),
            _ => panic!(
                "unexpected statement kind {:?} value {:?}",
                kind,
                self.lexer.slice()
            ),
        };
        statement
    }

    fn assign(&mut self) -> Statement {
        self.expect_token_kind(Kind::AliasSnakeCase);
        let alias = self.lexer.slice().to_string();
        let _ = self.expect_token_kind(Kind::Assign);
        let right = self.expression();
        Statement::Let(alias, right)
    }

    fn term(&mut self) -> Term {
        let token = self.expect_token();
        match token {
            Kind::Number => {
                let value = self.lexer.slice().parse().expect("Failed to parse number");
                Term::Literal(Literal::U32(value))
            }
            Kind::AliasSnakeCase => Term::Alias(self.lexer.slice().to_string()),
            // Kind::ParenthesisOpen => {
            //     let expr = self.expression();
            //     left = self.expect_token();
            //     if left != Kind::ParenthesisClose {
            //         panic!("Expected ')' after expression, found {:?}", left);
            //     }
            //     expr
            // }
            _ => panic!("Unexpected token {:?} in expression", token),
        }
    }

    fn expression(&mut self) -> Expression {
        let left = Expression::Term(self.term());

        let next = self.expect_token();
        let operator = match next {
            Kind::Add => ArithmeticOperator::Add,
            Kind::Sub => ArithmeticOperator::Sub,
            Kind::Mul => ArithmeticOperator::Mul,
            Kind::Div => ArithmeticOperator::Div,
            Kind::SemiColon => return left,
            _ => panic!("Unexpected token {:?} in expression", next),
        };

        let right = self.expression();

        Expression::BinaryOp(operator, Box::new(left), Box::new(right))
    }

    fn syscall(&mut self) -> Statement {
        self.expect_token_kind(Kind::AliasSnakeCase);
        let value = self.lexer.slice();
        match value {
            "exit" => {
                let expr = self.expression();
                Statement::SystemCall(SysCall::Exit(expr))
            }
            _ => panic!("unknown syscall token value {:?}", value),
        }
    }
}
