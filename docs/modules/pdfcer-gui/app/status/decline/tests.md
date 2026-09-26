# `pdfcer-gui/app/status/decline/tests`

## Item notes

### `fn a_decline_lives_exactly_as_long_as_its_reason`

The full matrix, both directions on every variant. The "still true"
direction is the one worth stating explicitly: a decline whose reason
still holds must survive, or the sentence would flicker off on the next
frame and the operator would never read it.

### `fn a_history_decline_is_retired_by_its_own_stack`

The cross terms are the reason this is a separate test rather than four
more lines in the matrix above. A build whose two arms read the same
field — the mistake a two-field struct makes available, and the reason
[`History`]'s doc comment argues for it over two loose booleans — would
pass every same-stack assertion and fail only here.

The remedy arriving *without a command* is the whole point: authoring a
rectangle is a canvas gesture that reaches no dispatcher, so [`retire`]
never runs and only this filter can end the sentence. That is
[`Declined::NothingToFrame`]'s property, and it is why both of these
have a live predicate at all rather than [`Declined::SaveFailed`]'s
unconditional `true`.

### `fn the_two_history_declines_do_not_share_a_slot_or_a_sentence`

[`record_history_empty`] takes the value rather than a `bool`, and the
property that buys is asserted here: pressing `Ctrl+Y` with an empty
redo stack must not leave the bar saying the document has no changes.
The ordering half is [`Declined::SaveFailed`]'s, already pinned above —
both record in the apply phase, after the frame's `retire`.

### `fn a_failed_save_is_recorded_and_retired_by_the_next_command`

The store half of [`Declined::SaveFailed`], and the ordering is the
interesting part: [`retire`] runs at the top of `dispatch_command` while
[`record_save_failure`] runs in the **apply** phase of the same frame,
which is later. A sentence recorded by a save therefore survives the
dispatch that raised it, and is cleared by the *next* command — which is
what makes a second `Ctrl+S` record a second sentence rather than
re-showing the first.

Reversing those two would be silent: the bar would simply never draw the
line, and a reader of the trace would still see `save-copy-failed`.

### `fn a_partial_grant_is_not_a_decline`

The one case this module is deliberately blind to. A region zoom past
the page's raster ceiling still zooms, still centres what was asked
for, and raises `Action::ZoomTo` carrying the clamped scale — so the
bar's own zoom readout states the truth on the same frame. Wording it
would word a non-event.

Asserted for the clamped case *and* the exact one, because a store that
happened to reject only the exact case would pass a test written the
obvious way and still ship the sentence nobody wants.

### `fn no_two_declines_share_a_sentence`

Three now rather than two, and asserted pairwise: the operator gets one
line, and "nothing is selected", "the page is still drawing" and "the
copy was not written" have three different remedies. A shared sentence
would be a decline that does not say which command declined.

### `fn a_decline_can_be_raised_again_after_the_operator_moves_on`

This is the property an edit-epoch key **cannot** express, and the
reason this module has a store of its own: a decline changes no
document, so an epoch-keyed sentence would be identical on both presses
and would never retire in between. Here the sequence
*decline → the operator does something else → decline again* puts the
sentence back, which is what makes the second press an answer rather
than a swallowed keystroke.

### `fn the_dispatcher_words_a_decline_and_the_next_command_retires_it`

Driven through `PdfcerApp::dispatch_command`, which is the same entry
point a ribbon click, a quick-access click and a keyboard chord all
reach — so what is asserted is the real routing rather than a
hand-assembled approximation of it.

Three steps, and the middle one is the point of the whole module:


   `view.zoom_actual` rather than an id no token names: `retire()` runs
   *above* the `match`, so an unimplemented id reaches the catch-all and
   the assertion passes either way. A test whose subject is "any other
   **command**" must name one that exists, or it quietly asserts
   something weaker than it says.

### `fn a_worded_decline_does_not_change_the_bar_height`

# Why this needs its own test beside the edit-disclosure one

Same rule, different arrival, and this arrival is the awkward one. The
edit disclosure follows a drag; this follows a **keyboard chord**, and
a chord is precisely the gesture where the operator is looking at the
page rather than at their hands. If this line grew the bar, an active
`FitMode` would recompute its zoom from a viewport one row smaller on
the next frame, and the page would visibly shrink in response to a
command that **did nothing at all**. R128's measured symptom is *"the
page jumped when I clicked an object"*; this variant would read as
*"the page moved when the command was refused"*, and it would be
investigated in the zoom code, where nothing is wrong.

