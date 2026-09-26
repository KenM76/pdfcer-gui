# `app::actions::pages` — the four page verbs, and the resync a structural
edit owes

[`super::apply`] is the interpreter for every [`Action`]; this file is the
body of the four that act on **pages** rather than on the marks drawn on
them — [`Action::RotatePages`], [`Action::DeletePages`],
[`Action::ReorderPages`] and [`Action::ExtractPages`] — plus the one
function every *other* edit in the application now calls as well,
[`resync`].

## Why these four are not four more arms in `apply.rs`

Rule R2's own justification decides it, exactly as it decided the
`apply.rs` split from `actions.rs`: *"the value of the limit is that the
file has to have a single subject."* `apply.rs`'s subject is **the
cancel–mutate–bump–invalidate protocol** — what happens when a request to
change the document is granted, and the ordering that makes it safe. That
protocol is the same for every verb and changes only when the protocol
changes.

This file's subject is a different one, and it is the reason page verbs
could not simply be four more `vector_edit` calls:

> **A page index is a position, not an identity**, and the application
> holds four things that are stated in page indices.

Those four are the flattened page vector, every cached raster, the canvas's
object selection, and the Pages panel's own picks. The general rule is that
a selection is an identity — page, object, subpath, node — not a position,
and `crate::canvas::interact`'s header states the measured half of it:
*"`move_*` renumbers nothing … the `delete_*` family is the one that
renumbers."* A page delete is that sentence one structure up, and a page
**reorder** is a third case neither of them names.

## The table this whole file exists to implement

| | page vector | rasters | canvas selection | panel picks | `view.page_index` |
|---|---|---|---|---|---|
| markup, move, fill *(existing verbs)* | unchanged | current page's dropped | survives — resolved against the new epoch | untouched | valid |
| **rotate** | `/Rotate` differs | **all stale** — every turned page's picture is sideways | survives; a rotation adds and removes no operator | **survives** — the same sheets are still the same sheets | valid |
| **reorder** | order differs | **all stale** — page *N* is a different sheet | **cleared** | **remapped** — the permutation says where each went | valid, may now show a different sheet |
| **delete** | shorter | **all stale** | **cleared** | **cleared** — the picked sheets are gone | **may be past the end**; clamped |
| **extract** | unchanged | unchanged | untouched | untouched | valid |

Every row of that table is asserted below or in
[`crate::panels::pages::select`], because every cell is a way for the
application to end up drawing the wrong sheet or aiming a destructive verb
at one nobody chose — and none of them fails loudly.

## [`resync`] is called from `vector_edit`, not from these four arms

That placement is the one design decision in this file worth arguing: it is
the one-choke-point rule applied to a *consequence* rather than to a
dispatch.

The naive arrangement is for each page arm to do its own tidying after its
own `vector_edit` call. It is wrong for a reason that is invisible until
undo exists — and undo exists: `Action::Undo` runs the *same* engine
commands backwards, through the *same* `vector_edit`, and an undone
`DeletePages` puts four sheets back. Tidying in the four arms would leave
the undo of a page delete showing a page vector that is four sheets short,
with a page count in the status bar to match, and nothing anywhere would
error.

So the resync sits at the one place every document change already passes
through, and it is **self-describing rather than told**: it compares the
page vector it has against the one the session now reports and acts on the
difference. A verb it has never heard of gets the right treatment, which is
what makes it a choke point rather than a fifth copy of a rule.

Its cost on an edit that changed no page is one page-tree walk and one
`Vec` comparison, paid **per operator gesture** rather than per frame. That
is the same order as the `Arc::get_mut` and the epoch bump beside it.

## What [`resync`] cannot do, and who does it instead

The **Pages panel's picks** live on `PdfcerApp::panels`, not on `OpenDoc`,
and `vector_edit` takes an `&mut OpenDoc`. They are therefore handled in
`apply.rs`'s two arms that know which edit ran — which is sound because
those are the only two verbs that can move a pick, and it is *stated* here
rather than left to be discovered, because the consequence of forgetting is
a Delete aimed at sheets the operator did not choose.

