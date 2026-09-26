# `pdfcer-gui/shell/manifest/format`

The **Format** tab — contextual, appearing only while something is
selected.

`RIBBON_IA.md` §5.8. One group: Selection.

# What a contextual tab is, in this manifest

It lives in `contextual_tabs` rather than `tabs`, and it carries a
`visible_when` condition — `"selection.any"` — that the application
publishes each frame in its `egui_shell::commands::ConditionSet`. The
separation is not cosmetic: a **mode** names a fixed tab set, and a
contextual tab's whole nature is that its presence is decided by
application state rather than by configuration. `egui-shell` refuses a
mode that names one, by design.

It is therefore present in **all three modes** and in none of their tab
lists. Selecting a markup in Review mode shows the Format tab exactly
as selecting one in Edit mode does. That is correct: Review is the
stance in which you place and adjust *your own* markup, and a reviewer
who cannot recolour a cloud they just drew has been given half a tool.

# Why this tab is nearly empty, and why it ships anyway

§5.8 calls the contextual tab *"the single largest usability change
proposed here"* and then sets the build order:

> Build order: **panel first, tab second.** The panel is the harder
> half and the tab's contents are a subset of it, so building the tab
> first would mean writing the property editors twice.

Every property editor the tab is eventually made of — colour, fill,
line width, line style, opacity, arrowheads, note text, dimension
group, scale, precision, units, standard, witness lines, size,
position, crop, stroke, winding rule, node tools, font, spacing,
alignment — is therefore **N**, and under P3 absent. Twenty-four
entries in [`super::PLANNED`] come from this one section.


What follows is the header this file carried for nineteen days, kept
verbatim because **the shape of the mistake is the record**. It named two
blockers, both of which were true when they were measured, both of which
were gone the **next day**, and neither of which anything checked:

> **1. `EditSession` has no verb that modifies an annotation.** Grepping
> every public `pub fn` for annotation work returns `add_markup`,
> `add_text_annotation`, `delete_annotation`, `delete_redaction_mark` and
> two deletion predicates. **Add and delete, nothing between them.** So a
> markup's colour, width, fill, opacity, arrowheads and note text cannot be
> changed after it is placed — which is §5.8's entire markup row.
>
> **2. The canvas selection cannot address an annotation.**
> `canvas::selection::identity::Selection` is `page + object + subpath +
> node` — four integers naming a **paint-order index into page content**.
> … a markup or a dimension is not selectable at all, so even a perfect
> `set_markup_style` would have nothing to name.

**`EditSession::set_markup_style` shipped on 2026-08-18**, and
`canvas::selection::annot::AnnotTarget` — an `ObjId`, a page, an
`AnnotKind` and the `/F` lock bit — landed the same day.
`panels::properties::markup` has been calling the verb through
`Action::SetMarkupStyle` since 2026-08-19. Every sentence above was false
from the moment the panel it argues for existed, and this file went on
stating both as present-tense facts about the engine until the operator
asked for *"full editing working for the Markup tools"* and somebody read
the header.

⇒ **A blocker is a measurement with a date, not a property of the world.**
`tools/gates/check-stale-blockers.sh` exists for exactly this class and did
not catch it, because the claim was prose in a module header rather than a
row in a register — which is the general lesson: *a reason that outlives
what made it true costs the next reader the whole feature.* The register
entries were caught (`manifest::PLANNED` names each absent command and is
asserted in both directions); the paragraph was not.

Two clauses of the old header **stand**, and they are kept as live text
rather than quotation because they still decide things:

- **Delete-and-re-add is not a workaround and is deliberately not built.**
  Re-adding loses the annotation's object identity, and with it its `/NM`,
  its place in the page's `/Annots` order (so its z-order), and any reply
  thread hung off it as an `/IRT` target. A "change the colour" button that
  silently detaches a reviewer's replies is worse than no button. This is
  also what `set_markup_style` is *for*: `MarkupStyleChange::annot_id` is
  documented as unchanged, and that is the whole point of the verb.
