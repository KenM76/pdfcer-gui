# `canvas::notepopup::model` — what a note says, and where its window goes

The **pure** half of the note pop-up: it reads the document and answers
three questions, and it draws nothing, stores nothing and decides nothing
about the interface.

| question | answer |
|---|---|
| *what notes are on this page, and where?* | [`notes_on`] |
| *which one is under the pointer?* | [`under`] |
| *what replies hang off this one?* | [`replies_to`] |

It is separate from [`super`] for the reason every `model` module in this
crate is: a `Ui` cannot be driven from a unit test here, so anything that
needs a `Ui` is untestable by construction. Everything in this file takes an
[`ObjectGraph`] and a [`Page`] and returns data, which means every rule it
states can be asserted.


> *"I could add a yellow sticky note but even in read mode I don't think I
> could figure out how to read it."*

He was right, and the measurement was worse than the report. Before this
module the **only** route to a note's `/Contents` in the whole shell was
the Comments panel, which is mounted from the `markup` tab — and
`crate::app::modes::defaults`' `"read"` arm gives Read the tab list
`["file", "view"]`. So in Read mode there was **no route at all**, which is
the posture exactly backwards: Acrobat *Reader* is a read-only product and
reading comments is its whole purpose.

A pop-up on the canvas is the fix that cannot regress that way, because it
is **canvas behaviour rather than a ribbon item** — it is mode-independent
by construction, and no future tab-list edit can take it away.

## What the file already contained, and what it did not

`pdfcer-core`'s sticky author — the private `sticky_note` behind
`annot_author::TextAnnotSpec::Sticky` — writes a `/Popup` companion for
every sticky note it authors, carrying the note's own `/Open` state and a
rectangle 150 pt wide placed to the right of the note. So the data is in
the operator's files already; this module is what draws it.

### `/Open` is READ from the file, never defaulted

§12.5.6.4 Table 172 gives `/Open` on a `/Text` annotation as *"a flag
specifying whether the annotation shall initially be displayed open"*, and
§12.5.6.14 Table 183 gives the same key the same meaning on the `/Popup`.
A note authored open must therefore **open on load**, with no click.

**The workaround that was here is GONE — 2026-09-06.** This paragraph
read: *"`pdfcer_core::annot::Annotation` does not model `/Open`. Confirmed
by audit on 2026-09-05: `b"Open"` appears exactly twice in the whole crate,
both write sites in `annot_author.rs`. So this module reads the raw
dictionary through `ObjectGraph::value`."* It was reported as a workaround
under pdfcer decision 058 — *anything the GUI has to work around is a place
the crate boundary was drawn wrong* — and filed as
`request_popup_open_state_cannot_be_read.md`.

`Pass 253.3` shipped [`pdfcer_core::annot::Annotation::open`], and
[`read_open`] is now two field reads. ⇒ **The shell's copy was deleted the
day the engine's existed**, which is the whole point of having reported it:
the request's own words were *"the day `Annotation` grows the field, two
places will answer the same question and one of them will be ours"*, and
the only way that never happens is to remove ours immediately rather than
leaving it beside the real one as a fallback nobody re-reads.

⚠ The `/Popup` companion's `/Open` is read from the **same walk** rather
than by a second lookup: [`notes_on`] already gathers every `/Popup` on the
page to find its rectangle, and `page_annotations` returns each one with its
own `open`. Table 170 gives geometric markup no `/Open` of its own, so for a
`/Square` the companion is the entire answer.

## Where a pop-up is drawn, in priority order

1. **The `/Popup`'s own `/Rect`**, when it has one. The file said where the
   window goes and honouring it is what makes a document look the same here
   as it does in the reader that wrote it.
2. **Beside the note**, when there is no `/Popup` or its rect is unusable —
   to the right, top-aligned, which is where every reader in the class puts
   one and where `pdfcer-core`'s own author places it.

[`PopupBox::rect`] is `None` in case 2 and [`super`] does the placing,
because case 2 needs the *drawn* size of a window this module cannot see.

## Rule 15: a ce dimension is a `/Line` and it is NOT excluded here

