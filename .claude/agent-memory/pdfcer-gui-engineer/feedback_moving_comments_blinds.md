---
name: moving-comments-out-can-fail-a-gate
description: Some gates read comment content in .rs (check-conventions reads `conventions:` blocks); a comment move must keep those blocks in source
metadata:
  type: feedback
---

Moving long `//!` headers into docs/modules (2026-09-25) failed
check-conventions on 16 files: it requires the `conventions: <class>` block to
be IN the source file. The fix put those sections back after the pointer line.

**Why:** comments are not inert here — several gates parse them. A move that
passes `cargo check` and a comment-only diff can still fail a gate.

**How to apply:** before any comment move/strip, grep tools/gates for what
comment text they read (`conventions:`, trace samples, exemption markers like
`<!--namesake:`), keep those in place, and run the full suite. Long headers
(>30 lines) belong in docs/modules per DEVELOPING §5.1 — see [[crate-split-is-standing-background-work]].
