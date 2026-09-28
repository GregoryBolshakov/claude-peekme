//! Development helper: run one explanation for a selection inside some screen
//! text, without a terminal. Prints the prompt sent and the streamed answer.
//!
//!     cargo run --example peek_text -- "<screen text>" "<selection>" [--deep] [--pid PID]
//!
//! `--pid` is the process id of a running interactive `claude`, whose
//! transcript then provides the conversation context.

#[path = "../src/context.rs"]
mod context;
#[path = "../src/explain.rs"]
mod explain;
#[path = "../src/transcript.rs"]
mod transcript;

use std::sync::Arc;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (text, selected) = (&args[1], &args[2]);
    let deep = args.iter().any(|a| a == "--deep");
    let pid = args
        .iter()
        .position(|a| a == "--pid")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse().ok());
    let chars: Vec<char> = text.chars().collect();
    let sel: Vec<char> = selected.chars().collect();
    let start = (0..=chars.len() - sel.len())
        .rev()
        .find(|&i| chars[i..i + sel.len()] == *sel)
        .expect("selection not in text");
    let req = explain::Request {
        screen: context::ScreenSel {
            text: chars,
            start,
            end: start + sel.len(),
        },
        cwd: std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        max_lines: 10,
        deep,
        claude_pid: pid,
    };
    let built = explain::prompt(&req);
    println!(
        "===== prompt ({} chars, {}) =====\n{}",
        built.prompt.chars().count(),
        built.label,
        built.prompt
    );
    let t0 = std::time::Instant::now();
    let pool = Arc::new(explain::Pool::default());
    explain::explain(&pool, req, |p| match p {
        explain::Progress::Started { model, source } => println!(
            "===== {model} · {source} · first token {:.2}s =====",
            t0.elapsed().as_secs_f32()
        ),
        explain::Progress::Delta(d) => print!("{d}"),
        explain::Progress::Done => {
            println!("\n===== done in {:.2}s =====", t0.elapsed().as_secs_f32())
        }
        explain::Progress::Failed(e) => println!("\n===== failed: {e} ====="),
    });
    pool.shutdown();
}
