---
name: an-absence-claim-is-a-claim-about-every-route
description: Never write "pdfcer-core cannot do X" without grepping source for every verb that could reach the operator's goal — checking one verb is not checking every route, and this failure has now recurred three times in two directions.
metadata:
  type: feedback
---

**Before writing that `pdfcer-core` cannot do something, grep the source for
every verb that could reach the same operator goal — not the one you thought
of.** An absence claim about a crate you do not build is a claim about **every
route**, and one verb is not every route. The grep happens before the file is
**saved**, not before it is sent.

**Why:** three instances in two directions, all in 2026-08.

1. **Ours, and it cost six weeks.** `OPERATOR_REQUESTS.md` O37 published a
   fourteen-row table with a column of crosses under *"on EXISTING text"*,
   concluding pdfcer could style text at creation and never afterwards. It had
   read `add_text`, `edit_text` and `delete_text_run` — a reasonable set — and
   not `format_text`, which had shipped three weeks earlier, been extended
   twice, and become form-retargetable five days before the request that said
   it did not exist.
2. **Theirs, same failure.** The engine measured `set_font`'s refusal, confirmed
   it was real, and inferred that bold on existing text was impossible — without
   asking whether a *second* verb reached the same goal. `set_synthetic` had
   shipped six weeks earlier. They minted R220 for it and retracted the same day.
3. ★ **Ours again, within the hour of reading their retraction.** A reply
   answering their correction asked for a `FormatReport::synthesis` field on the
   grounds it did not exist. It does — `format.rs:913`. The document whose whole
   subject was somebody making this mistake contained the same mistake.

Instance 3 is the one worth remembering: **the pull toward "I looked and did not
see it, therefore it is not there" survives reading a note about itself.** It is
not carelessness and it will not be fixed by intending to be careful.

**How to apply:**
- Any sentence of the form *"pdfcer-core has no …"*, *"there is no verb for …"*,
  *"blocked on the engine"* → grep `D:\Dev\pdfcer\crates\pdfcer-core\src` before
  saving the file that contains it. Search by the **operator's goal**
  ("bold", "colour", "restyle"), not by the mechanism you have in mind.
- The same applies to `NO_SURFACE.md` entries and to any `⛔` row in
  `FEATURES.md` — see [[a-backlog-row-is-a-record-not-evidence]], which is this
  rule pointed at our own documents rather than at the engine's.
- When you find you were wrong, **strike the claim in place rather than
  deleting it.** A wrong table that cost six weeks is worth more as a warning
  than as a gap; both the engine and this project now do it that way.
