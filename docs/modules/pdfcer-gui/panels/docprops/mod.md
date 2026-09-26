# `panels::docprops` — the **Document properties** panel: this file's own
title, author, subject and keywords, and the facts pdfcer read about it


> *"the document properties are still always visible in the properties tab.
> it needs to get out of there and be in its own document properties tab."*

Until this commit every line below was the **last section of the selection
inspector** (`crate::panels::properties`), drawn under the armed tool's
settings, the markup restyle controls, the ce-dimension section and the
focused object's read-only facts. It was the only one of those with no
condition attached to it at all, so the *"This document"* heading was on
screen every frame of every session, at the bottom of a panel whose subject
is what you have selected.

★★ **The previous arrangement was DESIGNED, not accidental, and this is a
supersession rather than a bug fix.** `file.properties`' shipped tooltip
commissioned both halves in one sentence — *"The document's own title,
author, subject and keywords, and the properties of whatever is selected on
the page."* — and `RIBBON_IA.md` §5.1 put that one command in File ▸
Document. One command, two subjects, one panel. That reading was coherent
and it is the operator's to overrule; he has.

★★★ **And it was the last thing in the inspector that was not the detail of
anything.** `OPERATOR_REQUESTS.md` O123 / A7 is his: *"I never understood
why there is a tool dock when everything can be in object and properties."*
Objects and Properties became one master–detail column so that picking a row
shows that row's detail — `app::modes::defaults`' Edit arm builds exactly
that shape, two adjacent stacks in one column with a draggable split. A
permanent document-metadata block at the foot of the detail pane is the one
block in that column that answers no selection, so it made the master–detail
reading false for every operator who scrolled to the bottom. That is why the
move is worth the churn, and it is the whole of the justification.

O75 is the same complaint arriving a fortnight earlier and being answered
with a **collapse** — *"the Properties section is always showing the This
document properties instead of just the properties of the objects I am
editing."* The section learned to fold itself shut whenever a
selection-scoped section above it had spoken. That machinery is deleted with
this move rather than kept: there is nothing above this section any more, so
`anything_above` would be permanently `false` and the edge-triggered
`set_open`/`store` dance would be a mechanism whose input never changes. The
reasoning it recorded — why neither `default_open` nor `open(Some(_))` can
express a collapse the operator may override — is preserved in
`crate::panels::properties`' header, because it is a finding about egui 0.35
and not about this panel.

## What it is, and what it is not

| | |
|---|---|
| **subject** | the file: `/Info`, its size, its version, its sheet count and size, its encryption, and whether pdfcer had to rebuild its index to open it |
| **command** | `file.document_properties`, on **File ▸ Document**, beside Properties and Fonts — *"inspection, not action"*, per `shell::manifest::ladder` |
| **modes** | all three. Reading a document's title is **reading**, and Read is shown the `file` tab |
| **not here** | anything scoped to a selection. Every one of those sections stayed in `crate::panels::properties`, which is now purely the detail of what is picked |

## ★ R9 — what it shows with no document open

Nothing of its own. [`crate::panels::Panel::show`] answers the empty case
**once**, for every panel, before any body runs: it forgets the panel state
and draws `crate::text::panels::panel_no_document`. So this module is never
called without a document and has no empty state to get wrong — which is
also why there is no "no document" sentence written here to drift from the
other eleven.

An empty *field* is a different matter and is answered below: a PDF with no
`/Info` dictionary at all renders four empty boxes, because absent is a
value and an empty box is how absent is spelled.


It opened, for months, by quoting `Panel::command_id`:

> Only the second half is built here; the first needs a `/Info` accessor
> that `pdfcer-core` does not expose on `Document` at all.

## ★ That last clause was TRUE when written and false when read

`EditSession::info_text` and `info_bytes` both exist, both are `&self`, and
both are documented as *"reflects unsaved edits"*. `InfoField::all()`
exists too, and its own doc comment was written **for this panel**:

> Every editable field, in the order a properties panel should show them.
> Provided so a front end enumerates the real list instead of hard-coding
> one that drifts when a field is added.

So the blocker had already cleared and the prose had not moved. That is the
**sixth** stale claim of this class found in this project, and the previous
five are recorded in `NO_SURFACE.md` §4 and `RIBBON_IA.md` §5.6. The
generalisable part is the one those notes already state: *a measurement in
a document is a measurement with a timestamp*, and a blocker quoted in
prose is a measurement. **Re-run it before believing it**, especially when
it names a crate somebody else is working on in parallel.

## ★ The disclosure this surface owes, and it is not the obvious one

Not "these are the metadata fields". It is `InfoText::exact`:

> `true` when every byte was decoded with certainty. When `false`,
> re-encoding [`InfoText::text`] would **not** reproduce the original
> bytes, so a front end must not write the field back unless the operator
> actually changed it.

A `/Title` written in an encoding pdfcer cannot fully resolve comes back
with U+FFFD where the unmappable bytes were. The operator sees a plausible
string. If the panel then wrote it back — on a focus change, on a save, on
any "keep everything in sync" impulse — it would **replace the document's
own bytes with pdfcer's guess at them**, silently, in a field nobody looks
at twice.

Two things follow, and the second is the one that is easy to skip:

1. **Never write a field the operator did not change.** Discharged by
   construction: this module commits through
   [`crate::panels::forms::rows::commit`], whose second condition is
   exactly *the draft differs from what the document already holds*. It is
   the same function the Forms panel and the canvas form editor use, so
   there is one rule and three callers rather than three rules.
2. **Say so.** Rule 4's half that survives is the inference the operator
   *cannot see* — and a substituted character in a metadata field is
   exactly that. The row carries a sentence when `exact` is false. It is a
   fact about the **document**, not a pdfcer failure, and is worded that way.

## Why an empty field CLEARS rather than sets an empty string

`set_info_field(field, None)` removes the key; `Some("")` would write an
empty string object. They are different documents, and the one an operator
means by deleting the contents of a box is the first: a document with no
title, not a document whose title is nothing.

The row says so, because it is not guessable and because it is the one
action here that *removes* something.
