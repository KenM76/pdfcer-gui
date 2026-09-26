# `panels::comments` — every annotation on this document, listed

The comment list a reviewer works through. The classification lives in
[`model`]; this file is the drawing, the disclosures and the actions the
panel can raise.

## What it deliberately excludes, and why each exclusion is safe

- **`/Widget`** — form fields have their own first-class surface (the Forms
  panel). `Annotation::is_widget()` already exists as the exact predicate;
  a second one would be a divergence waiting to happen.
- **`/Popup`** — a reader-UI window attached to a `Text`/`FreeText`
  annotation, never independent content. One row per real annotation; its
  pop-up is implementation detail. §12.5.6.14 is a `shall`: a pop-up
  *"shall not appear alone but is associated with a markup annotation, its
  parent annotation."*
- **ce dimensions are NOT excluded by type**, and that is worth stating:
  they are `/Line` annotations and so they appear here. They have their own
  home, but excluding them by subtype would also hide a genuine `/Line`
  markup an operator drew. Showing them is the lesser wrong, and it is
  honest — they ARE annotations on the document.

### `/TrapNet`, excluded twice over

Two independent arguments reach the same exclusion, and either alone would
be enough:

- It is **prepress output state** — the trapping a RIP applied to the page.
  Nobody wrote it, it is not a comment, and there is nothing in it for a
  reviewer to work through. That is the same shape as the `/Widget`
  exclusion: not *"we cannot act on it"* but *"this surface is not about
  it."*
- `pdfcer-core` refuses a `/TrapNet` deletion by name, and this panel has a
  Delete, so a row for one would carry a control whose every press is a
  refusal — the affordance R83 forbids.

**Nothing is silently omitted.** Every exclusion is counted and disclosed by
[`crate::text::panels::comments::comments_excluded`], so a reviewer looking
at six rows on a drawing they know carries forty annotations is told the
arithmetic and where each missing kind went. The numbers are stated on every
case, not only on the empty one.

**A filter is a FOURTH kind of omission**, and it is disclosed by the same
rule — see [`filter`]'s header and
[`crate::text::panels::comments::comments_filtered`]. It is the only one
the operator caused, which makes stating it more important rather than
less: an exclusion is a property of the document a reviewer learns once, a
filter is a switch they set an hour ago and have forgotten.

## Ordering: page order, then `/Annots` order

The ordering `pdfcer list-annotations` already produces, **reused by
name** rather than a second GUI-only rule that could disagree with it. See
[`model`]'s header for what that means concretely, and for why there is no
sort by date.

## Read the SESSION, not the file on disk

[`body`] hands [`model::collect`] `doc.session.view()` — the base revision
with **every unsaved edit applied**, which is the same thing the canvas
rasterizes. An operator who has just drawn three shapes must see three rows
without saving first. `crate::panels::forms`' body is the worked example
and carries the same sentence.

## Actions, not mutations — and every one of them is an `Action`

[`Action::GoToPage`], from a row's **Go to** control, exactly as
`crate::panels::bookmarks` does; and [`Action::Annot`] carrying
[`AnnotAction::SetNote`], [`AnnotAction::ClearNote`],
[`AnnotAction::Delete`] and [`AnnotAction::Reply`]. The body is handed
`&OpenDoc` — a **shared** reference, so this is a compile-time fact and not
a convention — it reads, and it pushes. It never touches the document.

⚠ **One thing this panel does write directly, and it is not the document.**
A row's **Go to** also opens that comment's canvas pop-up, through
`crate::canvas::notepopup::open::set`, which is `egui::Memory` — interface
state, per document, never saved.

It opens the **thread root's** window rather than the row's own, resolved
by [`model::thread_root`]: the canvas draws no window
for a reply, because `add_reply` places one at its parent's own `/Rect` and
a bubble there would cover the comment it answers. See
`canvas::notepopup::model::notes_on`'s exclusion table.

It is here rather than behind an `Action`
because an `Action` is drained *after* the frame and the pop-up must be
open on the frame the page arrives, and because the actions-not-mutations
rule is about the **document**: the thing it protects is the undo stack and
the edit epoch, neither of which a floating window touches.

Why it does it at all: jumping to page 14 and leaving the reviewer to
find which of its six clouds the row meant is half a navigation. Acrobat's
Comment pane opens the comment it takes you to, and that is the gesture
being matched.

