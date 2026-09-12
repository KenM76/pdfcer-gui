---
name: the-write-python-to-a-file-workaround-does-not-protect-an-escape
description: Writing the patch script to a file instead of a heredoc fixes the SHELL's mangling and nothing else — a `\n` or `\x00` inside a non-raw triple-quoted payload is collapsed by PYTHON, one layer down
metadata:
  type: feedback
---

**When a python script emits code containing backslash escapes, the payload
string must be RAW (`r'''…'''`) and plain ASCII.** Writing the script to a file
under `$SCRATCH` first solves the shell layer; it does nothing about the python
layer underneath it.

**Why:** `RESUME.md` carries a standing "Do not" bullet about heredoc-delivered
patch scripts — eleven-plus recorded occurrences of a multi-line Rust string
literal losing its trailing backslash. The recorded workaround is to write the
python to a file and run the file. **That workaround was followed on 2026-09-11
and the payload still broke, twice in the same hour, in two different ways:**

- `'\n'` inside a non-raw `'''…'''` became a **real newline** inside the emitted
  f-string literal, producing `SyntaxError: unterminated string literal` — and
  the reported line number was the *print* statement, not the edit, so the first
  reading was "the packager is broken" rather than "my patch script is".
- `b"\x00"` inside the same shape became an **actual NUL byte** in the emitted
  file, and python refused it with `source code cannot contain null bytes` — a
  message that names neither the escape nor the writer.

In the same session a sibling script that used `r'''…'''` came through clean,
which is the control: the difference is the `r`, not the delivery method.

**★★ And there is a THIRD layer, found sixty seconds after this memory was
first written — by this memory's own index entry.** The one-line pointer added to
`MEMORY.md` was supposed to contain the two escapes as literal text. It was
written through a correctly quoted heredoc, in a python string that doubled every
backslash, and **the file still came out with a real newline in the middle of the
index line**, splitting one entry across three lines. The collapse happened above
python, in the transport that carries the command.

⇒ So the rule is not *double the backslash*, which fails, and not *use a raw
string*, which only fixes the layer underneath. The rule is **do not put a
backslash in the payload at all.** Describe the escape in words, name the byte,
or build it with `chr(10)`. Three layers stack here and each one is individually
reasonable; only abstinence is reliable.

**How to apply:**
- Payload strings that will contain `\` get `r'''…'''`. Always. Even when the
  current payload has no escape — the next edit to it will.
- Prefer ASCII. If a test needs a byte, pick a printable one (`b"weights"`), not
  `\x00`; the null added nothing to the assertion and cost a debugging round.
- After emitting, **`python -c "import ast; ast.parse(open(p).read())"`** before
  running anything that imports it. It is one line and it localises the failure
  to the writer instead of the written.
- A `SyntaxError` whose line number sits in code you did not hand-write is a
  claim about the *generator*. Read the bytes with `cat -A` before reading the
  logic.
- Related: [[never-git-checkout-to-undo-an-experiment]] — keep a `cp` copy of the
  target before an emitted patch runs, because the recovery is a restore.
