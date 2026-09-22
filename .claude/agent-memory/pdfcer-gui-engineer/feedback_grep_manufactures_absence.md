---
name: grep-manufactures-absence
description: A grep returning nothing on a file a PROGRAM wrote may be binary-suppression, not an absence — re-run with -a before hypothesising about the program
metadata:
  type: feedback
---

When a `grep` over a file **a program wrote** (a `PDFCER_DIAG` trace, a captured
stderr, a gate log) comes back empty, re-run it with **`-a`** before forming any
hypothesis about the program. Do not reason from the empty result.

**Why:** GNU `grep` sniffs the first buffer and, if it finds one non-text byte,
suppresses *every* matching line — the `Binary file … matches` note goes to
stderr and vanishes under a redirect or a pipe. Stdout is then byte-identical to
what an empty or missing file produces. In this project, `grep -n "diag-keys"`
over two `ui-verify` trace files printed nothing and was read as *"the
instrumented build went quiet"*; `grep -a` showed every line, present all along.
A GL driver banner or a codepage mismatch is enough, and the offending byte need
not be near the lines you are asking about.

The error's direction is what makes it expensive: a wrong exit code leaves text
to read, a suppressed match leaves none. An absence cannot be cross-checked by
looking harder — only by changing the instrument. See
[[feedback_trace_grepping_check]] and [[feedback_count_command_wrong]] for the
two siblings.

**How to apply:** `-a` unconditionally on program-written files; it costs nothing
on a text file. Same for `rg --text`. A waiter of the form
`until grep -q "frame n=" "$LOG"` never terminates for the same reason and
presents as the application hanging — `grep -aq`. A reader written in Rust that
decodes with `from_utf8_lossy` (as `Trace::read` does) is immune; only the
shell-side spot checks lie.

Full lesson: `C:\personal_rag\claude_code\lesson_20260922_grep_binary_detection_prints_nothing.md`.
