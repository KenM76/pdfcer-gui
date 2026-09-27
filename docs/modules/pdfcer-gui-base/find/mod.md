# find — searching the page text, and showing the operator where it is

The whole of Find, across three files:

| module | subject |
|---|---|
| this one | the query and its options ([`FindState`]), the one place a search is run ([`search`]), the wrap rule, and what the position readout says ([`FindState::readout`]) |
| [`bar`] | the floating box: where it sits, the field, the step buttons, the readout, the options menu, and the three keys the field owns |
| [`reveal`] | how one hit reaches the operator's eye — the two-frame handshake, the scroll solve, and the projection out of PDF space |

## The trap, stated first because it is the whole reason this module
is written the way it is

`EditSession` has two search verbs and they are **not** interchangeable:

| verb | wildcards |
|---|---|
| `find_text(needle, case_insensitive)` | **on** — it passes `with_wildcards(true)` |
| `find_text_with(needle, &options)` | whatever the options say; the default is **off** |

In wildcard mode `#` matches any ASCII digit and `?` matches any single
character. **The old pdfcer shell's Find bar ran through `find_text`**, so
typing a `?` into it matched every character on the page and nothing said
why. `pdfcer-core` records that this was fixed *in the front end*, on
purpose: `find_text`'s pattern behaviour is its documented contract and
silently changing it would move results under every existing caller, so
what moved was [`pdfcer_core::edit::TextSearchOptions`]'s **default**, and
a front end that wants patterns now has to ask.

So this module calls **[`pdfcer_core::edit::EditSession::find_text_with`]
and never `find_text`**, wildcards default to off, and the control that
turns them on is labelled with what `#` and `?` do
([`crate::text::find::wildcards`]). `tests::the_default_search_is_literal`
and `tests::a_wildcard_search_is_only_ever_asked_for_explicitly` are the
regression tests; either one fails if someone reaches for the shorter
verb.

### The hazard next door, left where the next person will find it

`EditSession::mark_redactions_by_search` matches **literally** while
`find_text` patterns. A future *"redact every hit"* button built on a
wildcard search would therefore highlight hits that the redaction then
declines to mark — the highlight and the removal disagreeing about which
text exists, which on a redaction is not a cosmetic difference. **This
build has no such button.** If one is added, it must either force
wildcards off for its own search or refuse while they are on;
[`crate::text::find::wildcards_tooltip`] already tells the operator the
two match literally-versus-not, so the words exist and only the control
is missing.

## Where the bar is, and why

**A compact box floating over the top-right of the page**, drawn as an
`egui::Area` positioned from the canvas viewport's own rect — not from the
window's, so a dock opening moves it with the page rather than leaving it
stranded over a panel.

That is where Acrobat Reader, Chrome's PDF viewer and Edge's all put
theirs, and matching them is most of the argument: Ctrl+F is a chord an
operator arrives already knowing, and the surface it produces should be
where their eyes go. This application is meant to replace Reader, and
Reader's Ctrl+F box is a field, two arrows and a settings dropdown in the
top right of the document view.

The second reason is measured rather than conventional. **A docked bar was
built first**, spanning the window above the status bar, and driving the
binary showed what docking costs: the bar takes its height out of the
canvas, the canvas feeds `ViewState::apply_fit`, and under *Fit page*
pressing Ctrl+F moved the zoom from **85 % to 81 %** — and back to 85 % on
close. The page jumps every time the operator goes looking for a word on
it, and jumps back when they stop. An overlay consumes no layout, so it
cannot do that: the page does not move at all.

It costs what an overlay always costs — it covers a corner of the sheet.
Two things keep that small: the box is deliberately narrow (the four search
options are behind an `Options` menu rather than laid out along the row —
see [`bar`]), and it is at the **top** right, which on a drawing sheet is
usually clear where the bottom right is the title block.

**R128 does not reach this surface, and that is a consequence rather than
an exemption.** The rule is *a panel whose size feeds a fit-to-viewport
computation has a fixed size*, and an `egui::Area` feeds no such
computation. The box's width is fixed all the same, for a different reason
of its own: it is anchored by its top-right corner, so a width that changed
with the readout's text would move every control on it. [`bar`]'s header
carries that argument.

