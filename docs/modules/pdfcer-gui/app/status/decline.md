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
