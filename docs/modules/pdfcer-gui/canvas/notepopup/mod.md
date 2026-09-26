# `canvas::notepopup` — reading a comment where the comment is

The window that opens when an operator clicks a note on the page, and the
tooltip that appears when they hover one. **The canvas half of the review
surface**, and the half that was missing.

## The report this closes

The operator:

> *"check how the review functions work. unless something has changed I
> could add a yellow sticky note but even in read mode I don't think I
> could figure out how to read it. the review features should look and act
> the same as they do in Acrobat Reader."*

Two facts hold the answer up:

1. **`pdfcer-core` writes a `/Popup` for every sticky note.**
   `annot_author`'s `sticky_note` authors the companion dictionary, its
   `/Open`, and a rectangle 150 pt wide beside the note — so the state is
   already in an operator's files whether or not a shell draws it.
2. **Acrobat *Reader* is a read-only product and reading comments is its
   whole purpose.** A mode named Read that cannot read the comments has the
   posture exactly backwards.

⇒ Which decides where this lives. A pop-up on the canvas is **canvas
behaviour, not a ribbon item**, so it is mode-independent *by
construction*: no future edit to a tab list, a manifest or a panel default
can take it away again. That property is the whole reason this was built
before the panel work, and it is why it is not a fourth panel.

## The interaction, and every part of it is the convention rather than an
invention

This project's standing rule — *"use the conventional interaction, never
invent one — the convergence of the product class IS the spec"*:

- **Hover** a comment ⇒ a tooltip with the author and the words. The
  cheap half of the same question, in every reader in the class.
- **Click** a comment ⇒ its pop-up opens; clicking it again closes it. A
  *drag* moves the annotation and is a different gesture entirely — egui
  reports a click only when press and release land together — so the two
  cannot collide.
- **×** on the pop-up ⇒ closes it, as every window in the class does.
- A note the file marks `/Open` ⇒ **opens with the document**, no click.
  §12.5.6.4 Table 172 and §12.5.6.14 Table 183 both say so, and the
  state is in the file.

**A single click rather than a double.** In a reader, one click opens the
note; in an editor with the comment tool armed, one click selects and two
open. pdfcer has to serve both stances from one canvas, and a single click
serves both because opening a pop-up **does not consume the click**: in
Review and Edit the same press still selects the annotation, so the
selection outline, the grips and the Format tab all behave exactly as they
did. Nothing was taken away to add this.

## Rule 4: the pop-up is CHROME, and the page is untouched

*"Fuzzy never sneaky"*, and the one-line test this project uses for it:
**would a screenshot of the editing canvas differ from a screenshot of the
same document saved and reopened?**

A pop-up is the same class of thing as a selection handle or a snap marker
— the cursor's own furniture. So:

- It is drawn in an `egui::Area`, a **separate layer** above the page
  raster. Not one pixel of it is composited into anything that is saved.
- It is drawn at a **fixed size in screen points and does not scale with
  zoom**, which is what makes it unmistakably interface rather than
  content. Acrobat's pop-up behaves the same way and for the same reason.
  Its *position* does follow the page, because it is about a particular
  note; its *size* does not, because it is about the operator's eyes.
- Nothing it shows is inferred. The words, the byline and the date are
  verbatim from the file; the open state is read from `/Open`, never
  defaulted (see [`model::read_open`], and the request filed beside it).
- **Closing a pop-up changes the screen, not the file** — and
  [`crate::text::annotpopup::popup_close_tooltip`] says so, because an
  operator has every reason to assume otherwise.

## What it can do, by mode

| | Read | Review | Edit |
|---|---|---|---|
| hover tooltip | ✅ | ✅ | ✅ |
| open the pop-up and read the note | ✅ | ✅ | ✅ |
| read the thread of replies | ✅ | ✅ | ✅ |
| edit the note, remove it, delete the comment | — | ✅ | ✅ |

### Read shows and does not edit, and the reason is on screen

`MODES_AND_PANELS.md`'s stance for Read is *"the page content is not yours
to alter"*, and **reading is not editing** — which is the whole argument
for this module existing. So the editor is not drawn in Read.

It is not drawn *greyed*, either. R9: *"an unavailable capability renders
nothing; a temporarily unavailable one may grey and must explain on
hover."* Read mode is the purest example of temporary this program has —
the operator chose a stance and a labelled three-position control changes
it — so the pop-up carries **one sentence naming the mode that can**
([`crate::text::annotpopup::popup_read_only`]) and no dead text box. A
disabled `TextEdit` would be the half-built surface the no-placeholders
rule exists to forbid.

