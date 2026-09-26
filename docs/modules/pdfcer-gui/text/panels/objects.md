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

### `fn objects_dock_intro`

States the ordering convention, because "which end of this list is the
front of the page?" is otherwise a guess — and it is the convention the
panel's whole diagnostic value rests on. The old sentence continued
*"Click a row to select it on the page; Shift+click to add it to, or
remove it from, the selection."* Both clauses return with the selection
model.

### `fn objects_dock_empty_page_hint`

Deliberately distinct from [`objects_dock_decompose_failed_hint`] — a
genuinely blank page and a page pdfcer could not read must never look
identical. A failure state that is visually indistinguishable from a
success state is the same defect as no message at all.

### `fn objects_dock_summary`

Reads a census rather than a bare total because the breakdown is the
orienting fact — *"1,410 objects"* on a CAD sheet says only that the
sheet is big, while *"1,380 paths, 30 text"* says what kind of drawing it
is and which of the panel's row kinds to expect.

Kinds with a zero count are omitted rather than printed as `0`: a row of
zeros is noise, and noise is how the numbers that matter get skimmed
past.

### `fn objects_dock_row_tooltip`

One entry for every row rather than a per-kind variant: the row shows the
same three things whatever kind it is, and assembling this sentence from
per-kind fragments is how a catalog acquires four subtly different
versions of one idea.

It names the index's meaning, which is the row's least obvious and most
useful property: `#412` is the number `pdfcer object-list` prints and
`object-delete` takes, so an operator who wants to script the same change
across fifty files can read the argument straight off the panel.

### `fn object_tree_run_row`

**"Line", and the number counts lines, not show operators.** A producer
is free to write one line of a title block as nine `Tj`s, and the
operator sees one line; numbering the rung in show operators would put
nine rows under a row he reads as one. `ObjectModelProvider::part_count`
answers in lines for text, and this label is the same number.

**Not "label".** That presumes the CAD case that motivated the text half
of this rung; a line is just as often a fragment of ordinary prose.

### `fn object_tree_node_row`

The number is the **object-scoped** anchor index — it keeps counting
across a part boundary rather than restarting at 0, because the number
pdfcer shows and the number `pdfcer node-move --node N` addresses have
to be the same number (decision 025 §1.3(b)).

### `fn object_tree_points_capped`

**New at salvage, and it exists because of a measured number.** One path
object on a real CAD export holds **6,681 anchors**. The old tree listed
every one of them, relying on `ScrollArea::show_rows` to virtualize — but
virtualization makes a wall of rows cheap to *draw*, not useful to
*read*, and materialising 6,681 `ObjectTreeRow`s to find one costs a
frame on the sheet where it matters most.

So the point list is capped, and the cap is **stated with both numbers**
rather than the list being quietly shortened. A silently truncated list
is indistinguishable from a short one, which is the same defect
[`super::bookmarks_truncated`] exists to prevent one panel over.

### `fn object_kind_label`

The three image kinds get three different names rather than one, because
they are three different answers to "what is this?": an inline image
lives in the page's own byte stream, an image XObject is a shared
resource, and a form XObject is an entire nested drawing treated as one
opaque object — which is itself a common explanation for "why is this
object so much bigger than the thing I can see?".

### `fn paint_style_label`

**This is also where the winding rule is stated**, and it is stated by
naming the non-default: `even-odd` is called out, non-zero is not. That
is not brevity — non-zero is the rule a reader applies to `f`, and a row
that said "filled (non-zero)" on nine hundred ordinary shapes would bury
the thirty where the distinction changes what is drawn.

Words rather than the CLI's machine tokens (`fill-nonzero+stroke`): this
is prose an operator reads, and the two surfaces have different
audiences. The `n` case is spelled out at length because "paints nothing"
is the direct answer to "why is there an object here over blank paper?" —
a clip or discarded path is still a real, addressable object.

### `fn winding_rule_label`

[`paint_style_label`] names it only when it is even-odd, which is right
for a one-line row and wrong for a field list: a field headed "Winding
rule" that is blank for nine rows in ten reads as a value pdfcer failed to
read. A field list has room to state both, so it does.

`None` for an object that has no fill at all — a stroke-only or `n`-op
path has no winding rule in effect, and printing "non-zero" there would
name a rule that decides nothing.

### `fn rgb_hex`

Number formatting lives in the catalog, never inline at a call site.

Components are clamped before scaling: a PDF may set a colour component
outside 0..1 and the decomposition records what it read rather than
silently repairing it, so the clamp belongs here, at the point of
display, not in the model.

### `fn quoted_text`

Same quoting and same control-character treatment as the row's — that
half must not diverge, because an invisible `\n` is exactly as misleading
in a field as in a row — and a longer cap, because a field wraps. See
[`PANEL_TEXT_CHARS`].

### `fn font_label`

`/BaseFont` is preferred over the `Tf` resource name because `F1` names
nothing an operator can recognise — but the resource name is shown when
that is all there is, rather than dropping the font entirely, since
"which resource" is still the handle for a later edit.

The size is the `Tf` operand, **as the file states it** — see
`pdfcer_core::vector::TextFont::size`, which documents why it is not
scaled by a `Tm`/`cm`. It is written `10 pt` rather than `10.00 pt`
because a type size is conventionally a whole number and the trailing
zeros would read as a precision this value does not claim.

### `fn object_row`

