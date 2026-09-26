# `text::panels::objects` — the Objects panel, and how a page object is
described

Two jobs, and they are one module because the second is only ever read
through the first and through [`super::properties`]:

1. The Objects panel's own chrome — intro, summary, empty states, tree
   row labels and tooltips.
2. **The wording of every fact in
   [`crate::panels::objects::summary::ObjectSummary`].** That type
   deliberately holds no prose at all; it classifies, measures and counts,
   and this module alone renders. The split is what makes the classifier
   unit-testable without an egui frame, and what makes a copy edit a
   one-file change.

## One description, several renderings

[`object_row`] and [`super::properties::property_rows`] are two
**renderings of one record**. A path's fill colour is therefore never
described one way in a tree row and a different way in Properties, and
that is structural rather than careful: both read the same
`ObjectSummary`, and the colour resolution ("which colour does a viewer
actually SEE for this paint disposition?") happens once, in
`summary::describe_object`.

## [`object_note`] is rule 4's disclosure, in words

Every one of these sentences is a fact `pdfcer-core` already computed and
never showed: `TextObject::approximate`, `PaintStyle::is_invisible`, an
exact zero-extent comparison on the bbox. **None of them is a guess**,
which is what makes surfacing them a disclosure rather than an inference
the operator would have to review.

Each sentence says WHAT is true and WHY the operator is seeing what they
are seeing, because "approximate bounds" on its own is a label, not an
explanation — and an explanation is the entire deliverable here. They are
long, and that is deliberate: they answer the operator's own report,
*"sometimes I click and get a box highlighting on the screen that doesn't
seem to correspond to anything."*

**They live in a panel, and nowhere else.** The disclosure rule is
explicit that inference reporting belongs off-canvas — a status line, a
results panel, a properties field — and equally explicit that no badge,
tint or dashed outline may be drawn into the page view. A panel is the
right home; this module supplies its words and nothing else's.

## What changed at salvage

- **`object_detail` and its three helpers are private here too**, exactly
  as they were. [`object_row`] is the public surface; the Properties
  panel asks for the same facts one at a time rather than reusing the
  joined clause, because a vertical list and a one-line row want
  different punctuation and joining them would give one of the two the
  wrong shape.
- **The row-label copy no longer promises Shift+click**, and
  [`objects_dock_intro`] no longer promises clicking selects anything on
  the page. There is no selection model at S3. The old sentences return
  with the selection they describe; stating them now would be a control's
  documentation shipping ahead of the control, which is the defect the
  old shell's own panel header records twice.
- **[`objects_dock_summary`] counts kinds** rather than counting a
  selection, for the same reason, and reads a
  [`SelectionCensus`](crate::panels::objects::summary::SelectionCensus)
  so the selection form is a caller change and not a copy change.

## Item notes

### `const ROW_TEXT_CHARS`

**A second, shorter cap than the core model's**
`pdfcer_core::vector::MAX_TEXT_PREVIEW_CHARS` (64), and deliberately so:
that one is a memory budget for a page of 50,000 objects, this one is a
line-length budget for a ~320 pt dock. Tying the row width to the storage
cap would mean a future memory decision silently re-typesetting the
panel. 32 characters is enough to recognise a caption or a ce-dimension
label — which is the row's whole job — inside a row that also carries an
index, a kind and a font.

### `fn quoted_text_preview`

Three things happen here, each for a stated reason:

1. **Quoted**, so an empty or space-only string is visible as a string
   rather than as a gap in the row.
2. **Elided at `limit`** with a `…`, and the ellipsis is also appended
   when the CORE model already truncated (`truncated`), so a long string
   never presents its prefix as the whole.
3. **Control characters replaced** with `·`. A `\n` or a `\t` inside a
   show string would otherwise break the row's layout or, worse, silently
   vanish — and an invisible character in a label is exactly the kind of
   thing that makes an operator distrust the panel.

`limit` is a parameter rather than a constant because the row and the
Properties panel have different widths to spend; both still get the same
quoting and the same control-character treatment, which is the part that
must not diverge.

### `const PANEL_TEXT_CHARS`

Larger than [`ROW_TEXT_CHARS`] because a field wraps and a row does not,
so the panel can afford the whole preview the core model kept. It is
still a cap rather than "everything", for the same reason the row's is:
the two answer different questions from
`pdfcer_core::vector::MAX_TEXT_PREVIEW_CHARS`, which is a memory budget
for a page of 50,000 objects. 64 matches that budget today, which makes
the panel's elision purely a function of what the model kept — the honest
place for it to be.

### `fn without_subset_tag`

A subsetted font's `/BaseFont` is `AAAAAA+JetBrainsMono-Regular`: six
arbitrary uppercase letters, a `+`, then the name. §9.6.4 requires the tag
to be unique per subset and says nothing about it meaning anything, and it
does not — it is a uniqueness token, not a fact about the typeface.

# Why it goes, and why the driven harness is what decided it

`the_inspector_is_one_master_detail_column` drove the real binary against
the operator's own A1 sheet and reported **8 of 9 object rows shortened at
the default width**. O123 part 6 widened Edit's inspector to 360 pt
precisely so the common row would fit, and it still did not — so either the
width was wrong or the rows were too long. The width measured 354 pt of
content (360 less the splitter), which is what it is supposed to be.

⇒ The rows were too long, and **seven characters of every text row were a
token with no meaning.** Removing it is the only shortening available that
costs the operator nothing: every other clause — the paint style, the colour
hex, the node count, the size — is a fact he might act on.

It was already the approved answer. `mockups/pdfcer-shell.html`'s legend
carries it as its own line — *"Strip the AAAAAA+ subset tag in the row only.
Kept in Properties"* — with the argument that two subsets of one face
otherwise read identically in the list while naming different font objects.

**In the ROW only.** Properties keeps the whole `/BaseFont`, because there
the tag is the handle that tells two subsets apart, and that panel has the
room to show it. A list optimises for scanning; a detail pane optimises for
identity.

# What is NOT stripped, deliberately

A `+` that is not preceded by exactly six uppercase letters. `Arial+Bold`
is a real face name in the wild and losing its first half would be a
silent corruption of the one thing this label exists to say.

### `fn object_detail`

Built from whatever the summary actually carries, in a fixed order, so
the same object always reads the same way:

| Kind | Clause |
|---|---|
| Path | `stroked #1A73E8, 0.50 pt wide · 4 node(s)` |
| Text | `"Section A-A" · Helvetica 10 pt` |
| Image | `640 × 480 px` |

Every part is omitted when the fact is absent rather than filled with a
placeholder: a text object with no `Tf` shows only its string, an image
with unusable `/Width`/`/Height` shows no pixel clause, and an object with
nothing to add gets an empty clause and just its kind name.

### `fn headline_identity`

Deliberately **not** [`object_detail`] with clauses removed: that function
answers *"describe this object"* and this one answers *"which one is it?"*,
and expressing the second as a filtered version of the first would make
every future clause added to the description silently widen the row again.
The two lists are independent on purpose, and
[`tests::the_headline_is_the_description_with_the_describing_clauses_dropped`]
is what keeps them from disagreeing about a fact.

### `fn every_note_has_its_own_short_form_and_its_own_sentence`

This is the sweep the `ObjectNote::ALL` array exists for. A note
added without an explanation — or with one copy-pasted from its
neighbour — would ship as a disclosure that discloses nothing, and
that is worse than no note at all because it looks like the app
answered the question. Review does not catch that reliably; a sweep
does.

The four `ApproximateTextBounds` bases are the ones most likely to
collapse into one sentence, and they are exactly the ones where the
difference matters most: "measured from the font's own metrics" and
"a rough guess that may miss the letters entirely" are opposite
levels of confidence about the same field.

### `fn every_object_kind_has_a_distinct_name`

The three image kinds in particular: collapsing "Image", "Image
(inline)" and "Form" into one word throws away the distinction that
most often explains a form XObject's surprising size.

### `fn every_paint_disposition_is_named_and_the_no_paint_case_says_so`

Six combinations of `(fill, stroke)`. The `(None, false)` case is the
one that must never come out blank: an object over blank paper with
no description is precisely the "box over nothing" confusion.

### `fn a_text_preview_is_quoted_de_controlled_and_marked_when_elided`

An invisible `\n` in a label breaks the row's layout; a silently
dropped one makes the panel look like it is showing a different
string from the one in the file.

### `fn a_subset_tag_is_stripped_from_the_row`

The driven check `the_inspector_is_one_master_detail_column` reported 8
of 9 rows shortened at the intended width; this is the shortening that
costs the operator nothing.

### `fn a_blank_page_and_an_unreadable_one_read_differently`

"This page really is blank" and "pdfcer could not read this page" must
never look the same — a failure state indistinguishable from a
success state is the same defect as no message at all.

### `fn the_intro_does_not_promise_click_to_select`

The old sentence said "Click a row to select it on the page". Nothing
in this build selects anything on the page, and a panel that says it
does sends an operator hunting for a broken control.

### `fn the_point_cap_states_both_numbers`

A list quietly shortened to its first N is indistinguishable from a
list that is N long. Stating "the first 200 of 6,681" is the whole
difference.

### `fn the_headline_is_the_description_with_the_describing_clauses_dropped`

Both directions, and both are needed. Asserting only what survives
passes on a build that changed nothing; asserting only what is gone
passes on a build that returns the empty string.

### `fn a_path_that_paints_nothing_still_wears_the_mark`

`headline_note` skips `ObjectNote::PaintsNothing` because
[`paint_style_label`] spells it out — and the headline no longer prints
that label. So the mark is raised by *any* note, and this is the case
that proves the difference: a clipping path is real, addressable, and
invisible, and a row that said nothing about it would be the panel
failing at the one question it exists to answer.