## What happens to stale results

**An edit clears the highlights, keeps the query, and says so.**

A hit is a *quad*, and a quad is a claim about where particular glyphs
are. `delete_*` excises byte spans and renumbers; `move_*` rewrites
operands. After either, a quad recorded beforehand can cover different
glyphs, no glyphs, or the right glyphs in the wrong place — and rule 4
forbids painting a mark over content that does not say what the mark
claims. There is no cheap way to tell which hits survived: the geometry
comes from a full document text extraction, so "re-check one hit" costs
the same as "re-run the search".

The three available answers, and why this one:

| answer | rejected because |
|---|---|
| re-search automatically | a search is a whole-document text extraction — 5.6 MB of CAD drawing per edit, on the frame after every nudge of an object |
| keep drawing the old hits | draws a highlight that may be over the wrong text, which is the one thing rule 4 forbids outright |
| **clear the geometry, keep the query, say so** | ✔ |

Mechanically: [`Results`] records the `edit_epoch` it was computed at,
[`FindState::readout`] returns [`Readout::Stale`] the instant that epoch
moves, [`FindState::current_hit`] returns `None`, and the overlay
therefore draws nothing. The bar shows *"Document changed"* with a
tooltip saying to press Enter. Re-running is one keypress and it is the
**operator's** keypress.

Closing the document is the harder version of the same event and is
handled by [`FindState::forget_document`], called from the same two
places `crate::panels::PanelsState::forget_document` is
(`PdfcerApp::open_path` and `PdfcerApp::close_document`) — a hit list
naming pages of a file that is no longer open is not stale, it is
nonsense.

## Searching is not free, and nothing here searches on a keystroke

[`pdfcer_core::edit::EditSession::find_text_with`] runs
`text_extract::extract_document_view` over the **whole document** on
every call — every page, every content stream, decoded, tokenised and
walked, with fonts resolved. There is no cache in `pdfcer-core` and none
here. On the project's benchmark sheet (`ncored-benchmark-cad-drawing.pdf`,
5.6 MB, 129,758 objects on one page) that is a measurable fraction of a
second, and it is a *whole-document* cost that grows with page count
rather than with what is on screen.

A find bar that searched per keystroke would therefore run that
extraction once per character typed — five extractions to type `total`,
four of them for prefixes nobody asked about, each one blocking the UI
thread it is dispatched from. Incremental search is a feature of editors
whose document is already in memory as text; a PDF's is not, and
pretending otherwise is how a viewer becomes unusable on exactly the
files it exists for.

So: **a search runs only when the operator commits one** — Enter in the
field, the step buttons when the results are not current, or a change to
an option after a search has already been run (an explicit click, not a
keystroke, and one whose whole purpose is to change the hit list). Every
run reports its own cost on the `PDFCER_DIAG` channel:

```text
pdfcer-diag find needle="total" hits=47 current=1 page=3 ms=214 \
           case=insensitive whole=off wildcards=off boundary=Alphanumeric
```

…so the number in this header can be re-measured on any file by anyone,
rather than being a claim about one machine on one day.

## Actions, not mutations

Nothing in [`bar`] touches a document. Every commit becomes an
[`crate::app::actions::Action::Find`] carrying a [`FindRequest`], applied
after the frame through the one funnel — and that is not ceremony here,
it is a requirement: the search needs `&mut EditSession`, which means
`Arc::get_mut` on `OpenDoc::session`, which fails while the render worker
holds its clone. The funnel is what makes it legal to stop the worker
first. See [`search`] for that protocol and for how it differs from
`app::actions::vector_edit`'s.

## Why the state lives on `PdfcerApp` and not on `OpenDoc`

`crate::app`'s rule: *state that dies with the document lives on
`OpenDoc`; state that outlives it lives on `PdfcerApp`.* A find query and
its options outlive a document — closing one file and opening another is
the most likely moment to search for the same term again — so
[`FindState`] sits beside [`crate::panels::PanelsState`], with the same
`forget_document` seam for the half that does *not* outlive it.

