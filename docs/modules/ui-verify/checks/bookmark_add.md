# `ui-verify/checks/bookmark_add`

`bookmark_add` — **a bookmark can be written, and the panel gets it back.**

# Why the fixture having NO bookmarks is the point, not a limitation

`SW41177.pdf` is a CAD export with no outline. That is the state the
Bookmarks panel spent its whole life unable to leave: it drew *"no
bookmarks"* and returned early, so the one document that most needs a first
bookmark was the one document where nothing could be added.

The early return is gone, and this check is what proves it stayed gone. A
unit test cannot: the panel body takes an `OpenDoc` and an `egui::Ui`, and
the guard that would come back is a two-line `if outline.items.is_empty() {
return }` that every unit test of `read_outline` would still pass over.

# What is asserted, and the one number that must NOT be used

The engine's reply on `add_outline_item` spends its longest section on this
and calls it *"not a footnote — the entire difficulty of the feature"*:

> `/Count` on the outline root counts **visible** items. Adding a bookmark
> under a **collapsed** ancestor does not change it. A UI that says "added
> N" by diffing that number reports **zero for a correct save**.

So this check diffs `bookmarks-panel items=`, which is
`read_outline`'s **walked-tree** count and not `/Count` — every item at
every depth, open or closed. The distinction is invisible on this fixture
(a first top-level bookmark moves both) and would become visible the moment
anybody adds a nested case, which is exactly when a harness quietly
measuring the wrong quantity does its damage.

# The greyed Add button is asserted in BOTH states

R9 reserves greying for *temporarily* unavailable, always explained. An
empty title is the textbook case — one keystroke away from live — and
`panels::bookmarks::add` greys it with a hover explanation rather than
hiding the row, because the row is the whole of the feature and an operator
would go looking for where bookmarks are added.

Asserting only the live state would let the control ship permanently
enabled, which turns the empty-title press into an engine refusal an
operator never sees. Asserting only the disabled state would let it ship
permanently disabled, which is D5's family: a visible control that does
nothing.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | open the Bookmarks panel | `bookmarks.new_title` and `bookmarks.add` declared, on a document with no outline |
| B | read the census | `bookmarks-panel items=N` |
| C | click Add with the title empty | **no** `bookmark-add` trace — the control is greyed |
| D | type a title, click Add | `bookmark-add chars=5`, then `add-bookmark page=… epoch=…` |
| E | read the census again | `items = N + 1`, and no `add-bookmark-refused` |

## Item notes

### `const MODE`

`review`, and the choice is a claim about the PRODUCT rather than a
convenient route to a panel.

⚠ **The mode a check drives in is an assertion, whether or not anybody meant
it as one.** Read is the mode the application starts in, so it is always the
cheapest way to reach a panel that is shown in all three — and driving
authoring there quietly asserts that the mode whose whole promise is that it
changes nothing offers Add, Rename, Remove, Copy, Cut and a drag hint.

`MODES_AND_PANELS.md`'s panel table draws the line by name, giving Read
*"Comments (read)"* against Review's *"Comments (authoring)"*. The
authoring half here is gated on `Capabilities::authors_anything`, so
**Review is the lowest mode that has it** — which is also the right mode to
test it in, because it proves the gate admits more than Edit. A bookmark is
document *structure*, not page content, so Review must keep it.

⇒ This check fails if authoring is missing from Review, and
`read_mode_offers_no_bookmark_authoring` is the other half of the pair.

### `const PANEL_TAB`

The evidence that the panel is OPEN, independent of anything its body
draws — which is what lets an absence test tell "nothing is offered"
from "nothing opened".
