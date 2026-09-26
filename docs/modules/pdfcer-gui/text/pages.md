# `text::pages` — every string the Pages panel shows

One area of the catalog described in [`crate::text`]'s header, consumed by
[`crate::panels::pages`] — the grid, its captions and its tile states — and
by [`crate::app::actions::pages`], which words what a page **delete** broke.
Those are the only two readers.

The second joined the first rather than getting a module of its own because
both are sentences about *pages* in the same vocabulary — sheets, page
numbers, this document — and a reader who came here for one half would not
find the other. See the disclosure section at the foot of this file for the
rule-4 obligation those strings discharge.

It is a sibling of [`crate::text::panels`] rather than a module inside it
for the same reason [`crate::text::forms`] is: that directory's own header
declares it covers *"the three document-structure panels"* and their two
inspector siblings, and the Pages panel is neither. It is a **navigator**
whose copy is about pictures, page geometry and the cost of drawing —
vocabulary that has nothing in common with a font inventory or a signature
byte range, and that would be read past by anyone maintaining either.

## The posture: an undrawn thumbnail must SAY it is undrawn

This is the whole reason half the strings below exist, and it is the
project's no-placeholders rule (`RIBBON_IA.md` P3) applied to a picture
rather than to a control.

A page thumbnail that has not been rasterized yet is, on screen, a
rectangle. A rectangle the colour of paper **is a picture of an empty
page** — and an empty page is a thing a real PDF can contain. So a
thumbnail grid that draws blank rectangles while it works is not
"loading"; it is *asserting something false about the document*, and the
operator has no way to tell the two apart. The old shell drew exactly that
(`main.rs`'s `thumbnail_rail`: a bordered rect in `extreme_bg_color` with
the page number), and it is the one part of that rail this panel did not
carry across.

Every state a tile can be in therefore has **words**:

| State | String | Says |
|---|---|---|
| queued, previews on | [`thumbnail_not_drawn_yet`] | *this is not a picture of the page yet* |
| previews off | [`thumbnail_previews_off`] | *and it will not become one until you say so* |
| the render hit the time ceiling | [`thumbnail_abandoned`] | *pdfcer started and stopped* |
| the page would not draw | [`thumbnail_failed`] | *this page is the problem, not the panel* |

No spinner, and that is deliberate rather than an omission: a dozen
spinning icons is motion, not information, and only one page is ever
being drawn at a time anyway.

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.**
- **Name the thing and what the operator can do about it.**
  [`previews_skipped_note`] is the worked example: it names the page, the
  limit it exceeded, and the box that changes that limit.
- **Never state a capability the build does not have.**

## Item notes

### `fn no_orphans_means_no_clause_about_them`

The clause used to be unconditional — *"**Any** form fields on those
pages arrived as boxes…"* — because the shell had no count and had to
hedge. `InsertOutcome::orphaned_widgets` arrived on 2026-08-19 and the
engine's reply says the number is **exact rather than an upper bound**,
so a zero can be believed.

Worth a test rather than a glance, because the failure is silent in the
direction that matters: a paragraph about form controls on a drawing
that has none trains the operator to stop reading the sentence, and the
sheets this application is for almost never have any. The clause that
gets skipped is then the *bookmarks* one, which is always true.

### `fn a_source_with_no_structures_produces_no_clause_about_them`

The sentence used to end *"Bookmarks and page labels from that file did
not come across"* on **every** insert. On a CAD drawing whose source had
neither — which is most of them, in this application — that is a
paragraph about two things that never existed.

It is worth a test rather than a glance because the cost is not the
wasted words. It is that the same sentence carries the clause about
orphaned form controls, which is *actionable*, and an operator who has
learned that this sentence is boilerplate stops reading the part that is
not.

The clause was unconditional only because nothing reported the fact —
which is the difference between a disclosure and a disclaimer.

### `fn the_stale_clause_is_about_this_document_and_comes_last`

The one fact in this sentence that describes the sheets in front of the
operator rather than a file they have finished with: their own page
numbers have quietly stopped describing the pages they are on.

