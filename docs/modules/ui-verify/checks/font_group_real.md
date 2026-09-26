# `ui-verify/checks/font_group_real`

`the_font_controls_are_live_on_the_drawing_you_open` — **O198 claim 3 and
claim 4, measured on the operator's own file instead of on a fixture.**

# The report this exists to answer

`OPERATOR_REQUESTS.md` **O198**, in his words:

> *"Also get the font selector and editing tools like [bold] and italic
> working. That entire area is always greyed out in the menu, and the
> properties area is uneditable too. This is true even when I add a new line
> of text."*

Two claims, about two surfaces, in one sentence:

| claim | surface | what must be true |
|---|---|---|
| 3 | the ribbon's Format ▸ Font group | all five controls drawn **pressable** |
| 4 | the Properties panel | a font editor drawn, with its face row **inside the panel's clip** |

**And he said it about a drawing, not about a fixture.** The document is
`SW41177.pdf` — a SolidWorks export, 36 sheets, 1.8 MB, whose page 0 carries
5,899 paths against 4 text objects and whose labels are 5 pt. Everything
that makes that file hard is absent from `fixtures/paragraph.pdf`: the font
is subset, the text is a title block rather than a paragraph, the page is
1584 × 1224 pt rather than 612 × 792, and the objects the harness must hit
are two screen pixels tall at fit zoom.


Driven at `--doc-point 0,1140,62` against a build carrying the uncommitted
`panels::properties::tool::Slot` change. Every assertion held:

* the click selected exactly one object and it is text;
* the Properties panel drew a font editor **with its face row inside the
  clip** — claim 4 does not reproduce;
* all five Font controls were drawn pressable —
  `format.font: enabled=1 live=1, format.font_size: enabled=1 live=1,
  format.bold: enabled=1 live=?, format.italic: enabled=1 live=?,
  format.font_colour: enabled=1 live=1` — claim 3 does not reproduce.

`live=?` is not a hedge and not a missing measurement. Bold and Italic are
ordinary ribbon toggles with a single predicate, so they publish no `live=`
field at all, and [`super::font_group::describe`] prints `?` rather than
inventing a `1`. The three that DO carry a second predicate all reported
`live=1`, which is the reading that matters: the renderer's own read-back
resolved a face on a subset SolidWorks font.

**What this does NOT say is that the operator was wrong.** He is running a
published build that predates the `Slot` fix, and on that build the editor
drew below the fold of a Properties pane opening on three always-on switches.
This check green and his report accurate are the same state of the world one
release apart, and the next release is what closes the gap. Keep this check
aimed at his file for exactly that reason: it is the thing that will notice
if a later panel section takes the top of the pane again.

# THE TWIN, AND WHY THERE ARE TWO CHECKS AND NOT ONE

[`crate::checks::font_group`] asserts the same two surfaces and **pins its
fixture**: it opens `fixtures/paragraph.pdf` at a measured point and reads
`--pdf` and `--doc-point` only to ignore them. That pinning is correct for
what it is for — its subject is a *discoverability route*, and a route has to
be asserted on a page whose contents are known, or a red result is a question
about the aim rather than an answer about the program.

This check is the opposite half, deliberately:

| | `font_group` | `font_group_real` |
|---|---|---|
| document | pinned to a committed fixture | whatever `--pdf` names |
| aim | pinned to a measured point | whatever `--doc-point` names |
| question | *is the route there at all* | *does it survive HIS file* |
| a red result means | the program regressed | the program regressed **or** this file breaks it |

The second row of that last cell is the whole reason to have both. A
measurement taken only on a fixture answers *"the feature exists"*, which was
never the operator's question: the feature existed, was green, and he could
not use it. A measurement taken only on his drawing cannot tell a regression
from an aim, which is the mistake that cost this project a day on
2026-08-28. Two checks, two claims, and a reader who compares them gets the
diagnosis for free — green here and red there is a fixture problem, red here
and green there is something about real drawings.

# What is shared, and why it is shared rather than copied

The command list, the region names, the aim guard and the enablement
renderer all come from `font_group` as `pub(super)` items. Copying them would
have been the ordinary move and this repository has already paid for that
move nine times in private `click_tab` helpers and eleven in private
`workspace_root` helpers: a duplicated list diverges in silence, and a group
measured against a stale copy of its own membership still reports a measured
group.

# The oracle

1. The click left **exactly one object selected and it is text** — read from
   `properties-panel … kind=` and `canvas-selection … sel=`. Anything else is
   a **SKIP**, never a failure: it is a statement about where the harness
   aimed. See [`super::font_group::aimed_at_one_text_object`].
2. `properties.text` and `properties.text.face` are declared — claim 4.
3. All five `format.*` commands report `enabled=1` and, where they have a
   second predicate, `live=1` — claim 3.

**Step 2 asks `clipped_away` before it reports an absence**, and that is
not defensive coding — it is the actual defect O198 claim 4 turned out to be.
The font editor drew perfectly and drew *below the fold* of a Properties pane
whose first section was three always-on preference switches, so
`diag::ui_rect_visible` withheld the rect and the panel looked empty from the
trace. A check that reported that as *"the panel drew no editor"* would send
a reader to the renderer, which is not where the fix was.

# What this check does NOT do, said so nobody looks for it

It does not press Bold and it does not assert that a restyle reached the
document. That is `font_group`'s phase 2 and `restyle_text`'s whole subject,
both on the fixture, and repeating it here would make this check's red mean
four things instead of two. The question here is **reachability**: can the
operator, on his own drawing, get to the controls at all. Whether pressing
them works is a different question with its own check, and O198's sentence is
about the first one — *"that entire area is always greyed out"* is a
complaint about a surface, not about an outcome.
