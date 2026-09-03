---
name: a-proxy-condition-survives-one-correction
description: A gate corrected once for testing a proxy will usually still be testing a proxy, one level down — ask what the mechanism READS, not what a human would look at.
metadata:
  type: feedback
---

**When a check guards a mechanism, its condition must be the thing that
mechanism actually reads — and a check that has already been corrected once for
using a proxy is the most likely place to find a second proxy.**

## The incident, 2026-09-03

`tools/gates/check-engine-rename-shim.sh` was a tripwire on a temporary
`package = "pdfce-*"` bridge. Its header carried a proud, well-written record of
catching itself:

> *"★★★ THE CONDITION IS THE ENGINE'S CRATE, NOT ITS DIRECTORY, and the first
> version of this gate got that wrong within the hour."*

It had tested `-d D:/Dev/pdfcer` and fired between the engine's clone Pass and
its rename Pass. The correction tested `-d D:/Dev/pdfcer/crates/pdfcer-core` —
**the crate, on disk.**

That is still a proxy, and it failed for exactly the same reason one level down.
This shell does not build against the engine's disk. Its dependency form is

```toml
pdfcer-core = { git = "file:///D:/Dev/pdfcer", branch = "main", ... }
```

and a `git` dependency resolves **committed history only** — a form chosen
deliberately so the engine session's mid-rewrite working tree cannot break this
build. For about an hour the engine held **795 staged-but-uncommitted renames**:
`crates/pdfcer-core` on disk, `crates/pdfce-core` in `HEAD`. The gate failed the
build, and doing what it instructed would have produced an unresolvable
dependency.

The correct condition is the one Cargo asks:
`git -C "$ENGINE" cat-file -e "main:crates/pdfcer-core/Cargo.toml"`.

## Why: what makes the second proxy invisible

The first correction *feels* like the end of the story. The header now contains
a paragraph about proxies, which reads as evidence the question was settled —
so the corrected condition gets less scrutiny than the original did, not more.
And the second proxy is closer to the truth than the first, which makes it look
right.

★ The tell is not "is this condition plausible" but **"is this the same source
the guarded mechanism consults?"** Three different answers to *"does this crate
exist"*:

| dependency form | reads |
|---|---|
| `path = ` | the working tree |
| `git = ` + `branch = ` | committed history on that branch |
| registry version | the crates.io index |

A gate picking the wrong one is green or red for reasons unrelated to the build.

## How to apply

- When you correct a check for testing a proxy, **re-derive the condition from
  the mechanism**, do not merely refine the previous condition. "What does the
  build read?" not "what is a better stand-in?"
- Treat a header paragraph that congratulates itself on a past correction as a
  **flag**, not as reassurance.
- Falsify through every state, including the awkward intermediate one. Ours was
  driven through four; `git init -b main` was load-bearing, because on this
  machine `init.defaultBranch` is `master` and without it the should-fail case
  **passed** — the falsification itself needed falsifying.

⇒ The tripwire was otherwise right and it worked: two hours after the fix it
fired for real, the shim came out, and the gate deleted itself as its header
instructed. This is not an argument against tripwires. It is about their
conditions.

Related: [[feedback_a_check_that_cannot_fail_is_not_evidence]],
[[feedback_a_measurement_of_the_wrong_surface_looks_exactly_like_a_broken_one]],
[[feedback_a_temporary_shim_needs_a_tripwire_that_names_its_own_deletion]],
[[feedback_the_engine_session_runs_in_parallel]].
Full write-up:
`D:\dev\rag\rust\a_git_path_dependency_reads_committed_history_so_a_gate_on_the_working_tree_asks_the_wrong_question.md`