- **A ce dimension is a different verb.** `set_dimension_style`,
  `set_group_style`, `set_group_scale` and `set_group_standard` are its,
  and handing one to `set_markup_style` regenerates it as a bare line with
  its label and witness lines gone — which the engine refuses by name.
  `AnnotKind` carries the distinction **in the type**, so the Markup
  group's guard is a `match` the compiler checks rather than a comparison of
  `/Subtype` strings. Rule 15.

# What the tab carries now

Three groups: **Font** (2026-08-27), **Markup** (2026-09-06) and
**Selection**. §5.8's build order — *"panel first, tab second … the tab's
contents are a subset of it"* — was followed for both of the first two, and
it is why each was one pass rather than two: the property editors were
written once, in `panels::properties`, and the band reads the same actions.

What is left of §5.8's table in [`super::PLANNED`] is now the rows with no
verb behind them — a note's text (`MarkupNote`, a different struct), the
dimension property editors, and the vector-object rows.


⇒ Corrected rather than deleted, because it is the **third** stale-blocker
correction in this one header. The two above it stood for nineteen days
each; this one stood for an afternoon. The rule the header already states is
unchanged — *a blocker is a measurement with a date* — and what this instance
adds is the interval: **no reading of a blocker is fresh enough to skip
re-checking.** The register half of it *was* caught automatically, by
`planned_commands_are_genuinely_absent` failing by name the moment the
command was registered, which is the difference between a blocker that is a
row in a table and a blocker that is prose in a header.

**Delete** is the row that appears in *every* selection type's list in
§5.8's table. An unarmed canvas already does modeless select-and-delete —
that is what the removal of the `Editing on` master toggle relies on — so a
Delete command on a surface that only appears when something is selected is
real, not a stub.

Shipping the tab with one command rather than deferring the tab
entirely is a deliberate choice and worth defending, because it looks
like exactly the placeholder P3 forbids and is not:

- The tab **appears on selection**, which is itself the affordance
  §5.8 credits it with. That behaviour is the feature, and it is
  testable and demonstrable now.
- The command in it **does something**.
- The alternative — no contextual tab until the property editors land —
  means the appear-on-selection behaviour, the mode interaction and the
  one-command-one-tab consequences all get their first exercise at the
  same moment as twenty-four new controls.

# The other two surfaces

§5.8 is explicit that the contextual tab and a persistent **properties
panel** both ship, and that they answer different questions: the tab
carries what a user changes *while working*, the panel carries
everything including read-only facts and the editable X/Y/W/H geometry.
A **context menu** carries the same commands again for the user who
right-clicks — currently there is not one anywhere in the application.
Neither is a manifest concern at this stage; both are recorded here so
that "Format is nearly empty" is not read as "Format is all there will
be".

## Item notes

### `const FONT_VISIBLE_WHEN`

**Visibility, not enablement**, and R9 is the whole of the reasoning:
*an unavailable capability renders nothing; greying is reserved for
temporarily unavailable and is always explained on hover.* Read and Review
do not have a mislaid ability to restyle text — they do not have the
ability — so the group is **absent** there. Inside Edit the same controls
grey on `selection.text`, because there the capability is present and only
the operand is missing.

One condition would not do both jobs. `selection.text` alone would draw an
enabled Bold in Read, where pressing it must be refused; `mode.edit_content`
alone would draw an enabled Bold in Edit with nothing swept, which is a
control that does nothing on almost every press — the exact placeholder
shape P3 forbids.

### `const MARKUP_VISIBLE_WHEN`

# Why it is ONE fused fact and not two conditions

The Font group above takes two — `mode.edit_content` for visibility and
`selection.text` for enablement — and the reason is O37: an operator meets
the Font controls **greyed**, because reaching the operand means pressing
`T` first and nothing on screen says so, and a greyed control they can hover
is the one surface that can tell them. The greying is the feature.

