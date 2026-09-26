# `ui-verify/main`

## Item notes

### `fn refuse_if_self_is_stale`

A full 108-check sweep was launched with a freshly built *application* and a
`ui-verify.exe` stamped **two and three-quarter hours earlier** — five
commits of check fixes behind. It reported 15 failures. Several were the old
checks: one had been re-pointed at a different fixture that morning and the
stale binary drove the fixture the new one no longer uses, then failed
confidently about a program that was behaving correctly. **It cost a defect
investigation.**

⇒ The irony is the point: this harness has guarded the **application**
against exactly this since it was written — `LaunchSpec::allow_stale`, and a
message that opens *"the traces you are about to collect would describe code
that is NOT the code you just wrote."* It guarded everything except itself.

# Why the failure is worse here than for the application

A stale *application* produces a missing trace line, which reads as a broken
feature — bad, and the reason the original guard exists. A stale *harness*
produces a **confident, articulate failure about the wrong subject**: it
drives the wrong fixture, asserts on a trace name that has since been
renamed, and prints a diagnosis naming a module that is fine. That is harder
to see through, because everything about the report looks like evidence.

# What it compares

The running executable's mtime against the newest `.rs` or `.toml` under
this crate's own directory, resolved at compile time by `CARGO_MANIFEST_DIR`.
If that directory is gone — a binary copied elsewhere, which
`package-portable` does — the check is skipped rather than guessed at.

`--allow-stale` covers this too, deliberately: one flag for *"yes, I mean
to drive the older build"*, whichever binary is older, rather than a second
flag nobody would remember.
# And `--list` is behind this guard, which it was not until 2026-09-14


`RESUME.md` names `ui-verify --list | grep -cE '^  [a-z0-9_]+$'` as the way
to measure how many driven checks exist, and that figure is quoted into
`FEATURES.md`'s revision header and into the GitHub release notes. So the
one path this guard deliberately skipped was the one path whose output
reaches a shipped document.


A check was added and committed. `ui-verify.exe` on disk was an hour older
than that commit. `--list` answered **223** where the roster was **224** —
cheerfully, with no complaint anywhere, because the binary was reporting the
roster it had been COMPILED with. Nothing in the toolchain can notice that:
the number is not wrong *about the binary*, it is wrong *about the tree*.
It was caught only because a release rebuild happened to intervene between
the measurement and the document.

# Why the fix is the ORDER and not a warning

Every softer option is defeated by the pipe the count command is used in:

* a warning on stderr is discarded by `2>/dev/null`, which the measuring
  session had in fact typed;
* a warning in the `--list` header is invisible to `grep -c`, whose pattern
  matches only check-name lines;
* a non-zero exit is swallowed, because in `a | b` the shell reports **b**'s
  status and `grep` succeeded at counting what it was given.

⇒ Putting `--list` behind the refusal makes **stdout empty**, so the count
command answers **0**. Zero is not a plausible roster size and cannot be
mistaken for one; 223 can, and was.

# The general shape, which outlives this instance

**A guard is placed against the uses that existed when it was written.**
When a command later grows a second job, nothing re-asks which side of every
guard it belongs on — not the compiler, not clippy, not a test, because
nothing has changed about either the guard or the command. Re-ask it by
hand, at the moment the second job appears.

`--help` stays in front, deliberately. It prints the argument surface,
which is compiled in but is not a measurement of the tree, and it is exactly
what a reader reaches for when the tool has just refused them.
