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

## Item notes

### `const BOX_PT`

The same 220 pt [`super::stamp_size`] uses, for the same two reasons:
unambiguously a drag rather than a click the gesture machine might round to
one, and small enough to stay on the sheet from any `--doc-point` that is
itself on it.

### `const WANTED_PT`

**30, and the number is load-bearing in three directions.** It must not be
`12` (`StampStyle::default()`'s flat size, which a build that threw the typed
value away and re-defaulted would produce); it must not be `24` (what
[`super::stamp_size`] presses in the placing dialog, so a build that somehow
replayed the placing choice would be caught); and it must not be whatever a
220 pt box derives, which is **asserted at run time** rather than assumed —
see the module header's first falsification.

### `const SAME_PT`

Half a point, and it is a tolerance rather than an equality because the
number makes a round trip through a `/DA` string and back. The engine writes
what it was given; the reader parses what it finds. A build that wrote 30 and
read back `29.999999` is not the defect the operator reported, and a check
that failed on it would send whoever read the report into the wrong file.
Anything a *wrong* build produces here — a dropped edit leaving the derived
size, a re-default to 12 — is tens of points away, not tenths.

### `const PROPERTIES_TAB_REGION`

⚠ A dock draws only its **active** tab. In Review the right dock opens on
Comments, so a check that read the trace without pressing this would report
*"the panel published no size row"* about a build whose panel is perfect.
That mistake has been made on this project three times, in three checks, and
produced zero application defects.

### `fn row_size`

# Why the panel's own line and not the `ui-rect` line

`ui-rect` carries a name and a rectangle and no text whatsoever, so a check
that tried to read the number out of it would answer `None` on every build
that has ever existed — and would then go on to say the field *"reads
nothing"*, narrating an absence it never measured. The application grew
`stamp-label-row` for this. Every caller here treats `None` as a **failure to
observe** and reports it as such, never as a reading.
# `last`, and why a fossil is the RIGHT reading here

This suite's standing hazard is reading a whole capture's `last()` and
getting a line the surface stopped emitting some time ago — three wrong
defect reports came from exactly that. `stamp-label-row` is the one shape
where the fossil is the answer: it is written through `diag::trace_changed`,
a **state slot**, so the newest line is by construction what the row is
displaying now, and silence means *nothing changed* rather than *nobody is
looking*. Reading only lines newer than some anchor would turn "the panel
still shows the old number" — which is the operator's complaint, exactly —
into "the panel reports nothing", and send the reader to the wrong file.

[`row_size_since`] is the companion for the one question this cannot answer:
*did the row redraw at all since I pressed Enter?*

### `fn row_size_since`

The distinction [`row_size`] cannot draw. After a successful edit
there are three outcomes and only two of them are visible to a whole-capture
read: the row re-reported a new size (good), the row re-reported the same
size (a failed round trip), or **the row said nothing at all** — which means
the panel did not redraw it, and points at the selection rather than at the
size. Separating the third is what stops one message being written about two
different defects.

### `fn the_typed_size_collides_with_nothing_else_in_the_suite`

`12` is `StampStyle::default()`; `24` is what `stamp_size` presses in the
placing dialog. A build that ignored this field and re-defaulted, or one
that somehow replayed the placing choice, would trace a size that looked
like a pass if [`WANTED_PT`] were either of them. Asserted here because
the reasoning lives in a doc comment, and doc comments do not fail.

### `fn the_two_regions_are_distinct_and_belong_to_the_same_row`

They are the harness's hand-written copies of two constants in the
application, and a hand-written copy is exactly where two ends drift.
Equal names would make the fit assertion pass on a build that draws only
the spinner — the third defect the module header names, silently
unmeasured.
