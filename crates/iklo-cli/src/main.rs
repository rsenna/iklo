use rustyline::error::ReadlineError;

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
    println!("commands (at empty prompt): .quit, .revision, .env");
    println!("(incomplete input continues on the next line; a blank line cancels)\n");

    let mut rl = match rustyline::DefaultEditor::new() {
        Ok(rl) => rl,
        Err(err) => {
            eprintln!("iklo: failed to start line editor: {err}");
            return;
        }
    };

    let mut image = RuntimeImage::new();
    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() { "iklo> " } else { "iklo. " };
        let line = match rl.readline(prompt) {
            Ok(line) => line,
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                if !buffer.is_empty() {
                    eprintln!("(discarding incomplete input)");
                }
                break;
            }
            Err(_) => break,
        };

        if buffer.is_empty() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed == ".quit" {
                break;
            }
            if trimmed == ".revision" {
                println!("{}", image.revision());
                continue;
            }
            if trimmed == ".env" {
                for (k, v) in image.bindings() {
                    println!(":{k} = {v}");
                }
                continue;
            }
            // rustyline strips the trailing newline readline() would otherwise
            // include; re-add it so the parser sees the same source text across
            // multi-line continuation as it did reading raw stdin.
            buffer.push_str(&line);
            buffer.push('\n');
        } else {
            if line.trim().is_empty() {
                buffer.clear();
                continue;
            }
            buffer.push_str(&line);
            buffer.push('\n');
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