`crate::panels::comments`' header settles this and the argument is not
re-derived: **ce dimensions** (the ones pdfcer authors, `/Line` with `/IT
/LineDimension` and a `/PieceInfo` sidecar) are annotations on the
document, so hiding them by subtype would also hide a genuine `/Line`
markup an operator drew.

What *is* different here is that a ce dimension's `/Contents` is
**regenerated from its measurement** by `author_dimension`, so it is never
a note somebody wrote. [`NoteView::contents`] carries whatever the file
says and [`super`] captions it; this module does not filter on it.

## Cost, stated rather than discovered

[`notes_on`] walks one page's `/Annots` — one array read plus one
dictionary per entry, bounded by `pdfcer_core::annot::MAX_ANNOTS_PER_PAGE`,
decomposing nothing. It is the same walk
`crate::canvas::selection::annot::under_pointer` already pays per click.

[`replies_to`] walks **every page**, because §12.5.6.2 permits a reply to
live on a different page from the comment it replies to and `pdfcer-core`'s
own `locate_annotation` scans every page for exactly that reason. It is
called only while a pop-up is open, never otherwise.
`crate::panels::comments` pays the same walk every frame it is visible and
states so in its own header; this is the same bound with a stricter gate.

## Item notes

### `fn read_open`

This function's whole body was a `graph.value(id)` dictionary lookup for
`b"Open"`, reported as a workaround under pdfcer decision 058 (*anything the
GUI has to work around is a place the crate boundary was drawn wrong*) and
filed as `request_popup_open_state_cannot_be_read.md`. Its own docs said
*"the day the engine models `/Open` this function becomes two field reads."*

`Pass 253.3` shipped [`pdfcer_core::annot::Annotation::open`] and this is
that day: two field reads, the workaround deleted rather than left beside
the real thing. The prediction that mattered was in the request itself —
*"the day `Annotation` grows the field, two places will answer the same
question and one of them will be ours"* — and the way to make sure that
never happened was to remove the shell's copy the moment the engine's
existed.

# `Option<bool>` in, `bool` out, and the asymmetry is the point

The engine reports **facts**: `None` means the key was absent, which
§12.5.6.4 Table 172 distinguishes from an explicit `false` even though its
stated *default* is `false`. `Annotation::open`'s own docs argue the
distinction is load-bearing for exactly this case — a reader that could not
tell *"said closed"* from *"said nothing"* would silently shut every note
another producer authored open.

This function is where the default is finally applied, because a **window
is either drawn or it is not** and something has to decide. So the
three-state fact becomes a two-state answer here, once, in a named place,
rather than at the drawing site where it would be an `unwrap_or(false)`
nobody reads.

# The order: the note first, then its pop-up

Both carry the key and the standard gives both the same meaning — Table 172
for a `/Text` annotation, Table 183 for the `/Popup`. `pdfcer-core`'s author
writes the **same** value to both — the note's `/Open` is copied onto the
companion in the same call that creates it — so
on a file pdfcer wrote the order cannot matter. It matters on a file
somebody else wrote, and the note wins because it is the annotation the
operator interacts with — and because Table 170 gives a `/Square` or an
`/Ink` no `/Open` of its own, so for those the note half is `None` and the
companion is the whole answer.

Absent on both is `false`: Table 172's default value, stated.

### `fn canvas_rect`

A thin adapter so the two call sites above spell the conversion once.
`annot_canvas_rect` already rejects a degenerate or non-finite rectangle,
which is the whole of what "unusable" means here.

### `const MAX_THREAD_DEPTH`

Eight, which is far past any thread a human writes and far short of
anything that costs a frame. The bound exists for the malformed case rather
than the deep one: a `/IRT` cycle is legal syntax and would otherwise loop
for ever. See [`replies_to`].

### `fn a_note_authored_open_reads_as_open`

The assertion the whole `/Open` path exists for, and the one the
assignment names: *"A note authored open should open. Read it; do not
default it."* An implementation that returned `false` unconditionally
would pass every other test in this module.

### `fn a_note_with_no_open_key_is_closed`

And *absent on both* is the case that has to be spelled out, because
the engine deliberately reports absence as `None` rather than folding it
to `false`: somewhere the default has to be applied, and this asserts
that the somewhere is here.