Ordered last on purpose — a sentence whose most actionable clause comes
first is read and then abandoned at the part about a document nobody is
looking at any more. Asserted, because ordering is exactly the kind of
decision a later edit undoes without noticing.

### `fn a_real_count_is_stated_without_hedging`

The singular is its own arm rather than an `(s)`: one orphaned control
is an ordinary case — a single signature field on a title sheet — and
*"1 form controls"* is the shape that makes an operator distrust the
number beside it.

The route is asserted, not just the count. A disclosure that reports a
solvable problem without saying it is solvable leaves the operator with
a correct description and nothing to do with it.

### `fn the_recoverable_count_excludes_the_ones_that_cannot_be_recovered`

The engine's correction, asserted. `orphaned_widgets` counts every
orphan; `orphaned_widgets_unrecoverable` counts the subset whose field
identity is not in this file at all. Reporting the total as
re-registerable would send the operator to a panel where two of the
boxes refuse, with a sentence that had promised otherwise.

The measured case is the fixture: 13 orphans, 2 of them bare kids.

### `fn all_unrecoverable_means_no_re_registering_clause`

R9's rule applied to prose: *"0 form controls need re-registering"* is a
placeholder wearing a number, and it would be sitting immediately beside
a sentence saying two of them are gone for good.

### `fn every_conditional_clause_reads_alone_as_well_as_in_sequence`

Found by a driven run rather than by reading. On a source whose orphans
are all bare kids the re-registering clause is skipped, and the sentence
came out *"Inserted 2 pages after page 2. **3 more** lost their field
definitions entirely…"* — more than what? It reads as a sentence with
one deleted in front of it, which is exactly how an operator concludes
the program is losing text.

It is the **second** continuation-clause defect in this one function.
The first — *"Nor did its page numbering"* with no bookmarks clause
before it — was fixed an hour earlier, three clauses up, and the sweep
that should have followed it did not happen. Both arms are now asserted
together so the next one cannot be fixed alone.

### `fn an_impossible_pair_does_not_panic`

The invariant says it cannot happen — the second field counts a subset
of the first — and the subtraction saturates anyway. A disclosure runs
at the end of a *successful* edit, which is the worst possible moment to
panic on an arithmetic assumption about another crate's struct.

### `fn pages_count`

Singular and plural are spelled out rather than assembled with a `(s)`,
which reads as a form field rather than as a sentence. One page is a
reachable and perfectly ordinary case — most drawings are one sheet.

### `fn pages_selected`

**This number is the operand list the ribbon's Pages tab already
promises.** Every one of those commands' tooltips says *"the selected
pages"* — `pages.delete` is *"Remove the selected pages from this
document"* — so the count here is not decoration, it is the answer to
*"selected where?"* that the tooltips leave open. Wording it as a plain
count rather than as an instruction keeps it a statement of fact.

### `fn drag_landing`

The caret says *where* graphically; this says it in words, and both are
needed. A caret between two tiles in a scrolling grid of near-identical
drawing sheets is precise and not **checkable**: an operator cannot read a
page number off a hairline. The Insert-from-file dialog reached the same
conclusion for the same reason and its comment is the precedent — *"the
dialog is centred over a document the operator may have scrolled: the
number is what makes the choice checkable."*

# The vocabulary is the Insert dialog's, deliberately

*Before page N*, *the start*, *the end* — the same three phrasings the
insert-position radios use, because a drag and an insert answer the same
question about the same document and two different vocabularies for one
destination is how an operator ends up unsure whether they mean the same
thing. `gap` is a **gap index** in `panels::pages::ops`' sense: `0` is
before the first sheet, `page_count` is after the last.

### `fn drag_lands_nowhere`

**Said rather than shown by absence.** The alternative is to hide the
caret when the drop would not land, and that is worse: an operator whose
caret has vanished cannot tell *"this drop does nothing"* from *"the panel
has stopped tracking my pointer"*. The caret is dimmed and this sentence
explains the dimming.

It is the ordinary state at the beginning of every drag — a block hovering
over itself — so the wording has to read as information rather than as a
refusal.

### `fn pages_none`

Rare and not impossible: a damaged `/Pages` node can flatten to an empty
vector while the file still opens. Saying so beats an empty grid, which
reads as a panel that failed rather than as a document that is empty.

