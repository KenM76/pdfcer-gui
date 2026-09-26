# `panels::properties` — the detail of what is selected, and nothing else

## Every section here is scoped to a SELECTION

The file's `/Info` fields and the seven facts about the file itself are
[`crate::panels::docprops`], a panel of its own, reached by
`file.document_properties` on File ▸ Document and mounted in all three
modes. **Every section in THIS panel is scoped to a selection**, and that is
the property to preserve: the one thing this panel is for is answering
*what is the thing I picked*, which is what makes it the detail half of the
Objects/Properties master–detail column `OPERATOR_REQUESTS.md` O123 asks
for.

⇒ **A section added here must read the selection.** If it does not, it
belongs in `docprops` or in a panel of its own; a permanent block at the
foot of this one puts something on screen that is true of no selection, and
it takes an operator report to find, because every test in this file stays
green.

An egui finding kept here because it is about the toolkit rather than
about any one section: a
`CollapsingHeader` whose open state must follow a condition **and** stay
overridable by the operator cannot use `default_open` (consulted only
through `load_with_default_open`, so it is dead after frame one) and cannot
use `open(Some(_))` (it forces the state every frame and moves click
handling into an `else` branch, so the header stops responding at all). The
shape that works is `set_open` + `store`, run **only on the frame the
condition flips**. Checked against the vendored egui 0.35 source rather than
remembered.

## Built from the specification

**New.** Nothing like it exists in the shell this one replaces — a
properties panel of any kind is not among what is carried across — so this
is `RIBBON_IA.md` §5.8 built from the specification.

## Why the panel is built before the tab

§5.8, verbatim:

> Build order: **panel first, tab second.** The panel is the harder half
> and the tab's contents are a subset of it, so building the tab first
> would mean writing the property editors twice.

and, on the division of labour between the two surfaces:

> the **tab** carries what a user changes *while working* — colour,
> width, style, align, delete. The **panel** carries everything,
> including the read-only facts (winding rule, node count,
> embedded-font status, exact geometry) that belong beside the Objects
> panel's inventory rather than in a ribbon band.

Those four parenthesised facts are this module's brief, and all four are
below.

## One description, two renderings

Every value here comes from [`super::objects::summary::describe_object`]
and is worded by [`crate::text::panels::objects`] — the *same* record and
the *same* functions the Objects panel's row label uses. That is the
single-source-of-truth requirement made structural: a path's fill colour
cannot be described one way in a tree row and a different way in
Properties, because there is only one description.

What differs between the two is *shape*, not *content*: a row is one line
and joins its facts with separators; a panel is a list and labels them.
[`crate::text::panels::properties`] owns the labels — the left-hand
column — and nothing else.

## This panel is where rule 4's disclosure lands

Every [`super::objects::summary::ObjectNote`] the object carries is
spelled out in full at the foot of the list, under its own heading. That
placement is the disclosure rule's:

> **Disclosure lives off-canvas**: a status line, a results panel, a
> report after the command, a properties field. … **No badge, tint, red
> flag, dashed outline or "provisional" layer drawn into the page view.**

In the old shell, "these bounds are approximate" drove a **dashed outline
on the page**. Under the rule as it now stands that is content marking,
and content marking is forbidden — it is *"a second rendering path for
the same content, and two paths drift"*. Here the same fact is a
sentence, in a panel, and the canvas is untouched.

The heading is *"Worth knowing about this object"* rather than
*"Warnings"*, and the sentences are drawn at ordinary weight: every one
of them is a fact about the **document**, and warning styling would make
a property of the file read as a pdfcer failure.

## What it describes, given that there is no selection

`super::PanelsState::focus` — the object whose row the operator last
clicked in the Objects panel. That is **not a selection**, and the
difference is spelled out where the field is declared. The consequence
here is that the panel's empty state names the Objects panel by name: it
is the only route in, and an operator has no way to guess that.

## What is deliberately not built

§5.8 also commissions **editable X/Y/W/H** here, and calls the panel the
surface through which `/Rect` move-and-resize becomes reachable without a
drag. None of it is here, and the reason is not that typed geometry is
hard: there is nothing to edit. [`crate::app::actions::Action`] carries
zoom and page navigation, and this module may not add to it. Four
spinners bound to nothing would render, accept typing, and discard it —
not a harmless placeholder but a control that silently loses an
operator's work.

So the geometry is stated as facts in the same list as everything else,
and [`crate::text::panels::properties::properties_read_only_note`] says
so once at the top. `RIBBON_IA.md` P3: an unavailable capability renders
nothing; greying is for *temporarily* unavailable, and "the selection
model does not exist" is absence, not temporary unavailability.

## The embedded-font field is a name join, and it discloses that

