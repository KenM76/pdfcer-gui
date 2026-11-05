---
name: loop-tail-ran-harness
description: A leftover `for` loop appended to a compile command ran ui-verify with no --check and no --no-input — every check, real cursor, ten minutes on his desktop.
metadata:
  type: feedback
---

Never append a harness invocation to a compile command as an afterthought. Every `ui-verify` call carries `--no-input` and `--check <name>` on the same line, typed fresh, never left over from an edit.

**Why:** while fixing a borrow error I tacked a half-written `for e in lens p2; do cargo run -p ui-verify -- --exe …; done` onto the clippy command. With no `--check` it selected every check; with no `--no-input` it drove the REAL cursor and keyboard. It ran ten minutes in the background before I killed both ui-verify processes by PID (Ken's own OneDrive pdfcer-gui windows were left alone).

**How to apply:** before sending any Bash call that names `ui-verify`, read the line for `--no-input` and `--check`. If one is missing, the call is wrong. A harness run goes in its own call, never chained behind a build. See [[drive-off-screen-while-ken-uses-the-pc]].