### `fn page_number`

**1-based.** Everything inside pdfcer indexes from 0; a human counts from
1, and the conversion happens here and at no other point in the panel, so
there is exactly one place the off-by-one could be.

### `fn page_tile_tooltip`

The page size is in **millimetres**, from the page's own extent in points
— because the operator this panel is for is looking at a drawing sheet
set, and "A1" or "841 × 594" is how they identify a sheet. It is also the
one useful fact a tile can carry before its picture exists, which is why
the tooltip is worth having on an undrawn tile at all.

The gestures are named because none of them is discoverable: nothing on
screen says that Ctrl adds to the selection.
Takes **points**, not millimetres, and rounds here.

The caller has the sheet's extent in PDF user-space units and nothing else;
asking it to convert was how this surface came to disagree with the print
dialogue about a 210.5 mm sheet. `units::whole_mm_from_points` rounds half
away from zero, the same rule every other length in this program now uses,
and the result is printed with `{}` rather than `{:.0}` — `{:.0}` rounds
half to EVEN and is what produced the disagreement.

### `fn thumbnail_previews_off`

Distinct from [`thumbnail_not_drawn_yet`] on purpose. "Not drawn yet"
promises a picture is coming; with previews off, none is, and an operator
waiting for one that will never arrive has been misled by a word.

### `fn thumbnail_abandoned`

Reachable when the document is edited, or the panel closed, while a page
is being drawn. Not a failure — nothing is wrong with the page — so it
must not read like one.

### `fn thumbnail_failed`

Names the *page* as the subject, because that is what is true: the panel
works, and this one page did not draw. The canvas will say the same thing
at more length if the operator navigates to it, which is the right place
for the detail.

### `fn previews_tooltip`

**The number in this sentence is measured, not estimated.**
`BENCHMARK.md` records a real CAD drawing whose content stream costs
~0.74 s to interpret *at any scale* — a one-by-one-**point** region of it
costs 691 ms — so a thumbnail of such a page is not cheap merely because
it is small. That is the single most surprising fact about this panel and
it belongs where the operator meets it.

### `fn previews_budget_suffix`

Seconds rather than milliseconds because the operator is choosing how long
they are prepared to wait for one picture, and nobody has ever had an
opinion about that in milliseconds. The leading space is the gap between
the number and its unit; `egui` concatenates the two without one.

### `fn previews_budget_prefix`

A bare `2.0 s` beside a checkbox is a number with no verb — the operator
has to hover to learn whether it is a limit, a delay, or an interval.
`≤ 2.0 s` reads as a bound at a glance, in three characters, which is what
a 260 pt panel can afford. The tooltip carries the sentence.

### `fn previews_budget_tooltip`

The operator cannot pick a per-page time limit without knowing what a page
costs, and pdfcer is the only party that has measured it. Quoting the two
ends of the measured range — an ordinary drawing-office sheet against the
densest CAD page in the benchmark set — turns "type a number" into a
choice between two outcomes the operator recognises.

⚠ Both numbers are measured, not estimated: `thumbnails.rs`'s
`PAGE_BUDGET_DEFAULT` carries the table they come from.

It also states the consequence of the limit being hit, because that is the
part an operator would otherwise have to infer from a tile going blank:
**that page alone** is skipped. The rule this replaced skipped the rest of
the document too.

### `fn previews_budget_never`

# Why the box shows a word and not the number

Because `0` and *no limit* are opposite readings of the same glyph, and
the wrong one is the intuitive one. A box reading `≤ 0.0 s` beside a
grid of blank tiles says *give up immediately*, which is what O187
predicted the operator would otherwise be left to discover — and the
operator who typed `0` on purpose would have no confirmation that pdfcer
understood them. One word removes both readings.

It is deliberately the whole contents of the box, prefix and suffix
included: `≤ never s` is not English, and a control that keeps its units
while its value stops being a quantity reads as a formatting accident.

⚠ It is also what `crate::panels::pages::previews::parse_budget` must
accept back, because a
`DragValue` re-parses what it displayed the moment the operator clicks
into it. A word shown and not accepted is a control that empties itself
on a click.

