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
    // A missing OR unreadable/corrupt file both fail to load; either way we
    // must not later overwrite whatever's on disk with an empty in-memory
    // history — `had_existing_history` tracks whether we actually have
    // something loaded, not merely whether a file existed.
    let had_existing_history = rl.load_history(history_path).is_ok();

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
            if trimmed == ".quit" {
                record_history_entry(&mut rl, &mut entries_added_this_session, line.as_str());
                break;
            }
            if trimmed == ".revision" {
                record_history_entry(&mut rl, &mut entries_added_this_session, line.as_str());
                println!("{}", image.revision());
                continue;
            }
            if trimmed == ".env" {
                record_history_entry(&mut rl, &mut entries_added_this_session, line.as_str());
                for (k, v) in image.bindings() {
                    println!(":{k} = {v}");
                }
                continue;
            }
            // Ordinary expression input is NOT added to history line-by-line
            // here — a multi-line expression must recall as one unit (see
            // below), not as disconnected fragments, and a cancelled
            // continuation (blank line / Ctrl-C) must leave no trace.
            push_repl_line(&mut buffer, &line);
        } else {
            if line.trim().is_empty() {
                buffer.clear();
                continue;
            }
            push_repl_line(&mut buffer, &line);
        }

        match parse(&buffer) {
            Ok(program) => {
                record_history_entry(&mut rl, &mut entries_added_this_session, buffer.trim_end());
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
                // Record the failed attempt too, so the user can recall and
                // fix it with Up-arrow rather than retyping from scratch.
                record_history_entry(&mut rl, &mut entries_added_this_session, buffer.trim_end());
                eprintln!("parse error: {err}");
                buffer.clear();
            }
        }
    }

    if should_save_history(had_existing_history, entries_added_this_session) {
        if let Err(err) = rl.save_history(history_path) {
            eprintln!("iklo: failed to save history: {err}");
        }
    }
}

/// Adds `entry` to `rl`'s history and flips `entries_added_this_session` to
/// `true` only when rustyline confirms it was actually recorded (not
/// silently ignored, e.g. as a duplicate of the previous entry).
fn record_history_entry(
    rl: &mut rustyline::DefaultEditor,
    entries_added_this_session: &mut bool,
    entry: &str,
) {
    if let Ok(true) = rl.add_history_entry(entry) {
        *entries_added_this_session = true;
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

/// Whether to call `save_history`: skip only when we didn't successfully
/// load existing history *and* this session added no entries — otherwise a
/// session with zero input would create an empty `.iklo_history` (spec.md
/// User Story 1 Acceptance Scenario 3), or worse, overwrite a history file
/// that existed but failed to load (corrupt/unreadable) with nothing.
fn should_save_history(had_existing_history: bool, entries_added_this_session: bool) -> bool {
    had_existing_history || entries_added_this_session
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
    fn load_history_from_missing_path_returns_error() {
        let path = std::env::temp_dir()
            .join(format!("iklo-test-history-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);

        let mut rl = rustyline::DefaultEditor::new().expect("editor");
        assert!(rl.load_history(&path).is_err());
    }

    #[test]
    fn record_history_entry_flips_flag_on_success() {
        let mut rl = rustyline::DefaultEditor::new().expect("editor");
        let mut entries_added_this_session = false;
        record_history_entry(&mut rl, &mut entries_added_this_session, "let :x be 1");
        assert!(entries_added_this_session);
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

