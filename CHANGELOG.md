# Changelog

## 0.0.1 (2026-09-28)

First working version.

- Runs Claude Code (or any command) in a pseudo-terminal. Output goes to the terminal unchanged
  and is also kept in a terminal emulator in memory.
- Alt+P explains the selected text. On Claude's full screen the selection is read from the
  OSC 52 copy Claude sends when you release the mouse. On the classic screen it is the X11
  PRIMARY selection. The box opens under or above the text without clearing the screen. Works
  with the kitty keyboard protocol and with any keyboard layout.
- Esc restores the screen from a copy and then sends the Claude output that arrived meanwhile.
  The box is never drawn in the middle of an escape sequence from Claude.
- Explanations come from `claude -p` on Haiku with your Claude Code login, so no API key is
  needed. The session is not saved.
- The model gets the text around the selection with the selection marked, the request it
  answers, a short list of earlier requests and earlier uses of the same words. It reads them
  from the transcript of the running session.
- Alt+P again on the same selection explains with the whole conversation.
- Text from the model is cleaned before it is drawn, so it can't send terminal control
  sequences.
