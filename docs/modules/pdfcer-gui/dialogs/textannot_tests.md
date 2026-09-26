# `pdfcer-gui/dialogs/textannot_tests`

## Item notes

### `fn on_screen`

Two fields of `RawInput::default()` are wrong for driving this dialog,
and each cost a debugging round when it was left alone.

**`screen_rect` is `None`.** This dialog sizes itself from
`content_rect` — `420.min(width - 40)` — so a default input hands
`egui::Window` a degenerate size and the field inside it a width nothing
can be focused in. A test that lays out differently from the application
is measuring a different program.

**`time` is `None`, and egui then fills it from the wall clock.** That
makes frame timing depend on how loaded the machine is, so a test that
drives several frames is reproducible when run alone and intermittent
when run beside a thousand others — which is precisely the flake that
gets re-run until it is green and then believed. Time is supplied here,
one 60 Hz tick per frame, so the sequence is the same every time.

### `fn the_note_window_does_not_open_in_the_corner`

The whole of the defect in one assertion. `dialogs/textannot.rs`
computed this position and then wrote `let _ = pos;`, so the dialog was
placed by `Host`'s corner inset instead — on every open, dozens of times
in a markup session, because a dialog dismissed that often almost never
has a remembered position to restore.

The position is asserted as a **relationship** rather than as two
numbers: centred across the window and between a fifth and half of the
way down it. Pinning the exact pixels would fail the next time the
window's size changed for an unrelated reason, which is how a test stops
being read and starts being edited.

### `fn each_kinds_window_is_as_tall_as_its_body_needs`

**The text-box assertion is the positive control** and it is what
makes this a test at all. Asserting only *"the sticky and the stamp are
taller"* passes on a `window_size` that had gone taller for **every**
kind — the change would be invisible, the text box would grow a strip of
empty window, and nothing here would say so.

⚠ **This test was renamed on 2026-09-10, and the old name is the
lesson.** It was `only_the_sticky_notes_window_grew_for_its_chooser`,
and it asserted `stamp.y == WINDOW_PTS.y` with the message *"the stamp
has no chooser and must not have grown"*. That sentence was true when it
was written and became false the moment the stamp got one. A test whose
**name and message state a property the program no longer has** is worse
than no test: it reads as a measurement, and the next person to grep for
*"which kinds have choosers?"* finds an answer rather than a question.


Until that day this test asserted `sticky.y > stamp.y`, with the reason
*"seven radio rows and a disclosure is a taller addition than a heading,
one combo row and a disclosure"* and the standing instruction that an
inversion means somebody changed one constant without reading the
other's argument. **The argument was read, and the premise was wrong** —
it compared the two ADDITIONS while ignoring that the additions sit on
different bodies.


| body | what it holds | measured |
|---|---|---|
| text box | the four-line field | `54 … 118` — 64 pt |
| stamp | seven gallery radios, heading, disclosure | `12 … 278` |
| stamp | … **and** the size heading, combo and disclosure | `360 … 440` — 80 pt |

The sticky holds the field **and** a seven-radio gallery. The stamp holds
a seven-radio gallery **and** the size section, and no field. So the
comparison reduces to 80 pt of size section against 64 pt of text field,
both measured — and the stamp is the taller window.

⇒ The assertion is therefore `stamp.y >= sticky.y` and the operator is
`>=` on purpose: the two constants are currently equal at 190 pt, the
derivation only supports a 16 pt difference, and an assertion tighter
than its evidence is the defect this whole doc block is about.

### `fn the_chosen_icon_reaches_the_commit_action`

The whole placement half of `Pass 253.2` in one assertion: before this,
every sticky note pdfcer ever authored carried `/Note` because the field
did not exist.

The `stamp` assertion beside it is the **positive control for the
route**, not decoration. `StampName` already travelled this exact path,
so asserting the two together is what says the icon was added *to* a
working carrier rather than replacing one — and if a later edit dropped
either field out of the `Action::CommitTextAnnot` literal, the surviving
assertion would still be about a live route.

### `fn a_fresh_dialog_is_empty_and_defaulted`

**The size assertion is the load-bearing one**, and it is not merely
completeness. `StampSize`'s own header argues that a fresh gallery must
offer the size DERIVED from the drawn box — the behaviour of every build
before engine `Pass 287.0` — rather than the engine's flat 12 pt,
because adopting the engine default would have shrunk every stamp on the
operator's drawings as a side effect of a fix he asked for. That
argument is only enforced if something asserts the value.

### `fn the_page_and_rect_are_captured_at_open`

The property that stops a page change under an open window redirecting
the annotation. Asserted on the stored values because there is nothing
else to assert it on — the whole point is that nothing re-reads them.

### `fn readiness_follows_the_gallery_rule`

The readiness rule, which is the gallery exception stated once more at
the control that depends on it. A stamp whose Accept required typing
could never be authored; a callout whose Accept did not would author an
empty box.

