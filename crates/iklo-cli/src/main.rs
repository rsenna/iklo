use std::io::{self, Write};

use iklo_parser::parse;
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
    println!("commands: :quit, :revision, :env\n");

    let mut image = RuntimeImage::new();
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("iklo> ");
        if io::stdout().flush().is_err() {
            break;
        }

        line.clear();
        let read = stdin.read_line(&mut line);
        let Ok(count) = read else {
            break;
        };
        if count == 0 {
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input == ":quit" {
            break;
        }
        if input == ":revision" {
            println!("{}", image.revision());
            continue;
        }
        if input == ":env" {
            for (k, v) in image.bindings() {
                println!("{k} = {v}");
            }
            continue;
        }

        match parse(input) {
            Ok(program) => match image.eval_in_tx(&program) {
                Ok(value) => println!("{value}"),
                Err(err) => eprintln!("error: {err}"),
            },
            Err(err) => eprintln!("parse error: {err}"),
        }
    }
}

