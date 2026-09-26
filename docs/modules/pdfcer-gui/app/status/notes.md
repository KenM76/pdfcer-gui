# `app::status::notes` — the narrator, demoted behind a disclosure triangle


> The left half carries four things, and only the first is the narrator.
> The others look similar and are governed by different rules.

- Everything left in the parent answers *"how is the bar laid out, and what
  does each group show?"* — a fixed row, two rule-4 disclosure lines, a
  worded decline, and two clusters of stateless controls.
- Everything here answers *"what did the renderer have to compromise on,
  and how is that said in one line?"* — which is a different question with
  its own widget state (an open/closed flag in `egui::Memory`), its own
  pure decision function ([`notes_line`]), and its own editorial rule about
  which of the renderer's counters an operator can act on.

## Why this line is *narration* and the three below it are not

`DEFECTS.md`'s "Not defects" table records the old shell opening with a
substitute-glyph census:

> The first thing a user reads is the app talking about itself. Excellent
> information, wrong prominence — put it behind the disclosure triangle
> that is already there.

So the report is complete, still here, and **closed by default**. The words
"Render notes" stay visible so it is discoverable; only the report itself
is one click away.

That prominence argument is exactly what does **not** apply to the lines
beside it. A rule-4 disclosure and a worded decline are facts about the
operator's own document and their own gesture, and a disclosure the
operator has to *open something* to find is a disclosure that did not
happen — the opposite failure. Hence: this one is demoted, those are not.

## Opening it does not make the bar taller (R128)

The parent's header carries the measurement — a content-driven status
panel takes space from the central panel, an active `FitMode` recomputes
its zoom from the canvas viewport every frame, and the result on pdfcer was
a page that shrank 230 % → 224 % → 215 % across three frames with no zoom
input. This module is the half of that defence that lives in the widget:
the line is drawn **beside** the triangle, inside the parent's single
allocated row, elided at [`super::NOTES_WIDTH_FRACTION`] of the bar with
the whole text on hover.

It is also why there is no [`egui::CollapsingHeader`] anywhere in this
file. Changing its own height is that widget's entire behaviour, which is
the one thing this surface may not do.
[`super::tests::the_bar_is_exactly_as_tall_open_as_closed`] pins it, from
the parent, where the whole bar can be measured at once.

## Item notes

### `type NoteEntry`

A named type rather than an inline tuple so [`findings`]' table reads
as a table. The `fn(usize) -> String` half is a plain function pointer
rather than a closure on purpose: it names a *catalog entry*, so the
pairing is a lookup that can be read down the page, and a reviewer
checking that every reported field has a sentence has one place to look.

### `fn notes_line`

The **join and the empty case**, and nothing else. Which findings exist, in
what order, and which two counters are excluded are all [`findings`]' — see
there. This function's whole subject is that the bar gets *one* line
(**R128**) and that an empty one would be indistinguishable from a
disclosure that failed to fill itself.

### `fn annotations_not_drawn`

## Why this is computed here rather than read off one counter

The engine reports two numbers and refuses to combine them, for a reason it
states in its own doc comment: `annotations_without_ap` is a **fact about
the file** — these annotations really do carry no appearance stream, and
the map is the demand signal for the remaining appearance-generation work —
while `annotations_icon_painted` is a fact about **what the operator saw**,
counting the ones `Pass 289.0` nevertheless drew from pdfcer's own
standard-icon artwork. *"Folding them together would make one of the two
numbers a lie."*

This surface wants the second question and only the second question: *how
much of this page's markup is invisible to the man looking at it?* So the
subtraction happens **at the consumer**, once, with the argument written
down — which is the shape this project keeps arriving at whenever an engine
deliberately hands out the parts instead of the sum.

⇒ The subtraction is sound because the engine increments them at the same
site: `Appearance::None` counts into the map unconditionally and then adds
to `annotations_icon_painted` only if the icon was actually drawn, so the
painted set is a **subset** of the appearance-less set on every page.

## ⚠ The one case where the subtraction would lie, and how it is stopped

A **narrowed annotation scope**. The census is taken under every scope so
that a suppressed render still discloses what it is not showing, but the
icon painter only runs for annotations in scope — so under a narrowed scope
the subtraction would say *"not drawn"* about content that was **withheld
on request**. Those are opposite facts, and the engine's own row is
emphatic that *"withheld"* must stay distinguishable from *"tried and
failed"*.

[`findings`] therefore drops this entry entirely when
`annotations_out_of_scope` is non-zero, rather than reporting a smaller
number. Today no screen render in this shell narrows the scope — the only
`AnnotationScope` this crate sets lives in the **print** dialog
(`app::prefs::printing`), and printing does not feed this bar — so the
guard is dormant. It is written anyway because the day somebody adds a
*View ▸ Display ▸ Comments* toggle is the day this sentence would quietly
start accusing the file of something the operator did on purpose.

### `fn a_page_whose_resources_were_supplied_says_so_first`