### `fn typing_into_the_open_window_reaches_the_draft`

Every test above asserts on the struct's fields, which is exactly the
blind spot `DEFECTS.md` D1 was: they all pass on a build whose window
accepts no keystrokes, because none of them ever draws one. This drives
a real `egui::Context` through two frames — one to build the field and
take its one-shot focus, one carrying a real `Event::Text` — and asserts
the words arrived.

### `fn focus_stolen_on_the_opening_frame_is_taken_back`

The defect this replaced latched on having *asked* for focus rather than
on holding it, so a request that lost its frame was never retried and
the field sat there looking typeable while every keystroke went
elsewhere. That is unreachable in a bare `egui::Context` — the request
always wins when nothing competes — which is why the test above passed
on the broken build and why this one takes the focus away by hand.

The theft models what the real frame does: the dialog's first draw is
the frame AFTER the gesture that opened it, so the pointer release that
finished the drag is still being resolved around the request.

### `fn the_focus_retry_gives_up_so_another_control_can_hold_it`

The objection the original one-shot latch was written to answer, and it
is still correct: a field that asks for focus every frame takes it back
from whatever the operator clicked, and a window that cannot be
dismissed is worse than one that cannot be typed into.

The competitor is a **real drawn button**, not a bare `Id`. egui drops
focus for an id no widget registered that frame, so focusing an invented
id proves nothing about who won — it only proves egui tidied up.

### `fn a_custom_stamp`

The values are his: `Signatures` is the category his own collection
declares and `Ken` is one of the two stamps in it. Using the real shape
rather than `foo`/`bar` costs nothing and means a failure message names
something a reader can go and look at.

### `fn exactly_one_of_the_two_galleries_holds_the_selection`

This is the assertion that stands between the operator and the
worst bug this feature could have had — picking `Approved` and getting
his signature, silently, on the second click of a session. `radio_value`
would have shipped it: it writes one variable and knows nothing about
the other, and the commit path reads `custom` first.

Both directions, deliberately. A one-way test passes on a
[`TextAnnotDialog::select_standard`] that forgets to clear, or on a
[`TextAnnotDialog::select_custom`] that forgets to set — and the
project's own standing lesson is that a suite trying one sign is not
testing the value.

### `fn the_operators_own_stamp_reaches_the_commit_action`

Asserted alongside `stamp` still carrying a NON-default value, for
the positive-control reason `the_chosen_icon_reaches_the_commit_action`
gives: if a later edit dropped `custom` out of the action literal, the
surviving assertion is still about a live route.

### `fn only_the_stamp_kind_scans_the_stamps_folder`

A sticky note and a text box cannot reach a gallery, so opening one
must not walk Acrobat's stamps folder. This asserts the *observable*
consequence — an empty library — rather than counting `Document::load`
calls, because the count is the mechanism and the emptiness is the
contract.

⚠ It is NOT an assertion that the machine has no stamps. On a machine
with none, all three kinds produce an empty library and this test is
vacuous — a real limitation, written down rather than papered over. The
falsifying case is the operator's own machine, where the stamp kind
finds two and this test would go red if the guard were removed.

### `fn the_custom_half_appears_only_when_there_is_something_in_it`

An unavailable capability renders nothing — no empty group, no heading
over an empty list, no *"you can add your own"* invitation on a machine
that has never made a stamp.

The second half is what stops this being vacuous. An
absence-assertion alone passes on a `custom_stamps` that returns early
unconditionally — which is to say, on a build where the whole feature
is missing. The populated case is the control: it fails if the function
never draws, and the empty case fails if it always does.

### `fn the_stamp_window_grows_for_the_operators_own_stamps`

`custom_stamp_reaches_the_page` armed Markup ▸ Stamp, dragged a box, and
captured a dialog that showed the seven standard stamps and the Add/Cancel
row with **none** of his three stamps on it. They were laid out below the
scrolled body's fold, at content y 298, 326 and 354, inside a body that
ended at 270 — published as rectangles, invisible as controls, and the
harness clicked the first of them into the dialog's own drop shadow.

Nothing was clipped wrongly and nothing was laid out wrongly.
[`window_size`] added a per-kind constant written for a body that did not
yet contain this section, and **a guessed size is a claim about the content
that the content can outgrow**.

The two negative controls are what make this a test rather than an
observation. Asserting only *"the stamp window got taller"* passes on a
`window_size` that ignored its new argument and grew unconditionally, and
passes on one that applied the growth to every kind — the sticky note and
the text box would each gain a strip of empty window, and no assertion here
would say so.

### `fn the_custom_half_asks_for_more_room_up_to_a_stated_limit`

Both halves matter and neither is obvious from the other. Growth is the
feature; the cap is what stops a forty-stamp collection opening a dialog
the full height of the application window, standing over the drawing being
annotated. Past the cap the body scrolls, which is what a scroll area is
for.