A text object records the `/BaseFont` in effect; the document's font
inventory records a program per font **dictionary**. Joining them by name
is the only join available — the object model does not carry the font
dictionary's object id — and a name is not a key. One document can
declare two font dictionaries with the same `/BaseFont` (two independent
subsets of one face, which the survey behind the Fonts panel found in
87 % of embedding files), and they need not agree about embedding.

So the field has **three** answers, not two: yes, no, and *"pdfcer could
not tell — the Fonts panel lists each one separately"*. Picking one when
the name is ambiguous would be an inference presented as a fact, and
unlike most inferences this one is invisible: a confidently wrong "Yes"
looks exactly like a right one.

## Item notes

### `mod dimension`

It makes this panel's founding premise false and says so: the ui-spec's
*"nothing else competed for the word Properties"* stopped holding the day a
canvas selection became a second claimant on it. Its own header carries the
argument for broadening this panel's purpose rather than inventing a ninth.

### `mod disclose`

The 47-word refusal sentence that has never been readable needed a surface
whose width is decided before its body draws. The status bar is not one
(R128), and its own header checks that rather than assuming it.

### `mod markup`

Its header carries why this is the PANEL rather than the Format tab
(`RIBBON_IA.md` §5.8) and why every control raises one action carrying one
field.

### `fn body_sections`

Split out so the scroll area is impossible to forget: a section added to
this function is inside it by construction, where a section appended to
`body` after the `.show(..)` call would silently be outside it again — which
is the defect this arrangement exists to make unwritable.

### `fn object_section`

`something_drew` is passed rather than re-derived so this function knows
whether the panel is already saying something — see the *nothing focused*
arm.

The parameter is named `something_drew` and not after any one section: it
carries a disjunction of every section above, and a name that points at one
of them is a trap for the next reader.

# It returns whether it DREW, and nothing reads that

`true` on the two paths that draw property rows, `false` on both early
returns **including** the *nothing is selected* label — that sentence draws,
but it is not a description of a selection.

This is the last thing in the panel, so the value is bound to `_drew_object`
at the call site rather than deleted: the distinction it encodes is real, it
is one line, and the next section appended here will want it. Deleting it
leaves the next author to rediscover that *"nothing is selected"* must not
count as having spoken.
**No `PanelsState`**, deliberately. Taking one to read a panel-local
`focus` is what the read below replaces; the absent parameter is what keeps
a future reader from wiring panel-local state back in without noticing they
are recreating it.

### `const REGION_OBJECT`

Published only on the frames it draws, so its ABSENCE is evidence that
the panel is not describing a selection. That is the distinction the row is
about, and without a region a driven check could only observe the document
section being present — which it always was.

### `fn the_four_commissioned_facts_are_all_reported`

> the read-only facts (winding rule, node count, embedded-font
> status, exact geometry) that belong beside the Objects panel's
> inventory rather than in a ribbon band

Three of them are on a path and the fourth is on text, so the check
takes two objects. This is the panel's acceptance criterion, and it
is the one a later refactor is most likely to erode a field at a
time.

### `fn an_absent_property_is_omitted_and_an_unstated_one_says_so`

The distinction the panel's whole legibility argument rests on. A
path has no font — the row must not exist. An object with no finite
bounds *has* a position the file does not state — the row must exist
and say so, or the operator is left to notice an absence.

### `fn a_no_paint_path_reports_no_colour_at_all`

Its fill colour is default black and appears nowhere on the page.
Printing it would be a confidently wrong answer about a real,
addressable object — the exact class of error the panel exists to
avoid.

### `fn a_stroked_path_reports_the_colour_a_viewer_sees`

The one-description rule at work: the resolution happens once, in
`describe_object`, so this panel and the Objects row cannot name
different colours for one object.

### `fn the_embedded_font_join_declines_when_the_name_is_not_a_key`

The join is by `/BaseFont`, and a name is not a key. Two dictionaries
with one name need not agree about embedding, so pdfcer declines —
because a confidently wrong "Yes" is indistinguishable from a right
one, which makes this the one field on the panel that could mislead
silently.

Driven through a real inventory rather than a hand-built one, so the
`/BaseFont` values are the ones a document actually produces.

### `fn no_field_is_ever_rendered_blank`

A blank value is indistinguishable from a field pdfcer forgot to fill
in, and the panel's whole value is that its silences are as legible
as its numbers. Swept across several object kinds so a kind-specific
arm cannot slip through.

### `fn a_text_object_always_has_something_to_disclose`

Text is always approximate, so it always carries at least the
bounds-basis note. This pins the panel's most-used disclosure path:
if it ever came out empty, the heading would draw over nothing.

### `mod annotdelete`

`pub` rather than private, unlike [`dimension`] and [`markup`], for two
reasons that are both about a single derivation being reachable from
elsewhere: `crate::panels::PanelsState` holds its `(id, epoch)`-stamped
memo, and `crate::app::conditions` calls its `gate` to publish
`selection.delete_permitted` — the condition that decides whether
`format.delete` is drawn at all. One question, two consumers; see its header.

