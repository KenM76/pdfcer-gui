# `ui-verify/checks/tab_navigation`

`tab_moves_between_form_fields` — **Tab pressed inside a form field goes to
the next field, not into the ribbon** — `OPERATOR_REQUESTS.md` O204.

# The operator's report

> *"when I press tab while in a form I end up tabbing through the menus
> instead of the form items. The tab should tab through whatever space I
> have clicked on (example if I have an object selected on the canvase it
> should tab through to the next object as expected, and if I've clicked on
> a form item it should tab forward and shift-tab backwards to the next
> one."*

# Why only a driven run can answer it

The mechanism lives in `eframe`'s `raw_input_hook`, which runs **before**
`Context::run` and therefore before one line of application `ui` code. A
unit test that calls the ring's own step function proves the ring steps; it
cannot prove that the Tab press ever reached the ring, because the thing
that would steal it — `Focus::begin_pass` latching a focus move out of
`RawInput.events` — only exists inside a real frame with real widgets. That
is the shape of this project's founding defect: a guard that was
*"analysis-confirmed, NOT empirically verified"*, with a unit test whose
bare context had no widget the condition could occur in.

So the oracle is the operator's sentence, driven: click a field, press Tab,
and assert the focus landed on **another field** rather than anywhere else.

# The three assertions, and why the claim needs all three

1. `tab-claim scope=field` — the hook took the press. Without this a
   passing run could mean egui's own focus walk happened to land somewhere
   plausible.
2. `tab-field … from=A to=B` with `to` differing from `from` — the ring
   actually moved. A ring of one traces `tab-field-alone` instead, which is
   correct behaviour for a one-field form and no evidence at all here.
3. The same, backwards, under Shift — because *"shift-tab backwards"* is
   half of what he asked for and a forward-only implementation satisfies
   the other half completely.

# It needs a document with at least three fillable text fields

Two is not enough: with a ring of two, *"Tab moved to the next stop"* and
*"Tab wrapped straight back"* produce the same `to=` and the check cannot
tell a working ring from one that only ever bounces. `fixtures/
three-text-fields.pdf` exists for this, and `the_three_field_fixture_offers
_three_clickable_text_boxes` is its guard — **not** one of the engine's own
form fixtures, none of which carries a text field with a drawn `/AP`, and
an `/AP`-less field is not drawn on the canvas at all.