### `fn a_shapes_open_state_comes_from_its_popup`

The case that matters on a file another product wrote: `/Square`,
`/Ink` and every other geometric markup have **no `/Open` in Table
170** — the key belongs to `/Text` — so their open state lives only on
the `/Popup`. Reading the parent alone would report every shape's note
as closed however the producer saved it.

Both directions, because a fall-through that always answered `true`
would pass a one-sided check and open every shape's window in the
document.

### `fn the_note_outranks_its_popup`

Asserted in the direction where the two DISAGREE and the note says
*closed*: a build with the operands swapped passes any test where they
agree, and passes the `Some(true), Some(false)` case as well by
accident.

### `fn the_topmost_note_takes_the_click`

`/Annots` is paint order, so a sticky dropped over a cloud is drawn
last and is what the operator sees. A hit test taking the first match
would open the note underneath — which reads as the click having
missed, because a window appears about something the operator was not
pointing at.

### `fn the_fixture_carries_a_real_thread_with_words_an_author_and_a_date`

# Why this test is here and not in `tools/ui-verify`

`RESUME.md`'s falsification discipline: *"a fixture note with empty
`/Contents` makes 'the pop-up shows the words' pass on a build that
shows nothing."* A driven check aimed at a document that cannot
exercise its case reports **SKIP or a green pass**, and neither is
distinguishable from the feature working. This project has been bitten
by that seven times in one month, three of them on one afternoon.

So the fixture's own properties are asserted **here**, in a cheap unit
test that runs on every `cargo test`, rather than trusted. If the
generator is edited, or the file is regenerated by a different hand,
this goes red long before a driven sweep would notice anything.

It doubles as the only end-to-end exercise of [`notes_on`] and
[`replies_to`] against a real document, which is why the assertions
below are about the model's output rather than about the bytes.

### `fn only_a_note_with_somewhere_to_record_it_may_record_it`

# What the engine will and will not write

`set_annotation_open` writes `/Open` on the annotation itself **only for
`/Text`** — §12.5.6.4 Table 172 gives it there and Table 169 gives it to
no other subtype, and the engine refuses to invent it: *"writing it onto
a `/Square` would add a key the standard does not define there, which is
noise a later reader could mistake for meaning."* It writes the `/Popup`
companion's when there is one, and it **will not manufacture a
companion**, because choosing a `/Rect` the caller did not pick is
authoring rather than a state change.

With neither, the call succeeds, writes nothing and pushes no undo
entry. That is the right contract for an engine acting over a mixed
selection and the wrong affordance for a window: a tick box whose whole
effect is a sentence explaining that it had none.

# All four combinations, because two of them are the interesting ones

A build that asked only *"is it a `/Text`?"* withholds the control from
every shape an operator commented on and gave a pop-up — which is what
`pdfcer-core`'s own sticky-note author writes, and what Acrobat writes
for a highlight. A build that asked only *"does it have a `/Popup`?"*
withholds it from a `/Text` whose companion another producer left out,
where the annotation's own `/Open` is the entire answer.

### `struct NoteView`

Deliberately **not** `pdfcer_core::annot::Annotation` passed through: that
type carries sixteen fields of which four matter here, its `/Rect` is in
PDF user space rather than canvas space, and it cannot answer the `/Open`
question at all. A projection with a name is what lets [`under`] and
[`super`] agree about what they are talking about.

### `mod ified`

The same ruling `crate::text::panels::comments::comment_row_byline`
records at length and it is not re-argued: §12.5.2 gives `/M`'s type as
*"date **or** text string"* and requires a conforming reader to accept
any format, so formatting it here would mean writing a parser whose
failure mode is either rejecting a legal value or mangling one. Two
surfaces, one rule — and if this module formatted while the panel did
not, the operator would see two different dates for one comment.

### `fn notes_on`

# What is excluded, and why each one

The same list `crate::canvas::selection::annot::selectable_on` uses, with
one addition and one deliberate difference, because this surface answers a
different question — *what did somebody write here* rather than *what may I
restyle*.