### `fn previews_skipped_note`

Named parts: the page, the limit, and the control that changes the limit.
A message that said only "a page was skipped" would leave the operator
hunting for a cause pdfcer already knows.

⚠ **It states the limit, not the cost**, and the difference is not
pedantry. pdfcer abandoned the render, so it does not know what the page
would have cost; printing a number as though it did would be inventing
evidence. *"more than 2.0 s"* is the whole of what was measured.

Printed in **seconds to one decimal** because the number's job is to be
compared against the box beside the checkbox, which is also in seconds. A
disclosure quoting a different unit from the control it points at makes
the operator do arithmetic before they can act on it.

### `fn deleted_dangling_bookmarks`

Singular and plural spelled out rather than assembled with `(s)`, exactly
as [`pages_count`] does and for the same reason: one broken bookmark is a
perfectly ordinary outcome of deleting one page, and `1 bookmark(s)` reads
as a form field rather than as a sentence.

### `fn deleted_dangling_links`

"on the pages that remain" is load-bearing: links that left with their own
page are deliberately not counted by the engine, because reporting them
would inflate the number with references that no longer exist to be broken.
The sentence says which set it is talking about so the number can be
trusted.

### `fn deleted_dangling_destinations`

Named destinations are reached from *outside* this document as well as from
within it — another PDF's link, a URL fragment, a script — which is why they
are disclosed separately from bookmarks rather than added to that count.

### `fn deleted_page_labels_stale`

A sentence rather than a count, because the underlying fact is a boolean:
the tree is one object and the operator's question is *"are my page numbers
wrong now?"*.

It says pdfcer left them **deliberately**, because the alternative reading —
that pdfcer failed to update them — invites the operator to report a bug
against behaviour that matches Acrobat's and was chosen. Acrobat leaves them
stale and silent; this is the "and says so" half.

### `fn deleted_separations_repaired`

The one class of broken reference the engine **repairs** rather than
reporting, and it is still disclosed for exactly that reason: something in
the file changed that the operator did not ask for. `DeleteOutcome`'s own
docs draw the line — a bookmark's target is a question about *authorial
intent* that pdfcer must not guess at, while a separation dictionary's
`/Pages` array is a *structural* fact pdfcer knows the answer to.

### `fn insert_dialog_title`

Not the Open dialog's title. The two pick a PDF and mean opposite things —
one replaces what is on screen, the other adds to it — and a picker headed
*"Open a PDF"* over a document the operator is editing is a sentence that
says the wrong thing at the moment they are most likely to read it.

### `struct Structures`

# This sentence was WRONG for two hours, and the correction is the point

It read: *"Bookmarks, form fields and page labels from that file did not
come across — its pages did."* Three nouns, one verb, and the verb is true
of two of them and **false of the third**.

Measured rather than assumed, after the operator asked why the structures
could not simply be re-added: a source with **12 form fields** inserted into
a blank document produces **13 widget annotations and no `/AcroForm` at
all**. The widgets came across — `insert_pages` copies everything reachable
from the page, and a page's `/Annots` reaches its widgets. What did not come
across is the **field tree that names them**.

So they are not absent. They are **orphaned**: boxes that draw exactly like
form fields, that an operator will click on, and that nothing can fill
because no field claims them. That is this project's own *"visible control,
silently inert"* failure arriving through a document instead of a ribbon —
and the old sentence would have sent an operator looking for the missing
fields rather than at the ones in front of them.

**A disclosure that names the wrong failure is worse than none**, because it
is believed. It was written from the engine's summary — *"does not merge the
source's document-level structures"* — which is accurate about `/AcroForm`
and says nothing about the widgets, and I did not check.

# The three fates, which is what the sentence now distinguishes

| structure | what happens |
|---|---|
| pages, content, resources, fonts | copied at fresh object numbers |
| **form fields** | widgets **arrive**, `/AcroForm` does not — inert boxes, and the engine now **counts** them exactly |
| bookmarks, page labels, named destinations | genuinely absent |

# Why the page number is 1-based