## What it CANNOT do, and what kind of absence each is

Neither absence here is an engine gap. `EditSession::add_reply` writes
`/IRT` and `/RT /R` with its own `/Popup`, and `EditSession::add_review_state`
writes `/State` and `/StateModel` (§12.5.6.4 Table 171); both are reached
from the **Comments panel** (`crate::panels::comments`), which is where a
reviewer's work list already lives.

- **No Reply control on this surface.** The window already *shows* the
  thread ([`thread`]); composing in it is wiring nobody has asked for.
- **No Accepted / Rejected / Completed control on this surface.** The same
  decision and the same place: a review status is a property of the work
  list rather than of one open window.

Both are **scope** decisions rather than capability ones, and the
distinction is worth keeping sharp, because the two expire on different
events: a capability absence is a statement about the **program** and ends
when the engine moves, while a scope absence is a statement about **one
surface** and ends when somebody decides that surface should compose.
Conflating them is how a scope decision comes to be defended with a
capability argument that is no longer true.

⚠ Where an absence *is* a capability one, R9 governs and **nothing is
drawn**: no empty status row. A control that no state of the program could
enable is not an affordance, it is a promise.

## Recording `/Open`

`EditSession::set_annotation_open` writes the window state **into the
document**, and `controls`' `open_default` is the control for it. The read
half is `model`'s `read_open`, over
`pdfcer_core::annot::Annotation::open`.

⚠ **Opening and closing a bubble on screen still writes nothing**, and that
is a decision rather than a leftover. [`open_default`]'s doc comment carries
the undo argument in full; the short form is that reading a marked-up
drawing *is* opening and closing bubbles, and an undo log in which nineteen
entries in twenty say *"looked at a comment"* cannot do the job an undo log
is for.

## Where the pieces are

- [`model`] — the pure read: what notes are here, where their windows
  go, what replies hang off them.
- [`open`] — which pop-ups are showing, and the override rule that lets
  the file speak first.
- This file — the drawing, the two hooks, and the trace.

## The two hooks, and how small they are

[`show`] is called from `crate::app::surfaces` on the line after
`canvas::show` returns — **one statement** — because a floating layer does
not belong in the canvas paint order at all (`crate::canvas::painting`'s
header states that order and every position in it is an argument; a pop-up
has no position in it). [`clicked_on`] is called from
`crate::canvas::clicking` beside the annotation hit test — **one
statement**, consuming nothing.

[`show`] takes this frame's mapping from `crate::canvas::zoom::last_frame`
rather than being handed one. `canvas::present` publishes it through
`remember_frame` **before** it calls `interact`, so by the time this runs
it is this frame's map and not the previous one — which is what keeps the
window from lagging a pan by a frame.

## A pop-up NEVER covers the annotation it belongs to

An `egui::Area` at `Order::Middle` takes every press inside it, so a pop-up
laid over its own note swallows every gesture on that note — and the
symptom reads as a canvas defect rather than as a placement one: the move
drag and the grip drag never arrive, while the rotate handle, drawn clear
of the box, keeps working.

[`popup_origin`] flips rather than slides, and [`clear_of_anchor`] carries
the candidate order, the measurement and the one case that has no
answer. ⇒ **A window that describes a thing must not be laid over the
thing**, and on an immediate-mode canvas that is not a cosmetic rule: the
window is an input surface, and the thing underneath becomes unreachable.

## WHEN a pop-up opens, which is a separate question from where

A comment with no words must not open an empty pop-up. [`model::under`]
answers for every annotation that *can* carry a note rather than for those
that do, so [`model::has_something_to_read`] is asked at the click site;
without it, a click meant only to select a shape produces a blank window.

The rule is **not** simply "has words" — a sticky note is a note whether or
not anybody has typed in it, and an operator who has just placed one needs
the window in order to write. Subtype decides for the two whose purpose is
the note; content decides for every mark that merely *may* carry one.

Under a continuous or facing display mode, pop-ups are drawn for the
**acting page's** annotations only. That is not a decision of this module:
it inherits `crate::canvas::selection::annot::under_pointer`'s frame of
reference exactly — one `page_index`, one `PageMapping` — so a pop-up
appears wherever an annotation is *selectable*, and nowhere else. Fixing it
is the same piece of work as making annotation selection reach a second
visible page, and doing it here alone would put a window over a note the
canvas will not let you click.

