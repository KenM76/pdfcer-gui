---
name: the-write-python-to-a-file-workaround-does-not-protect-an-escape
description: Three quoting layers stack between a patch script and the file it writes, and each eats a different backslash — the only reliable rule is to put no backslash in the payload at all and spell it chr(92)
metadata:
  type: feedback
---

**Do not put a backslash in a patch script's payload. Spell it `chr(92)`, spell
a newline `chr(10)`, and keep the payload plain ASCII.** Writing the script to a
file under `$SCRATCH` instead of a heredoc solves the *shell* layer and nothing
else; there are two more layers under it and they disagree with each other.

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
which is the control for *that* pair: the difference was the `r`, not the
delivery method. ⚠ But see the 2026-09-13 addendum at the foot of this file —
the `r` is not a general answer, and taken as one it produces a *different*
wrong emission that compiles.

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
- Build every backslash with `chr(92)` and every newline with `chr(10)`, in
  every payload, including the ones that currently have no escape — the next
  edit to that payload will.
- Reserve `r'''…'''` for its one honest use: a payload that emits a **regex**,
  where the backslashes belong to the emitted code and are meant to arrive
  doubled. Do not reach for it as general protection; see the addendum below.
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

---

**★ 2026-09-13 — the GUARD against the escape is the new failure mode, and it
killed two patch scripts in one session.** The discipline above produced
`assert`s whose property was wrong rather than whose logic was wrong:

- `assert BS + "U" not in NEW and BS + "x" not in NEW` — **unsatisfiable**, because
  the payload legitimately contained `"C:" + BS + "Users"`. The guard could never
  pass and was aimed at a property the payload is supposed to have.
- `assert BS not in s.replace("chr(92)", "")` — failed because the emitted file
  contained regexes with `\s` and `\b` **inside raw strings**, which is correct
  code. The script died before writing, so the target was untouched; that is luck,
  not design.

⇒ **Do not assert about backslashes. Assert about the only property that matters:
does the emitted file compile without a `SyntaxWarning`?**

```
python -W error::SyntaxWarning -c "import py_compile,sys; py_compile.compile(p, doraise=True)"
```

That catches the unrecognised-escape class the guard was reaching for, accepts
every legitimate backslash, and cannot be unsatisfiable. For the payload's own
integrity, keep a literal sentinel check instead (`assert "PLACEHOLDER" not in s`),
which is cheap and says what it means.

⇒ Same family as
[[an-assertion-both-outcomes-satisfy-is-not-a-measurement-of-which-one-shipped]]:
a guard is a measurement, and a guard that forbids a property the subject must
have is measuring the wrong thing in the most expensive direction — it blocks work
that is correct.

---

**★ 2026-09-13 — a RAW payload does not protect a Rust line continuation, and
the two habits in this file contradict each other.**

Stated plainly it is obvious, and it is not obvious at all with a patch script
half-written:

- In a **non-raw** payload, `\\` is **one** character. It reaches the emitted
  file as a lone backslash — a Rust line continuation, a path separator.
- In a **raw** payload, `\\` is **two** characters. It reaches the emitted file
  as two backslashes.

So *"always use a raw payload"* and *"always double the backslash"* cannot both
be followed, and the combination is the dangerous one. A doubled backslash at
the end of a line in emitted Rust is an **escaped backslash inside a string
literal**, not a continuation. The crate still compiles. The string is simply
wrong: it carries a literal backslash and every space of the indentation a
continuation would have swallowed. Nothing goes red at the point of the mistake
— what goes wrong is downstream, in whatever reads that string.

⇒ This adds no third rule; it **removes** one. The lead of this file and its
first *How to apply* bullet used to say "make the payload raw", and that advice
is now deleted from both. What survives is the rule that was already written
here: **no backslash in the payload at all.**

⚠ And the emitted-file compile check above does **not** catch this class,
because the wrong output is valid source. The only oracle for *"it compiled and
it is wrong"* is reading the emitted region back with `sed -n` or `cat -A` and
looking at it, which costs one command.

---

**★ 2026-09-19 — the Bash-first preference has a SIZE ceiling, and the error
names neither size nor Bash.**

A new `ui-verify` check of about 700 lines was delivered as a single
`cat > path <<'RUSTEOF' … RUSTEOF`. It did not write a truncated file, and it
did not report a quoting problem. It failed with:

```
ENAMETOOLONG: name too long, uv_spawn
```

The whole heredoc is part of the **command string** handed to the process
spawner, so a large payload exceeds the spawn limit before any shell runs.
Nothing is written; the file simply does not exist afterwards. ⚠ The tell is an
absence, so the next step ("declare it in `mod.rs`") proceeds happily and the
build is the first thing to notice.

⇒ The session's standing *"prefer the Bash tool over Read/Edit/Write"* is a
preference about **which tool reads and edits**, not a claim that Bash can
deliver an arbitrarily large payload. Above roughly a few hundred lines, use the
`Write` tool — it takes the content as a tool argument rather than as a command
line, and it has none of the three quoting layers above either.

**How to apply:**
- New source file over ~300 lines ⇒ `Write`. Small targeted edits to an existing
  file ⇒ Bash `sed`/python, unchanged.
- After any `cat >` heredoc, `ls -l` the target. A spawn failure leaves no file
  and no partial file, which looks identical to "I have not got there yet".
- The same limit applies to a long python script delivered by heredoc — split it
  or write it to `$SCRATCH` with `Write` and run the file.
- **A `sed -i 's/…/…/'` replacement is a shell payload too**, and carries every
  one of these layers with it. A replacement text containing an escaped newline
  put a literal newline inside a python string literal and left the file at
  `SyntaxError: unterminated string literal`. Prose or code going INTO a file
  belongs in `Edit`, whatever the delivery verb is called.