### The row deletes its own comment

Operator, `OPERATOR_REQUESTS.md` O130: *"the review features should look and
act the same as they do in Acrobat Reader."* A reviewer's work list is where
a comment is most naturally removed, and it is the only surface that can
reach a **hidden** one at all.

`crate::app::actions::annot::AnnotAction::Delete { page, id }` is the
variant, `crate::app::actions::apply` is the dispatch arm, and
`crate::app::actions::annots::delete` is the body, which reports the
engine's collateral through `crate::text::markup::deleted_collateral`. The
canvas Delete key and the Format tab reach the same verb, so the three
surfaces cannot disagree about what a deletion does.

#### What the Delete carries, and where each piece comes from

- **R83 — the control is omitted where the engine would refuse.**
  `EditSession::annotation_deletion_refusal` answers the two document-wide
  cases (encrypted, certified) and [`delete_control`] asks it. A locked
  annotation and a ce dimension are withheld by the row itself, for reasons
  the row already states.
- **"Delete is not redaction"**, in the tooltip, per §3.4 — an incremental
  save leaves the previous revision in the file.
- **The collateral is reported after the call, not predicted before it.**
  §3.4's own reason: a preview *"is not a perfect oracle"* and the real call
  can still refuse. `delete_annotation`'s report names what actually went — a
  `/Popup` removed, replies orphaned, group members promoted — and
  `annots::delete` already surfaces it. One statement of record beats a
  guess before and a fact after that can disagree.

