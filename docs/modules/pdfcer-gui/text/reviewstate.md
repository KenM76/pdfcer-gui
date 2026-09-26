# `text::reviewstate` — every word the review-status control says

One module for one subject: `/State` and `/StateModel` (§12.5.6.3, Table
171 — 2.0's Table 174), read by `pdfcer-core` `Pass 253.1` and authored by
[`pdfcer_core::edit::EditSession::add_review_state`]. It covers the status
line on a comment row, the control that records one, the status chooser
beside the existing filter, and the two sentences the status row says after
the edit lands.

It is deliberately **not** in [`crate::text::panels::comments`], even
though most of these strings are drawn by that panel. The subject here is a
*vocabulary the standard defines and the engine refuses to interpret*, and
the wording decisions below are all consequences of that one fact rather
than of anything about panels. Keeping them together is what makes the next
reader able to check them against §12.5.6.3 in one pass.

## ★★★ THE FACT THAT DECIDES ALMOST EVERY STRING BELOW

**A review status is APPENDED, not set.** `add_review_state`'s own doc
comment states it twice with the standard's `shall`:

> *"THE STATUS IS NOT WRITTEN ONTO THE ANNOTATION IT DESCRIBES … §12.5.6.3
> puts it on a **separate** `/Text` annotation that points at the reviewed
> one through `/IRT` … That is why this verb returns a new `ObjId` rather
> than mutating the target, and why nothing about the target changes."*

and

> *"AND A SECOND STATUS CHAINS ONTO THE FIRST, PER AUTHOR … 'Additional
> state changes shall be made by adding text annotations **in reply to the
> previous reply** for a given user.' So this verb walks the `/IRT` graph
> rooted at `target` … building a per-author chain rather than a star."*

⇒ The file holds a **log**, not a field. So nothing here may say *set the
status*, *change the status* or *the status is now*. [`record_label`] says
**Record status**; [`status_recorded`] says what was added and how deep the
operator's own history on that comment now is; [`row_status_history`] says
how many earlier statuses stand behind the one being shown. A control
labelled *Set status* would describe a different document format.

## ★★ THE SECOND FACT: THE ENGINE DOES NOT INTERPRET THE STRINGS

`Annotation::state` and `Annotation::state_model` are `Option<String>`,
decoded verbatim, and `state`'s own doc says why:

> *"Neither key carries a 'shall be one of' anywhere in either edition, so
> a value outside Table 171's vocabulary is **unhandled, not illegal**.
> Reporting it verbatim is therefore reading the file rather than tolerating
> it. `ReviewState` is the closed set pdfcer AUTHORS; this is the open set
> it reads."*

⇒ The vocabulary is **this shell's to present**, and an unknown value must
be **shown**, never normalised. That is why there are three families of
string for one value — [`state_name`] for the seven pdfcer authors,
[`state_unmodelled`] for a value in a model pdfcer authors, and
[`state_foreign`] for a value in a model it will not author. The
distinction, and the reason collapsing the last two would be wrong, is
[`crate::text::buttonaction`]'s, reused rather than re-derived; see
[`crate::panels::comments::reviewstate::StateReading`].

## ★ Where these strings are NOT

**Never on the canvas.** R8b — *"fuzzy, never sneaky"* — and its clause
about the original GUI: *"the nagging and red flagging … made for a lot of
extra bugs in the visibility when editing."* A review status is a
**disclosure about a comment**, not part of the comment's appearance, so it
is drawn in the panel and in the status row and nowhere else. A mark whose
status is *Rejected* is drawn exactly as the file will draw it.

## Conventions, restated from [`crate::text`] because they bind here

- Sentence case, no trailing period on labels; full sentences for prose.
- The **file's own spelling** for any value that came out of the document,
  including `Cancelled`, which Table 171 writes in British English and
  which is therefore not "corrected" anywhere below.
