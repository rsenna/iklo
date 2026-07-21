use std::cell::Cell;

use rustyline::error::ReadlineError;
use rustyline::{
    Completer as DeriveCompleter, Helper as DeriveHelper, Highlighter as DeriveHighlighter,
    Hinter as DeriveHinter, Validator as DeriveValidator,
};

use iklo_parser::{parse, ParseError};
use iklo_runtime::{RuntimeImage, Value};
use iklo_substrate::Substrate;

const REPL_COMMAND_NAMES: [&str; 3] = ["quit", "revision", "env"];

/// Environment-variable fallback for the Turso database path (used only when
/// `--turso-db-url` is absent and `--substrate turso` is selected).
const ENV_TURSO_DB_URL: &str = "IKLO_TURSO_DB_URL";

/// Offers tab-completion for slash-commands, but only when
/// `is_repl_command_position` confirms we're at a fresh (non-continuation)
/// prompt with a leading `/` — gated via `at_fresh_prompt`, which
/// `run_repl()` updates through `rl.helper_mut()` immediately before each
/// `readline()` call (plan.md Key Design Decision 2), since
/// `Completer::complete` only ever sees the current line, never the
/// REPL's own buffer state. Does its own prefix-filtering over partial
/// input rather than calling `parse_repl_command`, per Key Design
/// Decision 1.
struct ReplCompleter {
    at_fresh_prompt: Cell<bool>,
}

impl ReplCompleter {
    fn new() -> Self {
        Self {
            at_fresh_prompt: Cell::new(true),
        }
    }

    fn set_fresh_prompt(&self, fresh: bool) {
        self.at_fresh_prompt.set(fresh);
    }
}

impl rustyline::completion::Completer for ReplCompleter {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<String>)> {
        if !is_repl_command_position(self.at_fresh_prompt.get(), line) {
            return Ok((pos, Vec::new()));
        }
        // Guard against a panic: pos can be 0 if the cursor was moved back
        // to the start of the line (Home/Ctrl-A) before Tab — giving the
        // invalid range 1..0 — and pos can in principle land on a non-UTF-8
        // char boundary. Either way there's nothing sensible to complete.
        let end = pos.min(line.len());
        if end < 1 || !line.is_char_boundary(end) {
            return Ok((pos, Vec::new()));
        }
        let prefix = &line[1..end];
        let matches: Vec<String> = REPL_COMMAND_NAMES
            .iter()
            .filter(|name| name.starts_with(prefix))
            .map(|name| (*name).to_string())
            .collect();
        Ok((1, matches))
    }
}

#[derive(DeriveCompleter, DeriveHinter, DeriveHighlighter, DeriveValidator, DeriveHelper)]
struct ReplHelper {
    #[rustyline(Completer)]
    completer: ReplCompleter,
}

type ReplEditor = rustyline::Editor<ReplHelper, rustyline::history::DefaultHistory>;

/// REPL input history, persisted in the current working directory.
const HISTORY_FILE: &str = ".iklo_history";

/// Which substrate backend the runtime image should use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubstrateKind {
    Memory,
    Turso,
}

/// Raw flags/positionals recovered from the argument vector, before any
/// env-var resolution or cross-flag validation. Deliberately does **not**
/// retain the `--turso-auth-token` value: the token is accepted (so setting it
/// isn't an "unknown flag" error) but is currently a no-op (blocker B001: no
/// remote/auth connectivity in this epic) and must never be stored, printed,
/// or logged — not even via this struct's `Debug`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ParsedArgs {
    /// Raw value of `--substrate` (unvalidated), if provided.
    substrate: Option<String>,
    /// Raw value of `--turso-db-url`, if provided (takes precedence over env).
    turso_db_url: Option<String>,
    /// The positional source-file path, if provided.
    file: Option<String>,
}

/// The fully-resolved run configuration: which backend, the Turso db path
/// (present iff `Turso`), and the optional file to evaluate.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RunConfig {
    substrate: SubstrateKind,
    turso_db_url: Option<String>,
    file: Option<String>,
}

