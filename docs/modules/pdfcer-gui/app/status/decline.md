# `pdfcer-gui/app/status/decline`

The worded decline: telling the operator that a command did *not* run.

A decline ("Nothing to zoom to") occupies the status bar's left half beside
the two rule-4 disclosures in [`super`], and is a different speech act from
them:

| | says | is true because |
|---|---|---|
| disclosure | this happened, and here is the part you cannot see | a document changed |
| decline | this did not happen | a document did **not** change |

They share the place and the discipline — the same [`super::disclosure_line`],
the same named-region publication, the same fixed row — and nothing else.
They must not share a store or a wording: one slot for both would make a
completed gesture and a refused one wear the same sentence in the same place.

# Retirement

A decline is **not** keyed on [`crate::app::state::OpenDoc::edit_epoch`],
which is what makes the two disclosures safe and would make this wrong:

1. A decline changes no document, so the epoch never moves and an
   epoch-keyed decline would never retire.
2. Two identical declined chords are two events, and the operator needs the
   second to register. The epoch key is identical across both, so it cannot
   tell a repeat from an unretired first.

It is retired instead by the operator's next act, in two places:

- **[`retire`], at the dispatcher.** `crate::app::dispatch` is the choke
  point every command arrives at, so it is the one place that knows the
  operator has just invoked something. Retiring there, before the new
  command's arm runs, means re-pressing a declining chord ends the sentence
  and raises it again — reason 2 made mechanical.
- **[`live`]'s still-true filter, at the bar.** Selecting something is a
  canvas gesture and reaches no dispatcher, so the bar draws the sentence
  only while the reason that produced it is still true, asked through *the
  same predicate that produced it* ([`zoom::can_zoom_to_selection`],
  [`zoom::last_frame`]) rather than a second spelling that could drift. A
  decline can therefore go stale but never become a lie, and the dispatcher
  handles stale.

The filter is a filter rather than a clear, for the same reason
[`crate::app::actions::last_edit_disclosure`]'s epoch comparison is: state
that must be cleared is state that will one day be shown against the wrong
document.

# What this module deliberately does not word

**A region zoom clamped by the raster ceiling is a partial grant, not a
decline.** [`ZoomOutcome::Zoomed`] carries both the scale asked for and the
scale pinned, and [`ZoomOutcome::ceiling_changed_the_answer`] reports when
they differ. Wording it here would be wrong twice over: the region *is*
framed at the closest scale the page can reach, and the clamp already
reports itself where the operator is already looking, because the framing
verb raises `Action::ZoomTo` carrying the clamped number and the zoom
readout states it on the same frame. A line that fires when nothing was
declined trains the operator to stop reading the line.

# Why the store is a thread-local

It should be a field on `OpenDoc`, and `crate::app::state` is not this
module's to extend — a territory boundary rather than a design judgement,
stated here so whoever lifts it knows the preferred shape.

It is sound regardless, and more obviously so than the same pattern in
[`crate::app::actions::last_edit_disclosure`] and
`crate::panels::forms::edit`: this is not document state. It records that a
command declined, it cannot change a pixel, nothing reads it but the bar
deciding whether to draw a sentence, and `eframe`'s update loop is one
thread — so writer and reader are the same thread, while a test on another
thread gets its own empty slot rather than another test's leftovers.

It needs no document identity. A decline that outlived a document close is
filtered out on the next frame anyway, because a freshly-opened document has
drawn no page and has nothing selected, which makes the sentence true rather
than stale.

## Item notes

### `const REGION_DECLINE`

Named for the same reason its two disclosure siblings are: the whole
requirement of a decline is that it is **on screen and legible**, and
`ui-verify` can only assert that about a rect the application published.
Matched literally by `tools/ui-verify`, so renaming it silently un-aims
whatever check was measuring it.

### `mod fresh`

See `decline/fresh.rs`'s header for the seam. In one line: it is the
**pure** half of this module's second job, and it leaves behind the half
that needs a context and a document ([`live`], [`show`]), so each file now
has one testability story instead of two.

### `mod canvas`

Its own file rather than a function in `record`, for `clipboard`'s and
`textedit`'s stated reason: it carries an argument of its own — why an
action raised by a read-only surface must carry a two-armed vocabulary and
not a [`Declined`] — and that argument would be buried among twenty
siblings. The size gate decided it as well: this file stood at 1,367
lines and the variant above is fifty-seven of them.