The one piece that does live on the document is [`Reveal`], and it has
to: it is *view* bookkeeping that spans two frames, exactly like
`ViewFrame::zoom_anchor`, and for the same reason — the page it targets has
not been navigated to yet on the frame the request is made.

## Item notes

### `fn default`

Three of the four are [`TextSearchOptions`]'s own defaults. The fourth
is not, and the divergence is a decision:

**Case.** `TextSearchOptions::default()` is case-*sensitive*, because
its job is to reproduce `find_text(needle, false)` byte for byte for
existing callers. A find **bar** is not an existing caller. Reader's
*Case-Sensitive* toggle is off by default, so is every browser's, and
an operator who types `total` and is not shown `TOTAL` on the next
line reads that as a search that did not work. So this shell starts
case-insensitive and the control turns it off.

**Wildcards.** Off, which is core's default and the whole subject of
this module's trap section.

**Word boundary — `Alphanumeric`, and here is the justification the
brief asks for.** ISO 32000-1 §14.8.2.5 NOTE 1 says outright that
*"the notion of a word is not precisely defined"*, and NOTE 4 offers
three reader strategies without preferring one, so there is no
standard answer to import — only a choice, which is precisely the
shape the operator's standing directive covers: *where standards are
ambiguous those should become settings, with the initial installed
default as the best guess of what is usually followed.*

`Alphanumeric` is that best guess on two independent grounds, and
`pdfcer-core` classifies it as **evidence tier (c)** — what other major
implementations do, as documented, rather than a bare guess:

- Acrobat Reader's own *Whole Words Only* is recorded as an
  exact-boundary match where `stick` does not match `tick` or
  `sticky`, which is what this variant produces.
- It is `\w`/`\b`, the boundary model every mainstream search box and
  regex engine ships, so it is what the operator's habits already
  predict.

The alternatives are better for narrower work and are offered rather
than hidden: `NonSpace` is right when the text is part numbers or
file paths (`A-12/B` is one token), `NonSpaceOrDash` when hyphenated
compounds matter. Neither is a good *default*, because under
`NonSpace` the string `(total)` does not contain the whole word
`total` — which is a surprising answer to give somebody who ticked a
box called "Whole word" and typed an ordinary English word.

### `impl Default`

`#[derive(Default)]` would give [`FindState::zoom_on_jump`] `false`, which
is the **opposite** of what ships — and it would do it silently, in a way
no test that did not name the field could see. That is the same hazard
[`FindOptions::default`] is hand-written for, one struct up, and it is why
the derive was removed here rather than the field being stored inverted.

### `fn search`

# The borrow protocol, and how it differs from an edit's

[`pdfcer_core::edit::EditSession::find_text_with`] takes `&mut self` —
it is a *read* that needs a mutable borrow — and `OpenDoc::session` is an
`Arc` precisely so a render worker can hold a clone while it rasterizes.
`Arc::get_mut` fails while any other strong reference exists, so the
worker is stopped first, exactly as `app::actions::vector_edit` does:
`RenderWorker::cancel_and_wait`'s own docs call itself *"the choke point
that makes `Arc<EditSession>` sound"*.

Two steps of `vector_edit`'s four are **deliberately absent**, and their
absence is the whole difference between a search and an edit:

- **`edit_epoch` is not bumped.** Nothing about the document changed. A
  bump would throw away the page decomposition and the font inventory, and
  would immediately make the results this function just produced *stale by
  its own rule* — a search that invalidated itself.
- **The page texture is not dropped.** The picture on screen is still a
  picture of the page. Dropping it would re-rasterize a CAD sheet on every
  Enter.

A cancelled render is re-spawned by `settle_and_rasterize` at the end of
the same frame if the texture is stale, and left alone if it is not — so
the cost of the cancel is a rasterization that was going to happen anyway,
restarted.

# An empty query is not a search

