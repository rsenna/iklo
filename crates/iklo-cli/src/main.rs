use std::io::{self, Write};

use iklo_parser::{parse, ParseError};
use iklo_runtime::RuntimeImage;

fn main() {
    let mut args = std::env::args().skip(1);
    if let Some(path) = args.next() {
        if let Err(err) = run_file(&path) {
            eprintln!("iklo: {err}");
            std::process::exit(1);
        }
        return;
    }

    run_repl();
}

fn run_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let src = std::fs::read_to_string(path)?;
    let program = parse(&src)?;
    let mut image = RuntimeImage::new();
    let value = image.eval_in_tx(&program)?;
    println!("{value}");
    Ok(())
}

fn run_repl() {
    println!("iklo IK0 REPL");
    println!("commands (at empty prompt): :quit, :revision, :env");
    println!("(incomplete input continues on the next line; a blank line cancels)\n");

    let mut image = RuntimeImage::new();
    let stdin = io::stdin();
    let mut buffer = String::new();
    let mut line = String::new();

    loop {
        let prompt = if buffer.is_empty() { "iklo> " } else { "iklo. " };
        print!("{prompt}");
        if io::stdout().flush().is_err() {
            break;
        }

        line.clear();
        let read = stdin.read_line(&mut line);
        let Ok(count) = read else {
            break;
        };
        if count == 0 {
            if !buffer.is_empty() {
                eprintln!("(discarding incomplete input)");
            }
            break;
        }

        if buffer.is_empty() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed == ":quit" {
                break;
            }
            if trimmed == ":revision" {
                println!("{}", image.revision());
                continue;
            }
            if trimmed == ":env" {
                for (k, v) in image.bindings() {
                    println!(":{k} = {v}");
                }
                continue;
            }
            buffer.push_str(&line);
        } else {
            if line.trim().is_empty() {
                buffer.clear();
                continue;
            }
            buffer.push_str(&line);
        }

        match parse(&buffer) {
            Ok(program) => {
                if !program.is_empty() {
                    match image.eval_in_tx(&program) {
                        Ok(value) => println!("{value}"),
                        Err(err) => eprintln!("error: {err}"),
                    }
                }
                buffer.clear();
            }
            Err(ParseError::UnexpectedEof) => {
                // Incomplete input — keep buffer and re-prompt with continuation.
            }
            Err(err) => {
                eprintln!("parse error: {err}");
                buffer.clear();
            }
        }
    }
}

