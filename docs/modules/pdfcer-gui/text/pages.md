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