# The three assertions, and why none of them is the obvious one

1. **A measurement happened at all** (`Some(_)`, never `None`) — assert
   that the measurement HAPPENED, not only its value. `cargo test -p
   egui-shell` and `cargo test --workspace` compile `egui` with different
   features (no fonts vs `default_fonts`), so a layout assertion can be
   entirely vacuous under one of the two commands a developer runs.
2. **The sentence reached the painter** — more shapes with the decline
   live than without it. Without this, assertion 3 is satisfied just as
   well by a [`show`] that returned early and drew nothing, which is
   true and proves nothing.
3. **The height did not move.** Asserted as `Some(true)` rather than
   with a bare `assert!`, so a run in which either frame failed to
   measure reads as `None` and fails, rather than reading as agreement.

[`Declined::NothingToFrame`] is the case tested because it is the one
an operator will actually reach, and because its sentence is the longer
of the two — the defence against a long sentence is eliding inside a
bounded sub-region with the whole text on hover, never wrapping,
because wrapping is how a one-row bar becomes a two-row bar.

### `fn a_clipboard_verb_the_mode_refuses_is_worded_and_then_retired`

Both halves matter and the first is the one that closes the defect.
`app::modes::capability::offers_command` does not refuse the clipboard
chords by tab, so `app::dispatch::clipboard`'s two mode gates are the path
an operator in Read or Review actually walks. A chord refused at the gate
at least traces `chord-not-offered`; a gate that is a bare `return` traces
nothing on any surface, which is a quieter defect than the one it
replaces.

Recorded through [`record_mode_refusal`] rather than by writing `LAST`
directly, so this exercises the same function the dispatcher calls.

### `fn a_mode_refusal_reads_like_no_other_decline`

`no_two_declines_share_a_sentence` above makes this claim for four
variants and cannot reach these, because they carry a payload the
catalog words. The cross-family half is what this adds: a mode refusal
that read like `EditRefused`'s *"that change was refused"* would tell
the operator the engine said no, when what said no is a control two
inches away that they can move.

### `fn a_paste_the_mode_refuses_reaches_the_bar_through_the_dispatcher`

`a_clipboard_verb_the_mode_refuses_is_worded_and_then_retired` above
proves the store works; it would pass unchanged on a build where
`app::dispatch::clipboard` never called it, which is a shippable state.
This drives `dispatch_command` for real, so deleting the
`record_mode_refusal` call in that module fails **here** and names it.

Read with an empty clipboard is the operand, and it is the cheapest
honest one: `dispatch::clipboard`'s paste gate sends an empty clipboard
down the **markup** branch on purpose — *"the refusal an operator gets in
Read is the mode's rather than 'nothing has been copied', which would be
true and useless"* — so `PasteMarkup` is the sentence that must arrive,
and asserting the variant rather than merely `is_some()` is what catches a
gate that refused for the wrong reason.

It does **not** cover the content branch, which needs a real clip on a
real OS clipboard. `a_paste_review_may_not_do_says_so` owns that, drives
it in Review, and has not been run — said here so the gap is stated rather
than implied by this test's confidence.

### `fn the_custom_stamp_declines_say_two_different_things`

The comparison against [`Declined::EditRefused`] is the one that matters,
because that floor is what these two stand in front of: without them a
stamp whose collection has moved reaches the operator as *"that change was
refused"* — true, and useless. An edit that paraphrased the floor here
would undo the feature while leaving it reading as though it were still
there.

And against each other, because they are the pair most at risk: both are
about a stamp that is not where it was, both end by telling him to reopen
the window, and the whole reason there are two is that one means the FILE is
gone and the other means the file was REWRITTEN.

### `fn a_refused_stamp_is_not_reported_as_an_edit`

R8b rule 4's honesty clause applied to prose rather than to pixels: the
gesture was a drag on the page, nothing was placed, and the drawing is
exactly as it was. A sentence beginning *"About your last edit"* — which is
where `record_note` puts things, and where these two lived for an afternoon
— would be a confident small lie.

Asserted on the CHANNEL rather than on the words. Checking that the string
avoids the phrase "last edit" would pass on a rewrite that said "your stamp
was added but"; checking that the decline slot holds it proves it renders
under `⊗`, which is the thing that is actually true.