The **thumbnail cache** needs nothing at all: it is keyed on
`(edit_epoch, pixels_per_point)` and `ThumbnailCache::sync` empties itself
the moment the epoch moves, which `vector_edit` has already done by the time
the next frame draws the panel. That is the key-carries-the-staleness design
`apply.rs` wishes the page texture had.

## Item notes

### `fn delete_disclosures`

`vector_edit`'s disclosure channel carries exactly this shape of thing —
*"the drawing is unchanged but the file is not, and rule 4 forbids letting
the operator find that out from a diff"* — and a page delete is the verb
with the most to disclose in the whole application.

`EditSession::delete_pages` computes the census and deliberately does **not**
repair; its own documentation names surfacing the result as the front end's
job and calls Acrobat's silence *"a low bar, not a target to literally
copy."* This is that half.

# Returns

One sentence per fact that is true, in the order an operator would care
about them: the references they navigate by first, then the numbering, then
the prepress structure. **Empty** when nothing was broken, which is the
ordinary case for a drawing set and which makes `vector_edit` record no
sentence at all rather than an empty one — see its own docs on why that
distinction matters.

The wording is [`crate::text::pages`]', under rule R1. This function decides
*which* sentences and in what order, and no words at all.

# Why the two halves arrive separately

`DeleteOutcome` is `#[non_exhaustive]`, so no test outside `pdfcer-core` can
build one — and a rule-4 disclosure whose *selection* logic cannot be
asserted is a rule-4 disclosure nobody has checked. `DanglingReport` is
`Default` and constructible field by field, so taking it plus the one
separation count keeps the decision testable. The caller does the
destructuring, which is one line and is the line that would not compile if
the engine's shape changed.

### `fn edit`

The four-step protocol is not re-run here — there is no render worker in
a unit test and nothing else holds the `Arc` — but the two steps the
assertions depend on are: the mutation, and the epoch bump that
[`resync`] traces. Anything that needs the *whole* protocol is
`tools/ui-verify`'s job, which is where the join is proven.

### `fn select_object_on`

Through [`crate::canvas::selection::SelectionState::marquee`], which is
a real gesture entry point, rather than by reaching into the struct:
there is no setter, deliberately — the canvas is the only writer — and a
test that needed one would be asking for an API the application does not
have. A marquee of one target lands at the Object rung, which is the
state a page edit has to invalidate.

### `fn a_delete_shortens_the_page_vector_and_clamps_the_view`

The defect this catches is silent and total: `OpenDoc::pages` is
*"resolved once at open"*, so without [`resync`] the panel would go on
saying "4 pages", the status bar would go on saying `n/4`, and the
canvas would go on rendering a `Page` whose object the engine has
**freed**. Every test in the crate would pass, because nothing else in
the application ever re-reads that vector.

### `fn a_delete_clears_the_canvas_selection`

A selection is an identity, not a position, and this is that rule at
page level: an entry that survived would resolve against another
sheet's decomposition
on the next frame and draw an outline round an object nobody selected —
with `format.delete` one keystroke away.

### `fn a_reorder_renumbers_and_clears_the_canvas_selection`

The middle case, and the one a length comparison alone would miss
entirely: the page count is unchanged, so a resync that only watched
`len()` would leave every strip raster and the canvas selection pointing
at sheets that have moved.

### `fn a_rotation_refreshes_the_pages_without_clearing_the_selection`

The falsifying half of the two tests above. A resync that cleared the
selection on any change at all would pass both of them and would make
every rotate throw away work the operator had done — and no assertion
anywhere else would notice, because a cleared selection is a valid
state.

`crate::canvas::interact`'s header states the rule this pins: a verb
that adds and removes no operator renumbers nothing.

### `fn a_delete_discloses_one_sentence_per_broken_thing`