`find_text_with` already returns an empty vector for an empty needle, so
this could simply run. It does not, because the two states must not look
the same on the bar: "you have not typed anything" is [`Readout::Idle`]
and "there is nothing here" is [`Readout::Empty`], and running a search
for `""` would put the second sentence in front of an operator who had
merely cleared the box.

### `fn step_to`

Declines — visibly, on the trace, and with the bar's own controls already
unavailable — when the results are not current. The bar never raises this
in that state (it raises [`FindRequest::Search`] instead), so reaching
here means a keymap or a future surface got to the verb another way, and
the honest answer is to do nothing rather than to step through geometry
this module has already declared untrustworthy.

### `fn next_index`

Wrapping rather than stopping, which is the opposite of what
`crate::viewer::ViewState::next_page` does — and the difference is not an
inconsistency. Page navigation saturates because *"wrap-around page
navigation silently teleports an operator from page 400 to page 1"*: the
operator is reading, and the pages have an order they care about. Stepping
hits is a **search**, the hit list is a ring the operator is working
around, and stopping at the last one would leave them pressing a live
button that does nothing with no way to tell that from a broken one. Every
find bar in the product class wraps.

`len == 0` is not reachable through [`step_to`], which checks
[`Readout::At`] first, and is handled anyway: an action can be raised from
anywhere and an index into an empty list is a panic waiting for a
customized keymap to find it.

### `struct FindOptions`

A shell-side struct rather than [`TextSearchOptions`] itself, and the
difference is deliberate in three places:

1. **`case_sensitive`, not `case_insensitive`.** The control on the bar
   says *Match case* — the thing the operator switches **on** — and a
   field whose polarity is the inverse of its checkbox is how a `!` gets
   dropped. The inversion happens once, in [`Self::to_core`], with a test.
2. **`TextSearchOptions` is `#[non_exhaustive]`**, so it cannot be
   written as a struct expression from this crate and cannot be exhaustively
   matched. Owning a plain struct keeps the bar's state a plain value that
   `PartialEq` and `Default` work on.
3. **The default differs, on purpose.** See [`Self::default`].

### `fn to_core`

**The one place the case polarity is inverted**, and the one place
`wildcards` is stated at all — which is what makes the trap
checkable rather than a promise: there is exactly one construction of
a [`TextSearchOptions`] in this crate, it is this function, and
`tests::the_default_search_is_literal` reads it.

### `const WORD_RULES`

Narrowest-word-first: `Alphanumeric` splits at the most characters,
`NonSpace` at the fewest. A chooser whose entries are in an arbitrary
order makes the operator read all three every time.

