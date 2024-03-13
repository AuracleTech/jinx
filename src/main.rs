use jayce::{Duo, Token, Tokenizer};
use std::{fmt::Debug, process::Command, sync::OnceLock};

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

    #[cfg(target_os = "linux")]
    SystemCall,

    ParenthesisOpen,
    ParenthesisClose,

    BracketOpen,
    BracketClose,

    AliasSnakeCase,
    Number,

    Assign,
    Add,
    Sub,

    SemiColon,
}

fn duos() -> &'static Vec<Duo<Kind>> {
    static DUOS: OnceLock<Vec<Duo<Kind>>> = OnceLock::new();
    DUOS.get_or_init(|| {
        vec![
            Duo::new(Kind::Whitespace, r"^[^\S\n]+", false),
            Duo::new(Kind::CommentLine, r"^//(.*)", false),
            Duo::new(Kind::CommentBlock, r"^/\*(.|\n)*?\*/", false),
            Duo::new(Kind::Newline, r"^\n", false),
            #[cfg(target_os = "linux")]
            Duo::new(Kind::SystemCall, r"^syscall", true),
            Duo::new(Kind::ParenthesisOpen, r"^\(", true),
            Duo::new(Kind::ParenthesisClose, r"^\)", true),
            Duo::new(Kind::BracketOpen, r"^\{", true),
            Duo::new(Kind::BracketClose, r"^\}", true),
            Duo::new(Kind::AliasSnakeCase, r"^[a-z][a-zA-Z0-9_]*", true),
            Duo::new(Kind::Number, r"^[0-9]+", true),
            Duo::new(Kind::Assign, r"^=", true),
            Duo::new(Kind::Add, r"^\+", true),
            Duo::new(Kind::Sub, r"^-", true),
            Duo::new(Kind::SemiColon, r"^;", true),
        ]
    })
}

#[derive(Debug)]
enum ParserError {
    NoStatementsFound,
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
#[allow(non_camel_case_types)]
enum SysCalls {
    exit(u32),
}

#[derive(Debug)]
enum Construct {
    Program(Vec<Statement>),
}

#[derive(Debug)]
enum Statement {
    #[cfg(target_os = "linux")]
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

    fn expect_token_u32(&mut self) -> u32 {
        let token = self.expect_token_kind(Kind::Number);
        let number = token
            .value
            .parse::<u32>()
            .expect(format!("expected u32 but got {:?}", token).as_str());
        number
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

    fn parse_statement(&mut self) -> Statement {
        let token = self.expect_token();
        let statement = match token.kind {
            Kind::SystemCall => self.parse_syscall(),
            _ => panic!("unexpected statement as {:?}", token),
        };
        let _ = self.expect_token_kind(Kind::SemiColon);
        statement
    }

    fn parse_syscall(&mut self) -> Statement {
        let syscall_alias = self.expect_token_kind(Kind::AliasSnakeCase);
        match syscall_alias.value {
            to_string!(SysCalls::exit) => {
                let number = self.expect_token_u32();
                Statement::SystemCall(SysCalls::exit(number))
            }
            // Other syscalls
            _ => panic!("unknown syscall {:?}", syscall_alias.value),
        }
    }
}

struct Transpiler;

impl Transpiler {
    fn transpile_construct(&mut self, construct: Construct) -> String {
        let mut output = String::from(format!(
            "# {} {}\n\n",
            chrono::Local::now().format("%e %B %Y"),
            chrono::Local::now().format("%H:%M:%S")
        ));

        match construct {
            Construct::Program(statements) => {
                output += &format!("global _start\n_start:\n");
                for statement in statements {
                    output += self.transpile_statement(statement).as_str();
                }
            }
        }

        output.clone()
    }

    fn transpile_statement(&mut self, statement: Statement) -> String {
        match statement {
            Statement::SystemCall(syscall) => match syscall {
                SysCalls::exit(number) => {
                    format!("\tmov rax, 60\n\tmov rdi, {}\n\tsyscall\n\n", number)
                }
            },
        }
    }
}

const SOURCE: &str = include_str!("../code/code.x");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = Tokenizer::new(SOURCE, duos());

    let mut parser = Parser::new(tokenizer);
    let ast = parser.parse_program();
    println!("AST {:?}", ast);

    let mut compiler = Transpiler {};
    let output = compiler.transpile_construct(ast);
    println!("ASM {:?}", output);

    std::fs::create_dir_all("transpiled")?;
    std::fs::create_dir_all("object")?;
    std::fs::create_dir_all("bin")?;

    #[cfg(target_os = "linux")]
    {
        let mut output_file = std::fs::File::create("transpiled/out.s")?;
        std::io::Write::write_all(&mut output_file, output.as_bytes())?;

        let nasm = Command::new("nasm")
            .args(&["-felf64", "transpiled/out.s", "-o", "object/out.o"])
            .output()
            .expect("Failed to execute nasm");
        println!("{:?}", nasm);

        println!("nasm Stdout: {}", String::from_utf8_lossy(&nasm.stdout));
        println!("nasm Stderr: {}", String::from_utf8_lossy(&nasm.stderr));

        let ld = Command::new("ld")
            .args(&["object/out.o", "-o", "bin/out"])
            .output()
            .expect("Failed to execute ld");
        println!("{:?}", ld);

        println!("ld Stdout: {}", String::from_utf8_lossy(&ld.stdout));
        println!("ld Stderr: {}", String::from_utf8_lossy(&ld.stderr));

        let bin = Command::new("./bin/out")
            .output()
            .expect("Failed to execute binary");
        println!("{:?}", bin);

        println!("bin Stdout: {}", String::from_utf8_lossy(&bin.stdout));
        println!("bin Stderr: {}", String::from_utf8_lossy(&bin.stderr));

        let exit_status = bin.status.code().unwrap();

        println!("Exit status {:?}", exit_status);
    }

    // println!("{:?}", to_string!(SysCalls::exit));

    Ok(())
}
