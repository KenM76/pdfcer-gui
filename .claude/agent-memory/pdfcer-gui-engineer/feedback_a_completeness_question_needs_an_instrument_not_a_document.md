---
name: a-completeness-question-needs-an-instrument-not-a-document
description: When Ken asks "confirm you have built every X", build the mechanical check keyed on the OTHER side's API — the project's own documents structurally cannot answer it
metadata:
  type: feedback
---

When Ken asks to **confirm** that something is complete — *"confirm that you
have built every editable surface into the GUI that has been implemented in
pdfcer"* (2026-08-28) — the answer is a **script**, not a reading of this
project's documents.

**Why:** on 2026-08-28 the question could not be answered from `FEATURES.md`,
`NO_SURFACE.md` or `GUI_ROADMAP.md`, and the reason is structural rather than a
gap in any of them: **all three are keyed on what this shell does.** None is
keyed on the engine's verb list, so none can answer *"is there a verb
`pdfcer-core` implements that nothing here calls?"*

A 60-line script (`tools/verb-coverage.py` — parse `impl EditSession` out of
`edit.rs`, grep this crate for each `pub fn` name) answered it in two seconds
and found **twelve gaps**, including:

- two operator **settings** that were persisted, validated, drawn in a window
  and honoured by nothing;
- three capabilities the engine had shipped **in answer to this shell's own
  requests** and this shell had then never consumed.

★★★ The last one is the pattern worth carrying: **a reply arriving is not a
capability landing.** The engine session runs in parallel and answers within the
hour, and three separate times its answer sat in
`D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\` while our own doc comments
still said the capability was blocked.

**How to apply:**

1. Any "have we done all of X" question → find the authoritative *list* of X
   (usually on the other side of a crate boundary), enumerate it mechanically,
   diff it against what this side names.
2. Commit the instrument, not just its output — a register that is trusted
   rather than re-measured becomes the next stale blocker. `EDITABLE_SURFACES.md`
   says *"re-run it before quoting any number in this file"* on its first screen.
3. State what the measurement is worth: a **miss** (identifier appears nowhere)
   is strong; a **hit** is weak (a call site behind a condition nothing sets is a
   hit here and dead in the running program).
4. The same shape works for guards: a funnel keyed on option *constructors* was
   blind to a setting delivered by a *setter*, and the way to find the second
   delivery mechanism was to enumerate the engine's API, not to re-read the
   guard. See [[a-backlog-row-is-a-record-not-evidence]].