Two assertions in one test because the ordering is the substance, not
the presentation. §7.8.3 and the ISO 32000-2 erratum let a form
XObject, a Type 3 font and an **annotation appearance stream** inherit
the page's `/Resources` — so a page with no dictionary of its own can
genuinely be the reason a font below it went missing. A findings list
that printed *"text from 1 font not drawn"* above *"this page names no
resources"* would send the operator looking for a font that was never
the problem.

### `fn a_page_that_names_its_own_resources_is_not_mentioned`

Without this, the assertion above would pass on a build that printed
the line unconditionally — which would put a permanent, meaningless
sentence in the status bar of every document the operator opens.

### `fn appearance_less_annotations_are_counted_as_not_drawn`

Two `/Square` marks with no `/AP`, nothing rescued by the icon painter.
Both are invisible on the page, and clean paper is what an unannotated
drawing looks like, so nothing about the render tells him to go looking.

### `fn an_icon_pdfcer_painted_is_not_reported_as_missing`

This is the whole reason the subtraction lives in this crate. The
engine counts an appearance-less annotation into `annotations_without_ap`
**and then** into `annotations_icon_painted` if it drew the standard icon
anyway (`Pass 289.0`), and refuses to fold them because the map is a fact
about the file while the counter is a fact about what the operator saw.
A surface that read the map alone would tell him three comments are
missing while two of them are on the screen in front of him.

### `fn a_page_whose_icons_were_all_painted_reports_clean`

The complement of the test above, and the one that stops this finding
becoming the nagging R8b rule 4 forbids: a page whose sticky notes
pdfcer painted from its own artwork has nothing withheld from the
operator, so the line must be the clean one.

### `fn a_narrowed_scope_withholds_the_finding_instead_of_miscounting_it`

⚠ The one way the subtraction could lie. The census is taken under every
scope so a suppressed render still discloses what it is not showing, but
the icon painter only runs in scope — so under a narrowed scope the
subtraction would report *"not drawn"* about content **withheld on
request**. Those are opposite facts and the engine's own row insists they
stay distinguishable.

Dormant today: the only `AnnotationScope` this crate sets is the print
dialog's, and printing does not feed this bar. Pinned anyway, because the
day somebody adds a *View ▸ Display ▸ Comments* toggle is the day this
sentence would start accusing the file of something the operator did.

### `fn an_impossible_engine_pair_saturates_rather_than_panicking`

`annotations_icon_painted` is incremented only inside the arm that has
just incremented the map, so it can never exceed the sum — today. That is
the ENGINE's invariant, not this crate's, and it reaches here across a
pinned git dependency that moves several times a week. `saturating_sub`
costs nothing and turns a future arithmetic overflow in a status bar into
the honest answer zero.

### `fn tolerated_and_compat_skipped_are_not_reported`

A tolerated structural oddity was absorbed and drawn correctly, and a
`BX`/`EX` skip is the file telling readers to skip it (§7.8.2 Table
32). Reporting either would put reassurance in front of the findings
that need reading.

### `const NOTES_OPEN_ID`

`pub(super)` because the parent's R128 test drives the flag directly — it
measures the bar open and closed, and it can only do that by writing the
same key this module reads.

### `fn show`

Drawn only when a page has actually been rasterized: the notes describe a
raster, and `page_texture` is `None` only before the first render and
after a failure the canvas already reports in words.

**Opening this does not make the bar taller.** The line is drawn beside
the triangle, inside the same row, elided at [`NOTES_WIDTH_FRACTION`] of
the bar with the whole text on hover. See the R128 section of this
module's header for why that is a requirement rather than a layout
preference.

### `fn findings`

Empty is a real answer and means the page drew clean. Callers word that
themselves, because the bar and the dialog have different room for it.

# What is reported, and what is deliberately not

Every field here changes **what the operator can see on the page**: text
that was not drawn, images that were not drawn, glyphs whose shapes are
not the document's, layers that were hidden, content the file does not
actually contain. Those are facts an operator can act on — supply a font,
turn a layer back on, go and find the missing stream.

`Diagnostics::tolerated` and `Diagnostics::compat_skipped` are **not**
reported. Both count divergences that leave the picture correct: a
tolerated structural oddity (an unbalanced `Q`, a mid-path `cm`) is
something the renderer absorbed and drew right anyway, and a `BX`/`EX`
skip is spec-sanctioned (§7.8.2 Table 32) — the file is *telling* readers
to skip it. Listing them **here** would put two numbers that mean "nothing
is wrong" in front of the seven that mean something is.

They are not lost: the Render-diagnostics dialog shows them, separately
and with a sentence saying they are not faults
([`crate::text::diagnostics::absorbed`]). That is the distinction this
exclusion has always rested on — *"this is a status bar, not a report"* —
finally having a report to be distinguished from.

# Order

Most consequential first: content the file is missing, then whole
surfaces that were not drawn, then glyph-level substitution, then the
operator's own hidden layers, then operators pdfcer has not implemented. A
line that opens with "3 unrecognised drawing operators" and buries "text
from 2 fonts not drawn" is sorted by the renderer's interest rather than
by the reader's. The dialog lists them top-down in the same order, for the
same reason.
