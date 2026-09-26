# `pdfcer-gui/find/bar/tests`

## Item notes

### `fn a_page_with_no_text_at_all_offers_recognition`

The one combination that offers OCR, and the operator's actual rule
stated as a case: there is nothing on this page for any search to have
matched, so the empty result is a fact about the *document*.

### `fn an_ordinary_empty_result_on_a_text_page_offers_nothing`

This is the assertion the whole feature turns on, and it is the one a
plausible wrong implementation fails. Offering OCR on any zero-hit
search is one character simpler to write, passes
[`Self::a_page_with_no_text_at_all_offers_recognition`] perfectly, and
would put *"this page has no text on it"* under every mistyped part
number on a drawing full of text.

The operator named this trap in the specification rather than leaving it
to be discovered: *"the trigger is 'this document is images', NOT 'this
search had no matches'"*, and `FEATURES.md` records that the two "must
not be collapsed."

### `fn the_page_is_not_extracted_unless_the_search_found_nothing`

The short-circuit, asserted rather than assumed — and it is a
correctness property, not an optimisation. `page_has_extractable_text`
costs one page extraction on a cache miss, and the bar draws on every
frame it is open; a version that evaluated the closure first would put
that extraction on the frame budget while looking identical in every
other test here. That is a whole defect class: the right work, charged at
the wrong moment, invisible to a suite that only asks whether it happened.

### `fn the_offer_raises_the_registered_recognise_command`

An id written at a call site and nowhere else goes stale in silence, and
the symptom here is a button that traces `command-unimplemented` and does
nothing.

### `fn the_overlay_is_pinned_inside_the_hosts_top_right_corner`

The distinction is invisible until a dock is open, and then it is the
whole difference between a find bar over the page and a find bar over
the Objects panel. Asserted against a host rect deliberately offset
from the origin, so an implementation that forgot `host.right()` or
`host.top()` and used the screen's still fails.

### `fn a_narrow_host_still_yields_a_pivot_on_the_canvas`

Reachable: the canvas viewport shrinks with every dock the operator
opens, and `MIN_WINDOW_SIZE` is 640 points wide before any of them.
egui's `constrain_to` is what pulls the *box* back in that case; what
is asserted here is that it is not being handed a nonsense point to
start from.

### `fn enter_searches_when_there_is_no_current_answer_and_steps_when_there_is`

The whole of the bar's key behaviour, asserted without a frame. The
interesting rows are `Stale` — which must search rather than step,
because stepping through geometry the module has already declared
untrustworthy is exactly what the staleness rule exists to prevent —
and `Empty`, which must do nothing.

### `fn enter_does_not_re_run_a_search_that_found_nothing`

A search is a whole-document text extraction — 350 ms on the benchmark
drawing, measured. Re-running one that just matched nothing, once per
keypress, is how a viewer becomes unusable on the files it exists for
— and it would produce the same answer, because if anything had changed
the readout would be `Stale`.

### `fn the_word_rule_chooser_appears_only_with_whole_word`

P3, applied to the one control on this surface whose availability is
conditional. Driven through a real `Ui` so what is asserted is what the
menu actually builds, and counted by *widgets that were laid out*
rather than by reading the branch — a test that read the branch would
pass on a build where the `if` had been inverted and the label moved.

### `fn painted_text`

Read off the **painted shapes**, not off the source. A test that
asserted `t::find_zoom()` equals `"Zoom"` would prove the catalog agrees
with itself and would pass on a build where the checkbox was never added
to the menu at all — which is the defect worth catching, because a
control the operator cannot see is the same as a control that does not
exist (**R9**).

### `fn the_options_menu_offers_a_control_named_zoom`

The operator's request named the control: *"add a checkbox option to our
search bar called zoom"*. A request that carries a name is a request for
that name — he will look for that word — so the word is asserted, not
just the presence of a fourth widget.

Painted rather than read from the catalog: see [`painted_text`].

### `fn toggling_zoom_leaves_the_search_options_alone`

The load-bearing property of the split in [`options`]: a `FindOptions`
change re-runs the search, and re-running a search because a *view*
preference moved would throw away the operator's place in the result
list. So the menu writes the Zoom value through a separate `&mut` and
must leave the options struct untouched, whichever way it is flipped.

### `fn every_glyph_the_find_bar_draws_has_a_glyph`

`⏴`, `⏵` and `×` are the entire visible text of three controls. A
codepoint egui's bundled fonts (Ubuntu-Light + NotoEmoji +
emoji-icon-font) cannot draw renders as a tofu box, which is defect
D2's shape — an invisible label — on a control the operator has to hit.

The status bar has the identical test, and the reason is concrete: `◀ ▶
▸ ▾` are **all four missing** from that font set, so a catalog written
with them renders as tofu on the two controls an operator touches most.
This test is not a duplicate of it —
it cannot see this file's strings, and this file cannot see the status
bar's.

### `fn the_box_is_the_same_size_whatever_the_readout_says`

The property the module docs argue for: the overlay is anchored by its
top-right corner, so a box that changed width would move the search
field the operator is typing into, and the ⏴ ⏵ buttons out from under
a pointer aimed between two clicks.

Four readouts, four very different strings, one size.

### `fn the_step_buttons_are_inert_until_there_is_something_to_step`

Driven through a real frame rather than by calling the arm, so what is
under test is the wiring — the failure this catches is a button that
draws, is enabled, and reports nothing — a shape a panel can ship in with
every unit test green, because no unit test touches the wiring.

### `fn typing_raises_no_search`

The cost rule, from the other end: a search is a whole-document text
extraction — 350 ms on the benchmark drawing — so a bar that raised one
per keystroke would spend 1.4 seconds of blocked UI thread on the word
`part`. This is what "never searches on a keystroke" means in a test.

### `fn searched`

Built by writing `super`'s private fields directly, which a child
module may do. The alternative — a constructor on `FindState` that only
tests call — would be a second way to assemble a result set, and the
currency key is exactly the thing that must have one.
