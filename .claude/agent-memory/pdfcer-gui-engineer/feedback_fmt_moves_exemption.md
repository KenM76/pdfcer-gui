---
name: fmt-moves-a-literal-off-its-exemption-line
description: cargo fmt can wrap a format! so the literal is no longer on the line after a `// ui-text-exempt` comment; check-ui-strings then fails. Put the exemption at the END of the literal's own line.
metadata:
  type: feedback
---

Put a `// ui-text-exempt: <reason>` trailing on **the literal's own line**, never
as a comment above `format!(`.

**Why:** 2026-09-23. The exemption sat above `format!(`. `cargo fmt` then split
the call, which moved the literal down a line. The comment no longer sat
directly above the literal, so a full gates sweep went 78/1 red on
check-ui-strings, after a 10-minute run.

**How to apply:** after `cargo fmt`, run `bash tools/gates/check-ui-strings.sh`
(it takes seconds) before starting the full sweep.