### `mod face`

`pub(crate)` reach rather than private, because the ribbon is not under
`crate::panels` and must draw the identical body: *"a face offered in one
surface and not the other"* is a divergence this project has found more than
once, and a disclosure added to one copy and not the other would be worse
than either.

Its header carries the two things this module could not have invented: the
evidence that `EditSession::format_text` performs the standard-14 resource
write itself, and why the fourteen are offered without being
coverage-tested first.

### `mod mkcolour`

Its header carries the reason it is hand-built rather than
`ui.color_edit_button_srgb`: `egui`'s own colour button marks itself changed
on **every frame of a drag inside the picker**, so a caller acting on
`.changed()` authors one undo entry per frame.
One `/MK` colour key as a labelled swatch, shared by the properties pane
and the placement dialog. `O202` asks for colour before AND after
placement, and its header carries why one module answers both.

### `mod refusedchar`

> *"if the character isn't available in a pdf are we able to change to a
> different font?"*

Yes, and each piece is separately ordinary: the engine refuses by name and
hands back `Refusal::character`, [`face`] offers the fourteen standard
faces, and `set_font` authors a resource the page does not carry. **What
connects the refusal to the chooser is this section**, and nothing else
does.

`pub` for [`text`]'s reason: `crate::panels::PanelsState` holds its state,
because the face list costs a provenance extraction and a pre-flight.

### `mod text`

`pub` rather than private, like [`geometry`] and unlike [`markup`], because
`crate::panels::PanelsState` holds its draft: the read-back costs a
provenance extraction, so it is stamped and kept rather than re-taken every
frame.

### `mod textobject`

Every text colour control in the program was gated on a swept range, and
clicking text selects the *object*, so the swatch he went looking for was
greyed with no guessable way to un-grey it. This section acts on the object,
through the same `Action::TextStyle` a sweep raises, with the operand
derived by **byte-span containment** rather than by any geometric inference
— see its header, and `crate::canvas::textedit::pin::object_text`.

`pub` for [`text`]'s reason: `crate::panels::PanelsState` holds its draft,
because the read-back costs a provenance extraction.

### `mod tool`

`OPERATOR_REQUESTS.md` O123: *"I never understood why there is a tool dock
when everything can be in object and properties."* They are properties of
what is about to be drawn, and this is the panel that owns that category.
`pub` rather than private because `block_for` is the shipped decision a
driven check and a unit test both assert against.

### `fn font_embedded`

A pure function over the inventory so it can be tested without a frame,
and so the disclosure rule it implements — *never pick when the join is
ambiguous* — is visible as an assertion rather than as a comment.

**Zero matches is [`FontEmbedded::Unknown`], not [`FontEmbedded::No`].**
"This font is not embedded" is a claim about a font dictionary pdfcer
found; "no font dictionary answers to this name" is a claim about pdfcer's
own inventory, and the Fonts panel already states which surfaces that
inventory does not cover. Reporting the second as the first would turn a
coverage gap into a statement about the operator's document.

### `fn body`

## Every section is scoped to a selection, and that is the whole rule

| section | subject | when |
|---|---|---|
| [`dimension`] | the **ce dimension** selected on the canvas | only while one is |
| [`markup`] / [`annotdelete`] | the **annotation** selected on the canvas | only while one is |
| [`formfield`] / [`fieldedit`] / [`widgetedit`] | the **form field** clicked on the page | only while one is |
| [`text`] / [`textobject`] / [`paint`] / [`geometry`] | the **content** swept or clicked | only while something is |
| [`object_section`] | the **page object** the canvas selection names | only while one is |

**The row that would break that pattern is the file's own title, author,
subject and keywords**, which have no selection to be scoped to and so draw
with no condition of any kind — on screen every frame under everything else.
They are [`crate::panels::docprops`], with a tab of their own. See this
module's header for the rule that leaves behind.

## Why `object_section` is a function rather than inlined

Because it has two early returns — no selection, and a selection naming an
object that has gone — and an early return written straight into `body`
would skip everything after it. Nothing follows it today, so the shape costs
one function and buys the guarantee back the moment a section is appended.

### `fn property_rows`

A pure function, and that is the point: it is where every "is this fact
present?" decision lives, so every one of them is testable without a
frame. The drawing code above does nothing but lay these out.

**A field is omitted when the object has no such property, and present
with [`crate::text::panels::properties::value_not_stated`] when it has
one the file does not state.** Those are different situations and the
panel distinguishes them: a path has no font at all (omit), while an
object with no finite geometry *has* a position that the file does not
give (say so). A blank row is never produced, because a blank is
indistinguishable from a field pdfcer forgot to fill in — and this panel's
whole value is that its silences are as legible as its numbers.