## `PDFCER_DIAG` proves what this computed

One `note-popup` line per frame with something to say: how many notes the
page carries, how many carry words, how many pop-ups are open, how many of
those the **file** asked for rather than the operator, and whether a
tooltip was shown. A screenshot cannot tell you that a pop-up opened
because `/Open` was true rather than because a click landed, and that
distinction is the whole of [`open`]'s contract.

## Item notes

### `mod controls`

Its header carries the seam: the rest of this module **reads** — a heading,
a byline, the words, the thread, a tooltip, a placement — and that one is
the only part that produces an `Action`.

### `const POPUP_WIDTH`

# Screen points, not page points, and that is the rule-4 half

A pop-up sized in page space would grow to fill the sheet at 800 % and
vanish at 20 %, which is what *content* does. Chrome does not. 260 pt is
about forty characters of the shell's body face — wide enough for a
sentence of review prose without wrapping every third word, narrow enough
that four open pop-ups on a D-size sheet do not tile over the drawing.

The `/Popup`'s own `/Rect` is still honoured **for position** (see
[`popup_origin`]). Its width is not, deliberately: `pdfcer-core`'s
`annot_author::sticky_note` authors 150 pt, which at 100 % zoom is under
twenty-five characters, and a producer's chosen width is a statement about
their reader's font rather than about ours.

### `const POPUP_MAX_BODY`

A note is arbitrary operator text and can be a page of it. Without a
ceiling one long comment would produce a window taller than the canvas,
whose Save button is off screen — a control that exists, is enabled, and
cannot be reached. The body scrolls; the title row, the
byline and the controls never do, so the two things an operator needs
(whose note is this, and how do I close it) are always in view.

### `const POPUP_GAP`

To the right of the annotation's box, which is where `pdfcer-core`'s own
author places one and where every reader in the class puts it. Eight points
of gap so the window does not touch the mark it belongs to — a pop-up flush
against a cloud reads as part of the drawing.

### `struct Ctx`

A struct rather than eight parameters, and the grouping is a statement:
every member is a property of *the frame*, while the two arguments that
stay loose — the note and the draft — are what distinguishes one window
from the next.

### `const POPUP_BOX_WIDTH`

`popup` sets `ui.set_max_width(POPUP_WIDTH)` twice: once on the `Area`'s
own `Ui` and once inside `egui::Frame::popup`, whose inner margin and
stroke sit **outside** the contents. Measured on a real build the drawn
window is 274 pt for a 260 pt content width — fourteen points of frame.
Sixteen is used here rather than fourteen because the frame is a *style*
value and a theme with a fatter popup margin must not silently reintroduce
the overlap [`popup_origin`] exists to prevent. Erring wide costs at most a
two-point gap; erring narrow costs the gesture.

### `fn popup_origin`

Two sources of a *preferred* origin, in priority order:

1. **The `/Popup`'s own `/Rect`**, when the file gives a usable one. The
   producer said where the window belongs and honouring it is what makes a
   document look here the way it looked in the reader that wrote it.
2. **Beside the note**, to the right and top-aligned, when there is no
   `/Popup` or its rectangle is unusable. Where `pdfcer-core`'s own sticky
   author puts one — 150 pt wide, to the right of the note — and where
   every reader in the class puts one.

…and then one **invariant that outranks both**:

> ### A pop-up must never be laid over the annotation it belongs to

# Why the clamp cannot be left to place a window

An `egui::Area` at `Order::Middle` takes every press inside it: egui
resolves interaction on the topmost layer, so a window drawn over its own
note means the canvas response never sees the press at all. The symptom is
asymmetric and misleading — the move drag and the grip drag never arrive,
rotation still works, because the rotate handle is drawn *above* the box's
top edge and clear of the window — and it reads as a fork in
`canvas::interact` eating the gesture rather than as a placement.

`Area::constrain_to` produces exactly that state on its own, and the
measured case is ordinary: `beside` puts the origin at
`anchor.max.x + POPUP_GAP` = 559.7, a 274 pt window does not fit in a
viewport ending at 772, and the clamp slides it **left** to 498 — back
over the anchor. The clamp is doing exactly what it was written to do, and
*sliding is the wrong recovery*: the one direction a pop-up must not be
pushed is onto its own subject.

⇒ **Flip, do not slide.** The candidates are tried in order and the
first that clears the anchor *and fits* wins:

1. The preferred origin — the file's, else right of the note — taken
   as-is when the box it implies does not intersect the anchor.
2. **Left** of the note, right-aligned to its left edge. Horizontal.
3. **Below** or **above**, whichever side of the anchor has more room, x
   pinned into the viewport. Vertical.

Candidates 1 and 2 separate on **x alone**, which makes them independent
of the window's height — and the height is the one dimension this function
cannot know, because it is decided by the note's own words during layout.
A placement that needed the height would have to read the *previous*
frame's measured rect, and `D:/dev/rag/egui/` records what that costs: a
surface whose position depends on its own size oscillates, and the
oscillation is invisible to unit tests and to screenshots alike. Candidate
3 needs a vertical decision and takes it from the **room available**
(`clip.max.y - anchor.max.y` against `anchor.min.y - clip.min.y`) rather
than from the window's height, for the same reason: room is a property of
the page and the viewport, and nothing about it moves when the window does.

# ⚠ The case that has no answer, named rather than hidden

When the anchor is wider than the viewport minus a pop-up **and** taller
than half of it — an annotation zoomed until it fills the screen — no
candidate clears it, and the preferred origin is used unchanged. That is
honest: at that zoom every position covers part of the subject, and the
operator has the whole rest of the shape to press on. It is stated because
the alternative — refusing to draw the pop-up at all — would make a note
unreadable at exactly the zoom an operator uses to read one.

Only the **origin** comes from the file; the size does not. See
[`POPUP_WIDTH`] for why, and note the consequence: a pop-up whose `/Rect`
is 150 pt wide is drawn 260 pt wide from the same top-left corner, so it
extends further right than the file's rectangle. That is correct — the
rectangle is where the window *is*, and how big a window needs to be is a
property of the reader's typeface.

The file's own rectangle is a *preference*, not a licence to overlap.
A producer that placed a `/Popup` over its own note is asking for a window
the annotation cannot be grabbed through, and honouring that would be
honouring a defect. Candidate 1 keeps the file's origin whenever it clears
the anchor, which is what every `/Popup` a real producer writes does.

# The clamp is still here, and still a fallback

Whatever candidate wins is clamped into a viewport grown by the window's
own size, so that an `Area` is never handed a position off in the millions:
at deep zoom a page point maps to a screen coordinate far outside any
viewport, and an `Area` positioned there is constrained back to the edge —
every pop-up on the sheet stacked in one corner. Clamping into a
slightly-grown viewport first means an off-screen note's window arrives at
the edge *nearest to it*, which is at least a direction.

### `fn clear_of_anchor`

Separated from [`popup_origin`] so it can be tested without a
`PageMapping` — the decision is pure rectangle arithmetic and every
interesting case is a specific arrangement of three rectangles, which is
exactly the shape a unit test can state and a driven check cannot.

See [`popup_origin`]'s header for the candidate order and for why the
first two separate on **x alone**. `width` is [`POPUP_BOX_WIDTH`]; the
height is deliberately not a parameter, because this function must not
depend on a quantity that is decided by the window's own contents.

### `fn body`

# The order, and why it is this one

Top to bottom in the order a reviewer needs them: **who and what** (so they
know whose comment they opened), **the words** (what they came for), **the
thread** (the rest of the conversation), then **the controls** — last,
because an operator scanning downward reads and stops when they reach a
button.

The close control is the exception and sits on the title row at the right,
which is where every window in the class puts it and where a hand reaches
for it without reading.

### `fn thread`

# Read-only HERE, which is a scope decision and not a limit

`EditSession::add_reply` writes `/IRT` and `/RT /R`, and this shell authors
replies from the **Comments panel** (`crate::panels::comments::editor`'s
`reply_control`), which is where a reviewer's work list already lives.

⇒ So what is absent here is a *control*, not a *capability*, and the two
expire on different events. Adding composition to this window is wiring: the
thread is already gathered, already drawn, and the action bus already
carries `AnnotAction::Reply`.

**Every reply in this list is transitive**, which is why it can afford
to be flat. [`model::replies_to`] gathers anything whose `/IRT` chain
reaches the root, so an answer to an answer appears here beside the answer
rather than being lost — and the panel makes the same choice for the same
reason. `crate::panels::comments::body`'s threading section carries the
argument and the cost.

# Cost

