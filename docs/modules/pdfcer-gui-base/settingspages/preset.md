# `settingspages::preset` — a named vector of answers


> *"I'd like a preset setting for rendering things to what the [print
> conformance suite] page needs to render correctly … since it is for
> conformance to PDF/X-4 (ISO 15930-7) … maybe we should have a dropdown to
> select view options between the different standards."*

## What a preset IS, and what it deliberately is not

A preset is a **named bundle of settings that already exist**, applied in one
click, and **individually editable afterwards**. It is not a mode, not a
second rendering path, and not a lock. Choosing one writes values into the
draft exactly as though the operator had set each control by hand, and the
window then behaves as it always did.

That is the whole design, and it is what makes the feature cheap and safe:
there is no new state to persist, no new thing that can disagree with
`settings.txt`, and no way for a preset to express something the individual
controls cannot. A preset that could would be a second source of truth.

## Where the values came from, and why every one of them is graded

Not from here. A control labelled *ISO 15930-7* carries that standard's
authority whether or not it was meant to, so the vector was **asked for**
and the engine answered with an API rather than a table
(`settings::presets`, Pass 128.1). Its reply quoted this project's own
reason back at it, and then made the sharper point:

> *"The interesting column is not the value. It is how much weight the
> value can bear, and for most of these axes the answer is less than the
> button implies."*

Only **one** of PDF/X-4's six answers is a claim about the standard at all,
and it is graded `implied` rather than `sourced`. So the row shows the
grading beside the choice; a row that showed the name and hid the grade
would be the over-claim this whole request was careful to avoid.

Three consequences the engine had to spell out, all of which shape this
file:

1. **Not every standard binds a renderer.** PDF/A and PDF/UA both put
   *"operational details of rendering"* outside their own scope. `PdfUa1`
   exists and correctly **sets nothing** — surfaced as an answer, not
   hidden as an omission, because *"nothing, and here is the measurement"*
   cannot be mistaken for unfinished work.
2. **A third of the grid is axes a standard does not reach.** No PDF/X part
   contains a shading clause, so none of them says anything about mesh
   padding. `PresetAction::LeaveAlone` is a real state and is rendered as
   one — blank would read as missing data, a value would assert a
   requirement that does not exist.
3. **`cmyk_intent` has no conformant value.** Every PDF/X level guarantees a
   *colorimetric* definition of device colour, and `CmykIntent` selects
   among fixed built-in tables, which is not one. The preset takes the
   least-wrong value and **discloses that the file's own output intent was
   not applied** — mandatory under rule 4, because a colour transform that
   did not happen leaves nothing on screen to notice.

## What DOES ship today, and why it is not a consolation prize

**pdfcer's own recommended answers.** The operator's report that opened this
request included *"touching some of our presets caused some test to show up
as failed"* — which is a person who has changed several settings while
investigating and now wants a way back. That is a real need, it is
answerable today with complete authority (these are *our* defaults; no
external standard is being spoken for), and it is the half of the request
that was never blocked.

## Item notes

### `fn apply_pdfcer`

# It is `Settings::default()`, and getting here took two corrections



The second is the more instructive mistake. A doc comment asserting a
difference that does not exist is worse than no comment: it invites the next
reader to preserve a line that does nothing, and it makes a *deliberate*
override indistinguishable from a copied one. **Read the value, not the
prose about the value.**

So this assigns nothing. Every answer is the engine's, taken by *not
assigning it*, so a default the engine changes tomorrow arrives here for
free and cannot rot into a stale literal.

### `fn live_choice`

The operator's own choice while it still describes the working settings,
and otherwise the derived reading. See [`row`]'s comment for why the
order is that way round and not the other.

### `fn still_holds`

Asked against the preset rather than remembered as a flag, because the
operator can change any control in the window after choosing a preset and
nothing tells this module when they do. A flag would need every other
control to remember to clear it — the shape of a guard that is correct until
somebody adds a widget.

### `fn identical_siblings`

