---
name: a-shell-gates-cost-on-windows-is-process-spawns-not-work
description: A tree-wide shell check on Windows is priced in process spawns, not in the work it does — 4,100 spawns took 5 minutes, three greps doing the same job took 3 seconds
metadata:
  type: feedback
---

On Windows a shell gate's runtime is **process creation**, not the work. The
first cut of `check-engine-citation.sh` looped over 1,378 files running about
three greps each plus a subshell per hit — roughly 4,100 spawns — and took
**5m12s**, which is not a per-commit gate. The identical logic rewritten as
three tree-wide greps, pure-bash `[[ =~ ]]` predicates, a `classify` that sets
a global instead of echoing into `$( )`, and one `xargs -0 wc -l | awk` for
the premise measurement runs in **2.9s**. Same output, 100x.

**Why:** `fork`/`exec` is cheap on Linux and expensive on Windows; a loop body
that *looks* like three cheap commands is three process creations multiplied
by the file count. Nothing in the script reads as slow, so this is not found
by reading it — only by timing it.

**How to apply:** write a tree-wide scan as **one grep that emits every
candidate**, then classify in bash. Never `$(…)` inside a per-item loop —
command substitution is a spawn; set a global. Measure the premise with one
batched `xargs`, not a per-file loop. Two Windows traps come with it: a
`path:line:match` record split on its first colon yields the **drive letter**
as the file name, so run the sweep from inside the root and grep `.` so every
record is relative; and `xargs` batches, so a `wc -l` summary has a `total`
row **per batch** — exclude it by name (`$2 != "total"`), never by reading the
last line (see [[a-count-command-can-be-wrong-not-just-its-quoted-answer]]).
Handle exhaustion is the other side of this: see
[[disk-is-tight-and-target-grows-unbounded]].
