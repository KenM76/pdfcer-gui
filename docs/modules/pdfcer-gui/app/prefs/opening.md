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
