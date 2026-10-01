# Live link: driving the user's open window with pdfcer-remote.exe

The preferred way to change a document the user has open. Edits happen inside
the open document with the same undo as the user's own click, so there is no
hand-off and nothing to reload. `pdfcer-remote.exe` sits beside `pdfcer-gui.exe`.
If it is missing, use the hand-off procedure in file-safety.md.

## Command line

```
pdfcer-remote list                       live windows: "<pid>\t<active document path>", newest first
pdfcer-remote help                       the window's own verb list (needs permission like any verb)
pdfcer-remote [--pid N] [--client NAME] [--purpose TEXT] <verb> [args...]
```

- Options come before the verb. Everything from the verb on is the request.
- `--client` defaults to `pdfcer-remote`; `--purpose` defaults to empty. Always
  pass a purpose a person can decide on: `--purpose "apply the red markup on page 3"`.
- Without `--pid`, the target is the newest live window. "Newest" means the
  most recently modified discovery file, so it can change. Use `--pid` whenever
  more than one window is listed.
- Each invocation opens one connection: hello, one request, bye.
- stdout on success: the message line (if any), then the body lines.
- stderr on failure: `refused (<code>): <msg>` when permission is denied;
  `error <code>: <msg>` when the request fails.
- Exit codes: 0 success; 1 refused or the verb failed; 2 bad usage, no
  listening window, `LOCALAPPDATA` unset, or the connection failed.
  The no-window message is `no pdfcer-gui window is listening (is one open, and is Remote control not set to Never?)`.

## Permission (Settings ▸ Remote control; preference `remote_control`)

- `ask` (default): the first request puts a bar across the user's window naming
  the client and purpose, and **the call blocks** until the user answers, for up to
  120 s. A timeout counts as a refusal.
  - Allow once: this connection only. Because each `pdfcer-remote` call is a
    new connection, the next call asks again.
  - Allow for this session: any program, until that window closes.
  - Always allow: sets the policy to `always`.
  - Refuse: that client name is refused without asking for 60 s
    (`refused cooling-off`). Do not retry in a loop; ask the user.
- `always`: no question. `never`: `refused disabled`.
- Only the same Windows user on the same machine can open the pipe.
- A second program asking while a question is pending gets `refused busy`.
- The client name is self-declared. The user can press Disconnect
  (`err disconnected`). A closing window replies `err closing`.
- The status bar shows the connected client, with a log of up to 500 requests.
- When there are several requests to make, ask the user to choose
  "Allow for this session".

## Verbs (all act on the active document tab)

| Verb | Reply |
|---|---|
| `state` | `documents=N`, `mode=<read\|review\|edit\|->`, then with a document: `document=<abs path>`, `page=n/total`, `zoom=<percent>`, `unsaved=true\|false`, `selected=i,j` (current page), `undo_depth=N` |
| `commands` | one line per command: `id<TAB>enabled\|disabled\|hidden<TAB>label`, sorted |
| `run <id>` | runs it as a ribbon click; replies `ok <id>` + state. Errors: `unknown-command`, `hidden` (the mode lacks it), `disabled` |
| `page <n>` | 1-based; replies with state. `no-such-page 1..N` |
| `objects` | `i<TAB>path\|text\|image<TAB>x0,y0,x1,y1`: current page, paint order, page-space points to 0.1. `no-objects` if the page did not decompose |
| `select <i,j,...>` / `select none` | replaces the selection on the current page; replies with state. `no-such-object 0..count` |
| `undo [n]` / `redo [n]` | n defaults to 1, capped at 100; replies `ok <n>` + state |
| `history` | `undo_depth`, `undo_top`, `redo_top`, `redo_depth`, `undo_kinds=A,B` (newest first), `redo_kinds`; `-` when empty |
| `render [page] [dpi]` | defaults to the current page at 96 dpi, max 600. Writes `%TEMP%\pdfcer-remote\<pid>-page<N>.png` (white backdrop, annotations and layers as currently shown). Replies `ok <path>`. `render-failed` |
| `open <path>` | opens in a new tab; replies with state |

Errors any verb can give: `no-document` (open a PDF first), `unknown-verb`, `usage`.

## Wire protocol (for a client of your own)

- Named pipe `\\.\pipe\pdfcer-remote-<pid>`, opened as a file (read+write). Discovery files are
  `%LOCALAPPDATA%\pdfcer\remote\<pid>.txt`, holding `pipe=`, `pid=`, `app=`, `document=`.
  A file whose pipe is gone (a crashed window) is skipped.
- Request: one UTF-8 line. Quote any argument containing spaces; the only
  escapes are `\"` and `\\`.
- Reply: `ok [message]` or `err <code> <message>`, then body lines, then a line
  holding only `.`. Body lines starting with `.` are dot-stuffed.
- The first line must be `hello <client> "<purpose>"`. Anything else first gets
  `err not-enabled`. A refusal is `err refused <reason>`. `bye` closes.
- A request taking over 300 s gets `err timeout`.

## The loop that works

`state` → `render` → read the PNG → `objects` → `select` → `run` → `render` again.
`run mode.edit` first when a command is `hidden`.

## Pitfalls

- A label ending `…` opens a dialog **on the user's screen**. Prefer commands
  that act on the selection directly.
- `objects` indices are paint order and shift after a delete, and a delete clears
  the selection. Re-read `objects` after every edit.
- The user can scroll at any time. Check `page=` in a reply before `select`.
- Do not `run file.save` unless asked. Unsaved edits let the user review and undo.
- `commands` lists only what this build registered. A missing id means the
  capability is absent from this build.
- `render` shows what the window shows: a hidden layer is not drawn.
