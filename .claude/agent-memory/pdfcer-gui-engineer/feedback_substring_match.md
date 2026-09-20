---
name: a-substring-match-on-another-crates-prose-survives-a-narrowing
description: Selecting behaviour by `err.to_string().contains("…")` keeps firing after the engine narrows the condition — the words stay, the meaning shrinks — and "delete this arm when it ships" can be wrong on the day it ships
metadata:
  type: feedback
---

**Never select behaviour on a substring of another crate's error prose. Read
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
