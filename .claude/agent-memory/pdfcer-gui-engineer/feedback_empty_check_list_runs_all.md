---
name: feedback-empty-check-list-runs-all
description: ui-verify with no --check runs all 700+ checks; a $(grep …) that builds the list silently empty turned a 3-check re-drive into a 60-min full sweep
metadata:
  type: feedback
---

`ui-verify.exe` given **no** `--check` drives every registered check (723),
not none. A re-drive whose check list was built by a command substitution
(`$(grep -ho 'fn name…' | …)`) came back empty — the name literal sits on the
line *after* `fn name(&self)` — and the run went on for an hour on form and
clipboard checks before anyone looked at its command line.

**Why:** an empty list reads as "nothing to do" to the writer and as "do
everything" to the harness; the output was piped through grep, so nothing
printed until the end.

**How to apply:** build the list, `echo` it, and refuse to run if it is empty
(`[ -n "$C" ] || exit 1`) before passing it to ui-verify. Get names with
`grep -h -A2 'fn name(&self)' … | grep -o '"[a-z0-9_]*"'`. See
[[feedback-drive-only-affected]].