/// Hand-rolled argument parser (no `clap`/`pico-args` dependency for a
/// three-flag surface). Recognizes `--substrate`, `--turso-db-url`,
/// `--turso-auth-token`, and a single positional file path. Returns a
/// human-readable error message (never a token value) on malformed input.
fn parse_args(args: impl Iterator<Item = String>) -> Result<ParsedArgs, String> {
    let mut parsed = ParsedArgs::default();
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--substrate" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--substrate requires a value (memory|turso)".to_string())?;
                parsed.substrate = Some(value);
            }
            "--turso-db-url" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--turso-db-url requires a value (a local file path)".to_string())?;
                parsed.turso_db_url = Some(value);
            }
            "--turso-auth-token" => {
                // Consume and discard the value: accepted for forward
                // compatibility (FR-011) but currently unused (B001). Never
                // stored or echoed — do not surface it in any error message.
                if args.next().is_none() {
                    return Err("--turso-auth-token requires a value".to_string());
                }
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown flag '{other}'"));
            }
            _ => {
                if parsed.file.is_some() {
                    return Err(format!(
                        "unexpected extra argument '{arg}' (a single file path is allowed)"
                    ));
                }
                parsed.file = Some(arg);
            }
        }
    }

    Ok(parsed)
}

/// Applies env-var fallback and cross-flag validation to produce a
/// [`RunConfig`]. `env_turso_db_url` is threaded in as a parameter (rather than
/// read from the process environment here) so this stays a pure, unit-testable
/// function. Flag value takes precedence over the env value (FR); selecting
/// `turso` without a resolvable db path is a hard error, never a silent
/// fallback to memory (FR-007).
fn resolve_config(
    parsed: ParsedArgs,
    env_turso_db_url: Option<String>,
) -> Result<RunConfig, String> {
    let substrate = match parsed.substrate.as_deref() {
        None | Some("memory") => SubstrateKind::Memory,
        Some("turso") => SubstrateKind::Turso,
        Some(other) => {
            return Err(format!(
                "unknown --substrate value '{other}' (expected 'memory' or 'turso')"
            ));
        }
    };

    match substrate {
        SubstrateKind::Memory => Ok(RunConfig {
            substrate,
            // Turso-specific flags/env are ignored entirely in memory mode.
            turso_db_url: None,
            file: parsed.file,
        }),
        SubstrateKind::Turso => {
            // Flag wins over env.
            let db_url = parsed.turso_db_url.or(env_turso_db_url).ok_or_else(|| {
                format!(
                    "--substrate turso requires a database path via --turso-db-url \
                     or the {ENV_TURSO_DB_URL} environment variable"
                )
            })?;
            Ok(RunConfig {
                substrate,
                turso_db_url: Some(db_url),
                file: parsed.file,
            })
        }
    }
}

fn main() {
    let parsed = match parse_args(std::env::args().skip(1)) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("iklo: {err}");
            std::process::exit(2);
        }
    };

    let config = match resolve_config(parsed, std::env::var(ENV_TURSO_DB_URL).ok()) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("iklo: {err}");
            std::process::exit(2);
        }
    };

    match config.substrate {
        SubstrateKind::Memory => {
            // Default path — behaviourally identical to before flag parsing
            // existed: a fresh in-memory image, then file-eval or REPL.
            run_with_image(config.file.as_deref(), RuntimeImage::new());
        }
        SubstrateKind::Turso => run_turso(config),
    }
}

/// Constructs a Turso-backed image and dispatches to file-eval or REPL.
/// Split out (and feature-gated) so the memory path never references the
/// optional `iklo-substrate-turso` crate.
#[cfg(feature = "turso")]
fn run_turso(config: RunConfig) {
    use iklo_substrate_turso::TursoSubstrate;

    // `turso_db_url` is guaranteed `Some` for Turso mode by `resolve_config`.
    let db_url = config
        .turso_db_url
        .expect("resolve_config guarantees a db path for turso mode");

    let substrate = match TursoSubstrate::<Value>::new(&db_url) {
        Ok(substrate) => substrate,
        Err(err) => {
            // Explicit error, non-zero exit, no panic, no fallback (FR-007).
            eprintln!("iklo: failed to open turso database: {err}");
            std::process::exit(1);
        }
    };
    run_with_image(config.file.as_deref(), RuntimeImage::with_substrate(substrate));
}

