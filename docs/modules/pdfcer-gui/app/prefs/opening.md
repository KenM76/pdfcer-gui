# `app::prefs::opening` — what an operator is shown when a page first appears

Two preferences, both read **exactly once per document open** and never
again: how the first page is fitted, and which of the three View ▸ Display
overlays are already on.

## Why these are preferences and not compiled-in defaults

Without them both are constants in `crate::viewer::ViewState::default`, and
`NO_SURFACE.md` §2 is where that state is catalogued: the View ▸ Display
toggles exist, and **the default is not settable**. The operator's words for
it were *"there is no surface for changing or editing the settings for
them"*.

It is worse than that phrase implies, because **the toggle is per
document**. Nothing remembers it: `viewer::remembered` persists the
page-display arrangement and nothing else, deliberately (see its header). So
an operator who works with rulers on flicks the same switch on every
document they will ever open, forever, and the program never learns.

## The trio is ONE setting, not three, because they interlock

`canvas::guides`' [`ruler_drag`](crate::canvas::guides) states the coupling
in its own doc comment:

> Registers nothing when the rulers are hidden, which is why the guides
> toggle is usable on its own but *creating* a guide needs rulers.

So an operator who uses guides needs **two** switches before they can place
the first one, and they need them on every document. Presenting the three as
three separate settings, each with its own title, its own silence line and
its own radius line, would bury that relationship under three copies of the
same three sentences. One setting, three switches, one explanation — and the
explanation gets to say the thing that actually matters, which is that
placing a guide needs the ruler it is dragged from.

## What is deliberately NOT here

**The page-display arrangement** — single, continuous, facing. It already
has a per-document store (`viewer::remembered`) built to an explicit
operator requirement: *"Mode persists per document, not globally — opening a
drawing set must not inherit a report's setting."* A global default for it
would be a second axis colliding with the per-document one, and is
deliberately unbuilt.

The distinction is worth stating because it looks arbitrary from outside:
the display arrangement is remembered per document **because the right
answer differs per document** (a drawing set and a report want different
things). Ruler visibility does not vary that way — it is a property of how a
person works, not of what they are looking at — so a global preference is
the right shape for one and the wrong shape for the other.

## Item notes

### `fn every_opening_fit_has_a_distinct_stable_token`

Same property [`super::RenderQuality`] is held to, and for the same
reason: two values sharing a token makes one unreachable from a
hand-edited file.

### `fn the_shipped_opening_fit_is_what_the_constant_held`

A build whose operator never opens the Settings window must behave
exactly as one with no preference at all — the standing rule for a
capability becoming choosable. This is also the check that catches a
reordering of `ALL` that moved the `#[default]`.

### `fn every_opening_fit_yields_a_positive_zoom`

Not a tautology: `to_view` is the only place the `(FitMode, zoom)`
pairing is stated, and a zero or negative zoom for any of the three
would divide by zero in the canvas geometry rather than fail here. The
two fitting modes recompute against the viewport on the first frame, but
`OpenDoc::assemble` copies the zoom into `observed_zoom` **before** that
frame runs.

### `fn exactly_one_opening_fit_stops_following_the_window`

The distinction the enum exists to make: two of the three values are
*rules* recomputed every frame, and one is a *pinned number*. If a future
edit made `Width` produce `FitMode::None`, the page would open at the
right size and then stop resizing with the window, which reads as a
layout bug rather than as a preference.

### `fn a_bool_round_trips_and_a_typo_does_not_parse`

The second half is the point. A lenient parser that read every unknown
token as `false` would give a typo and a correct `false` the same
outcome and no report, which is the silent substitution the whole
note mechanism exists to prevent.