| excluded | why |
|---|---|
| `/Widget` | a form field is not a comment; the Forms panel owns it, and its `/Contents` is a tooltip rather than a remark |
| `/Popup` | §12.5.6.14 is a `shall`: a pop-up *"shall not appear alone but is associated with a markup annotation, its parent annotation."* It is the window, not the note |
| `/Link`, `/Movie`, `/PrinterMark`, `/TrapNet` | nobody wrote them. `/TrapNet` in particular is prepress output state a RIP applied |
| **not drawn on screen** (§12.5.3 bit 2 `Hidden` **or** bit 6 `NoView`) | nothing is painted there, so a pop-up would hang off a point on blank paper with no visible anchor. The Comments panel is where such an annotation is reached, and it marks it as hidden — the same split the canvas already makes for an undrawn form field |

### This is STRICTER than the selection's exclusion, and the difference
is a finding

`crate::canvas::selection::annot::selectable_on` excludes `flags.hidden()`
alone. `pdfcer_core::annot::AnnotFlags::suppressed_on_screen` — used here —
is `hidden() || no_view()`, and §12.5.3 Table 165 bit 6 (`NoView`) means
*"do not display on screen, but do print"*. So a `/NoView` annotation is
**selectable on the canvas today with nothing drawn under the pointer**:
the operator gets a selection outline round blank paper.

Found by building this door, which is the second time in this project a new
route onto an existing capability has exposed a divergence the old one was
hiding. **Not fixed here** — `canvas/selection/**` belongs to a concurrent
track — and reported instead. This module takes the stricter reading
because a pop-up window is a much louder wrong answer than an outline.
| **no object id** | there would be nothing for Edit or Delete to name — see [`NoteView::id`] |
| **no usable `/Rect`** | §12.5.5's placement target is missing, so there is no anchor and the renderer drew nothing either |

**A `/FreeText` is NOT excluded**, and that is worth stating because it
is the one case where a pop-up duplicates what is already on the page: a
free-text annotation paints its own words. Acrobat still gives it a
pop-up — the words on the page are the *appearance*, which a producer may
have styled, clipped or rotated, while `/Contents` is what was typed — and
a reviewer correcting a typo needs the second one.

# Ordering

`/Annots` order, which is paint order: later entries draw on top. [`under`]
takes the **last** match so the topmost note wins a click, which is the
rule page content and annotation selection both already follow.

### `fn has_something_to_read`

# The defect this closes

Until 2026-09-05 a click opened a pop-up for *every* annotation that **could**
carry a note, not for those that **do**. So clicking a revision cloud you
only meant to select produced an empty window over the drawing — and, until
the placement fix landed the same day, one that sat on top of the shape and
swallowed the drag as well. It was recorded as a known limit in [`super`]'s
header and on `OPERATOR_REQUESTS.md` O133 as *"a question about WHEN a pop-up
opens rather than where it goes"*. This is that question, answered.

# The rule, and why it is not simply "has words"

**A sticky note is a note whether or not anybody typed in it.** Opening it
empty is not noise — it is the annotation's entire purpose, it is what
Acrobat does, and an operator who placed one and has not written in it yet
needs the window in order to write. The same is true of a free-text box,
whose words *are* its appearance.

⇒ So the subtype decides for the two whose **purpose** is the note, and the
**content** decides for everything else. A square, a circle, a line, a
cloud, a polygon, freehand ink and a text-markup highlight are all *marks
on a drawing* that may additionally carry a comment; where they carry none,
there is nothing to show and the click belongs to selection.

A byline with no words is deliberately **not** enough. Knowing that
B. Reviewer drew this cloud is a fact about the drawing, and the Comments
panel lists it — putting a window over the page to say only that would be
the noise this function exists to remove. The panel is where facts live;
the pop-up is where a *message* lives.

⚠ Whitespace does not count. A `/Contents` of `"   "` renders as an empty
window just as surely as an absent one, and a file that carries it was not
trying to say anything.

### `fn can_record_open_state`

`EditSession::set_annotation_open` writes `/Open` on **up to two objects**
and nowhere else:

