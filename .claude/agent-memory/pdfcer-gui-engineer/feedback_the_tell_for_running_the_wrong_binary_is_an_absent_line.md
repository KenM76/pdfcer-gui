---
name: the-tell-for-running-the-wrong-binary-is-an-absent-line
description: When a new instrument's line is missing from a trace, suspect the binary before the code — a stale exe produces a trace that is entirely valid and silently one build old
metadata:
  type: feedback
---

**A trace with your new diagnostic line MISSING is a claim about which binary
ran, not about the code. Check the exe's timestamp before you debug the
instrument.**

**Why:** 2026-09-13, adding `src=` to the `canvas-place` line. The first launch
after the change produced a trace full of correct, well-formed `canvas-place`
lines with no `src=` field on any of them. Every instinct said *the format
string is wrong* or *the constant is not marked* — and the honest answer was
that `target/scratch/drive/pdfcer-gui.exe` was a copy taken before the rebuild.
**A stale binary does not emit a wrong line; it emits the OLD line, which is
indistinguishable from a correct trace unless you know what you just added.** A
wrong value shouts. An absent value reads as "that code path did not run",
which is a plausible and entirely wrong conclusion.

This bites specifically here because the harness deliberately drives a **copy**
of the exe (never the published build — that is his install), so there are
always at least two binaries on disk and the copy can silently lag.

**How to apply:** when a field, event, or trace slot you *just added* does not
appear, do these three, in order, before reading any source:

1. `stat -c '%s %y'` on both the built exe and the copy the harness drives — if
   the copy is older than the build, that is the whole defect.
2. `grep -a -c '<the new literal>' <the exe that actually ran>` — the format
   string is in the binary; its absence is proof, not inference.
3. Only then suspect the code.

★ And note the failure that produced the stale copy: on Windows a `Copy-Item`
onto a **running or recently-run** exe fails, and in a chained command that
failure can be swallowed while every later step succeeds — see
[[a-commit-message-can-describe-work-that-never-landed]] for the same shape.
Assert the copy landed (compare byte counts), do not assume it.

Related: [[a-value-cannot-identify-which-producer-made-it]] — that one is about
a line whose number is right; this one is about a line that is simply not there.