A `const` list rather than a `match` over the enum because
[`WordBoundary`] is `#[non_exhaustive]` — a fourth variant (core names
UAX #29 as a candidate) cannot be matched exhaustively here, and a
wildcard arm would silently drop it from the chooser instead of
failing to compile. This list is the one that has to be extended, and
`tests::every_word_rule_the_chooser_offers_has_a_label` is what says
so out loud.

### `struct Hit`

Not [`pdfcer_core::edit::TextMatch`] itself, for two reasons that both
matter:

- `TextMatch` is `#[non_exhaustive]`, so a test in this crate cannot
  construct one — which would leave every rule in this module
  (stepping, wrapping, the readout, staleness) testable only through a
  real document and a real search;
- the **canvas-space rectangle is computed once, here, at search time**
  rather than per frame. See [`Self::canvas`].

### `struct Results`

The three fields above `hits` are the **currency key**: results describe a
query, under options, against a revision, and any of the three moving
makes them something other than an answer to the question now being
asked. Storing the key with the answer is what lets
[`FindState::readout`] be a pure function of state rather than a flag
somebody has to remember to clear.

### `enum Readout`

A four-way enum rather than an `Option<(usize, usize)>` because the four
states have four different sentences and an operator has to be able to
tell them apart: *I have not searched yet* is not *I searched and there
is nothing*, and neither is *the answer I gave you is no longer true*.
Collapsing any pair of them produces a readout that is silent exactly
when the operator most needs a word.

### `fn open`

Idempotent about the *open* half and deliberately **not** about the
focus half: `Ctrl+F` pressed while the bar is already open is a
request to type in it, which is what every browser and editor does
with that chord, and it is the recovery an operator reaches for after
clicking on the page.

### `fn close`

**The results go with it**, which is what makes the highlights
disappear: the overlay reads [`Self::current_hit`] and the hit list,
and a closed bar with live hits would leave marks on the page with no
surface saying what they are or how to get rid of them. The query and
the options survive — see [`Self::query`].

### `fn set_options`

Does **not** clear the results, and does not need to: [`Self::readout`]
compares the options the results were computed under against the ones
now set, so changing an option makes the results non-current by the
same rule that a changed query does. One currency test, three inputs.

### `fn set_zoom_on_jump`

**Does not touch the results, and must not.** The three
[`FindOptions`] controls make the standing hit list wrong, which is why
changing one re-runs the search; this one does not change which glyphs
matched. Clearing the results here would throw away a correct answer
and make the operator search again to get the same list back.

Two callers: `PdfcerApp::new`, mirroring the persisted preference in at
startup, and the [`PrefAction::FindZoom`](crate::app::actions::prefs::PrefAction::FindZoom)
arm, carrying the operator's
click. Both go through here rather than writing the field so there is
one place to read when the value is wrong.

### `fn set_trim_query`

**Clears the results, unlike [`Self::set_zoom_on_jump`]**, and the
asymmetry is the point: this preference changes which text matches,
so a stored hit list computed under the old value is wrong rather
than merely stale. Leaving it standing would let the operator step
through hits for a needle the setting says is no longer the needle.

Two callers, the same two [`Self::set_zoom_on_jump`] has:
`PdfcerApp::new` mirroring the persisted value in at startup, and the
`Action::SetFindTrim` arm carrying the operator's tick.

### `fn forget_document`

Called from `PdfcerApp::open_path` and `PdfcerApp::close_document`, the
same two sites `crate::panels::PanelsState::forget_document` is called
from and for the same reason: page indices and page-space rectangles
are positions in one file, and carrying them into another one is not
staleness but nonsense. The bar stays open if it was open — the
operator did not ask for it to close — with an empty readout and the
query they last typed, ready for Enter.

### `fn answered`

True when a search has been run for exactly this query under exactly
these options, whatever has happened to the document since. That is
deliberately weaker than [`Self::readout`] returning [`Readout::At`],
and it is the right test for its one caller: [`bar`] asks it before
changing an option, to decide whether the change should re-run the
search. An edit having intervened is not a reason to *skip* the
re-run — the operator has just asked for a different hit list — and
the epoch is not reachable from that call site anyway.

### `fn unsearchable_fonts`

Returns `0` unless the last search is the one the bar is showing, so a
stale or edited-away result cannot leave a sentence on screen about a
query the operator has moved on from. Same staleness rules as
[`Self::readout`], deliberately: two surfaces describing one search must
not disagree about which search it is.

### `fn current_hit`

`None` covers every non-[`Readout::At`] state, staleness included —
which is the mechanism by which an edit stops the highlights: the
overlay asks this, and a stale result answers no.

### `fn page_highlights`

The overlay's input. Empty — not merely all-`false` — whenever the
results are not current, so a stale or superseded search paints
nothing at all rather than painting hits without a highlighted one.

Returns [`FindHighlight`]s, which carry a canvas-space rect and a
flag and nothing else: `crate::canvas::overlay` is not told what a
page index or a quad is, and this module is not told what a `Painter`
is.

### `enum FindRequest`

Two variants and no more. In particular there is **no** `Open`/`Close`
variant: opening the bar changes no document state and needs no frame
boundary, so it happens in the `edit.find` dispatch arm directly, exactly
as `file.properties` mounts a panel there. What has to go through the
funnel is what needs the *document* — and both of these do, one because
it borrows the session mutably and one because it navigates.

### `fn apply`

Called from `PdfcerApp::apply`, **after** the frame that raised it, which
is the only place the two borrows this needs can be had at once: the state
and the open document are separate fields of `PdfcerApp`.
