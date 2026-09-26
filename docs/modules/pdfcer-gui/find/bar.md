# `find::bar` — the Find overlay's controls, and the keys they own

One compact box **floating over the top-right of the page**, which is
where Acrobat Reader, Chrome's PDF viewer and Edge's all put theirs:

```text
                         ┌────────────────────────────────────────────┐
                         │ Find [ total       ] ⏴ ⏵ 3 of 47  Options ⏷  × │
                         └────────────────────────────────────────────┘
```

[`super`]'s header carries the placement note. The short version is that an
overlay consumes **no layout space**, so opening Find does not resize the
canvas and therefore does not re-fit the page under the operator's eyes.
That is not a theoretical advantage: a docked version of this bar was built
first and driven, and pressing Ctrl+F under Fit page took the zoom from
85 % to 81 % and back again on close — a page that jumps every time you go
looking for a word on it.

## The four search options live in a menu, not on the bar

Match case, Whole word, Wildcards and the whole-word **rule** are behind
the `Options` button. Three reasons, in order of weight:

1. **An overlay has to be narrow**, because it covers the page. Laid out in
   a row, those four controls are wider than the search field, the step
   buttons and the readout put together; the box would span most of the
   window and hide the thing being searched for.
2. **It is what the reference product does.** Reader's Ctrl+F box is a
   field, two arrows and a settings dropdown holding *Whole words only* and
   *Case-Sensitive*. An operator arriving from Acrobat finds the options
   where they left them.
3. **The rule chooser can then appear and disappear without moving
   anything.** It is meaningful only while Whole word is on (see
   [`options_menu`]), and inside a menu its arrival costs a row of a popup
   rather than shifting every control to its right — which, on a bar the
   operator is aiming at, is the difference between a tidy layout and a
   mis-click.

## The width is fixed, and nothing on the bar may move

The box is anchored by its **top-right corner** to the canvas viewport, so
its left edge is `right − width`: any change of width moves every control
on it. The search field, the readout and the buttons therefore all have
reserved widths ([`FIELD_WIDTH_PTS`], [`READOUT_WIDTH_PTS`]) and the
options are in a menu — so `3 of 47` becoming `No matches`, or a search
finding nothing at all, cannot slide the ⏴ ⏵ buttons out from under the
pointer between two clicks. That is the same reason the status bar reserves
a width for its zoom readout.

The height is fixed too, at [`ROW_HEIGHT_PTS`], but for a much weaker
reason: layout tidiness. **R128 does not reach this surface.** That rule is
*a panel whose size feeds a fit-to-viewport computation has a fixed size*,
and an `egui::Area` feeds no such computation because it consumes none of
the layout. Docking the bar is what would have made R128 bind, and that is
one of the reasons it is not docked.

## The three keys, and the one that is shared

| key | while the field has focus | otherwise |
|---|---|---|
| Enter | search, or step to the next hit | belongs to whatever has focus |
| Shift+Enter | search, or step to the previous hit | as above |
| **Escape** | close the bar | **belongs to the canvas** |

Escape is the interesting one, because three surfaces want it: a canvas
drag in flight wants to abandon itself, the selection ladder wants to
ascend a rung, and this bar wants to close. There is no arbitration code,
and there does not need to be — `crate::canvas::interact` already reads
Escape as `!ctx.text_edit_focused() && …`, so while the operator is typing
here the canvas is not offered the key at all. This file takes it only
under the same condition, from the other side.

The consequence is worth stating because it looks like a gap: **Escape does
not close the bar after the operator has clicked on the page.** That is
deliberate. At that moment Escape is the selection ladder's, and a bar that
stole it would cost the operator the rung they were working in — the same
one-press-one-effect rule `canvas::interact` applies between the gesture
machine and the ladder. The close button and Ctrl+F are the routes out from
there, and both are visible.

## What Enter does depends on whether the answer is still current

[`super::FindState::readout`] is the single test, and [`enter_intent`] is
the whole decision as a pure function of it:

| readout | Enter | Shift+Enter |
|---|---|---|
| `Idle` — nothing searched for what is in the box | **search** | **search** |
| `Stale` — the document has been edited since | **search** | **search** |
| `At` — there are hits | step **next** | step **previous** |
| `Empty` — searched, nothing found | **nothing** | **nothing** |

