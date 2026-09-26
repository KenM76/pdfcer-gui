# `app::markupband` — the five Format ▸ Markup controls the ribbon cannot
draw itself

`RIBBON_IA.md` §5.8's *Markup annotation* row, and the operator's ask of
2026-09-06: *"getting full editing working for the Markup tools."*

## What this module is

The Markup group has five controls and **not one of them is a button**: two
colours that must also *show* the mark's current one, a width the operator
drags, a percentage, and a four-way choice of arrowheads.
`egui_shell::manifest::Item::Custom` is the extension point for exactly that
— it hands the application a `Ui` and gets out of the way — and this module
is what goes in that `Ui`.

[`crate::app::fontband`] is the precedent and this file is deliberately its
twin: same shape, same four obligations, same park-and-report contract. Read
that module's header first; what is below is what differs, and every
difference is argued rather than inherited.

## The shell reserves the slot; everything else is ours, including the
greying

`egui_shell::ribbon::control::render_command` does four things for a command
item: it evaluates `enable`, it draws the control greyed when the predicate
is false, it shows the tooltip through `on_hover_text` **or**
`on_disabled_hover_text`, and it publishes the control's rect under
`ribbon.item.<id>`. For a custom item it does **none** of them — it cannot,
because it does not know what is being drawn.

So all four are done here, in the same shapes and under the same names. The
rect matters as much as the greying: it is published under
`egui_shell::ribbon::report::band_item(id)`, the same name the shell builds
for a command control, so a driven check finds the fill swatch the way it
finds a Delete button. A second naming scheme for *the same kind of thing,
drawn by the other half of the program* is how a harness comes to have two
lookup paths, which is the defect `driving::declared_or_in_overflow` was
written to end.

## It reports; it does not dispatch

Every control here parks a [`MarkupEdit`] and returns the command's
`HandlerToken`. It raises no `Action` and touches no document.

That is `egui-shell`'s contract — *"the shell reports, the application
dispatches"* — and R8's dispatch choke point besides: a capability that
edits the document is a registered command, and a registered command is
invoked through `PdfcerApp::dispatch_command`, which is the same point a
chord and a context-menu row reach.

⚠ **`panels::properties::markup` pushes an `Action` straight into the queue
and this must not copy it.** That is the panel's established path and
`app::surfaces` states the rule at the ribbon's own custom-item closure: the
three Font controls *"DO return a token, where the pen's swatch above does
not, and the difference is what the control acts on."* A ribbon control that
built its own `Action` would put the operand derivation — *which annotation,
on which page* — in the renderer, where a chord never reaches it, and the
copy in `app::dispatch::format` would be the one that went stale.

## One field per raised action, never a struct reassembled from widgets

[`MarkupEdit`] carries exactly one property and [`MarkupEdit::into_style`]
sets exactly one field of `MarkupStyle`. `MarkupStyle`'s own doc comment is
the rule and the reason:

> Every field is `None` by default … That shape is deliberate: a Format tab
> whose colour picker also had to restate the current width would overwrite
> whatever the operator had set from the other control.

The failure that prevents is specific and this surface is where it would
have happened: five controls drawn from one annotation, one of them stale by
a frame, and a colour change that silently reverts a width set a moment
earlier. `panels::properties::markup` §"Every control is `None` unless the
operator touched it" argues it at length; this module obeys the same rule
through a type rather than through care.

## Absence and greying, and why this group's answers differ from Font's

| state | Font group | Markup group |
|---|---|---|
| the mode cannot author | **absent** (`mode.edit_content`) | **absent** (`selection.markup_restylable`) |
| no operand | **greyed**, and the tooltip says how to get one | **absent** — same condition |
| the operand refuses | the run is CMYK: greyed swatch, own sentence | the mark is **locked**: greyed, own sentence |

The middle row is the whole difference. A greyed Font control is the one
surface in this application that can say *sweep with the Text tool first*
(O37); a greyed Markup control could only say *select a mark*, which the
operator has already done or the contextual tab would not be on screen. R9
then requires absence, and `manifest::format`'s `MARKUP_VISIBLE_WHEN`
carries the argument in full.

⇒ The bottom row is what greying is left for, and it is R9's textbook case:
§12.5.3 Table 165 bit 8 is a fact about **this annotation** rather than about
the build or the mode — click a different mark and the controls work. The
sentence is `text::panels::properties::markup_locked`, which is the string
the Properties panel shows, so the two surfaces cannot refuse for different
reasons.

## WHICH subtype takes WHICH property is the ENGINE's question, and this
module no longer holds an answer to it


> *"That list is the engine's to know. The first subtype that gains or loses
> a border is the day our copy is wrong and nothing tells us."*

⇒ The engine answers it. **`pdfcer_core::edit::MarkupStyleSupport`** carries
`takes_border`, `takes_interior` and `takes_endings`, and
`MarkupStyleSupport::for_subtype` derives them from a `/Subtype`; that
sentence is quoted into the type's own doc comment as its justification.
[`Current::support`] is the answer, asked once per frame off the
annotation's `/Subtype`; the three controls read it and **nothing here
re-derives it**.

### ⚠ The distinction that must not be collapsed

| question | whose | where it is answered |
|---|---|---|
| *"does this subtype take a border width?"* | the **engine's** | [`Current::support`] |
| *"what IS this mark's border width?"* | this module's | the `MarkupSpec` arm |
| *"which anchors do I paint for this shape?"* | this module's | `canvas::annotnodes` |