[`model::replies_to`] walks every page, because a reply may legally live on
a different page from the comment it replies to. It runs only for a pop-up
that is **open**, which is the gate that makes it affordable: closed notes
cost nothing at all. `crate::panels::comments` pays a comparable walk on
every frame it is visible and states so in its own header.

### `fn tooltip`

Returns whether one was shown, for the trace.

# Why it is suppressed over an open pop-up

Because the answer is already on screen, three inches away and in full. A
tooltip repeating a truncated copy of it would be noise, and it would
appear *under the operator's pointer* at the moment they are reaching for
the window's own controls.

# Why it is drawn as an `Area` rather than through `Response::on_hover_text`

Because there is no `Response` to hang it off. The thing being hovered is a
rectangle inside a page raster, not an egui widget — the canvas is one
`Image` response covering the whole strip. An `Area` at `Order::Tooltip` is
what egui's own tooltip machinery resolves to anyway, and building it
directly means the offset, the constraint and the layer are this module's
to state rather than inherited from a widget that does not exist.

### `fn trace`

# Why this is more than a debug print

Because the two things that could be wrong here are both **invisible in a
screenshot**. A pop-up that is open because the file said `/Open` and one
that is open because the operator clicked look identical, and the whole of
[`open`]'s contract is the difference between them — an implementation that
silently defaulted `/Open` to `false` would look perfect until somebody
opened a file another product authored. `from_file` is the only oracle for
that available from outside the process.

`with_note` is the second: a page of markup pdfcer drew carries no
`/Contents` at all, so *"the pop-up showed nothing"* has two causes — the
note is empty, or the reader is broken — and they need opposite responses.

Silent when there is nothing open and nothing hovered, so a trace of a
reading session is not one line per frame of noise.

### `fn draft_key`

Per document, exactly as [`open`]'s overrides are and for the same reason:
object ids collide freely between files, so a shared draft would put one
document's half-typed note into another's pop-up.

### `fn load_draft`

**A second draft from the Comments panel's, deliberately.** They are two
editors on two surfaces and an operator may legitimately have one open in
each; sharing one draft would mean typing in the panel silently rewriting
what is in the window. `NoteDraft`'s own `(annotation, edit epoch)` stamp
is what keeps *this* one honest — a draft stamped at an older epoch
describes a document that no longer exists, and [`show`] calls `sync`
before anything is drawn.

### `fn window`

Height is a stand-in: [`clear_of_anchor`] is specified to separate on
**x alone** for its first two candidates, so a test that asserted with a
real height would be asserting something weaker than the contract.

### `fn the_popup_that_ate_the_drag_is_placed_clear_of_its_annotation`

Measured geometry: a markup selected at
`[[464.0 464.5] - [551.7 550.2]]` whose pop-up, left to the clamp, is
drawn at `[[498.0 465.0] - [772.0 565.0]]` — on top of it, so every
press meant for the shape goes to the window instead and neither the
move nor the resize reaches the canvas.