The last row is the one that needs defending. Re-running a search that just
returned nothing would re-extract the whole document's text — **350 ms on
the benchmark drawing, measured**; see [`super`]'s cost section — to
produce the same empty answer, and an operator leaning on Enter would do it
once per press. Nothing has changed since the search ran; if something had,
the readout would be `Stale` and the first row would apply.

## Actions, not mutations

Every commit leaves here as an [`Action::Find`]. The two exceptions are the
ones `crate::app::status`'s page box already takes and for the same reason:
the **text buffer** and the **option flags** are widget state, they describe
the control rather than the document, and deferring a keystroke to after
the frame would make typing lag by a frame.

## Where the strings are

[`crate::text::find`], all of them. Nothing here is a literal an operator
can read; `tools/gates/check-ui-strings.sh` is the mechanical half of that
rule and [`crate::text`]'s header is the reason for it.

## Item notes

### `const MARGIN_PTS`

Enough that the box reads as floating *over* the page rather than as
something welded to the edge of the window, and enough that its shadow has
somewhere to fall.

### `const FIELD_WIDTH_PTS`

Wide enough for a part number or a short phrase — the two things drawing
reviewers actually search for. Reserved rather than proportional, for the
reason the module docs give: everything to its right is positioned from it.

### `const READOUT_WIDTH_PTS`

`3 of 47`, `No matches` and `Document changed` are three very different
widths, and without a reserve every search would shove its neighbours
sideways — and, because the box is right-anchored, would move the search
field the operator is typing into. Sized for the longest of the three at
the default text size; anything longer elides, with the whole string on
hover.

### `const REGION_BAR`

Published so `ui-verify` can reach the bar at all. A check that wants to
assert *"Ctrl+F produced a find bar, and its text is legible"* has exactly
two honest sources for **where to look** — the application measures the
rect on the frame it reports, or the harness hard-codes a fraction of the
window and goes stale the first time a panel moves. This is the first
source, and for a *floating* surface it is the only one: an overlay's
position depends on the canvas viewport, which depends on the dock, so no
constant a harness could hold would survive opening a panel.

### `const REGION_OCR_OFFER`

Published so `ui-verify` can assert on the offer's **presence and absence**
rather than on a screenshot. That matters more here than for the other
regions: the offer is one line of muted text and a small button on a
floating box over a drawing sheet, which is exactly the kind of thing a
pixel oracle cannot distinguish from the frame before it. A declared rect is
a claim the application makes about
itself, and the absence of one is the harness's evidence that the offer was
not drawn.

### `const FIELD_ID`

**Stable and explicit, because defect D1 depends on it.**
`crate::app::keyboard::collect` guards its unmodified bindings with
`ctx.text_edit_focused()`, which resolves the focused id and asks whether a
`TextEditState` exists *for that id*. The field therefore has to be a real
[`egui::TextEdit`] with an id that does not move between frames, or
`PageDown` would step the page while the operator was halfway through
typing a search term. It also has to be stable for
[`super::FindState::take_focus_request`] to be able to focus it.

### `fn host_rect`

Read from `crate::canvas::zoom::last_frame`, which is the canvas's own
record of where it drew. That matters as soon as a dock is open: anchoring
to the window's top-right would put the box over the right-hand panel
rather than over the page, and would move it every time a splitter was
dragged even though the page had not moved.

The fallback is the whole screen rect, and it is reachable rather than
defensive: a document with no pages, or one whose current page will not
rasterize, makes `canvas::show` return before it records a frame — and
both of those documents still have text worth searching. The box then sits
at the window's top-right, over the sentence explaining why there is no
page, which is the best available answer.

### `fn anchor_right_top`

A free function, and taking no width, because that is the whole point of
the right-top pivot: the placement is a corner, not a corner minus a
measurement. See the pivot's comment in [`show`] for the frame-one defect
that made it one.

Clamped into `host` on both axes so that a host smaller than the margin
cannot produce a point outside the canvas. `constrain_to` would pull the
box back anyway; this keeps the constraint a safety net rather than the
thing deciding the layout.

### `fn enter_intent`

The table is in this module's header; the argument for the `Empty` row —
the only one that returns `None` — is there too, and it is about cost: a
re-search of a query that just matched nothing re-extracts the whole
document's text to produce the same answer, and nothing has changed since
it did.

`Stale` searches rather than steps, which is the mechanism by which the
bar's own "press Enter to search again" tooltip is true.

### `fn offer_ocr`