## Rule 4: everything here is disclosure, and none of it is on the page

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`'s first
non-negotiable: *"Disclosure lives off-canvas."* **A panel is the right
home**, and this one draws not a single pixel on the canvas — no badge on a
hidden annotation, no tint on an unresolved appearance, no outline round the
row under the pointer. It must not start to. The one-line test is *would a
screenshot of the editing canvas differ from a screenshot of the same
document saved and reopened?*

Three of this panel's row captions exist **only** because of that rule, and
each names the inference it discloses:

| Caption | The inference |
|---|---|
| `comment_row_hidden` | none — this one is a *document fact* the file states and the page therefore cannot show, so it is listed and marked hidden |
| `comment_row_appearance_unresolved` | pdfcer chose to paint nothing, under a default core documents as **evidence tier (d), a reasoned guess** |
| `comment_row_is_group_member` | pdfcer shows the raw `/Contents` where §12.5.6.2 says a reader should show the group primary's — so another viewer legitimately disagrees |

Highlighting the shape under the hovered row **would be permitted** — rule
4's fourth clause allows *"a snap indicator, a hover highlight, a
rubber-band, a selection handle — these are the cursor"*. It is simply not
built here, and not for want of a mechanism:
`crate::panels::forms::spotlight` is a panel→canvas channel of exactly that
shape and `crate::canvas::forms` draws what it carries. Named as a
*permitted* affordance so nobody later reads its absence as a rule.

## The two layout rules, and which one applies

1. **Scrollbars must be visible.** `crate::panels::scroll_style` is applied
   by [`crate::panels::Panel::show`] before any body runs, so this panel
   inherits it. egui's default `floating()` bar allocates zero space and is
   fully transparent when the pointer is elsewhere, which makes a scrolling
   area indistinguishable in a capture from content clipped at the
   container edge.

2. **A fixed-size child inside a scroll area needs the container's width
   stated.** **This panel has no fixed-size child**, so
   [`crate::panels::content_width`] is deliberately not called — stated here
   rather than left to look like an omission, because skipping the second
   layout rule silently is how a panel ships clipped rows.

   Every child is a `Label`, which wraps to whatever width it is given, so
   the clamping defect cannot arise: there is nothing whose *requested*
   size could exceed the pane and be silently squeezed. The one fixed-width
   child is the **Go to** button, at a couple of dozen points against a
   dock that opens at 320. A note body is arbitrary operator text and can
   be a paragraph; stating a container width computed from it would either
   scroll a 4,000 pt row sideways or defeat its own wrapping. Vertical-only
   scrolling with wrapping labels is the correct shape here, and
   `crate::panels::forms` — whose rows have the same character — takes it
   too.

## Cost, stated rather than discovered

[`body`] walks **every page's `/Annots`** every frame, and lays out every
row it finds. Both are bounded —
`pdfcer_core::annot::MAX_ANNOTS_PER_PAGE` caps the walk, and the walk reads
one array plus one dictionary per annotation rather than decomposing any
content — so on the documents this project measures against
(`SW41177.pdf`, 36 sheets) it is nothing beside a raster.

The one thing that would change the picture is a document with thousands of
comments, where the *layout* rather than the walk becomes the cost. The fix
is `ScrollArea::show_rows`, which needs a uniform row height, which these
rows do not have — a row is one to six labels depending on what the
annotation carries. Named here so the next hand does not have to measure it
twice; `crate::panels::forms` carries the same note for the same reason.

## `PDFCER_DIAG` proves what the panel computed

One `comments-panel` line per frame carrying the counts: rows found, rows
with note text, rows with an author, ce dimensions, suppressed rows,
unresolved appearances, relations — and the three exclusion counts
separately, so *how many it excluded and why* is answerable from the trace
alone. That is the founding rule of this project applied to a surface whose
correctness is entirely arithmetic: a screenshot of this panel cannot tell
you that a widget was excluded, and the trace can.

## Item notes

### `struct RowSink`

Every line below the heading is conditional, and each condition is a real
state of a real document rather than a formatting choice. A row is between
two and seven lines tall depending on what the annotation actually carries,
which is why this panel cannot use `ScrollArea::show_rows` — see the module
header.
**What one row can raise**, collected so the row function takes a subject
and a destination rather than eight loose parameters.

Three `Option`s and a flag, and each is `None`/`false` for the whole frame
unless exactly one row sets it — which is the invariant that makes them
scalars rather than `Vec`s: **two rows cannot be pressed in one frame**, and
a `Vec` would invite a future reader to queue two navigations or two edits
that would each bump the epoch under the other.

### `fn delete_control`

# Four reasons it is not drawn, and every one is R83 rather than R9

R83 — *an affordance that cannot be honoured is not drawn* — rather than
R9's *unavailable renders nothing*, and the distinction is real: the
capability **exists**, and what is missing in each case is the permission
to use it on *this* annotation. None of the four is a build limitation, so
none of them is a sentence about pdfcer.

| not drawn when | why |
|---|---|
| the row has **no object id** | a direct dictionary in `/Annots` is a malformed file (§12.5.2 Table 164 requires an indirect object) and there is nothing to name. The row already says so where its editor would be |
| the document **refuses deletion** | encrypted, or a certification signature forbids the change. `EditSession::annotation_deletion_refusal`, asked once per frame — see [`RowSink::deletable`] |
| it is a **ce dimension** | rule 15. `delete_annotation` would remove the `/Line` and leave the `/PieceInfo` sidecar describing a ce dimension that no longer exists. The Dimension groups panel owns that verb |
| the annotation is **hidden** | deliberately NOT a reason. A hidden annotation is exactly the one a reviewer cannot reach from the canvas, so this panel is the only place it can be removed — which is the whole argument for listing it in the first place |

# No confirmation, and no hover preview

The collateral is reported **after** the call rather than predicted before
it, because `docs/core-api/03-capabilities.md` §3.4 says a prediction *"is
not a perfect oracle"* and the real call can still refuse.
`delete_annotation`'s report names the pop-up it removed, the replies it
orphaned and the group members it promoted, and
`crate::app::actions::annots::delete` surfaces it. One statement of record
beats a guess before and a fact after that can disagree with it.

Undo is the safety net, and it is one press: the deletion is a single
`CommandKind` on the same stack as every other edit.

### `fn filter_strip`

# Built from the UNFILTERED rows, always

A chooser built from what survived the current filter would drop every
other author from the menu the moment one was chosen, leaving no route from
one reviewer's comments to another's except *Show all* and starting again.
The lists are therefore derived from the whole listing and are stable while
the operator works — which is also what makes them a map of the document
rather than a map of the current view.

# Why `ComboBox` and not a row of toggles

Because the number of authors and the number of subtypes are properties of
the **document**, not of this build: a drawing set that went round six
reviewers has six names, and six toggles would be six lines of a 320 pt
dock. A chooser is one line whatever the document contains.

# `Show all` is drawn only when a filter is set

R9's shape applied to a control that would do nothing: with no filter in
force, *Show all* is a button whose entire effect is a repaint. It appears
with the first narrowing and goes when the last one is lifted, which also
makes its presence a second, wordless statement that something is hidden.

### `fn chooser`

Written once for the author and the type because the two differ only in
their label and their values — and because a second copy is a second place
for the `None` entry to be forgotten, which would leave a filter nobody
could lift.

### `fn sort_label`

A `match` rather than a method on [`filter::Sort`], because a label is copy
and copy lives in `crate::text` — and because the compiler makes this
exhaustive, so a fourth ordering cannot ship without a word for it.

### `fn trace`

# Why this is more than a debug print

R1 — a surface is verified by driving the binary, not by a passing test —
and a screenshot of this one cannot tell you that four widgets were
excluded, that two rows are hidden annotations, or that the `/Line` on
page 3 was recognised as a ce dimension. Every one of those is arithmetic,
and arithmetic is what a trace is for.

Every count that drives a *decision* is here, which is the test for what
belongs: `with_note` decides the document-wide disclosure, the three
exclusion counts decide the exclusion line, and `ce_dimensions`,
`suppressed`, `unresolved`, `replies` and `group_members` each decide a row
caption. If a number here is wrong, something on screen is wrong with it.

# `listed` is the CENSUS, `shown` is what is on screen

[`filter::apply`] sits between [`model::collect`] and the draw, so the two
numbers differ whenever a filter is set. One field carrying both meanings
would let a reader outside the process watch a census shrink with no way to
tell a filtered list from a document that had lost annotations — exactly the
omission this panel's founding discipline forbids, held on the diagnostic
channel as well as on the screen, and the more load-bearing because the
driven `save_copy_round_trip` and `undo_redo_round_trip` use this line as
their **only** oracle for whether an annotation reached the document.

So the line carries both, and they answer different questions:

| field | question | source |
|---|---|---|
| `listed` | how many annotations does this document have that a reviewer may work through | [`model::collect`], unfiltered — the census |
| `shown` | how many rows is the operator actually looking at | the same rows after [`filter::apply`] |
| `filtered` | is the operator's filter narrowing the list right now | [`filter::Filter::is_narrowing`] |

`filtered=1` says the operator is narrowing the list, and `shown` says by
how much.

⚠ **The filter is read one frame late**, and that is deliberate rather than
overlooked: this runs *before* [`filter_strip`] draws, so a filter the
operator changes on frame *N* appears here on frame *N+1*. The panel
repaints continuously, so a filter that is on is reported as on within a
frame; what the lag rules out is reading a single frame's line as evidence
about a filter set in that same frame, which nothing does.

### `mod reviewstate`

Its header carries the two facts everything in it follows from: a status is
**appended, not set** — the file holds a log rather than a field — and the
engine reads both keys **verbatim without interpreting them**, so the
vocabulary is this shell's to present and an unrecognised value is shown
rather than normalised.

### `const COMMAND_ID`

Named here as well as on [`crate::panels::Panel::command_id`] so this
module's own reachability test can assert it, exactly as
`crate::panels::forms` does.

# Why `markup.comments` and not a `view.panel_*` id

`RIBBON_IA.md` names Comments **twice** and the two placements cannot both
be honoured, because P1 gives a command one tab:

- **§5.2** lists `Comments` among View ▸ Panels, beside Pages, Objects,
  Bookmarks, Layers, Signatures and Forms.
- **§5.5** gives the Markup tab its own `Comments` group, with `Comments
  panel` in it.
- **§7's migration map** then settles it explicitly, naming the source and
  the destination: `Review ▸ Comments ▸ Comments` → `Markup ▸ Comments`.

The migration map is the more specific statement — §5.2's row is a list of
panel names, while §7 is a per-control ruling on this control — so Markup ▸
Comments it is. `crate::shell::manifest::markup` already reached the same
conclusion in the same words when the tab was built, and the command is
already registered and already on the ribbon; only the panel behind it was
missing.

**The mode taxonomy agrees, which is what makes this safe.** The Forms
panel had to move *off* the Edit tab because Read is shown `file` and
`view` alone and Read mounts Forms. Comments is mounted by **Review and
Edit only** (`crate::app::modes::defaults`), and both of those are shown
the `markup` tab. So no mode can mount this panel without also being able
to reopen it — which is the failure the Forms move existed to prevent.

### `const REGION_EDIT`

# Why only the first, when every row draws the control

A region name is a key. Publishing the same name from thirty rows would
leave the harness clicking whichever one happened to be drawn last, which is
a coordinate nobody chose and which moves when a row above it grows a
caption. The first row is the one deterministic choice available without
inventing a per-row naming scheme that nothing would consume.

⇒ **A control with no published rect is a control R1 cannot reach**, and
the gap leaves no failing test behind — the harness simply never presses it,
and nothing goes red to say so. Publishing this one is what keeps the
panel's writing controls drivable.

### `const REGION_POST`

A **separate** name from [`REGION_SAVE`], deliberately. The two controls
occupy the same place on the row and reach different engine verbs, so one
shared name would leave a driven check unable to tell *"the note editor is
open"* from *"the reply editor is open"* — which is precisely the pair a
harness must distinguish, because one of them writes over the comment the
other one answers.

### `fn body`

The one entry point. Shape and signature match every other panel body — see
[`crate::panels::Panel::show`].

## `state` carries one thing, and it is not a selection

It holds a [`note::NoteDraft`] — one annotation's `/Contents` while the
operator is typing it — and **nothing else**. It is emphatically not a
"selected comment": the draft names one annotation by `ObjId` for the
duration of one edit, and it decides nothing about what the canvas outlines,
what the Format tab describes or what Delete acts on. That distinction is
the one [`crate::panels::ObjectTreeUi::focus`]' docs refuse to blur, and it
is what stops this panel growing a second, weaker selection that the canvas
would then have to be kept in step with.

# THREADING DEPTH: the file nests, the panel does not

`EditSession::add_reply` permits a reply to a reply — its scope note 2 wants
one and does no cycle checking beyond the obvious — so **does a reply to a
reply draw indented under it, or flat?**

## The answer, in each of the three places it shows

| | |
|---|---|
| **in the file** | the true parent, always. `AnnotAction::Reply` carries the **row's own** `/IRT` and never rewrites it to the thread root |
| **in this panel** | flat. Every annotation is one row in document order, and a reply is marked by its caption ([`t::comment_row_is_reply`]) rather than by an indent |
| **in the canvas pop-up** | flat, one level under the root — `canvas::notepopup::model::replies_to` gathers the whole transitive thread and lists it |
| **when the two differ** | the operator is told, on the row, at the moment they are about to answer an answer — [`t::comment_row_reply_to_a_reply`] |

## Why flat, and what the alternative would have cost

Three reasons, in the order they decided it:

1. **The canvas pop-up is flat**, for the reason `replies_to` records:
   every reader in this class draws a comment thread as a flat chronological
   list under its root rather than as a nested tree, and a tree drawn in a
   260 pt window would be four indents of two words each. Nesting the panel
   while the pop-up stayed flat would give one document two shapes in one
   program, which is worse than either shape.
2. **This panel is a work list, not a conversation.** Its rows are headed by
   subtype and page because a reviewer scanning forty of them is looking for
   *the cloud on sheet three*, and its filter strip narrows by author and
   subtype. An indent tree fights both: a filtered tree either hides parents
   whose children matched or shows rows the filter excluded, and there is no
   third option.
3. **An indent has to be computed from something that can be malformed.**
   §7.3.10 makes a dangling `/IRT` not an error and says nothing about a
   circular one, and `pdfcer-core` models both rather than repairing them.
   A depth column therefore needs a cycle bound, a rule for a parent that is
   not in the list, and a rule for a parent that was filtered out — three
   decisions in service of a visual that the surface it would appear on does
   not want.

**The cost is named rather than hidden**: a reader of this list cannot
tell, from the list alone, which comment a given reply answers. That is
what [`t::comment_row_is_reply`] admits by saying *"a reply to another
annotation on this document"* rather than naming one, and it is why *Go to*
resolves through [`model::thread_root`] — the canvas window is where the
conversation is legible, and this panel's job is to get the operator there.

⇒ If an operator asks to see who answered whom without opening each
comment, that is a surface-level decision and belongs in
`MODES_AND_PANELS.md` rather than here.
