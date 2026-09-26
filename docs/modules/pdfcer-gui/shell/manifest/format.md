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