It is disclosed because the operator's reason for wanting the control was
*"especially PDF/X-4 … to see how far we are along with matching the
[conformance suite's] tests"*, and switching to it will change nothing on screen. Discovering that
by staring at an unchanged page costs an hour and reads as the setting being
broken. Saying it costs a line.

Computed rather than written down, so the day a standard's answers diverge
the sentence corrects itself instead of becoming a stale claim — which is
this file's own recorded lesson: *read the value, not the prose about the
value.*

### `fn same`

Compares the render-radius fields explicitly rather than deriving
`PartialEq` on `Settings`, and the reason is not tidiness. `Settings` also
carries `theme` — the *program's* appearance — and the two write-radius
entries that change bytes on disk. A preset is about how a **document
renders**; an operator who picks a dark theme has not stopped using pdfcer's
recommended rendering, and a comparison that said otherwise would clear the
radio for a reason that has nothing to do with rendering.

### `fn detail`

Indented under its radio, muted, and **only for the selected one** — the
full grid for nine standards at once would be a wall of text nobody reads,
and the operator only needs the caveats for the answer they have chosen.

### `fn operator_title`

The engine names these `mesh_patch_padding`, `image_minify` and so on —
field identifiers, correct for an API and wrong for a window. Every one of
them already has a title in [`crate::text::settings`], written by the
symptom that would send somebody looking for it, and this is the one place
the two vocabularies meet.

A `PresetKey` the engine adds later falls through to its own name rather
than to a placeholder: an unfamiliar identifier is ugly and honest, whereas
a guessed title would be a sentence pdfcer never wrote.

### `fn a_pdf_x_preset_says_a_composite_viewer_will_differ_and_pdf_a_does_not`

This test began life asserting the output of a nine-line workaround in
this window, because `disclosures()` emitted a `why` only for keys a
preset LEAVES ALONE. The engine shipped the same rule the same day; the
workaround is gone and this now points at the real thing.

What it checks is deliberately the *content* that makes this a rule-4
disclosure — that another viewer will show these areas differently — and
not merely that some sentence mentioning spots appeared. The two values
render visibly differently, and without that sentence an operator
comparing pdfcer with Acrobat sees pdfcer being wrong on a page where it is
being deliberately more correct.

PDF/A is the counter-case in the same test, because the engine's sourced
reply was explicit that PDF/A does not reach this axis: its Scope clause
excludes the operational details of rendering, in every part from 2005 to
2020. Showing the sentence there would assert a requirement that does not
exist.

### `fn a_best_effort_value_is_counted_and_not_spelled_out`

The distinction the engine adopted, in one sentence: *a count is an
honest summary of a judgement and not of a claim*. So a value pdfcer
CHOSE where the standard is silent is reported as a number, and a value
pdfcer says the standard IMPLIES must show its citation, because that is
something an operator may want to check.

Pinned because the obvious "improvement" is to print every `why`, and
that turns a disclosure into a wall of prose nobody reads — six entries
times ten standards. `image_minify` is best-effort on every PDF/X and
PDF/A preset and is the stable example to test against.

### `fn applying_a_choice_leaves_settings_that_match_an_equivalent_one`

Because **two standards can mean the same thing to pdfcer**, and two
of them do: applying `pdf-x3` leaves settings that `matching` reports as
`pdf-x1a`. That is not a defect in either — it is a fact about the
domain. PDF/X-1a and PDF/X-3 differ in what colour spaces a *file* may
contain, and pdfcer's render-radius settings cannot see that difference,
so the two produce an identical vector.

The first version of this test asserted the stronger property and failed
on the second standard it tried. Weakening it was the right response
rather than adding state to remember which button was pressed: the radio
reflects **what the settings are**, not what was last clicked, and a
remembered choice could disagree with `settings.txt` — which is the one
thing this feature's design set out to make impossible.

What must hold is the round trip that the radio actually depends on: the
settings after applying a choice are the settings of *whatever* choice
is reported, so the selection shown is never a lie about the values.

### `fn some_standards_are_indistinguishable_from_the_settings_alone`

If this ever fails — because pdfcer gains a setting that distinguishes
them — the test above can be strengthened back, and this is the note
that says so.

### `fn the_recommended_preset_is_the_engines_defaults`

Its predecessor asserted the opposite and failed twice in one evening,
which is the whole reason this one is worded as it is. See
[`super::apply_pdfcer`] for what those two failures taught.

If a real divergence is ever added, this fails — and that failure is the
prompt to state the reason in `apply_pdfcer`'s doc comment *before*
amending the assertion, so an override always arrives with its
justification attached rather than as a bare line somebody later deletes
as redundant.

### `fn every_key_a_standard_leaves_alone_has_an_operator_facing_title`

`mesh_patch_padding` is correct for an API and wrong for a window. The
fallback arm exists so a key the engine adds later degrades to an
unfamiliar identifier — ugly and honest — rather than to a guessed
title, which would be a sentence pdfcer never wrote appearing under a
standard's name.

This fails when the engine adds a `PresetKey`, which is the point: the
failure is the prompt to write a title for it.

### `fn the_weight_tally_excludes_what_the_standard_does_not_reach`

`NotApplicable` is deliberately excluded from the tally and named in
the left-alone list instead. *"Does not apply"* is a different kind of
fact from *"we chose"*, and adding them together is the arithmetic that
would make a preset look better sourced than it is.

### `fn every_preset_in_the_list_can_actually_be_selected`

Drives the real rule — click a radio, then ask what the control would
show — for all ten choices. Before the fix, eight of them answered
`pdf-x1a` and one answered `pdfcer`, because `matching` returns the
FIRST choice whose settings equal the current ones and all eight
conformance presets apply byte-identical settings.

It asserts the property the operator cares about — *can I choose
this?* — rather than the mechanism. A test that asserted
`chosen_preset == Some(id)` would pass against a version that stored the
choice and still drew the dot somewhere else.

### `fn choosing_any_standard_after_another_leaves_something_to_save`

> *"When I go to settings and select some of the standards the save
> button is greyed out and I can't save the change."* — 2026-08-26

Both halves were true and the second explains the first. `is_dirty`
compares values, and [`identical_siblings`] measures that **all eight
PDF/X and PDF/A presets apply byte-identical render settings** — so
choosing a second one moved nothing and Save was correctly greyed about
a draft that really did equal what was saved.

The test is written over **every pair** of choices rather than the two
that were reported, because the reported pair is not special: any two of
the eight reproduce it. Picking two would have passed the day a ninth
standard arrived and collided with a tenth.

### `fn a_chosen_standard_is_what_the_window_shows_when_it_reopens`

The half the operator had not seen yet. Before this was persisted the
window showed whichever standard `matching` found first, so choosing
PDF/X-4 and coming back read PDF/X-1a — the window contradicting the
operator about what they had asked for.

Asserted for **every** standard, since the failure is invisible for
whichever one happens to sort first.

### `fn a_hand_edited_setting_retires_the_stored_choice_as_well_as_the_shown_one`

The guard that stops persistence turning into a lie. `live_choice`
already declines to show it; this asserts the same of
[`still_chosen`], which is what `commit` asks before writing — because
a window that stopped claiming a standard while the file went on naming
it would reopen claiming it again.

### `fn an_unknown_stored_standard_falls_back_rather_than_failing`

Reachable two ways: a hand-edited preferences file, and a file written
by a newer pdfcer that knows a standard this build does not. Neither is a
fault in the file, so neither may become a parse note — the window falls
back to the derived reading, which is the honest answer for a choice it
cannot offer.

### `fn changing_a_setting_by_hand_retires_the_chosen_preset`

The other half of the fix, and the reason the choice is filtered through
`still_holds` rather than simply believed. Without this a window could
read *"PDF/X-4"* over settings that are nobody's.

### `fn the_conformance_presets_give_the_same_render_answers_today`

This is not asserting that they SHOULD be identical. It records what is
true of the engine this build links, so that if a standard's answers
ever diverge, this test fails and whoever reads it learns that the
sentence under the radio has become interesting rather than routine.

It is also the falsification for the test above: with the presets all
distinct, `every_preset_in_the_list_can_actually_be_selected` would pass
against the OLD code, and would have proved nothing.

### `enum Choice`

Two kinds, and the distinction is the whole model:

* [`Choice::Recommended`] — pdfcer's own shipped answers. **We** are the
  authority, so it can say what it does without qualification.
* [`Choice::Standard`] — a published standard's answers, from
  `pdfcer_core::settings::presets`. **We are not the authority**, so every
  value carries the engine's own evidence grade and the row shows it.

### `fn choices`

pdfcer's own answers first, then every standard the engine knows about.
The list is *derived* from `RenderStandard::all()` rather than restated, so
a standard the engine adds appears here with no change at all — R8's
registration rule, reached through the crate boundary instead of through a
command registry.

### `fn resolve_id`

The seam between `Prefs`'s owned `String` and the `&'static str` every other
reader here compares against. Resolving through [`choices`] rather than
leaking the string is what keeps an unknown id — hand-edited, or written by
a newer pdfcer that knows a standard this one does not — from travelling any
further than this function. It is not an error: the window simply falls back
to the derived reading, which is the honest answer for a choice this build
cannot offer.

### `fn still_chosen`

[`live_choice`]'s own filter, exposed so the display rule and the stored
value cannot drift apart. Without one function answering for both, the
window would stop showing a standard while the preferences file went on
naming it, and the next session would open claiming a choice the values
contradict.

`true` when nothing is chosen: there is no claim to retire.

### `fn matching`

Returns `None` for "none of them", which is the **normal** state once an
operator has adjusted anything, and is not a fault. The control shows no
selection rather than pretending the nearest one is chosen — a radio that
claimed "pdfcer recommended" over settings that are not pdfcer's recommended
answers would be lying about the thing it exists to report.

### `fn row`

Above the groups rather than inside one, because it acts on all of them —
`widgets::group`'s convention is that a group holds settings sharing a
*subject*, and a preset shares a *purpose*.

## What is shown BESIDE the choice, and why it is not decoration

The engine's reply that supplied these values spent most of its length on
one point: *"the interesting column is not the value, it is how much weight
the value can bear."* Only one of PDF/X-4's six answers is a claim about the
standard at all, and that one is `implied` rather than `sourced`. A row that
showed the name and hid the grading would be exactly the over-claim the
whole request was careful to avoid.

So a selected standard shows:

* **its disclosures**, verbatim from the engine. These are not advisory —
  `cmyk_intent` has no conformant value at all, so choosing a PDF/X preset
  means a colour transform did *not* happen, and rule 4 requires saying so
  because nothing on screen would reveal it.
* **what it leaves alone**, named. Roughly a third of the grid is axes a
  standard does not reach — no PDF/X part contains a shading clause, so none
  of them says anything about mesh padding. Showing those as blank would
  read as missing data; showing them as values would assert a requirement
  that does not exist.