### `enum Declined`

A *narrower* type than [`ZoomOutcome`] on purpose: that enum's third
variant is a zoom that **did** happen (possibly clamped, which is a partial
grant and not a decline — see the module docs), and a store that could hold
it would be a store a future edit could word. This one cannot represent a
grant at all.

### `fn of`

`None` for [`ZoomOutcome::Zoomed`] **including the clamped case**. See
the module docs: a clamped framing zoom is a partial grant that already
reports itself through the zoom readout, and wording it here would word
a non-event.

### `fn retire`

Called at the top of `crate::app::dispatch::PdfcerApp::dispatch_command`,
before the arm for the new command runs. That placement is the whole
retirement rule and it is deliberate on both counts:

- **the dispatcher**, because it is the one choke point that knows an
  operator has invoked *something*, and "the next thing you did" is the
  only honest lifetime for a sentence about a gesture. See the module docs
  for why an epoch cannot serve here;
- **before the arm**, so that re-pressing the declining chord retires the
  old sentence and then [`record`]s a new one. Two presses are two events
  (module docs, reason 2), and this is where that becomes mechanical rather
  than aspirational.

Idempotent and free: one `Option` write per *invoked command*, which is an
operator click, not a frame.

### `fn live`

The bar's read. Both facts are gathered from the modules that own them —
[`zoom::can_zoom_to_selection`] is the same predicate `view.zoom_selection`
is gated on and the same one [`zoom::zoom_to_selection`] declines from, and
[`zoom::last_frame`] is the same record the framing verbs check for
[`ZoomOutcome::NoCanvas`]. Asking the producing predicate rather than an
equivalent-looking one (`doc.page_texture.is_some()`, say, which is a
*different* question by one frame) is what keeps the retirement rule from
drifting away from the decline it retires.

Filters rather than clears; see the module docs.

### `fn record_inside_form`

# Recorded by the DISPATCHER, not by an apply arm

[`record_history_empty`]'s docs argue the opposite placement for undo, and
the argument holds there: *"is there anything to undo?"* is a question about
the document that the apply phase has to ask anyway, so asking it twice is
how the greyed control and the sentence come to disagree.

This one is different in the way that matters. *"Is this selection inside a
form?"* is answered from the **selection**, which the dispatcher holds, and
there is no apply phase to reach: the refusal is that no `Action` is raised
at all. An arm that raised a doomed action so that the apply phase could
decline it would be manufacturing an edit in order to have somewhere to
refuse it.

### `fn recorded_for_test`

[`live`] is the bar's read and applies the retirement filter, which needs a
document and a context. A test asserting that a *dispatcher* recorded a
decline is asking a narrower question — did the sentence get written down? —
and routing it through the filter would make the assertion depend on zoom
bounds and canvas state that have nothing to do with what it is testing.

`cfg(test)` rather than `pub(crate)` unconditionally, so nothing in the
shipped build can read the store without the retirement rule.

### `fn live_for_test`

# Why this exists when [`recorded_for_test`] is right there

Because they answer different questions and one of them was standing in
for the other. `recorded_for_test` reads the store; `live` re-asks the
sentence's predicate and is the only thing the bar calls. A decline whose
predicate is false on the frame it is written is **recorded and never
readable**, and a test that stops at the store cannot tell the two apart.


`live` is `pub(super)` and stays that way: the bar is the one reader.
This is a `#[cfg(test)]` widening, so it cannot become a second reader in
a shipped binary.

### `fn show`

Drawn through [`super::disclosure_line`] rather than by hand, which is the
point of that function existing: the R128 defence is four small rules that
only work together — a bounded sub-region, a fixed row height,
`truncate()` rather than wrapping, and the full text on hover — and a third
hand-written copy would be a third chance to omit one of them.

**It does not make the bar taller**, and that matters more here than for
its neighbours rather than less. A decline arrives from a *keyboard chord*,
which is the gesture during which the operator's hands are furthest from
the thing they are looking at; if this line grew the bar, an active
`FitMode` would recompute its zoom from a smaller viewport on the very next
frame and the page would shrink under a gesture that, by construction,
changed nothing. "The page moved when the command did nothing" is the
worst-reading symptom on this surface.
[`tests::a_worded_decline_does_not_change_the_bar_height`] pins it.
