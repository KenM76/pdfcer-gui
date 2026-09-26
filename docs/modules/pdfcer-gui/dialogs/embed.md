# `dialogs::embed` — the confirmation before font programs go into a
document



This header used to say the window has **no** settings, that *"there is no
option to set"*, and that a form here *"would imply choices that do not
exist"*. That was true when it was written and it is not true now, and the
sentence is corrected in place rather than left standing beside its
replacement.

There is exactly one control: **use pdfcer's own copy of a standard-14 face
where none of your fonts answers**. It is off when the window opens, it is
drawn only when it would change something, and the fonts it would stand in
for are named beside it before it is ticked. Everything else in the window
is still a report, and the shape below still governs.

### Why the "no settings by design" argument did not survive




## ★★★ WHY THE SWITCH IS OFF BY DEFAULT, and it is not a matter of taste

Two reasons, and the second is the one that decided it.

**1. The letters change.** Embedding a stand-in changes what the document
looks like on the screen of whoever it is sent to. That is disclosed per
row either way — see [`crate::text::embed::embed_row`]'s `Bundled` arm — so
on its own it argues for loud disclosure rather than for a default.

**2. ★★★ It is a LICENCE the operator takes on, not a look they accept.**
pdfcer's fourteen substitute faces are BSD-3-Clause (`THIRD_PARTY_LICENSES.md`,
*"Bundled Foxit substitute faces"*), and embedding one puts it **inside a
file the operator then distributes**, carrying that licence's attribution
condition with it. `pdfcer`'s own CLI says so in the argument's doc comment
and draws the only defensible conclusion: *"That is your decision to make,
so pdfcer does not make it for you."* — `pdfcer-cli`'s
`EmbedFont::use_bundled_fonts` (`--use-bundled-fonts`), off by default.


## What did NOT change

* The rung order. pdfcer's own faces are still consulted **last**, after an
  exact name match and after a standard-14 family equivalence, so a machine
  with fonts configured reaches a real face first whether the box is ticked
  or not.
* The disclosure. Every substituted row still says *"none of your fonts
  matched, so pdfcer used …. It is a stand-in, not the font the document
  asks for."*
* **Nothing is marked on the canvas.** No badge, no tint, no provisional
  styling on substituted text. Rule 4's surviving half is that an inference
  the operator cannot see owes them a report **off** the page — which is
  this window before the act and the disclosure row after it. Both;
  neither on the drawing.

## ★★ The preview is `embed_preview`, and it is the SAME computation the
commit runs

`EditSession::embed_preview(&request)` is `&self` and side-effect-free, and
`embed_fonts` calls it internally before mutating — the engine's own words
for the shape are *"the same value is returned by the preview query and by
the committing call, produced by the same function, so a front end cannot
show one thing and do another."*

★ That is the property `preview_font_resources` had to be fixed to have
twelve hours earlier, and the reason it mattered there applies here: a
preview and a commit that compute the same answer separately eventually
disagree, and the disagreement is silent.

## ★ The plan is computed ONCE, when the window opens

Not per frame. Building it scans every configured font folder — reading and
parsing every font file in each — which measured **3,359 face names** on an
ordinary Windows font directory, and then reads each matched donor's bytes
into memory. A window that redid that sixty times a second would be
unusable, and nothing it depends on can change while it is open: the
document is not editable behind this window, and the folder list is not
either.

## Rule 4

Nothing here marks the canvas, and the disclosures this window shows are the
*pre*-commit half. What the embed actually did lands in the disclosure line
through `app::actions::fonts`.