`page_has_text` is a closure rather than a `bool` so that the caller's
answer is **not computed unless it is needed** — the short-circuit is the
affordability argument, and passing an already-evaluated `bool` would make
the call site pay for a page extraction on every frame the bar is open while
this function still looked correct. That is the shape of the defect: right
work, wrong moment, invisible to every test.

# The rule, and the trap inside it

| readout | page has text | offer |
|---|---|---|
| `Empty` | no | **yes** — there is nothing here for any search to have found |
| `Empty` | yes | no — an ordinary empty result; the words are there, that one is not |
| `At` / `Idle` / `Stale` | either | no |

The second row is the operator's stated rule and the whole reason this is a
function with a test rather than an `if` in the layout: *"the document is
images"* is not *"this search had no matches"*, and a build that collapsed
them would offer to recognise a text PDF every time somebody mistyped a part
number. [`tests::an_ordinary_empty_result_on_a_text_page_offers_nothing`] is
the falsifying case.

The third row is not merely "nothing to offer". A `Stale` readout means the
document has been edited since the search ran, so the *page* answer is about
a revision the hit list does not describe; and `Idle` means nothing has been
asked at all, where an offer would be the bar volunteering an opinion about a
document the operator has not yet questioned.

### `fn ocr_offer`

Drawn **below** the control row rather than inside it, and that is a
consequence of this module's fixed-width rule rather than a layout
preference. The box is anchored by its top-right corner, so anything added
to the row would move the search field the operator is typing into; a row
added underneath grows the box downwards, over the page, and moves nothing.
An `egui::Area` consumes no layout, so the extra height costs the canvas
nothing either — which is the same property that made the bar an overlay in
the first place (see the module header's 85 %-to-81 % measurement).

It appears and disappears with the condition rather than being greyed. P3
permits greying only for a *temporarily* unavailable capability that is
always explained on hover, and "this page happens to have text on it" is not
a state an operator can act their way out of — a permanently dead control
explaining a fact about the document is the placeholder the rule forbids.

### `const OCR_COMMAND`

Named here rather than typed inline so that
[`tests::the_offer_raises_the_registered_recognise_command`] can assert it
against the registry — an id that is merely spelled at a call site is an id
that goes stale silently.

### `fn position`

The two buttons are **enabled only when there is something to step**, and
each explains its greyed state on hover — P3 permits greying only for
*temporarily* unavailable and only when it is always explained. Both
conditions hold here: an empty or stale result set is a state that ends the
moment the operator presses Enter, and
[`crate::text::find::step_unavailable_tooltip`] says so.

### `fn readout_text`

Split out as a pure function so the four sentences can be asserted without
a frame, and so the "Idle draws nothing" case is a value rather than a
branch somebody has to notice in the layout code.

### `fn options`

**Changing an option re-runs the search**, when and only when a search
has already been run for what is in the box. Both halves matter:

- re-running is what makes the control *do* something — a "Whole word"
  checkbox that left the old hit list on screen would be an inert control,
  which is the shape defect D1 took;
- only after a search, because otherwise ticking a box on a bar the
  operator has not yet used would run a whole-document text extraction for
  a query they have not finished typing.

The test is [`super::FindState::answered`] *before* the change is applied —
"the bar is currently showing an answer to what is in it", which is exactly
the state in which leaving the old answer up would be wrong.

### `fn options_menu`

A free function taking `&mut FindOptions` so the whole menu is testable
without a popup: what matters about it is which controls appear under which
conditions, and that is a property of the options value.

**The word-rule chooser exists only while it means something.**
P3: an unavailable capability renders nothing, and greying is for
*temporarily* unavailable with an explanation. A rule chooser beside an
unticked *Whole word* is neither — it is a control that would change a value
nothing reads, which is worse than a greyed one because it looks like it
works. So it appears with the option and disappears with it, and
[`crate::text::find::whole_word_tooltip`] warns the operator that it will.

**`zoom_on_jump` is a second `&mut`, not a fourth field of
[`FindOptions`]**, and the split survives all the way down to this
signature on purpose. Everything reachable through the first argument
changes *what matches* and so re-runs the search; the second changes what
the view does with an answer that is already correct. A menu that took one
struct would have made the two indistinguishable at the only place a reader
looks to find out which controls are expensive.

### `fn unsearchable_note`

# Off-canvas, and that is the whole design

Rule 4 as narrowed by pdfcer's decision 059: an inference the operator cannot
see still owes them a report, **and the report does not go on the page.** No
badge over the offending run, no tint, no dashed outline, nothing drawn into
the page view at all. Applied content renders exactly as saved content
renders; the disclosure lives in a status line, a results panel, or — here —
the bar's own second row, which already exists for the OCR offer.

That is not fastidiousness. A provisional styling layer is a **second
rendering path for the same content**, and two paths drift. The operator's
own words on the old shell: *"the nagging and red flagging made for a lot of
extra bugs in the visibility when editing."*

# Why it sits beside the OCR offer rather than replacing it

They answer different questions and can be true at once. The OCR offer fires
when **this page** has no extractable text at all — a scan. This fires when
the **document** contains a font whose text is unreachable, which is
perfectly compatible with the current page being ordinary searchable text.
A file with a Type 3 titleblock on page 1 and normal text everywhere else
produces this note and no OCR offer, which is exactly right.

### `fn blanks_note`

Written when the note landed, because the two are drawn in the same row and
the obvious mistake is to make one an `else` of the other. They are not
alternatives:

| | OCR offer | unsearchable note |
|---|---|---|
| scope | **this page** | the **whole document** |
| fires when | the page has no extractable text at all — a scan | some font's text is unreachable, anywhere |

A drawing with a Type 3 titleblock on page 1 and ordinary text everywhere
else produces the note and **no** OCR offer, and that is correct: the page
in front of the operator is searchable, and the document still contains
something no search will ever reach.
**The blank at the end of the query, said out loud** —
`OPERATOR_REQUESTS.md` **O180**.

A trailing space, tab or newline on the query stops a search from finding
text on the page that does not carry one, and text pasted out of a
spreadsheet routinely carries one.

# Why a row exists at all, when the setting already fixes it

Because trimming **silently** is the same defect in the other
direction. Before this, an invisible character decided the answer and
nothing said so; after a silent trim, an invisible character would be
discarded and nothing would say so. The operator who genuinely meant
the space — checking whether a field is padded — would get hits they
could not explain, which is the harder half of the same problem.

So the fact is disclosed in **both** states, with two sentences, and
the predicate behind it is about the RAW query rather than about
whether trimming changed anything. See [`crate::find::query`].

# Off-canvas, and nothing is marked

Rule 4. The page renders exactly as it will render when saved; the
highlight over a hit is unchanged; no badge, tint or flag appears
anywhere near the document. The report lives in the bar's own second
row, which already exists for the OCR offer and the unsearchable note,
and it is not blocking and not positioned relative to the page.

# Muted, not strong

`DEFECTS.md` **D11**: `RichText::strong()` is unusable in this theme.
`palette.text_muted` for its neighbours' stated reason — this is a
statement about the search, not a control, and it must not compete
with the readout one row up.

### `const BAR_WIDTH_PTS`

**Fixed, and load-bearing.** The box is anchored by its top-right corner,
so its left edge is `right − width`; a width that varied with the readout's
text would move every control on the bar every time a search ran.

Deliberately a little generous: a row that overflows its allocation *wraps*
in egui, and a wrapped Find bar is two rows tall with its close button
underneath its own search field.

### `const ROW_HEIGHT_PTS`

Layout tidiness rather than R128 — see the module docs on why that rule
does not reach a surface which consumes no layout. What it does buy is that
the box does not change shape as the readout changes, which matters for the
same reason the width does.

### `fn show`

Call it from `PdfcerApp::ui` **after** the canvas and **before** the modal
dialogs. Both halves are ordering decisions:

- **after the canvas**, because the box is positioned from the canvas
  viewport's own rect, which `crate::canvas::show` records through
  `zoom::remember_frame` as the last thing it does. Drawing first would
  position this frame's box from last frame's layout, which is visible as a
  one-frame lag every time a dock is resized;
- **before the dialogs**, because a modal takes the frame and must be on
  top of everything, this included.

# Two states draw nothing at all, and neither is an oversight

- **Closed.** No area, no widgets, no hit-test region over the page.
- **Open with no document.** `edit.find` is gated on `doc.pages`, so the
  bar cannot be *opened* without one; but a document can be closed while it
  is open, and a search box over nothing is a control whose every input is
  refused. The flag survives, so reopening a document brings the bar back
  exactly as the operator left it — the same courtesy the recent list
  extends, for the same reason.

### `fn word_rule_label`

A free function rather than a method on [`WordBoundary`] because that type
belongs to `pdfcer-core` and its operator-facing wording belongs to this
crate's catalog. `pub(crate)` so [`super`]'s test can assert that every rule
the chooser offers has one.
