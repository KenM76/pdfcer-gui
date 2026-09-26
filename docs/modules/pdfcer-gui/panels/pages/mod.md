# `panels::pages` — the document's pages, as pictures

The thumbnail grid — `FEATURES.md`'s Phase 3 row, and one of the surfaces
`MODES_AND_PANELS.md` Part 1's table gives **all three** modes.

| | |
|---|---|
| Ribbon command | `view.panel_pages` |
| Acts on the document | [`Action::GoToPage`], and **nothing else** |
| Owns | [`select::PageSelection`] — the operand list the ribbon's Pages tab already promises |

## Why a page panel sits in **Review**, and not only in Edit

It is in all three default arrangements (`crate::app::modes::defaults::spec`), and
Review is the placement that needed an argument. `README.md` records the
operator's, and it is the reason this panel offers page verbs rather than
only navigation:

> Reviewing a set means rotating a sheet to read it and extracting the
> pages you were asked about. The stance that matters is the content is
> not yours to alter, and page operations do not alter content.

That is what separates rotate/extract/delete from the Edit tab's verbs. A
rotation changes `/Rotate`; an extraction writes a *different* file. Neither
touches a single content-stream operator, so neither breaches the stance
Review takes.

## What this panel draws, and what that costs

The rendering and caching policy is [`thumbnails`]', and its header is the
one to read before changing anything here — it carries the measurements.
The one sentence that decides the shape of this file:

> **A two-pixel render of the benchmark CAD drawing costs 691 ms.** ~99 %
> of a page's cost is resolution-independent, so a thumbnail is *not*
> cheap because it is small.

Consequently this body renders **at most one page per frame**, only for
tiles that are actually on screen, and stops on its own the first time a
page proves expensive. An undrawn tile says so **in words**: a blank
rectangle the colour of paper is a picture of an *empty page*, which is a
thing a real PDF contains, so drawing one would assert something false
about the document rather than merely look unfinished.

## Two ways this panel can go silently missing, and what holds each shut

Neither failure is this module's to make: both live in `shell/`, and both
are **invisible rather than broken**, which is the expensive kind.

1. **An unregistered command hides the whole panel.**
   `crate::app::mod`'s panel registry registers a panel **only if its
   command is registered**, which is `SHELL_FRAMEWORK.md` §5b's capability
   rule — so a `view.panel_pages` left out of the manifest filters this
   panel out of every default arrangement with no error anywhere.
   `every_panel_is_reachable_from_the_ribbon` is what makes that visible.
2. **An undefined menu context detaches every right-click.** [`PAGES_ROW`]
   is attached to every tile below, on every frame, through the same
   [`MenuHost`] the canvas and the Objects panel use — and
   `egui_shell::menu::Menu::attach` treats an unknown context as *"this
   surface has no menu yet"*, so an id `crate::shell::menus::built_in`
   does not define opens nothing at all, silently.

That second seam is also the payoff of routing a right-click through a
context id rather than through a list of items at the call site: the attach
site needs no edit when the menu's contents change.

## What a click does, and what it does not

[`select`] owns the rule; the summary is that a plain click navigates and
picks one page, Ctrl+click toggles without navigating, and Shift+click
extends a range without navigating. **Only a plain click navigates**,
because building a five-page set that dragged the canvas through five
renders would cost ~4 s on a drawing set to perform a gesture that changes
nothing about what the operator is looking at.

## The selection is not a decoration

Every one of the ribbon's Pages-tab tooltips already says *"the selected
pages"* — `pages.delete` is *"Remove **the selected pages** from this
document"* — and `crate::shell::commands`' own comment on that band says
those commands *"respect the thumbnail rail's selection when there is
one"*. This panel is where that selection comes from, and
[`crate::panels::PanelsState::selected_pages`] is how a dispatch arm reads
it.

All six of the verbs this panel's own context menu offers work — rotate
left and right, delete, extract, move up and move down — through five
dispatch arms, since the two rotations share one and so do the two moves.
The reading path is through that accessor and through [`ops::operands`],
which is the single place the *"with nothing picked, act on the current
page"* rule is written down.

## What an edit does to this panel's own state

Nothing here has to remember anything, and that is by construction rather
than by discipline:

| | how it is kept honest |
|---|---|
| the **thumbnails** | keyed on `(edit_epoch, pixels_per_point)`; [`thumbnails::ThumbnailCache::sync`] empties itself the moment the epoch moves, and the epoch moves on every edit |
| the **page count** and the tiles | read from `doc.pages` every frame, which `crate::app::actions::pages::resync` refreshes from the session on every edit |
| the **picks**, after a delete | cleared by the apply arm — every picked sheet is gone, so there is nothing to point at |
| the **picks**, after a reorder | **remapped**, through [`select::PageSelection::remap`]: the permutation states where each sheet went, so the arrows stay usable twice in a row |
| the **picks**, after anything else | [`select::PageSelection::retain_below`] on the next frame, which is belt to all of the above |
