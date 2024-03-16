mod assembly;
mod utterances;
use assembly::{Instructions, Registers};
use logos::{Lexer, Logos, Skip};
use std::{collections::HashMap, fmt::Debug, process::Command};
use utterances::{Construct, Expressions};
use utterances::{Statement, SysCalls};

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
enum Kind {
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

    #[token(";")]
    SemiColon,
}

#[derive(Debug)]
enum ParserError {
    NoStatementsFound,
}

struct Parser<'a> {
    lexer: Lexer<'a, Kind>,
}

impl<'a> Parser<'a> {
    fn new(lexer: Lexer<'a, Kind>) -> Self {
        Self { lexer }
    }

    fn expect_token(&mut self) -> Kind {
        if let Some(token) = self.lexer.next() {
            token.expect("expected token but got None")
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

    fn parse_program(&mut self) -> Construct {
        let mut statements = Vec::new();

        while let Some(kind) = self.lexer.next() {
            statements.push(self.parse_statement(kind.expect("expected kind but got None")));
        }

        if statements.is_empty() {
            panic!("{:?}", ParserError::NoStatementsFound);
        }

        Construct::Program(statements)
    }

    fn parse_statement(&mut self, kind: Kind) -> Statement {
        let statement = match kind {
            Kind::KeywordLet => self.parse_let(),
            Kind::SystemCall => self.parse_syscall(),
            _ => panic!(
                "unexpected statement kind {:?} value {:?}",
                kind,
                self.lexer.slice()
            ),
        };
        let _ = self.expect_token_kind(Kind::SemiColon);
        statement
    }

    fn parse_let(&mut self) -> Statement {
        self.expect_token_kind(Kind::AliasSnakeCase);
        let alias = self.lexer.slice().to_string();
        let _ = self.expect_token_kind(Kind::Assign);

        let value = self.parse_expr();

        Statement::Let(alias, value)
    }

    fn parse_expr(&mut self) -> Expressions {
        let kind = self.expect_token();
        match kind {
            Kind::Number => Expressions::U32(self.lexer.slice().parse().expect("expected u32")),
            Kind::AliasSnakeCase => Expressions::Alias(self.lexer.slice().to_string()),
            _ => panic!("unexpected expression kind {:?}", kind),
        }
    }

    fn parse_syscall(&mut self) -> Statement {
        self.expect_token_kind(Kind::AliasSnakeCase);
        let value = self.lexer.slice();
        match value {
            "exit" => {
                let expression = self.parse_expr();
                Statement::SystemCall(SysCalls::Exit(expression))
            }
            _ => panic!("unknown syscall token value {:?}", value),
        }
    }
}

#[derive(Debug)]
struct Var {
    stack_location: u32,
}

struct Transpiler {
    output: String,
    vars: HashMap<String, Var>,

    stack_len: usize,
    max_stack_size: usize,
}

impl Transpiler {
    fn new() -> Self {
        let max_stack_size = 100; // TODO: make this configurable
        Self {
            output: String::new(),
            vars: HashMap::new(),
            stack_len: 0,
            max_stack_size,
        }
    }

    pub fn transpile_instructions(&mut self, instructions: Vec<Instructions>) {
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
                    self.transpile_instructions(vec![
                        Instructions::Mov(Registers::Rax.to_string(), 60),
                        Instructions::Pop(Registers::Rdi.to_string()),
                    ]);
                    self.output += "\tsyscall\n";
                }
            }
        }

