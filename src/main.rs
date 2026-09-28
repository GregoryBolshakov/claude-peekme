//! claude-peekme: select text in Claude Code output, press Alt+P, and read an
//! explanation that opens inline next to it.

mod app;
mod context;
mod explain;
mod input;
mod osc52;
mod overlay;
mod render;
mod select;
mod shadow;
mod transcript;

use std::io::IsTerminal;

/// Set in the child's environment, so a nested claude-peekme steps aside.
pub const ACTIVE_ENV: &str = "CLAUDE_PEEKME_ACTIVE";

const HELP: &str = "\
claude-peekme {version}
Select text in Claude Code's output with the mouse, press Alt+P, and an
explanation from a smaller model opens right next to it. Esc closes it.

USAGE:
    claude-peekme [CLAUDE ARGS...]     run claude with peekme
    claude-peekme -- COMMAND [ARGS]    run any other command with peekme

KEYS (while the box is open):
    Alt+P again     explain again with the whole conversation
    PgUp / PgDn     scroll the explanation (the mouse wheel too)
    Esc             close the box
    any other key   closes the box and goes to Claude

ENVIRONMENT:
    CLAUDE_PEEKME_MODEL       model for explanations (default: haiku)
    CLAUDE_PEEKME_CLAUDE_BIN  claude binary used for explanations (default: claude)
    CLAUDE_PEEKME_SELECTION   use this text instead of the mouse selection
";

/// `claude` flags and subcommands that don't open the interactive screen.
const BATCH_FLAGS: &[&str] = &[
    "-p",
    "--print",
    "-v",
    "--version",
    "-h",
    "--help",
    "--bg",
    "--background",
];
const BATCH_COMMANDS: &[&str] = &[
    "agents",
    "attach",
    "auth",
    "auto-mode",
    "doctor",
    "gateway",
    "import",
    "install",
    "logs",
    "mcp",
    "plugin",
    "plugins",
    "project",
    "respawn",
    "rm",
    "setup-token",
    "stop",
    "kill",
    "ultrareview",
    "update",
    "upgrade",
];

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let program = match args.first().map(String::as_str) {
        Some("--help") if args.len() == 1 => {
            print!("{}", HELP.replace("{version}", env!("CARGO_PKG_VERSION")));
            return;
        }
        Some("--version") if args.len() == 1 => {
            println!("claude-peekme {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Some("--") => {
            args.remove(0);
            if args.is_empty() {
                eprintln!("claude-peekme: no command after --");
                std::process::exit(2);
            }
            args.remove(0)
        }
        _ => "claude".to_string(),
    };
    let is_claude =
        std::path::Path::new(&program).file_name() == Some(std::ffi::OsStr::new("claude"));

    // Step aside when there is nothing to overlay: already inside claude-peekme,
    // not a terminal, or a claude command without its interactive screen.
    let nested = std::env::var_os(ACTIVE_ENV).is_some();
    let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let batch = is_claude
        && (args.iter().any(|a| BATCH_FLAGS.contains(&a.as_str()))
            || args
                .first()
                .is_some_and(|a| BATCH_COMMANDS.contains(&a.as_str())));
    if nested || !tty || batch {
        exec(&program, &args);
    }

    // A panic is logged, never printed over the child's screen.
    std::panic::set_hook(Box::new(|info| log(&format!("panic: {info}"))));

    if crossterm::terminal::enable_raw_mode().is_err() {
        exec(&program, &args);
    }
    let result = std::panic::catch_unwind(|| app::run(&program, &args));
    restore_terminal();
    let code = match result {
        Ok(Ok(code)) => code,
        Ok(Err(e)) => {
            eprintln!("claude-peekme: {e:#}");
            1
        }
        Err(_) => {
            eprintln!(
                "claude-peekme: internal error, see {}",
                log_path().display()
            );
            1
        }
    };
    std::process::exit(code);
}

/// Replace this process with the command (keeps its pid, signals and exit code).
fn exec(program: &str, args: &[String]) -> ! {
    use std::os::unix::process::CommandExt;
    let err = std::process::Command::new(program).args(args).exec();
    eprintln!("claude-peekme: could not start `{program}`: {err}");
    std::process::exit(127);
}

fn log_path() -> std::path::PathBuf {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".local/state")))
        .unwrap_or_else(std::env::temp_dir);
    base.join("claude-peekme").join("claude-peekme.log")
}

/// Append a line to the log file. Never fails loudly: this runs inside a panic hook.
pub fn log(msg: &str) {
    let path = log_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let _ = std::io::Write::write_all(&mut f, format!("{secs} {msg}\n").as_bytes());
    }
}

fn restore_terminal() {
    let _ = crossterm::terminal::disable_raw_mode();
    // End any half-written synchronized update and make the cursor visible.
    print!("\x1b[?2026l\x1b[0m\x1b[?25h");
    let _ = std::io::Write::flush(&mut std::io::stdout());
}
