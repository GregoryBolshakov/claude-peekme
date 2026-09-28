# claude-peekme

Peek me. Pick me. Select text in [Claude Code](https://github.com/anthropics/claude-code) output,
press **Alt+P**, and a short explanation opens right next to that text, inside the terminal. The
other lines move to make room, like Peek in VS Code. Esc closes it and the screen is exactly as
before.

The explanation comes from a small, fast model and it knows the conversation. If you select
"its split panes" in an answer that recommends Kitty, it tells you it is about Kitty.

This is the Claude Code version of [peekme](https://crates.io/crates/peekme), which does the same
for Codex CLI.

Status: 0.0.1, first working version. Tested on Linux with X11, Claude Code 2.1.283.

## Install

You need Rust and Claude Code (logged in).

```
cargo install --git https://github.com/GregoryBolshakov/claude-peekme
```

Then start Claude Code through it:

```
claude-peekme                  # same as `claude`
claude-peekme --resume         # any claude arguments work
claude-peekme -- COMMAND ...   # any other command
```

Commands without the interactive screen (`claude -p`, `claude mcp`, `claude doctor` and so on)
and output to a pipe or a file run plain `claude` directly.

## Use

Select text with the mouse as you normally do, then press Alt+P.

| Key | When | What it does |
|---|---|---|
| Alt+P | any time | explain the selected text |
| Alt+P again | box is open, same selection | explain again using the whole conversation |
| PgUp / PgDn, mouse wheel | box is open | scroll a long explanation |
| Esc | box is open | close the box |
| any other key | box is open | closes the box and goes to Claude |

Alt+P opens the model picker in plain Claude Code. Under claude-peekme it explains the selection
instead. `/model` still opens the model picker.

No API key is needed. claude-peekme runs `claude -p` with your existing Claude Code login. The
session is not saved, so nothing appears in `/resume`. It does use your plan, a little for each
explanation.

Nothing starts until the first Alt+P. After that one `claude -p` process waits in the
background for the next explanation, so it starts faster. It takes about 210 MB.

## How it works

Claude Code runs in a pseudo-terminal. Its output goes to your terminal unchanged, and the same
bytes also go to a terminal emulator in memory (`alacritty_terminal`). So claude-peekme always
knows what is on your screen.

Claude Code has two screens, and claude-peekme reads the selection in a different way on each:

- Full screen (`"tui": "fullscreen"`). Claude handles the mouse and draws the selection itself.
  When you release the mouse it copies the text with an OSC 52 escape sequence. claude-peekme
  reads the text from that sequence as it passes through, and knows where the drag ended. This
  works on any terminal, also over SSH.
- Classic (`"tui": "default"`). Your terminal makes the selection, and claude-peekme reads the
  X11 PRIMARY selection.

Then it finds the text on the screen and redraws only the rows next to it, with the box between
them. Nothing is cleared and there is no switch to another screen, so there is no flicker.
Claude output that arrives while the box is open is kept aside.

On Esc it draws those rows again from a copy taken when the box opened. After that it sends the
kept output, so the terminal ends in the same state as if the box never existed. Tests check
this cell by cell on real Claude Code output from both screens.

### What the model gets

A few words alone are not enough ("Desktop app", which one?). The whole chat is too much and too
slow. So the model gets small parts, each with a size limit:

1. One line saying this is Claude Code and in which directory.
2. The text around the selection, with the selection marked in place like `⟦this⟧`. It comes
   from the session transcript in `~/.claude/projects`. claude-peekme knows which transcript
   because Claude Code registers its process id in `~/.claude/sessions`. The selection is
   searched together with the words next to it on screen, so the right place is found when the
   same words appear many times. If the text is not in the conversation, it uses the screen.
3. Your request that this text answers, and one line for each of your earlier requests.
4. The first places in the conversation where the same words were used before.

When this is not enough, press Alt+P again. The model then gets the whole conversation up to
that answer, with long tool output cut short.

## Settings

| Variable | Meaning |
|---|---|
| `CLAUDE_PEEKME_MODEL` | model for explanations. Default is `haiku` |
| `CLAUDE_PEEKME_CLAUDE_BIN` | claude binary used for explanations. Default is `claude` |
| `CLAUDE_PEEKME_SELECTION` | use this text instead of the mouse selection, for testing |
| `CLAUDE_CONFIG_DIR` | read like Claude Code does, to find the transcript |

## Limits

- Only text that is on screen now can be explained.
- Full screen: if `copyOnSelect` is off in Claude Code, only a Shift+drag selection works (read
  from X11 PRIMARY).
- Classic screen: the terminal selection is read on Linux with X11 only, and not over SSH.
- A new explanation starts to arrive after about 1 second, and about 0.5 seconds when the
  waiting process is ready.
- Not tested yet in a real terminal window, only in an emulated one. Not tested on macOS or
  Wayland.

## Development

```
cargo test
cargo clippy --all-targets
cargo run --example vtdump -- capture.bin 120 40 [offset]
cargo run --example peek_text -- "<screen text>" "<selection>" [--deep] [--pid PID]
python3 spikes/capture.py out.bin 120 40 "<script>" -- claude-peekme ...
```

`vtdump` replays a captured byte stream and prints the screen. `peek_text` runs one explanation
without a terminal and prints the exact prompt. `capture.py` runs a command in a pseudo-terminal,
sends keys and mouse events from a script and saves everything it prints.

## License

MIT or Apache-2.0, at your choice.