The middle row is why the `match` on `MarkupSpec` survives: only
`MarkupSpec::Square` has a `border_width` field, so reading the **value**
out of an arm is something the compiler checks and something the engine has
no API for. What is gone is the arm deciding whether the control exists.
`canvas::annotnodes`' header draws the third distinction and is right; this
module does not touch it.

### Belt and braces — a predicate to ask, and a refusal if you ask anyway

The engine also refuses: `set_markup_style` raises
`EditError::StylePropertyNotApplicable { id, subtype, property }` **before**
anything is regenerated, so a width sent to a highlight leaves the file
untouched. Both mechanisms are in use
here and neither replaces the other: the predicate shapes the UI so the
refusal is unreachable, and the refusal is what makes a drifted shell loud
instead of silent.

Surfacing it costs this module nothing, and that is by design rather than
by omission: `app::actions::funnel::vector_edit`'s `Err` arm
already routes every `EditError` to the decline channel —
`crate::text::status::edit_declined_by_engine` on screen, the engine's own
sentence into `PDFCER_DIAG` — and `check-ui-strings.sh`'s exclusion 3 says
in as many words that an error's `Display` is *"not permission to route UI
text through an error type"*. So the refusal arrives; it arrives through the
one channel every refusal uses, and this module does not build a second.

## The genuine fifth state of the arrowhead chooser

`MarkupStyle::endings` became `Option<StyleEdit<(LineEnding, LineEnding)>>`
on the same day: `Set` writes `/LE`, **`Clear` removes it**. The four
positions this chooser offers all *write* the key, so *"no arrowheads"* —
`/LE [/None /None]` — and *"no line-ending entry at all"* are two different
files that draw the same line, and only the first was reachable.

### It is an ACTION below a separator, not a fifth peer — and here is why

A fifth position in the list would be a fifth answer to the question the
list asks (*which ends carry a head?*) that **draws identically to the
first**. Two entries a drafter cannot tell apart by looking is the one thing
this control cannot afford: the mistake is undiscoverable on screen and
shows up in a byte comparison of a drawing that has already gone out. It
would also make the combo's `selected_text` ambiguous, because a mark with
no `/LE` and a mark with `/LE [/None /None]` would both have a claim on it.

⇒ So the four positions stay four, and the removal sits under a separator as
what it is — an act, not a state. Three consequences fall out of that shape
and each of them is the reason:

- **It costs no band width.** `ribbon::plan::CUSTOM_ITEM_WIDTH` is 96 and
  this group is already sized to fit inside it; a button beside the combo
  would have spent the budget on the rarest control in the group.
- **It is ABSENT when there is no `/LE` to remove**, which is the rule
  [`fill`]'s Clear and `panels::properties::markup`'s three Clears already
  obey: *a Clear beside a mark that has nothing to clear is a control whose
  only possible effect is an undo entry the operator did not earn.* That is
  what [`Current::endings_key_present`] is for, and it is read off the
  dictionary rather than off the spec, because `spec_from_dict` supplies
  Table 176's default and so cannot tell an absent key from a written one.
- **It is where the operator already is.** They opened the chooser to change
  their mind about arrowheads; the way back to the file they were given is
  in the same popup, not on a second control they must go looking for.

The wording is a drafter's and lives in the catalog, once, read by **both**
surfaces: `text::panels::properties::markup_endings_clear` and its hover.
One string, so the tab and the panel cannot come to describe one act two
ways.

## Rule 15 — a ce dimension is a different verb and must not appear here

**ce dimensions** are the ones pdfcer authors; **pdf dimensions** are CAD
page content. A ce dimension is `panels::properties::dimension`'s and uses
`set_dimension_style`; handing one to `set_markup_style` regenerates it as a
bare line with its label and witness lines gone, and the engine refuses it
by name.

The guard is a `match` on `AnnotKind` that the **compiler** checks, in two
places that cannot disagree: [`resolved`] here and `app::conditions`'
publication of the condition. It is deliberately not a comparison of
`/Subtype` strings, because a ce dimension's `/Subtype` **is** `/Line`,
exactly like an arrow's — a string test would restyle the operator's
dimensions into bare lines and would look correct while doing it.

## Why this module reads the session itself rather than sharing the
panel's reader

[`Current::read`] is a twenty-line dictionary read that
`panels::properties::markup::Current::read` also performs. Duplicating it is
the deliberate choice: that function is private to its module, five tracks
were writing in this tree on the day this landed, and making it
`pub(crate)` would have been an edit to another author's file to save
twenty lines.

⚠ **Corrected 2026-09-06.** This paragraph used to end: *"The two are
allowed to differ, and one of them does — this one also reads `/LE` and the
interior, which the panel offers no control for."* That was true when it was
written and stopped being true the same day: `panels::properties::markup`
gained `fill_row` and `endings_row` in the session that added this band, so
**both surfaces now read all five terms**. The permission still stands — the
two readers are allowed to differ — but the example of a difference is gone,
and a header that keeps a stale example teaches the next reader a fact about
the panel that is no longer so.

⚠ What is **not** duplicated is the derivation that matters: both read
through `annot_author::spec_from_dict`, the author's view, which is what
`set_markup_style` reads when it plans. One derivation of *what this mark
currently is*, two readers of it.

## Rule 4 / R8b — fuzzy, never sneaky

Nothing here marks the canvas and nothing here can. The restyled mark
renders exactly as the saved file will render it; what the engine could not
reproduce is raised by `app::actions::annots` into the status bar, and what
a wider pen does to the annotation's `/Rect` is disclosed in this group's
tooltips and in the Properties panel's standing note.
