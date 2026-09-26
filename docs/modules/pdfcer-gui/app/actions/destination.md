# `app::actions::destination` — **arriving where a bookmark points, not
merely on its page**


> *"in Acrobat clicking on the nested bookmarks in the drawing package takes
> you to a zoomed in area of the page for the drawing bookmark that was
> clicked on. when we click on ours it just jumps us to the correct page,
> but doesn't send us to the spot on the page the bookmark actually points
> to."*

## Why the view is carried and not just the page

`outline::Destination::Page` carries **both**:

```text
Page { page_index, view: DestView }
```

A navigator that matches `page_index` and discards `view` reduces an entire
outline to a page list. On a drawing package, where every bookmark names a
**detail** on a shared sheet with `/XYZ` or `/FitR`, several bookmarks
pointing at different details of one sheet then all arrive at the same
place — indistinguishable from them being broken. So everything that
navigates to a destination comes through [`actions_for`], and nothing
pattern-matches a `Destination::Page` for its page number alone.

## The views this translates, and what each one means here

| `DestView` | §12.3.2.2 | what this does |
|---|---|---|
| `Xyz { left, top, zoom }` | a corner and a magnification | put that point at the top-left; honour `zoom` when it is given |
| `Fit` | fit the whole page | fit the page |
| `FitH { top }` | fit the width, `top` at the top edge | fit width, then scroll so `top` is at the top |
| `FitV { left }` | fit the height | fit the page, then scroll so `left` is at the left |
| `FitR { rect }` | fit a rectangle | frame that rectangle — the same act the zoom marquee performs |

`DestView` is `#[non_exhaustive]` and holds more than these — the `/FitB`
family, an unrecognised entry, no destination at all. Each of those gets
the page turn and nothing more; see the fallback arm of [`actions_for`].

## A `Point` is scrolled to, and it is the ONLY thing that sets the
## magnification here

Every row above that produces a `Point` produces a *position* and nothing
else. The magnification, if the destination asked for one, comes from the
`Action::Fit(..)` or `Action::ZoomTo(..)` raised beside it, one step earlier
in the same list — and because actions are applied in order, the scroll is
solved against the size that produced. `canvas::destscroll` does not read
the zoom at all.

That ordering is the repair of `DEFECTS.md` D47: a `Point` used to be grown
into a 150 pt square and handed to the framing solver, which raised its own
`ZoomTo` and discarded whatever the fit had just decided. A `/FitH` arrived
at whatever magnification 150 pt of paper requires — on a large sheet,
several hundred percent — and a Word table-of-contents link, `/XYZ x y
null`, arrived magnified despite explicitly declining to name a zoom.

**`null` is not zero.** Table 151 lets `left`, `top` and `zoom` each be
null, meaning *"leave this one as it is"* — and the standard states the
`0`-means-null equivalence **only for `zoom`**, never for the coordinates.
So a literal `0` left edge is a real left edge and must be honoured as one,
while a `zoom` of `0` means *"keep the current magnification"*. Collapsing
those is how a destination at the top-left corner of a page silently becomes
"no change".

## What this does NOT do

It does not clamp a destination into view. A bookmark pointing off the sheet
is a bookmark pointing off the sheet — the engine's own census counts
dangling ones — and quietly landing somewhere plausible would hide a
document defect the operator may need to fix.
