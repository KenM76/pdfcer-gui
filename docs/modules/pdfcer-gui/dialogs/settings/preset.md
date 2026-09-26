# `dialogs::settings::preset` — a named vector of answers


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

## ★★★ Where the values came from, and why every one of them is graded

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

★★ Three consequences the engine had to spell out, all of which shape this
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