With the anchor at x 464–551.7 there is no room on the right
(551.7 plus 8 plus 276 = 835.7, past the viewport's 772) and none on the
left (464 minus 8 minus 276 = 180, before the viewport's 288), so this
exercises candidate 3.

### `fn a_note_with_room_beside_it_keeps_the_placement_it_always_had`

This is the ordinary case and the one that must not move: a note near
the left of the sheet has room on its right, and the window goes there,
byte for byte where it went before this function existed.

### `fn a_wide_note_low_on_the_sheet_puts_its_window_above`

A note low on the sheet, too wide for either side. The window goes to
the top of the viewport, which is where every point of the available
room is.

### `fn a_producers_popup_rectangle_is_kept_unless_it_covers_the_note`

Two documents, one function. The first names a rectangle beside its
note and gets exactly that; the second names one on top of its note and
is overruled, because honouring it would be honouring a defect.

### `const _`

[`POPUP_BOX_WIDTH`] exists because `egui::Frame::popup`'s margin sits
outside `ui.set_max_width(POPUP_WIDTH)`. If the two were ever collapsed
into one constant the separation would be short by the frame and the
overlap would come back at the margin — silently, on exactly the notes
nearest the edge.
A `const` assertion rather than a runtime one: clippy refuses
`assertions_on_constants`, because an `assert!` over two constants is
decided when the crate is compiled and a test that cannot fail is not
evidence. `const _: () = assert!(..)` states the same fact where it is
actually checked — the build stops, with this message, and no test
has to run at all.

### `const REGIONS`

⚠ A hand-written list inside a completeness check is itself the
known weakness: a name added to the constants and not to this list is
simply not checked. [`REGION_OPEN_DEFAULT`] is published and is **not**
in this list, which is that drift in the present tense.

### `fn the_region_names_are_unique_and_namespaced`

A region name is a key a driven check aims a real pointer at. Two
controls publishing one name leaves the harness clicking whichever was
drawn last — a coordinate nobody chose — and the failure looks like the
feature being broken rather than like the check being blind.

### `fn a_note_with_no_popup_rect_opens_beside_itself`

The one placement failure an operator would report as the feature being
broken: a pop-up drawn on top of the icon that opened it hides the
thing they just clicked, and the second click — which they will
certainly try — lands on the window rather than on the note, so it does
not close. Asserting *strictly* to the right of the anchor's right edge
is what forbids the whole family of "close enough" placements.

### `fn the_files_own_popup_rectangle_is_honoured`

§12.5.6.14 makes the pop-up a separate annotation *with its own
placement*, and a producer who moved a note's window across the sheet
meant it. Ignoring that and always placing beside would make every
document laid out in Acrobat look rearranged here — and it is the
mistake an implementation makes by default, because "beside" is the
easier code path and it looks fine on a file pdfcer itself wrote.

### `fn a_note_far_off_screen_is_clamped_towards_itself`

The deep-zoom failure, and it is not hypothetical: at 300,000 % a page
point maps to a screen coordinate in the millions, and an `Area` handed
one is constrained back to the nearest edge — so every pop-up on the
sheet stacks in one corner, all of them claiming to belong to marks
nowhere near it. Clamping the origin first means an off-screen note's
window arrives at the edge *nearest to it*, which is at least a
direction.

### `const REGION_POPUP`

# Why the first, when several can be open at once

A region name is a key, and publishing one name from four windows would
leave a driven check clicking whichever happened to be drawn last — a
coordinate nobody chose, which moves when an unrelated note is opened. The
first is the only deterministic choice available without inventing a
per-annotation naming scheme that nothing would consume.

It is published with `ui_rect_visible` against the **canvas viewport**,
not with `ui_rect`, because a rect on its own proves *layout* and not
*visibility*. A pop-up constrained off the edge of the canvas lays out
perfectly and is invisible, so a check asserting only the rect would pass
on a window no operator could reach.

### `const REGION_OPEN_DEFAULT`

Which is exactly why it needs a region of its own rather than being
counted among the others: nothing about the pop-up changes when it is
pressed, so a driven check has no visual oracle for it at all and the rect
plus the `set-annotation-open-applied` trace line are the whole of what a
harness can see.

### `fn show`

The one entry point, called from `crate::app::surfaces` immediately after
`crate::canvas::show` returns. See the module header for why it is there
and not in the paint pass.

# Why it takes `caps`

To decide whether the editor is drawn at all — see the module header's mode
table. It is the frame's sampled value, passed in rather than read here,
for the reason every canvas sample is: two readings within one frame can
disagree, and a disagreement here would be an editor that appeared for one
frame.

# Why it takes `&OpenDoc` and `&mut Vec<Action>`

Actions, not mutations — the discipline every panel and every canvas
gesture in this crate follows. This function reads the document and pushes
intent; it never touches the session. The shared reference makes that a
compile-time fact rather than a convention.

### `fn clicked_on`

Called from `crate::canvas::clicking` beside the annotation hit test, and
**it consumes nothing**: the click goes on to mean exactly what it meant
before this module existed. That is the property that made a single click
the right gesture in all three modes — see the module header.

Returns the note it toggled, for the trace at the call site.

# Why the hit test is repeated here rather than reusing `annot_hit`

Because `crate::canvas::clicking`'s `annot_hit` is gated on
`caps.author_markup` — Review and Edit only — and **that gate is the
operator's complaint**. In Read mode it is `None` on every click, so a rung
that consumed it would do nothing in the one mode this whole feature exists
for.

The two hit tests also exclude different things, deliberately, and
[`model::notes_on`]'s header states the difference and why each is right
for its surface. This one is a *reading* question and the other is a
*restyling* question.

# Cost

One `/Annots` walk per click — not per frame — bounded by
`pdfcer_core::annot::MAX_ANNOTS_PER_PAGE` and decomposing nothing. It is
the same walk the annotation hit test beside it already pays, so a click on
the 129,758-object benchmark sheet costs what it did.