/// When the `turso` feature is not compiled in, selecting it is a clear error
/// rather than a silent fallback.
#[cfg(not(feature = "turso"))]
fn run_turso(_config: RunConfig) {
    eprintln!(
        "iklo: --substrate turso is not available: this binary was built without the \
         'turso' feature (rebuild with `--features turso`)"
    );
    std::process::exit(1);
}

/// Dispatches to single-file evaluation or the interactive REPL, against an
/// already-constructed image of whichever backend `S`.
fn run_with_image<S: Substrate<Value = Value>>(file: Option<&str>, image: RuntimeImage<S>) {
    match file {
        Some(path) => {
            if let Err(err) = run_file(path, image) {
                eprintln!("iklo: {err}");
                std::process::exit(1);
            }
        }
        None => run_repl(image),
    }
}

fn run_file<S: Substrate<Value = Value>>(
    path: &str,
    mut image: RuntimeImage<S>,
) -> Result<(), Box<dyn std::error::Error>> {
    let src = std::fs::read_to_string(path)?;
    let program = parse(&src)?;
    let value = image.eval_in_tx(&program)?;
    println!("{value}");
    Ok(())
}

fn run_repl<S: Substrate<Value = Value>>(mut image: RuntimeImage<S>) {
    println!("iklo IK0 REPL");
    println!("commands (at empty prompt): /quit, /revision, /env");
    println!("(incomplete input continues on the next line; a blank line cancels)\n");

    let mut rl: ReplEditor = match rustyline::Editor::new() {
        Ok(rl) => rl,
        Err(err) => {
            eprintln!("iklo: failed to start line editor: {err}");
            return;
        }
    };
    rl.set_helper(Some(ReplHelper {
        completer: ReplCompleter::new(),
    }));

    let history_path = std::path::Path::new(HISTORY_FILE);
    // A missing OR unreadable/corrupt file both fail to load; either way we
    // must not later overwrite whatever's on disk with an empty in-memory
    // history — `had_existing_history` tracks whether we actually have
    // something loaded, not merely whether a file existed.
    let had_existing_history = rl.load_history(history_path).is_ok();

    let mut buffer = String::new();
    let mut entries_added_this_session = false;

    loop {
        let prompt = if buffer.is_empty() { "iklo> " } else { "iklo. " };
        if let Some(helper) = rl.helper_mut() {
            helper.completer.set_fresh_prompt(buffer.is_empty());
        }
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
            if is_repl_command_position(buffer.is_empty(), &line) {
                if let Some(command) = parse_repl_command(&line) {
                    record_history_entry(&mut rl, &mut entries_added_this_session, line.as_str());
                    match command {
                        ReplCommand::Quit => break,
                        ReplCommand::Revision => {
                            println!("{}", image.revision());
                            continue;
                        }
                        ReplCommand::Env => {
                            for (k, v) in image.bindings() {
                                println!(":{k} = {v}");
                            }
                            continue;
                        }
                    }
                }
                // Eligible position, but not a recognized command (e.g.
                // unrecognized '/foo') — fall through to ordinary parsing,
                // per FR-006.
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
    rl: &mut ReplEditor,
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

/// A recognized REPL meta-command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplCommand {
    Quit,
    Revision,
    Env,
}

/// The shared eligibility gate for slash-command territory: true iff the
/// buffer is empty (a fresh, non-continuation prompt) and `/` is byte zero
/// of the *untrimmed* line (so leading whitespace correctly returns
/// `false`, per FR-003). Does NOT check whether the line is a complete
/// command — used by both the Completer (which then prefix-filters
/// whatever partial text follows `/`) and the submit-time dispatcher (which
/// then requires an exact match via `parse_repl_command`). See plan.md Key
/// Design Decision 1.
fn is_repl_command_position(buffer_is_empty: bool, line: &str) -> bool {
    buffer_is_empty && line.starts_with('/')
}

/// Exact-match parser for a complete, submitted REPL command line. Only
/// meaningful after `is_repl_command_position` has confirmed eligibility;
/// trims only the end (trailing whitespace/newline), never the start, so
/// it never contradicts the eligibility gate's leading-whitespace check.
fn parse_repl_command(line: &str) -> Option<ReplCommand> {
    match line.trim_end() {
        "/quit" => Some(ReplCommand::Quit),
        "/revision" => Some(ReplCommand::Revision),
        "/env" => Some(ReplCommand::Env),
        _ => None,
    }
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
    fn is_repl_command_position_fresh_prompt_bare_slash() {
        assert!(is_repl_command_position(true, "/"));
    }

    #[test]
    fn is_repl_command_position_fresh_prompt_partial_command() {
        // Partial input must still be eligible for completion — this is
        // the exact case the earlier, since-corrected single-function
        // design got wrong (see plan.md Key Design Decision 1).
        assert!(is_repl_command_position(true, "/q"));
    }

    #[test]
    fn is_repl_command_position_fresh_prompt_no_slash() {
        assert!(!is_repl_command_position(true, "foo"));
    }

    #[test]
    fn is_repl_command_position_continuation_rejected() {
        assert!(!is_repl_command_position(false, "/quit"));
    }

    #[test]
    fn is_repl_command_position_leading_whitespace_rejected() {
        // '/' must be byte zero of the untrimmed line, per FR-003.
        assert!(!is_repl_command_position(true, "  /quit"));
    }

    #[test]
    fn parse_repl_command_quit() {
        assert_eq!(parse_repl_command("/quit"), Some(ReplCommand::Quit));
    }

    #[test]
    fn parse_repl_command_revision() {
        assert_eq!(parse_repl_command("/revision"), Some(ReplCommand::Revision));
    }

    #[test]
    fn parse_repl_command_env() {
        assert_eq!(parse_repl_command("/env"), Some(ReplCommand::Env));
    }

    #[test]
    fn parse_repl_command_unrecognized() {
        assert_eq!(parse_repl_command("/foo"), None);
    }

    #[test]
    fn parse_repl_command_division_expression() {
        // Defensive: in practice is_repl_command_position already excludes
        // this (no leading '/'), but parse_repl_command must be safe on
        // its own too.
        assert_eq!(parse_repl_command("10 / 2"), None);
    }

    #[test]
    fn parse_repl_command_trailing_whitespace_tolerance() {
        assert_eq!(parse_repl_command("/quit  \n"), Some(ReplCommand::Quit));
    }

    #[test]
    fn record_history_entry_flips_flag_on_success() {
        let mut rl: ReplEditor = rustyline::Editor::new().expect("editor");
        rl.set_helper(Some(ReplHelper {
            completer: ReplCompleter::new(),
        }));
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

    // --- CLI flag parsing / substrate selection (T020-T023) ---

    fn args(items: &[&str]) -> ParsedArgs {
        parse_args(items.iter().map(|s| s.to_string())).expect("parse_args should succeed")
    }

    #[test]
    fn parse_args_defaults_are_all_none() {
        assert_eq!(args(&[]), ParsedArgs::default());
    }

    #[test]
    fn parse_args_reads_positional_file_path() {
        assert_eq!(args(&["prog.ik"]).file.as_deref(), Some("prog.ik"));
    }

    #[test]
    fn parse_args_reads_substrate_and_db_url() {
        let parsed = args(&["--substrate", "turso", "--turso-db-url", "state.db"]);
        assert_eq!(parsed.substrate.as_deref(), Some("turso"));
        assert_eq!(parsed.turso_db_url.as_deref(), Some("state.db"));
    }

    #[test]
    fn parse_args_accepts_and_discards_auth_token() {
        // The token is accepted (not an "unknown flag") but never retained.
        let parsed = args(&["--turso-auth-token", "s3cr3t", "prog.ik"]);
        assert_eq!(parsed.file.as_deref(), Some("prog.ik"));
        // Debug output of the parsed args must not leak the token value.
        assert!(
            !format!("{parsed:?}").contains("s3cr3t"),
            "auth token must never appear in ParsedArgs Debug output"
        );
    }

    #[test]
    fn parse_args_rejects_unknown_flag() {
        let err = parse_args(["--nope".to_string()].into_iter()).expect_err("must reject");
        assert!(err.contains("unknown flag"), "got: {err}");
    }

    #[test]
    fn parse_args_rejects_substrate_without_value() {
        let err = parse_args(["--substrate".to_string()].into_iter()).expect_err("must reject");
        assert!(err.contains("--substrate requires a value"), "got: {err}");
    }

    #[test]
    fn parse_args_rejects_second_positional() {
        let err =
            parse_args(["a.ik", "b.ik"].iter().map(|s| s.to_string())).expect_err("must reject");
        assert!(err.contains("extra argument"), "got: {err}");
    }

    #[test]
    fn resolve_config_defaults_to_memory() {
        let config = resolve_config(ParsedArgs::default(), None).expect("memory default");
        assert_eq!(config.substrate, SubstrateKind::Memory);
        assert_eq!(config.turso_db_url, None);
    }

    #[test]
    fn resolve_config_memory_ignores_turso_db_url_and_env() {
        let parsed = ParsedArgs {
            substrate: Some("memory".to_string()),
            turso_db_url: Some("flag.db".to_string()),
            file: None,
        };
        let config =
            resolve_config(parsed, Some("env.db".to_string())).expect("memory ignores turso opts");
        assert_eq!(config.substrate, SubstrateKind::Memory);
        assert_eq!(
            config.turso_db_url, None,
            "memory mode must ignore turso db path entirely"
        );
    }

    #[test]
    fn resolve_config_turso_flag_beats_env() {
        let parsed = ParsedArgs {
            substrate: Some("turso".to_string()),
            turso_db_url: Some("flag.db".to_string()),
            file: None,
        };
        let config = resolve_config(parsed, Some("env.db".to_string())).expect("flag wins");
        assert_eq!(config.substrate, SubstrateKind::Turso);
        assert_eq!(config.turso_db_url.as_deref(), Some("flag.db"));
    }

    #[test]
    fn resolve_config_turso_falls_back_to_env_when_no_flag() {
        let parsed = ParsedArgs {
            substrate: Some("turso".to_string()),
            turso_db_url: None,
            file: None,
        };
        let config = resolve_config(parsed, Some("env.db".to_string())).expect("env fallback");
        assert_eq!(config.turso_db_url.as_deref(), Some("env.db"));
    }

    #[test]
    fn resolve_config_turso_without_db_url_errors_and_does_not_fall_back() {
        let parsed = ParsedArgs {
            substrate: Some("turso".to_string()),
            turso_db_url: None,
            file: None,
        };
        let err = resolve_config(parsed, None).expect_err("turso needs a db path");
        assert!(err.contains("requires a database path"), "got: {err}");
        assert!(err.contains(ENV_TURSO_DB_URL), "should name the env var; got: {err}");
    }

    #[test]
    fn resolve_config_rejects_unknown_substrate_value() {
        let parsed = ParsedArgs {
            substrate: Some("postgres".to_string()),
            turso_db_url: None,
            file: None,
        };
        let err = resolve_config(parsed, None).expect_err("must reject unknown substrate");
        assert!(err.contains("unknown --substrate value 'postgres'"), "got: {err}");
    }

    /// A valid local Turso path yields a working image that actually evaluates
    /// against the Turso backend. Uses `:memory:` (a fresh, isolated Turso db)
    /// so the test needs no filesystem cleanup. Only meaningful with the
    /// `turso` feature compiled in.
    #[cfg(feature = "turso")]
    #[test]
    fn turso_mode_with_valid_path_constructs_and_evaluates() {
        use iklo_substrate_turso::TursoSubstrate;

        let config = resolve_config(
            ParsedArgs {
                substrate: Some("turso".to_string()),
                turso_db_url: Some(":memory:".to_string()),
                file: None,
            },
            None,
        )
        .expect("turso config with a valid path");

        let db_url = config.turso_db_url.expect("turso db path present");
        let substrate =
            TursoSubstrate::<Value>::new(&db_url).expect("opening :memory: turso db must succeed");
        let mut image = RuntimeImage::with_substrate(substrate);

        let program = parse("let :answer be 40 + 2").expect("parse");
        let value = image.eval_in_tx(&program).expect("eval against turso backend");
        assert_eq!(value, Value::Number(42.0));
        assert_eq!(image.revision(), 1);
        assert_eq!(image.bindings().get("answer"), Some(&Value::Number(42.0)));
    }
}

