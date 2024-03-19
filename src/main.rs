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

const SOURCE: &str = include_str!("../code/code.x");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lexer = Kind::lexer(SOURCE);

    let start_time = std::time::Instant::now();
    let mut parser = Parser::new(lexer);
    let ast = parser.program();
    println!("Parser took {:?}", start_time.elapsed());

    println!("{}", ast);
    std::fs::create_dir_all("ast")?;
    let mut output_file = std::fs::File::create("ast/out.json")?;
    let json = serde_json::to_string(&ast).unwrap();
    std::io::Write::write_all(&mut output_file, json.as_bytes())?;

    let start_time = std::time::Instant::now();
    let mut compiler = Transpiler::new();
    let output = compiler.construct(ast);
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

    Ok(())
}
