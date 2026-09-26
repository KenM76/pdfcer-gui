# `ui-verify/checks/stamp_size_properties`

`stamp_size_in_the_properties_box` — a stamp **already on the page** is given
a new label size by typing into the properties panel, and the number is
asserted to reach the engine and to come back out again.

# The report this closes


> *"still can't adjust the size of a stamp on the canvas, **or by entering a
> different size in the properties box**."*

[`super::stamp_size`] closed the first clause: the Size chooser in the
*placing* dialog is pressed for real and the number is asserted to arrive at
`canvas::textannot::spec`. That check says nothing whatever about a stamp
that is already on the page, which is the second clause and the one an
operator hits every time they change their mind — the placing dialog is gone
by then and never comes back.

This file is the payment for the other half. Engine `Pass 292.0` made it
possible at all: `TextAnnotStyle` grew `font_size` and `stamp_fit`, and
`EditSession::stamp_label_parameters` grew the read half, so a panel can now
both show what size a placed stamp's words are and write a new one.

# Why every unit test in the crate can be green while this fails

Because the panel sits at the far end of a chain that no test in the crate
can stand at the near end of. The hops, and the test in front of each:

| hop | its test | what that test cannot see |
|---|---|---|
| the session reports a stamp's label parameters | the round-trip test in `markup::tests` | it calls the reader directly; no panel is drawn |
| `markup.rs` puts them on the `Reading` | the `Reading` tests | most build a `Reading` by hand and never call `with_stamp_label` |
| `size_row` draws a spinner when the label is `Some` | — | **nothing**, because a row is drawn, not returned |
| the spinner raises `SetTextAnnotStyle` | — | same |
| the action reaches `set_text_annot_style` | the apply-side tests | they raise the verb themselves, in code |

Every one of those is green on a build whose properties panel draws the row
**below the fold of its own scroller**, or draws it for a `/Text` and not a
`/Stamp`, or draws it and never commits because the `size != read_size`
guard compares a value that was rounded on the way through. The operator's
sentence is a report about the last hop, and only a hand on the control can
measure it.


# The oracle, and why it is two trace lines rather than a screenshot

A driven check cannot read a number off a picture. Two lines carry it:

```text
pdfcer-diag stamp-label-row size=24.7 source=declared-in-da fit=grow
pdfcer-diag set-text-annot-style-applied id=17 subtype=Stamp icon=false colour=false size=true fit=as_requested was_foreign=0 ap=...
```

* `stamp-label-row` is what the **panel is displaying**, emitted through
  `diag::trace_changed` so a sixty-frame-a-second panel writes one line per
  actual change. It is read twice — before the edit and after it — and that
  pair is the round trip the operator's sentence is about.
* `set-text-annot-style-applied` is what the **engine was asked to do**.
  `size=` there is `TextAnnotStyleChange::font_size_written`, a boolean: the
  engine's own answer to *"did I write a new size?"*, which is not the same
  claim as *"the panel now shows one"*, which is why both are asserted.

Neither line existed before this feature. The lesson behind that is on the
record: **a trace line must carry the number a wrong build would get wrong**,
and before these two, every line this route emitted was byte-identical
between a build that carried the operator's number and one that dropped it.

# The falsifications, which are what make this a test

**The size typed is checked against the size already there.** The stamp is
placed with the dialog's default (*Fit the box I drew*), so its label size is
derived from a 220 pt box and is nothing like [`WANTED_PT`]. If it ever were,
the shell's own `size != read_size` guard would correctly decline to write,
and this check would report a defect that is not one — so the two are
compared, and an equal pair is raised as a **harness** error, named as such,
rather than being allowed to look like an application failure. This project
has filed four defects that did not exist off one harness input that was
wrong.

**The row's presence is asserted before anything is typed**, separately from
the value travelling. A panel that draws no size row and a panel that draws
one and drops the number are the same experience for the operator and two
entirely different defects, and a check that only looked at the end state
would report the second when it was the first.

**The fit chooser is asserted too**, and for its own reason: a build can draw
the number and clip the chooser under it, and the operator then has a size
they can change with no way to say what should give when the words stop
fitting the box. That is a third distinct defect behind one symptom.

# What this check does NOT claim

It does not assert what `pdfcer-core` paints. `size=true` proves the engine
was asked and answered *"I wrote a size"*; `stamp-label-row size=30` proves
the size can be read back out of the annotation the engine produced. Whether
30 pt text is legible, or whether the `/Rect` grew to hold it, is the
engine's own tests' business — and the box growing is disclosed off-canvas by
`annots::textannotstyle` rather than drawn, under R8b rule 4.
