---
name: a-proxy-condition-survives-one-correction
description: A gate corrected once for testing a proxy will usually still be testing a proxy, one level down — ask what the mechanism READS, not what a human would look at.
metadata:
  type: feedback
---
<!-- old-name-exempt-file: this memory is about the engine rename shim, so the old crate names ARE its subject. -->

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

Related: [[a-check-that-cannot-fail-is-not-evidence]],
[[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]],
[[a-temporary-shim-needs-a-tripwire-that-names-its-own-deletion]],
[[the-engine-session-runs-in-parallel-and-answers-within-the-hour]].
Full write-up:
`D:\dev\rag\rust\a_git_path_dependency_reads_committed_history_so_a_gate_on_the_working_tree_asks_the_wrong_question.md`

**Same shape:** **Never select behaviour on a substring of another crate's error prose. Read
the variant. And when a note says "delete this when X ships", check whether the
CONCERN survived X, not just the condition.**

**Why:** 2026-09-11, two findings from one arm.

**1 — the substring outlived its subject.** A redaction refusal selected its
operator sentence with `reason.contains("hybrid-reference")`. At `Pass 281.0`
the engine narrowed `WriteError::HybridFullRewrite` from *any hybrid file* to
*a hybrid whose `/XRefStm` could not be parsed*. The **words did not change** —
the message still says "hybrid-reference" — so the match kept firing, now for
a sentence that had become about a different and much rarer file. The operator's
own `SW41177 MATERIAL REQUIREMENTS.pdf`, which is what prompted the request in
the first place, had started redacting successfully days earlier. A locator for
another crate's message format living inside a GUI is a dependency on prose,
and prose is the part of an API with no compatibility promise. Replaced with
`RedactApplyRefusal { broken_xref_stream: bool }`, set from `matches!(err,
WriteError::HybridFullRewrite)` at all three construction sites — the variant
is in hand at every one of them, so the string was never necessary.

**2 — ★★★ the arm's own "delete me" note was wrong on the day it came true.**
It said *"delete this arm when it ships"*. It shipped. Deleting it would have
been the defect: the engine's remedy for the narrowed error is **"use
incremental save"**, and a redaction is the one operation forbidden to take
that remedy (R35 — an incremental save leaves the un-redacted bytes in a prior
revision). So an operator following the writer's advice produces exactly the
file the redaction existed to prevent. The arm was rewritten to the narrowed
fact, and now warns *"do NOT save it the ordinary way and assume the marks
took"*.

⇒ A retirement note records the **condition** the author expected, not the
**concern** behind it. When the condition arrives, re-derive the concern.

**3 —** the test that should have covered the arm never reached it: the fixture
used `reason: "hybrid"`, which does **not** contain `"hybrid-reference"`. A word
chosen because it *reads* like the case does the reader's convincing and none of
the assertion's work.

**How to apply:**
- `grep -rn 'contains("' crates/` periodically. Every hit that inspects an error
  message is a latent instance.
- A pin bump that "narrows" or "relaxes" an error is the moment to re-read every
  arm keyed on that error — including the ones that still compile and still fire.
- Related: [[a-capability-can-change-meaning-under-a-stable-signature]],
  [[a-check-whose-input-is-chosen-for-convenience-tests-the-assertion]],
  [[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]],
  [[delete-the-workaround-when-the-cause-is-removed]],
  [[a-temporary-shim-needs-a-tripwire-that-names-its-own-deletion]].
