mod assembly;
mod utterances;
use assembly::{Instructions, Registers};
use jayce::{Duo, Token, Tokenizer};
use std::{collections::HashMap, fmt::Debug, process::Command, sync::OnceLock};
use utterances::{Construct, Expressions};
use utterances::{Statement, SysCalls};

#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Whitespace,
    CommentLine,
    CommentBlock,
    Newline,

    KeywordLet,

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
            //
            Duo::new(Kind::KeywordLet, r"^let", true),
            //
            #[cfg(target_os = "linux")]
            Duo::new(Kind::SystemCall, r"^syscall", true),
            //
            Duo::new(Kind::ParenthesisOpen, r"^\(", true),
            Duo::new(Kind::ParenthesisClose, r"^\)", true),
            //
            Duo::new(Kind::BracketOpen, r"^\{", true),
            Duo::new(Kind::BracketClose, r"^\}", true),
            //
            Duo::new(Kind::AliasSnakeCase, r"^[a-z][a-zA-Z0-9_]*", true),
            Duo::new(Kind::Number, r"^[0-9]+", true),
            //
            Duo::new(Kind::Assign, r"^=", true),
            Duo::new(Kind::Add, r"^\+", true),
            Duo::new(Kind::Sub, r"^-", true),
            //
            Duo::new(Kind::SemiColon, r"^;", true),
        ]
    })
}

#[derive(Debug)]
enum ParserError {
    NoStatementsFound,
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

    fn expect_token_kind(&mut self, kind: Kind) -> Result<Token<Kind>, String> {
        let token = self.expect_token();
        if token.kind == &kind {
            Ok(token)
        } else {
            Err(format!("expected {:?} but got {:?}", kind, token))
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

    fn parse_statement(&mut self) -> Statement {
        let token = self.expect_token();
        let statement = match token.kind {
            Kind::KeywordLet => self.parse_let(),
            Kind::SystemCall => self.parse_syscall(),
            _ => panic!("unexpected statement as {:?}", token),
        };
        let _ = self.expect_token_kind(Kind::SemiColon);
        statement
    }

    fn parse_let(&mut self) -> Statement {
        let alias = self
            .expect_token_kind(Kind::AliasSnakeCase)
            .unwrap()
            .value
            .to_string();
        let _ = self.expect_token_kind(Kind::Assign);

        let value = self.parse_expr();

        Statement::Let(alias, value)
    }

    fn parse_expr(&mut self) -> Expressions {
        let token = self.expect_token();
        match token.kind {
            Kind::Number => Expressions::U32(token.value.parse().unwrap()),
            Kind::AliasSnakeCase => Expressions::Alias(token.value.to_string()),
            _ => panic!("unexpected expression as {:?}", token),
        }
    }

    fn parse_syscall(&mut self) -> Statement {
        let token = self.expect_token_kind(Kind::AliasSnakeCase).unwrap();
        match token.value {
            "exit" => {
                let expression = self.parse_expr();
                Statement::SystemCall(SysCalls::Exit(expression))
            }
            _ => panic!("unknown syscall token value {:?}", token),
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

    pub fn stringify_instructions(&mut self, instructions: Vec<Instructions>) {
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
                    self.output += "\tsyscall\n\n";
                }
            }
        }

        self.output += "\n";
    }

    fn transpile_construct(&mut self, construct: Construct) -> String {
        self.output += &format!(
            "# {} at {}\n",
            chrono::Local::now().format("%e %b %Y"),
            chrono::Local::now().format("%H:%M:%S")
        );

        match construct {
            Construct::Program(statements) => {
                self.output += "\nglobal _start\n_start:\n";
                for statement in statements {
                    self.transpile_statement(statement);
                }
            }
        }

        self.output.to_string()
    }

    fn transpile_statement(&mut self, statement: Statement) {
        match statement {
            Statement::Let(alias, expression) => {
                if let Some(var) = self.vars.get(&alias) {
                    panic!("variable named '{}' already declared {:?}", alias, var);
                }

                let stack_location = self.stack_len as u32;
                self.vars.insert(alias.to_owned(), Var { stack_location });

                let u32_value = match expression {
                    Expressions::U32(value) => value,
                    _ => panic!("expected integer expression but got {:?}", expression),
                };

                self.stringify_instructions(vec![
                    Instructions::Mov(Registers::Rax, u32_value),
                    Instructions::Push(Registers::Rax),
                ])

                // format!("\n\tmov rax, {}\n\tpush rax\n", number)
            }
            Statement::SystemCall(syscall) => match syscall {
                SysCalls::Exit(_number) => self.stringify_instructions(vec![
                    Instructions::Mov(Registers::Rax, 60),
                    Instructions::Pop(Registers::Rdi),
                    Instructions::Syscall,
                ]),
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

    let mut compiler = Transpiler::new();
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
    }

    // println!("{:?}", to_string!(SysCalls::exit));

    Ok(())
}
