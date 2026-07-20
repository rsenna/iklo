use rustyline::error::ReadlineError;

use iklo_parser::{parse, ParseError};
use iklo_runtime::RuntimeImage;

/// REPL input history, persisted in the current working directory.
const HISTORY_FILE: &str = ".iklo_history";

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

    let history_path = std::path::Path::new(HISTORY_FILE);
    let had_pre_existing_file = history_path.exists();
    let _ = rl.load_history(history_path); // missing/unreadable is non-fatal

    let mut image = RuntimeImage::new();
    let mut buffer = String::new();
    let mut entries_added_this_session = false;

    loop {
        let prompt = if buffer.is_empty() { "iklo> " } else { "iklo. " };
        let line = match rl.readline(prompt) {
            Ok(line) => line,
            Err(ReadlineError::Eof) => {
                if !buffer.is_empty() {
                    eprintln!("(discarding incomplete input)");
                }
                break;
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C cancels the current multi-line input (like a blank
                // line does) and reprompts — it must not exit and lose
                // `image`'s defined bindings. Only Ctrl-D (Eof) exits.
                if !buffer.is_empty() {
                    eprintln!("(discarding incomplete input)");
                    buffer.clear();
                }
                continue;
            }
            Err(_) => break,
        };

        if buffer.is_empty() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let _ = rl.add_history_entry(line.as_str());
            entries_added_this_session = true;
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
            push_repl_line(&mut buffer, &line);
        } else {
            if line.trim().is_empty() {
                buffer.clear();
                continue;
            }
            let _ = rl.add_history_entry(line.as_str());
            entries_added_this_session = true;
            push_repl_line(&mut buffer, &line);
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

    if should_save_history(had_pre_existing_file, entries_added_this_session) {
        let _ = rl.save_history(history_path);
    }
}

/// Appends `line` (as returned by `rustyline::Editor::readline`, which has
/// no trailing newline) to `buffer`, restoring the newline character the
/// parser's soft-terminator and comment grammar rules depend on — matching
/// exactly what `stdin.read_line()` would have included.
fn push_repl_line(buffer: &mut String, line: &str) {
    buffer.push_str(line);
    buffer.push('\n');
}

/// Whether to call `save_history`: skip only when there was no pre-existing
/// history file *and* this session added no entries — otherwise a session
/// with zero input would create an empty `.iklo_history` (spec.md User
/// Story 1 Acceptance Scenario 3).
fn should_save_history(had_pre_existing_file: bool, entries_added_this_session: bool) -> bool {
    had_pre_existing_file || entries_added_this_session
}

#[cfg(test)]
mod tests {
    use super::*;
    use iklo_runtime::Value;

    #[test]
    fn should_save_history_skips_when_nothing_to_save() {
        assert!(!should_save_history(false, false));
    }

    #[test]
    fn should_save_history_saves_when_entries_added() {
        assert!(should_save_history(false, true));
    }

    #[test]
    fn should_save_history_saves_when_file_pre_existed() {
        assert!(should_save_history(true, false));
    }

    #[test]
    fn should_save_history_saves_when_both() {
        assert!(should_save_history(true, true));
    }

    #[test]
    fn load_history_from_missing_path_does_not_error() {
        let path = std::env::temp_dir()
            .join(format!("iklo-test-history-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);

        let mut rl = rustyline::DefaultEditor::new().expect("editor");
        assert!(rl.load_history(&path).is_err());
    }

    #[test]
    fn push_repl_line_appends_a_real_newline() {
        let mut buffer = String::new();
        push_repl_line(&mut buffer, "let :x be 1 +");
        push_repl_line(&mut buffer, "2");
        assert_eq!(buffer, "let :x be 1 +\n2\n");
    }

    /// Regression test for the newline-restoration fix: a `#` comment only
    /// stops at a real newline. If push_repl_line joined lines with a space
    /// (or nothing) instead of '\n', the comment on the first line would
    /// swallow the second line whole and this would fail to parse.
    #[test]
    fn multiline_continuation_preserves_newline_across_comment_boundary() {
        let mut buffer = String::new();
        push_repl_line(&mut buffer, "let :x be 1 +  # comment");
        push_repl_line(&mut buffer, "2");

        let program = parse(&buffer).expect("parse");
        let mut image = RuntimeImage::new();
        let value = image.eval_in_tx(&program).expect("eval");
        assert_eq!(value, Value::Number(3.0));
    }
}