The empty case matters as much as the full one: `vector_edit` records
`None` for an empty list, so a build that returned a placeholder
sentence would put a line under every page delete and train the operator
to ignore the ones that mean something.

### `enum PageAction`

## Why this is a sub-enum rather than five more variants on `Action`

The same three reasons [`super::dimensions::DimensionAction`] gives, and the
first is again the one that decided it:

1. **They share a rule the flat enum could not express.** Every verb here
   can **renumber** the document, and what each owes the shell's derived
   state afterwards is different: a rotation preserves both selections
   (nothing renumbers), a reorder remaps the Pages panel's picks and clears
   the canvas selection, a delete clears both, an insert navigates to what
   arrived. As five flat variants that rule is re-derived in five arms; as a
   family it lives here, where a sixth verb has to answer it.

   The failure that guards against is specific and silent: a page verb with
   the wrong invalidation produces a **correct document** and a wrong
   screen, so nothing fails and the operator sees a selection pointing at a
   sheet that has moved.
2. **R2.** `super`'s enum crossed 1,500 lines when image placement landed,
   and the alternative to a seam is thinner prose — which the file-size
   gate's own header names as the incentive it refuses to create.
3. **The destination already existed.** This module has held the five
   verbs' *bodies* since page operations shipped, and `apply` already routed
   every one of them here. The enum was the only half still living
   elsewhere.

### `fn resync`

Called from `super::apply::vector_edit`'s success path — every document
change in the application, including an undo or a redo of one. See the
module header for why it lives there rather than in the four page arms, and
for the table of what each kind of edit invalidates.

# The comparison, and why it is `(id, rotate)` rather than the whole page

`pdfcer_core::page_tree::Page` is not `PartialEq` and comparing it fully
would compare two resolved `/Resources` dictionaries, which is expensive and
answers a question nobody asked. The pair below is the **complete** set of
page facts anything in this application caches:

* **`id`** — the page object's identity. A change in the *sequence* of ids
  is a reorder or a delete, and it is the only signal that separates "page 3
  is a different sheet now" from "page 3 looks different now". Nothing else
  in `Page` can tell those apart, which is why identity rather than geometry
  is the key.
* **`rotate`** — the one page attribute an edit in this build changes that
  does not change the id. A rotation leaves every index meaning the same
  sheet and makes every cached picture of it wrong.

A page whose *media box* changed would be missed. No verb in this build
changes one, and the honest note is here rather than in a comment claiming
completeness: the day a crop verb lands, its extent belongs in this pair.

# What a failed page walk does

Traces and returns, leaving the previous vector in place. The alternative —
emptying it — would turn a transient page-tree read failure into a document
that appears to have no pages, and an operator cannot save what the shell
has decided is empty. `page_tree::pages` fails only on structural damage,
which an edit through `EditSession` cannot introduce; this is the honest
answer for a case that should not arise rather than a case that is expected.

### `fn merge_into`

# Why this uses the SESSION verb and not `pageops::insert`

`pdfcer_core::pageops::insert` also inserts pages and returns the bytes of a
**new document**. Wiring that would have meant replacing `OpenDoc::session`
wholesale — which discards the undo stack, invisibly to any test that
checks page counts, and visibly the first time an operator presses Ctrl+Z
twice.

So it was filed rather than shipped, and `pdfcer-core` answered the same day
with `EditSession::insert_pages`: the missing member of the `delete_pages` /
`reorder_pages` / `rotate_pages` family. It records **one** undoable command
however many pages arrive, exactly as a reorder does however many move.

# What it does not carry

Page content, resources, fonts and XObjects come across at fresh object
numbers. The source's **document-level** structures do not — outlines, the
AcroForm field tree, named destinations, page labels. That is the honest
cost of staying incremental, because a document-level merge rewrites objects
an incremental save exists in order not to touch.

[`crate::text::pages::inserted`] says so in the disclosure, because an
operator whose bookmarks did not come across is entitled to know at the
moment it happened rather than by going looking for a bug.