Nothing here is like that. The operand is *the mark you clicked*, and the
gesture that produces it is clicking the mark — which the operator has
already done, or the tab would not be on screen. So a greyed Markup group
would explain nothing an operator did not already know, and R9 is explicit
about what that leaves: *an unavailable capability renders nothing; greying
is reserved for temporarily unavailable and is always explained on hover.*
**Absent**, in both of the two states that make it unavailable:

| state | why absent rather than greyed |
|---|---|
| the selection is not a markup — a page object, a form field, a swept text range, or a **ce dimension** | these controls have no operand of the right *kind*, and `set_dimension_style` is the ce dimension's verb (Rule 15) |
| the mode cannot author markup — Read | not a mislaid ability; Read does not have it, and the mode selector is the disclosure |

Fusing them is what `selection.formattable` and
`selection.delete_permitted` already do, and for the stated reason:
`egui_shell::commands::Enable`'s grammar is one condition name with an
optional leading `!` — *"a grammar in a string is a parser and a parser is a
thing that has its own bugs"* — so an `A && B` predicate is published as a
**named fact** rather than assembled here. The name says which fact.

# What is deliberately NOT folded in: the lock

§12.5.3 Table 165 bit 8 says a locked annotation's properties *"shall not be
changed by the user interface"*, and the engine refuses `set_markup_style`
for one by name. That is a property of **this annotation** rather than of
this build or this mode — click a different mark and the controls work —
which is exactly the case R9 reserves greying for, and exactly the case
where making the controls vanish would read as pdfcer being unable to
restyle anything at all.

So the lock greys, with a sentence, and [`crate::app::markupband`] does that
itself: a custom item gets no greying from the shell, and the sentence has
to be the locked one rather than the command's tooltip. The Properties
panel's *This mark* section takes the same position with the same string
(`text::panels::properties::markup_locked`), which is what keeps the two
surfaces from refusing for different reasons.

Note it is **not** spelled `selection.markup`. That name would claim only
half of what is published and would read, at the two call sites, as though
Read could restyle a mark.

### `const WRITERS`

# Why this is a list and not a predicate

Because "does this command write" is not a property the manifest can
see. The manifest holds an id and a condition string; whether the arm
behind that id calls `EditSession` is a fact about
`app::dispatch::format`. A test that tried to derive the answer would
be re-implementing the dispatcher, and a hand-written list inside a
completeness test is exactly the shape this project has already been
bitten by — a new module invisible to the check built to find it.

So the list is stated, and its JOB is to fail loudly when the Format
tab grows an item it does not name. `every_writer_is_accounted_for`
below is the half that makes the list honest: it asserts that the tab's
full command set is exactly the writers plus the explicitly-declared
readers, so a new command lands in neither bucket and fails.

### `fn every_custom_control_on_this_tab_is_withheld_from_a_mode_that_cannot_author`

`items()` filters `Item::Command`, because that is the only variant
carrying an id — which means the two tests above walked past the three
Font controls from the day they landed and would have walked past the
five Markup controls the same way. All eight **write to the document**:
a face chooser rewrites a content stream, a colour swatch regenerates an
annotation's appearance. They are the exact population A18 was about.

⇒ The gap is the shape this file's own `WRITERS` note warns of — *"a
hand-written list inside a completeness test is exactly the shape this
project has already been bitten by"* — arriving through a **variant**
rather than through a missing row. So this asserts over custom items,
where the check is stronger than the command one and needs no list:
there is nothing on this tab that a custom item may legitimately do
without authoring, so *every* one of them must carry a mode-bearing
condition. A custom item with no `shown_when` at all fails.

Two conditions are accepted rather than one, and they are not
interchangeable: `mode.edit_content` gates the Font group (page content)
and `selection.markup_restylable` gates the Markup group (an
annotation), and `Capabilities` keeps `edit_content` and `author_markup`
as separate questions precisely so a reviewer may recolour a cloud in a
drawing they may not otherwise touch. A test that demanded one string
would have taken the working verb away from the mode that owns it.
