---
name: a-fix-that-names-its-victims-can-still-miss-one
description: A guard installed on the write path does not cover the delete path — reaching a default state by REMOVING a file is a second route, and a fix whose doc comment lists the exact checks it repairs can still miss one of them.
metadata:
  type: feedback
---

**When you centralise a guard into "the only write path", ask what the other
ways of reaching the same state are — deleting the file is one of them, and it
is not a write.** A default state can be arrived at by *setting the defaults*
or by *removing the thing that overrode them*, and a header prepended to every
write covers exactly one of those.

**Why:** measured 2026-09-12 in the full driven sweep.

The application shows a first-run offer (*"Open PDFs with pdfcer"*, O173)
unless `preferences.txt` contains `ask_default_app = false`. The harness seeds
that key into every sandbox. Three checks wrote their own preferences file and
blew the seed away; one of them consequently measured a modal dialog's client
area and filed a **confident, specific, wrong FAIL against the application**.

The repair was excellent and is worth copying: the suppression became a
**header of the only write path** (`sandbox::write_prefs`), so a check can seed
whatever keys it likes and *cannot* restore the offer by doing so. Its doc
comment carries a three-row table naming each affected check and what each one
had done.

★★★ And it still missed one — **the one its own table names.**
`print_remembered` reaches the shipped defaults on its control run by
`std::fs::remove_file`, not by writing. Deleting is not writing, so the
write-path header never applied to it, and the sweep three weeks later still
found `dialog-refocus title="Open PDFs with pdfcer"` in that check's trace. The
sibling named in the same table was fixed correctly, by **resetting the file to
the seed rather than deleting it** — the right shape, applied to one of the two.

⇒ The tell was in the failure message: *"the click produced no
`ribbon-tab-activated` line, so no click reached the ribbon."* That sentence
names a cause it never measured (see
[[an-unevidenced-excuse-is-worse-than-silence]]). The trace said what really
happened in one line, forty lines above.

**How to apply:**
- Centralising a guard into a write path: grep for `remove_file`, `remove_dir`,
  truncate, and "reset to defaults" in the same module before believing the
  guard is universal. Absence is a state with two routes into it.
- Prefer **reset-to-seed over delete** whenever a check wants a pristine
  starting state. It reads the same, it is one call, and it cannot lose a
  suppression the sandbox installed.
- A fix's doc comment listing the cases it repairs is a **claim to verify per
  case**, not a receipt. Re-drive each named case, or the list is prose.
- Sibling of [[an-absence-claim-is-a-claim-about-every-route]] pointed at a
  *fix* rather than at a *claim*, and of
  [[a-proxy-condition-survives-one-correction]].

---

## ✓ CLOSED 2026-09-13 — and the closure is the instructive part

`print_remembered`'s two delete sites (the control launch and the `Neutral`
drop guard) now go through `sandbox::reset_prefs`, and the check **PASSES**:
twelve remembered print settings come back into the dialog's own fields and the
job is planned with them — a capability the suite had never once observed. The
failed reset is a SKIP now, not a verdict-neutral note, because a reset that
failed leaves print keys the check knows nothing about and the twelve values
below would then be somebody else's settings recorded as "the shipped
defaults": a baseline wrong without being empty.

**All five prefs-touching checks were re-driven this time**, which is what the
doc table should have got when it was written: `default_app_offer_is_asked_once`
(the one legitimate deleter, correct to be), `a_page_display_choice_survives_a_
close_and_reaches_a_new_document`, `ui_scale_resizes_the_chrome`,
`the_page_preview_limit_is_remembered_and_zero_means_never`, and this one. All
pass.

**★★★ What made the delete look free, and it is worse than the delete.** A long
comment above it asserted *"`userdata/` is not among the sibling directories the
sandbox brings, so every check begins with no preferences file of any kind"* —
and therefore *"the delete at the top of this function always finds nothing."*
**True the day it was written.** `sandbox::seed_prefs` then began writing
`userdata/preferences.txt`, and **no signature anywhere changed**, so neither
the compiler nor any test could see the comment become the opposite of the
truth. [[a-capability-can-change-meaning-under-a-stable-signature]] is the
engine-side twin of this; the in-tree version is cheaper to cause, because the
thing that moved was a **side effect** of a sibling function, not a meaning.

⇒ A comment asserting *what the environment contains* is a measurement with a
timestamp. When a function starts creating a file, grep for prose claiming that
file's absence.

**★★★ And the paragraph arguing for the delete was right in every clause.** It
said the neutral state is *no print preferences at all*; that a fresh
`userdata/` has exactly that; that `PrintPrefs::default` is specified against
it; and that writing the defaults back would make a later reader think the
operator had chosen them. All true. It missed that **the file holds one key that
is not a print preference.** ⇒ A correct argument about what a state means can
still be wrong about how to reach it; the conclusion is a separate claim from
the premises.

⚠ **The diagnosis above sat in this file for a day while three project
documents named the ribbon.** See
[[a-register-row-outranks-memory-so-correcting-the-row-is-the-work]].