*"after page 7"* is the sheet the operator was looking at, in the numbering
the page box and the thumbnails use. A 0-based index here would be the only
place in the application that counted differently.
# The orphan clause is now a NUMBER, and it is exact

This sentence used to hedge — *"**Any** form fields on those pages arrived
as boxes…"* — because the shell had no way to know whether there were any,
so a document with no form controls got a paragraph about form controls.

`EditSession::insert_pages` returns `InsertOutcome { pages_inserted,
orphaned_widgets }` as of 2026-08-19, and the engine's reply is explicit
that the count is **exact rather than an upper bound**: `/AcroForm` is
document-level and is not merged, and the copy remaps every object number,
so no field in the target can be claiming a widget that has just arrived.
*"There is no case where a counted widget turns out to have an owner, and
you can put the number in front of an operator without hedging it."*

So `orphans == 0` drops the clause entirely. That is not a cosmetic saving:
a sentence about form controls on a drawing with none trains the operator
to stop reading the sentence, and the drawings this application is for
almost never have any.

# And the clause is PERMANENT, which the wording has to survive

The engine over-ruled the framing this shell filed it under. It was
proposed as *"the count now, carrying the definitions later"*, and the
answer was that a field's widgets can be **split** across inserted and
non-inserted pages — so a residue survives *any* merge and the count exists
for ever. `Pass 102.1` will reduce the number and can never make it always
zero.

The sentence is therefore worded as a fact about what arrived, not as an
interim apology for a feature that is coming.
What an insert did to the two document-level structures that do not travel
with a page.

# Three booleans, and there are three because the REMEDIES differ

The engine's ruling on the two label fields, adopted verbatim and extended
to the third: *"a stale tree wants renumbering; a dropped one wants
creating. A single 'page labels are wrong' message names neither, which is
why I did not merge them."*

| field | what is true | what the operator would do about it |
|---|---|---|
| `outline_dropped` | the source file had bookmarks and they did not come | re-create them in the Bookmarks panel |
| `labels_dropped` | the source file had its own page numbering and it did not come | accept this document's numbering, or author one |
| `labels_stale` | **this** document's numbering now points at different sheets | renumber the ranges |

The third is the one worth having and the one this shell would never have
asked for. The first two are about a file the operator has finished with;
`labels_stale` is about the document **in front of them**, and it says the
page numbers they are looking at have quietly stopped describing the pages
they are on.

# Why the outline case is a boolean and not "always"

Because it used to be "always", and that made it a **disclaimer rather than
a disclosure**. The sentence said *"Bookmarks and page labels from that file
did not come across"* on every insert, including a CAD drawing whose source
had neither — a paragraph about two things that never existed, which is how
an operator learns to stop reading the sentence that also carries the clause
about form controls.


The engine's note on *why* bookmarks never came is kept here because it is
what makes the remedy obvious: `/Outlines` is a **catalog** entry,
unreachable from any page, so a copy that walks outward from the pages never
sees it. They are not lost in transit — they were never in the set of
objects being copied. Which is why carrying them means replaying the source
outline through `add_outline_item`, i.e. exactly what the Bookmarks panel
now does by hand.

# Why pdfcer deliberately does not match Acrobat on page labels

Carried in this type's own documentation, because the first review question
about a *"pdfcer wrote nothing"* disclosure is always *"what does Acrobat
do?"* — and the answer being **something worse** is what nobody would guess.

Acrobat does neither of the two things anyone assumes. It does not carry the
source's labels and it does not leave the inserted pages unlabelled: it
**overwrites every inserted page with a static copy of the label on the page
preceding the insertion point**. Not incrementing — the same string on all
of them. The engine sourced three independent Adobe Community threads
(2024-2025) in which a twelve-page chapter labelled `10-1`...`10-12`,
inserted after a page labelled `9-45`, came out with **all twelve showing
`9-45`**. Those threads are complaints about it.

So matching it would be matching a defect. This type is what makes *"pdfcer
wrote nothing"* a stated choice rather than a silence.

### `fn inserted`

`orphaned_widgets_unrecoverable` arrived hours after `orphaned_widgets`,
with the engine's own correction of the sentence it had suggested the day
before. It measured its own output — 13 orphans from a real AcroForm — and
found two shapes:

| shape | of 13 | registering it |
|---|---|---|
| **merged field-widget** (§12.7.3.1) | 11 | recovers the field exactly |
| **bare kid** (a radio group member) | 2 | **impossible** — its identity is not in this file |

The engine's words: the undifferentiated sentence *"is true of both rows and
useful for only one."* For the 11 it describes **a chore this shell can now
offer to complete**. For the 2 it describes **a permanent loss** whose only
remedy is going back to the source file — and the combined total says the
milder of the two, which is the wrong way for a disclosure to be wrong.

So the clause splits, and each half is dropped when its number is zero. A
document with 11 recoverable and 0 unrecoverable gets one sentence with a
route in it. A document with 0 and 2 gets one sentence with no route,
because there is none.

# The recoverable clause names WHERE, and that is the whole point of it

*"Forms ▸ Tab order lists them"* is the difference between a disclosure and
a complaint. Before `EditSession::adopt_widget` shipped there was nothing to
name, so the sentence could only report the damage; now the panel that
already computes exactly this set can register them one press at a time, and
a disclosure that stops short of saying so leaves the operator with a
correct description of a problem and no idea it is solvable.

# Why the unrecoverable clause says "re-insert from the original"

Because it is the only thing that works, and because the alternative an
operator would otherwise try — typing a name into the box — produces
something that *looks* like success. A named bare kid is a new, empty,
typeless field. It is not the radio button that was lost, and an operator
who believes it is will go looking for its group. See
[`crate::text::status::adopt_declined_no_name`], which refuses the word
*restore* for the same reason.

### `fn insert_failed`

`detail` is `pdfcer-core`'s own error `Display`, passed through for the same
reason [`crate::text::canvas_render_failed`] passes one through: those
errors are specific, and replacing one with *"could not open the file"*
discards the half that says whether it was encrypted, truncated or not a
PDF at all.

**Says nothing was inserted.** A failure part-way through a multi-page
insert would otherwise leave the operator wondering whether some of it
landed; the verb is one command and either records it or does not.

### `fn insert_empty`

A separate sentence from [`insert_failed`] because it is not a failure: the
file opened, it is a valid PDF, and it is empty. Collapsing the two would
send an operator looking for corruption in a file that has none.

### `fn insert_source`

The **count is the point**. An operator about to insert a document they
picked from a folder has no other way to know whether it is the four-page
revision or the forty-page one, and finding out afterwards means an undo.

### `fn insert_range_hint`

It states the two behaviours that surprise people, because both are useful
here rather than merely tolerated: a range is a **sequence**, so `3,1-2`
inserts page 3 first, and it is not de-duplicated, so `1,1` inserts a page
twice. An operator who wants either has no other way to ask for it in one
gesture.

### `fn insert_range_unparsable`

One sentence for every way it can fail — a number past the end, a backwards
range, a typo — because the remedy is the same for all of them and the
source's own page count is already on screen two lines above.

### `fn insert_commit`

The count is on the control rather than beside it, for the reason the print
dialog's commit button carries its clip count: it is on the thing the
operator's hand is already on, where it cannot be looked past.

### `fn merged`

# Two renames, and both are disclosures rather than warnings

**`fields_renamed`** — a field whose name was already taken here arrives
under a different one. The engine's note on why that is still the right
behaviour is worth carrying: *"two fields sharing a fully qualified name are
ONE field, and filling either fills both."* But the operator now has a field
whose name is not the one the source document showed them, and **any script,
FDF or calculation keyed on the old name no longer matches it.** Nothing
else would tell them.

**`named_destinations_renamed`** — the same for §12.3.2.3 destinations, with
a second consequence the engine spells out and this sentence must not lose:
pdfcer rewrites the *carried* bookmarks to the new keys, and **cannot rewrite
a link in a document it did not copy.** An outside `/GoToR` reference to the
old key now resolves to *this* document's destination rather than the
source's — a link that still works and goes somewhere else.

Each clause appears only when its count is non-zero. A clean merge of a
form-free drawing set says one thing: how many pages arrived.