`index` is the object's PAINT-ORDER index, printed verbatim so it
cross-references `pdfcer object-list`'s `index=` field and the
`object-move` / `object-delete` / `node-move` operands, which all address
an object by exactly this number. Showing a display-position number
instead (the list is drawn back-to-front) would produce a number that
looks equally authoritative and addresses a different object.

Takes an [`ObjectSummary`] rather than a `VectorObject`: the
classification (which colour is actually visible, how many nodes, which
disclosures apply) belongs to `summary::describe_object`, and this
function only words it. That is the single-source-of-truth requirement
made structural — this row and the Properties panel are two renderings of
ONE record, so a fill colour cannot be described one way here and another
way there.

The trailing note marker is what makes a row diagnostic rather than
decorative: a text row, a clip path and a hairline all look ordinary
until the row says out loud that its stated extent will not match what is
on the paper.

### `const OBJECT_ROW_DISCLOSURE_MARK`

One character in place of a phrase between nineteen and thirty characters
long, and the trade is stated rather than assumed: the row loses *which*
doubt applies and keeps *that* one does, and the which is one hover and one
pane away (see [`object_row_headline`] §3). A column of identical marks is
also scannable in a way a column of varied phrases is not — an operator
looking for "the objects pdfcer is unsure about" reads down the right edge
instead of reading eight different sentences.

U+26A0 rather than an exclamation mark or an asterisk: those two are
ordinary text and appear inside font names and quoted document strings, so a
row could wear one it did not mean.

### `fn object_row_headline`

[`object_row`] is the full description and stays exactly as it was; this is
the same record said shorter, and the panel draws this one while attaching
[`object_row`] on hover. Both are one rendering of one [`ObjectSummary`], so
the short form can never disagree with the long one about a fact — it can
only carry fewer of them.

# 1. Why a second form exists: every row was elided, on every fixture

The driven check `the_inspector_is_one_master_detail_column` reported
**8 of 8 object rows do not fit at the default width**, and the panel's own
`objects-rows overflow=` field — added for exactly this question — said how
far: the widest row wanted **473.6 pt** in a **314 pt** pane. At O123's
360 pt Edit inspector the pane is ~354 pt, so the dock would have to open at
roughly **526 pt** — half of an 1,100 pt window — for that row to fit.

⇒ **No width closes this**, which is what the `overflow=` field was added to
be able to say. Widening the inspector to follow the content would also be a
measurement fed back into a size (**R128**), and this change deliberately
feeds nothing back: the width is a constant, the row is what changed.

# 2. What is dropped, and where each dropped fact went

| dropped from the row | still shown |
|---|---|
| the paint-style phrase (*"filled (even-odd) and stroked"*, up to 40 chars) | the hover, and the Properties pane's own fields |
| the stroke width (*", 0.80 pt wide"*) | the hover, and Properties |
| the font name and size (*"JetBrainsMono-Regular 4.50 pt"*, ~30 chars) | the hover, Properties, and the Fonts panel |
| the worded disclosure (*"bounds from metrics"*) | the hover, and Properties' disclosure list — the row keeps [`OBJECT_ROW_DISCLOSURE_MARK`] in its place |

**This is what master–detail means**, and it is the arrangement the same
request asked for: *"Objects and Properties become master–detail in one
panel."* The detail pane is on screen, in the same column, an inch below the
row. A master row that restates it is spending the operator's width twice on
one fact and eliding the identity — the index and the words — that only the
master carries.

# 3. What is kept, and why each earns its characters

- **the index**, verbatim: it is the operand `pdfcer object-list`,
  `object-delete` and `node-move` all take, and nothing else on screen
  carries it;
- **the kind word**: the one classification that decides which verbs apply;
- **the quoted text preview** for a text object — the only way to tell one
  run from another, and the reason a text row is findable at all;
- **the visible colour and the node count** for a path — the two facts that
  distinguish two otherwise identical rules, and the pair the operator's own
  drawings are read by (a path carrying 4,405 anchors is a different animal
  from one carrying 2);
- **the pixel size** for an image;
- **[`OBJECT_ROW_DISCLOSURE_MARK`]** when the object carries any note at
  all.

The mark is raised by `!notes.is_empty()` rather than by
[`headline_note`], and the difference is load-bearing: `headline_note` skips
`ObjectNote::PaintsNothing` **because [`paint_style_label`] already says
it**, and this form no longer prints that label. Using `headline_note` here
would leave a clip path — an object that is real, addressable and paints
nothing — wearing no mark at all, which is the one disclosure on this panel
that explains a box the operator can see and cannot find ink for.

# 4. A long row still elides, and that is the mechanism working

The quoted preview is capped at [`ROW_TEXT_CHARS`] characters, so a row
quoting a full-length string plus a mark can still exceed a narrow pane —
and `panels::elide_to_width` shortens it with the whole string on hover,
exactly as O123 asked. What this function fixes is that **every** row needed
that, not that any row does.

### `fn headline_note`

`PaintsNothing` is deliberately skipped: [`paint_style_label`] already
spells it out inside the detail clause, and a line reading "…paints
nothing (a clip or discarded path) · 4 node(s) · paints nothing" says it
twice and explains it neither time. Every other note adds a fact the
detail clause does not carry. The FULL sentence for `PaintsNothing` is
still shown in the Properties panel's disclosure list, where it earns its
space by saying the object is real and addressable.

### `fn object_note_short`

Paired with [`object_note`]'s long form rather than replacing it: the row
flags that something needs explaining, the Properties panel explains it.
Two lengths of the same fact, never two different facts.