# The three ways it can decline, and why they read differently

| condition | sentence |
|---|---|
| the file would not open | [`crate::text::pages::insert_failed`], carrying the engine's own reason — encrypted, truncated, not a PDF |
| it opened and has no pages | [`crate::text::pages::insert_empty`] — **not a failure**, and collapsing it into one would send the operator looking for corruption that is not there |
| the insert itself refused | `vector_edit`'s own decline path, as every other edit |
**Merge a whole document into this one**, with its form, its bookmarks and
its named destinations.

Raised by `pages.merge_into` on the Pages tab.

# It is not [`insert_from_file`] with "all pages" ticked

`insert_pages` takes some pages and **orphans** the widgets on them: a form
field arriving that way is drawn and cannot be filled. `merge_document`
re-parents each widget to its field, so the field arrives working — the
engine's own words, *"that is the whole point of the verb"* — and carries
the source's `/AcroForm`, its outline and its named destinations with it.

So the two commands on the Pages tab are *"pages"* and *"a document, with
the things that make its pages work"*, and an operator choosing between them
has no way to find that out except from the tooltips.

# The blocker this had was real, and it was answered

Its `SCAFFOLDED` entry read: *"`insert` returns the bytes of a NEW document
rather than mutating the session … wiring it means replacing
`OpenDoc::session` wholesale, which discards the command log the undo work
is building."* That was **true when written**, was filed rather than worked
around, and the engine answered it with an in-session verb that is one undo
entry.

What the entry then said — that this *"wants a destination document, and a
shell that can only edit the open document has nowhere to put it"* — had the
destination backwards: the manifest's own taxonomy is that Pages ▸ Merge
*adds to this document* and Tools ▸ Merge *combines files into a new one*.
The open document **is** the destination. Found by re-deriving the list.

# Position

`InsertPosition::End`, and it is not a placeholder. Merging is *"add this
document to mine"*, and every application that offers it appends — the
alternative, asking where, is `pages.insert_from_file`'s question and that
command already asks it. A merge that opened a position dialog would be the
insert command wearing a different label.

### `fn insert_from_view`

Sharing it is not merely tidy. `crate::text::pages::inserted` reports six
facts about what did and did not come across (orphaned widgets, of which
some are unrecoverable; a dropped outline; dropped page labels; stale page
labels), and a second copy of that reporting would be a second place for it
to fall behind what the engine actually returns.

# Returns

**How many pages the document actually gained**, which is `0` for every
refusal — an empty operand list, an engine decline, a source with nothing in
it. The cross-document *move* needs it: the source's pages are removed only
if the target's insert happened, and *"did it happen"* is a
question this function is the only one in a position to answer. A move that
deleted first, or deleted regardless, would lose the operator's sheets to a
refusal they never saw.

### `fn apply`

## Why this takes `panels` as well as `doc`

Because the answer to *"what does this edit do to what is on screen?"* is
**different for each of the five**, and three of the answers are about the
Pages panel's own picks, which live on [`crate::panels::PanelsState`] rather
than on the document:

| verb | canvas selection | panel picks |
|---|---|---|
| rotate | kept — nothing renumbers | kept |
| reorder | cleared by the resync | **remapped** through the permutation |
| delete | cleared by the resync | **cleared** |
| insert | — | — (the view navigates instead) |
| extract | — | — (no document changes at all) |

`vector_edit` cannot reach `PanelsState`, so the choice has to be made by
the caller of it — and making it *here*, beside the bodies, is what keeps
the table above in one place instead of spread across five arms in the
interpreter.

## Every guard is on the EPOCH, not on a return value

A refused delete — the engine refuses removing every page, §7.7.3.3 — must
leave the operator's selection exactly as they built it. Testing whether
the epoch moved is what distinguishes *"the edit applied"* from *"the verb
was called"*, and it is the one signal that is true for every path including
a session that could not be borrowed.