        self.output += "\n";
    }

    fn transpile_construct(&mut self, construct: Construct) -> String {
        self.output += &format!(
            "# {} / {}\n\n",
            chrono::Local::now().format("%H:%M:%S"),
            chrono::Local::now().format("%e %b %Y"),
        );

        match construct {
            Construct::Program(statements) => {
                self.output += "global _start\n_start:\n";
                for statement in statements {
                    self.transpile_statement(statement);
                }
            }
        }

        self.transpile_statement(Statement::SystemCall(SysCalls::Exit(Expressions::U32(0))));

        self.output.to_string()
    }

    fn transpile_expr(&mut self, expression: Expressions) {
        match expression {
            Expressions::U32(value) => {
                self.transpile_instructions(vec![
                    Instructions::Mov(Registers::Rax.to_string(), value),
                    Instructions::Push(Registers::Rax.to_string()),
                ]);
            }
            Expressions::Alias(alias) => {
                if let Some(var) = self.vars.get(&alias) {
                    self.transpile_instructions(vec![Instructions::Push(format!(
                        "QWORD [{}+{}]",
                        Registers::Rsp,
                        (self.stack_len - var.stack_location as usize - 1) * 8
                    ))]);
                } else {
                    panic!("undeclared alias '{}'", alias);
                }
            }
            _ => unimplemented!(),
        }
    }

    fn transpile_statement(&mut self, statement: Statement) {
        match statement {
            Statement::Let(alias, expression) => {
                if let Some(var) = self.vars.get(&alias) {
                    panic!("alias '{}' already declared {:?}", alias, var);
                }

                let stack_location = self.stack_len as u32;
                self.vars.insert(alias.to_owned(), Var { stack_location });
                self.transpile_expr(expression);
            }
            Statement::SystemCall(syscall) => match syscall {
                SysCalls::Exit(number) => {
                    self.transpile_expr(number);
                    self.transpile_instructions(vec![Instructions::Syscall]);
                }
            },
        }
    }
}

const SOURCE: &str = include_str!("../code/code.x");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lexer = Kind::lexer(SOURCE);

    let start_time = std::time::Instant::now();
    let mut parser = Parser::new(lexer);
    let ast = parser.parse_program();
    println!("Parser took {:?}", start_time.elapsed());
    std::fs::create_dir_all("ast")?;
    let mut output_file = std::fs::File::create("ast/out.json")?;
    let json = serde_json::to_string(&ast).unwrap();
    std::io::Write::write_all(&mut output_file, json.as_bytes())?;

    let start_time = std::time::Instant::now();
    let mut compiler = Transpiler::new();
    let output = compiler.transpile_construct(ast);
    println!("Compiler took {:?}", start_time.elapsed());

    std::fs::create_dir_all("transpiled")?;
    std::fs::create_dir_all("object")?;
    std::fs::create_dir_all("bin")?;

    let mut output_file = std::fs::File::create("transpiled/out.s")?;
    std::io::Write::write_all(&mut output_file, output.as_bytes())?;

    let nasm = Command::new("nasm")
        .args(&["-felf64", "transpiled/out.s", "-o", "object/out.o"])
        .output()
        .expect("Failed to execute nasm");
    println!("nasm Command Output: {:?}", nasm);
    println!("nasm Stdout: {}", String::from_utf8_lossy(&nasm.stdout));
    println!("nasm Stderr: {}", String::from_utf8_lossy(&nasm.stderr));

    let ld = Command::new("ld")
        .args(&["object/out.o", "-o", "bin/out"])
        .output()
        .expect("Failed to execute ld");
    println!("ld Command Output: {:?}", ld);
    println!("ld Stdout: {}", String::from_utf8_lossy(&ld.stdout));
    println!("ld Stderr: {}", String::from_utf8_lossy(&ld.stderr));

    let bin = Command::new("./bin/out")
        .output()
        .expect("Failed to execute binary");
    println!("bin Command Output: {:?}", bin);
    println!("bin Stdout: {}", String::from_utf8_lossy(&bin.stdout));
    println!("bin Stderr: {}", String::from_utf8_lossy(&bin.stderr));

    let exit_status = bin.status.code().unwrap();
    println!("Exit status {:?}", exit_status);

    // println!("{:?}", to_string!(SysCalls::exit));

    Ok(())
}