| object | when the key is written | authority |
|---|---|---|
| the annotation itself | its `/Subtype` is `/Text` (or `/Popup`, which this shell never addresses directly) | §12.5.6.4 Table 172. Table 169 gives no other annotation the key, and the engine refuses to invent it: *"writing it onto a `/Square` would add a key the standard does not define there, which is noise a later reader could mistake for meaning"* |
| the `/Popup` companion | there is one | §12.5.6.14 Table 183 |

With neither, the call **succeeds and does nothing** — no keys, no undo
entry — and the engine is explicit that this is a report rather than a
refusal, so that a caller acting over a mixed selection need not filter by
subtype. That is the right contract for the engine and the wrong affordance
for a shell: R83 says a control that cannot be honoured is not drawn, and a
tick box whose entire effect is a sentence explaining that it had none is
exactly the control that rule exists to remove.

# It does NOT manufacture the companion, and that is the engine's line

`set_annotation_open` *"does not create a `/Popup`. An annotation without
one has no window to open, and manufacturing the companion — with a `/Rect`
the caller did not choose — is authoring, not a state change."* This shell
agrees and does not work around it: a `/Square` an operator commented on
with no `/Popup` in the file is a shape whose window state is not
expressible, and the honest response is to offer nothing rather than to
author a rectangle nobody asked for at coordinates nobody chose.

⚠ The `/Popup` half is a fact about the **file**, read through
`Annotation::popup`. It is not a fact about whether pdfcer is drawing a
bubble right now — this shell draws one beside the note when the file gives
no rectangle, which is a placement fallback and emphatically not a `/Popup`
the document contains.

### `fn under`

The **last** match in paint order, exactly as
`crate::canvas::selection::annot::hit` takes the last: a sticky dropped on
top of a cloud is the thing the operator sees and therefore the thing they
mean.

# The tolerance is the frame's, not a number of this module's own

Handed in from `crate::canvas::mapping::PageMapping::tolerance`, which is
the same click tolerance content and annotation selection both use. A note
icon must be exactly as easy to hit as the shape beside it, and a
separately chosen constant here would drift from that the first time either
was tuned.

# Rectangle containment, not ink

Unlike the selection hit test, which narrows a ce dimension to its drawn
segments, this claims the whole `/Rect`. That is deliberate and it is the
convention: in every reader in the class, clicking anywhere on a
highlight's span or inside a cloud's box opens its note. Narrowing to ink
would make the note on a hollow rectangle reachable only by clicking its
hairline border.

### `struct Reply`

Flattened deliberately: §12.5.6.2 permits a reply to a reply, but every
reader in the class draws a comment thread as a **flat chronological list**
under its root rather than as a nested tree, and a tree drawn in a 260 pt
window would be four indents of two words each.

### `fn replies_to`

# Every page, because a reply need not be on the parent's page

§12.5.6.2 puts no page constraint on `/IRT`, and `pdfcer-core` reaches the
same conclusion where it plans a deletion: `plan_annotation_deletion` is
handed every annotation on every page because *"a reply may live on a
**different page** from the annotation it replies to — nothing in §12.5.6.2
binds a thread to one page — so a per-page scan would under-report"*.
Scanning the current page alone would silently drop replies on a
forty-sheet drawing set, which is the shape of document this program is
for.

# Replies to replies are flattened onto the root

One transitive pass: anything whose `/IRT` chain reaches `root` is in the
thread. `MAX_THREAD_DEPTH` bounds it, because a file may legally contain a
cycle (`a` replies to `b`, `b` replies to `a`) — §7.3.10 says a dangling
reference is not an error and says nothing at all about a circular one, and
`pdfcer-core` surfaces `/IRT` in `Annotation::in_reply_to` *"unresolved,
same as `popup`: a dangling `/IRT` is modelled, not repaired"*. A depth bound is the
only thing standing between that and a hang.

# Order

Document order — page order, then `/Annots` order. The same ordering
`crate::panels::comments` uses, **reused rather than re-decided**, and for
its reason: a second GUI-only rule is a second thing that can disagree with
`pdfcer list-annotations`. There is deliberately no sort by date, because
`/M` is not reliably a date (see [`NoteView::modified`]).
