use jayce::{Duo, Token, Tokenizer};
use std::fmt::Debug;
use strum_macros::EnumString;

macro_rules! to_string {
    ($enum_type:ident::$variant:ident) => {
        stringify!($variant)
    };
}

#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Whitespace,
    CommentLine,
    CommentBlock,
    Newline,

    // #[cfg(target_os = "linux")]
    SystemCall,

    ParenthesisOpen,
    ParenthesisClose,

    BracketOpen,
    BracketClose,

    Alias,
    Number,

    Assign,
    Add,
    Sub,

    SemiColon,
}

lazy_static::lazy_static! {
     static ref DUOS: Vec<Duo<Kind>> = vec![
        Duo::new(Kind::Whitespace, r"^[^\S\n]+", false),
        Duo::new(Kind::CommentLine, r"^//(.*)", false),
        Duo::new(Kind::CommentBlock, r"^/\*(.|\n)*?\*/", false),
        Duo::new(Kind::Newline, r"^\n", false),

        // #[cfg(target_os = "linux")]
        Duo::new(Kind::SystemCall, r"^syscall", true),

        Duo::new(Kind::ParenthesisOpen, r"^\(", true),
        Duo::new(Kind::ParenthesisClose, r"^\)", true),

        Duo::new(Kind::BracketOpen, r"^\{", true),
        Duo::new(Kind::BracketClose, r"^\}", true),

        Duo::new(Kind::Alias, r"^[a-zA-Z_]+", true),
        Duo::new(Kind::Number, r"^[0-9]+", true),

        Duo::new(Kind::Assign, r"^=", true),
        Duo::new(Kind::Add, r"^\+", true),
        Duo::new(Kind::Sub, r"^-", true),

        Duo::new(Kind::SemiColon, r"^;", true),

    ];
}

#[derive(Debug)]
enum ParserError {
    NoStatementsFound,
}

// #[cfg(target_os = "linux")]
#[derive(Debug, EnumString)]
enum SysCalls {
    #[strum(ascii_case_insensitive)]
    Exit(u32),
}

#[derive(Debug)]
enum Construct {
    Program(Vec<Construct>),
    SystemCall(SysCalls),
}

struct Parser {
    tokenizer: Tokenizer<'static, Kind>,
}

impl Parser {
    fn new(tokenizer: Tokenizer<'static, Kind>) -> Self {
        Self { tokenizer }
    }

    fn expect_token(&mut self) -> Token<Kind> {
        if let Some(token) = self.tokenizer.consume().unwrap() {
            return token;
        } else {
            panic!("expected token but got None");
        }
    }

    fn expect_token_kind(&mut self, kind: Kind) -> Token<Kind> {
        let token = self.expect_token();
        if token.kind == &kind {
            return token;
        } else {
            panic!(
                "expected {:?} but got {:?} at line {} column {}",
                kind, token.kind, token.pos.0, token.pos.1
            );
        }
    }

    fn parse_program(&mut self) -> Construct {
        let mut statements = Vec::new();

        while let Some(_) = self.tokenizer.peek().unwrap() {
            statements.push(self.parse_statement());
        }

        if statements.is_empty() {
            panic!("{:?}", ParserError::NoStatementsFound);
        }

        Construct::Program(statements)
    }

    fn parse_statement(&mut self) -> Construct {
        let token = self.expect_token();
        let construct = match token.kind {
            Kind::SystemCall => self.parse_syscall(),
            _ => panic!("unexpected statement as {:?}", token),
        };
        let _ = self.expect_token_kind(Kind::SemiColon);
        construct
    }

    fn parse_syscall(&mut self) -> Construct {
        let syscall_alias = self.expect_token_kind(Kind::Alias);
        match syscall_alias.value {
            to_string!(SysCalls::Exit) => {
                let _number = self.expect_token_kind(Kind::Number);
                Construct::SystemCall(SysCalls::Exit(1))
            }
            // Other syscalls
            _ => panic!("unknown syscall {:?}", syscall_alias.value),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = Tokenizer::new(r#"syscall Exit 100;"#, &DUOS);
    let mut parser = Parser::new(tokenizer);
    println!("{:?}", parser.parse_program());

    // println!("{:?}", to_string!(SysCalls::Exit));

    Ok(())
}
