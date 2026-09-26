# `pdfcer-gui/dialogs/print/remembered`

**What the Print window remembers between jobs** — `OPERATOR_REQUESTS.md`
**O166**, the operator's words of 2026-09-10: *"the printer dialogue box
needs to remember our last settings."*

[`PrintDialog::habits`] reduces the open dialog to the subset of its state
that would still be the right answer for a **different document**;
[`PrintDialog::remember`] writes that subset to the preferences file. The
reading half is in [`super::PrintDialog::open`], which seeds every one of
those fields from what it is handed.

# ★★ The second direction, added at O185

[`PrintDialog::restore`] writes [`super::PrintDialog::opened_with`] back --
the settings the window opened with -- and it exists because Cancel has to
*undo*, not merely *decline*. The reason it has anything to undo is on that
field: a Print the driver refuses saves before it spools and leaves the
window open, so the preferences can genuinely hold values this window put
there and the operator then rejected.

Both directions are one private writer, [`PrintDialog::store`], which is
also the sole emitter of the `print-remembered` trace. That is the point of
the split rather than a tidiness argument: two copies of
compare/assign/save/trace would be two places for the disclosure to drift,
and the failure mode of a drifted disclosure is a trace that reports the
wrong direction while looking entirely well-formed.

# Why this is its own file rather than part of [`super::commit`]

`commit` is what calls `remember`, so proximity would be defensible. But the
subject here is not *printing* — it is a judgement about **which of this
window's twenty controls describe the operator rather than the document**,
and that judgement has a long argument attached to it. Splitting it out
under rule R2 was the opportunity to put it where its argument is.

The rule itself, and the reasoning for every inclusion and every omission,
lives on [`crate::app::prefs::PrintPrefs`]. Read that first; this file is
only the projection.

# The two properties that keep this honest

- **Writing is compiler-enforced.** `habits` is a struct literal with no
  `..Default::default()`, so a field added to `PrintPrefs` stops this file
  building rather than being silently written out as its own default.
- **Reading is test-enforced.** Nothing about the *other* direction is
  visible to the compiler: a field that `PrintDialog::open` never mentions
  compiles perfectly and is simply inert. That gap is closed by
  `crate::app::prefs::printing::tests::every_remembered_field_is_read_back_by_the_print_dialog`,
  which reads the struct declaration out of the source rather than carrying
  a hand-written list of its own.
