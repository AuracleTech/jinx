mod assembly;
mod ast_display;
mod parser;
mod transpiler;
mod utterances;
use logos::Logos;
use parser::Parser;
use std::process::Command;
use transpiler::Transpiler;
use utterances::Kind;

macro_rules! time_and_print {
    ($label:expr, $block:expr) => {{
        let start_time = std::time::Instant::now();
        let result = $block;
        println!("{} took {:?}", $label, start_time.elapsed());
        result
    }};
}

const FOLDERS: [&str; 4] = ["ast", "transpiled", "object", "bin"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Incorrect usage. Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    let source_code = std::fs::read_to_string(filename)?;

    for folder in &FOLDERS {
        std::fs::create_dir_all(folder)?;
    }

    let lexer = Kind::lexer(&source_code);

    let mut parser = Parser::new(lexer);
    let ast = time_and_print!("Parser", { parser.program() });

    println!("{}", ast);

    let mut ast_output = std::fs::File::create("ast/out.json")?;
    let serialized_ast = serde_json::to_string(&ast).unwrap();
    std::io::Write::write_all(&mut ast_output, serialized_ast.as_bytes())?;

    let mut transpiler = Transpiler::new();
    let transpiled = time_and_print!("Transpiler", { transpiler.construct(ast) });

    let mut asm_output = std::fs::File::create("transpiled/out.s")?;
    std::io::Write::write_all(&mut asm_output, transpiled.as_bytes())?;

    let assembler = Command::new("nasm")
        .args(&["-felf64", "transpiled/out.s", "-o", "object/out.o"])
        .output()
        .expect("Failed to execute nasm");
    println!("nasm Command Output: {:?}", assembler);
    println!(
        "nasm Stdout: {}",
        String::from_utf8_lossy(&assembler.stdout)
    );
    println!(
        "nasm Stderr: {}",
        String::from_utf8_lossy(&assembler.stderr)
    );

    let linker = Command::new("ld")
        .args(&["object/out.o", "-o", "bin/out"])
        .output()
        .expect("Failed to execute ld");
    println!("ld Command Output: {:?}", linker);
    println!("ld Stdout: {}", String::from_utf8_lossy(&linker.stdout));
    println!("ld Stderr: {}", String::from_utf8_lossy(&linker.stderr));

    let runner = Command::new("./bin/out")
        .output()
        .expect("Failed to execute binary");
    println!("bin Command Output: {:?}", runner);
    println!("bin Stdout: {}", String::from_utf8_lossy(&runner.stdout));
    println!("bin Stderr: {}", String::from_utf8_lossy(&runner.stderr));

    let exit_status = runner.status.code().unwrap();
    println!("Exit status {:?}", exit_status);

    Ok(())
}
