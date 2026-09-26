# `pdfcer-gui/canvas/gesture/meaning/tests`

## Item notes

### `fn probe`

Named rather than spelled out at each call: eight fields of which six
are `None`/`false` in almost every row, and a literal repeated a dozen
times is where a `true` gets left behind after an edit.

### `fn the_caret_tool_clicks_for_a_caret_and_drags_for_a_box_on_edit_content_alone`

Three claims in one loop, and each fails against a different plausible
wrong implementation:

* `drag.is_none()` fails a build that gave the tool a `DragKind` "for
  symmetry" — which would put a rubber band on screen promising a
  gesture nothing implements;
* `click == edit_content` fails a build that copied the measure rung and
  left `author_measure` in it — which would arm the caret in Review and
  refuse it in Edit, i.e. exactly backwards;
* the zoom assertion fails a build in which the rung was placed *below*
  the armed-zoom branch, where a press would rubber-band a zoom region
  under an I-beam.

Over the whole capability lattice rather than the three shipped modes,
for the reason this module's other tests are: a mode is a manifest entry
and can be customized, and the rule is about the flags.
**This test asserted `drag.is_none()` until 2026-08-21**, and the
sentence it carried — *"a caret is placed, not dragged"* — was true of
the gesture and wrong about the tool.

The operator: *"I should be able to make it multi line."* Multi-line
needs a width to wrap against, because a PDF has no paragraph and each
visual line is its own show operator at its own position. A width is a
rectangle, and a rectangle is a drag.

So the tool now has **both**: a click places a caret for one line, a drag
draws a box for a paragraph. What is asserted here is that neither has
taken the other away, and that **both** answer to `edit_content` and to
nothing else — the half of this test that was always the point.

### `fn an_armed_markup_tool_outranks_the_grips_and_the_region_zoom`

Both rows matter and both are failure modes with teeth: a markup drag
classified as a `Resize` would be consumed and author nothing (a tool
that arms and does nothing over any selected object), and one classified
as a zoom marquee would zoom the page instead of drawing.

### `fn read_mode_gives_a_content_press_no_meaning_but_keeps_the_region_zoom`

This is the operator's ask (*"in read mode the document shouldn't allow
editing"*) at the point where it is decided. Every one of the four
content meanings is asserted, because they are four separate arms and
gating three of them would look exactly like gating all four right up
until someone dragged a grip.

**The bare press is no longer `NOTHING`, and that is the text-selection
row arriving.** It used to assert *"no marquee-select, and no selecting
click either"* against `PressMeaning::NOTHING`, which was the right
assertion while Read had no press meaning at all — and would be the wrong
one now, because it would pass on a build that had silently taken text
selection away again. What must remain true is the thing the operator
actually asked for: the press means **text**, never
[`DragKind::Marquee`], so nothing on the page can be selected as
*content*. That is asserted by naming the variant rather than by
asserting an absence.

The region-zoom row is the one that would be easy to get wrong in the
other direction: marquee-**zoom** is navigation, it is armed
deliberately, and refusing it would take a viewer feature away from the
viewing mode.

### `fn no_press_offers_both_a_text_sweep_and_a_content_marquee`

The exclusivity `canvas::textsel`'s header §3 rests on, asserted at the
point where a press is given its meaning rather than only at the
predicate that decides it. A build in which both were reachable would
have one primary button with two meanings and no rule to choose between
them — which is the ambiguity `CanvasTool::Text` exists to remove.


* with **Select**, the guarantee is the original one — exclusive *by
  construction*, because `takes_the_press` and `content_gesture` read the
  same flag in opposite senses, so exactly one of the two is offered;
* with **Text**, the guarantee is *by precedence* — both underlying facts
  can be true in Edit, and rung 2 decides. So the assertion there is not
  an exclusive-or but the stronger and more specific one: the drag is
  `TextSelect` and **never** a content meaning, in every mode.

Written as one test rather than two because the property is one property
— *one press, one meaning* — and splitting it would let a future reader
change the branch order and fix only the half that failed.

### `fn the_text_tool_sweeps_in_edit_and_retiring_it_gives_the_marquee_back`

`Capabilities::FULL` is Edit, whose primary drag is the content marquee.
Every content meaning must be absent, and the click must still be
reported — because three of the text gesture's four meanings are clicks
(double-click takes a word, triple-click a line, Shift+click extends, a
plain click clears), and a build that suppressed it would leave a sweep
that selects and no way to unselect.

The second half asserts the thing that must **not** have changed: with the
tool retired, the same mode's press is the marquee it always was. Without
it, a build that had simply deleted the mode gate would pass the first
half perfectly while having removed the only content-selection gesture the
product has.

### `fn a_region_zoom_outranks_the_text_tool_but_not_a_pen`

The two orderings around rung 2, asserted together because they point in
opposite directions and the reason is stated once, at the branch: markup
**authors**, so the loss of its drag is a mark that was never made, while
a text sweep loses nothing an operator cannot re-make with one more drag —
and the zoom is a one-shot the operator armed deliberately from the
ribbon, spent by the very next drag.

The text half is not a new rule: the *un-armed* reading-mode text row has
yielded to the zoom since it shipped, and this asserts the armed tool
borrows that ordering rather than inventing a second one. Both modes are
covered, because a build that consulted `caps.edit_content` while
deciding would answer differently in each.

### `fn a_vertex_markup_tool_takes_the_click_and_offers_no_drag`

Three claims, and each has a distinct failure:

* **`drag` is `None`** — a build that gave these a `DragKind::Markup`
  would put a rubber band on screen for a gesture nothing implements, and
  `band::drag`'s family guard would then draw and author nothing, so the
  operator would see a band appear and vanish on every press.
* **`click` is live**, gated on `author_markup` — a build that reused the
  general `caps.edit_content || text` tail would leave these two tools
  inert in **Review**, which is the mode a reviewer draws a cloud-shaped
  polygon in, and would leave them placing vertices in Read.
* **The grips and the armed zoom do not change the answer**, because the
  early return is above both. A vertex click that fell through to the
  marquee rung would place no vertex and replace the selection instead.

### `fn a_dimensions_rotate_handle_is_gated_on_measure_not_markup`

The two rows together are the whole reason [`RotatableAnnot`] is a variant
rather than a bool. A build that gated both families on one capability
passes exactly one of these and fails the other, whichever gate it picked —
and the failure it ships is a handle that is painted and inert in one mode,
which is indistinguishable from a handle that is broken.

### `fn the_handle_outranks_the_content_branch_in_edit`

The ordering is stated in `press_kind` rather than relied on, and this pins
it: with a markup selected in **Edit** — where `edit_content` is true and
the content branch's own `Grip::Rotate` arm exists — the press must still
reach the annotation's rotation. A build that let it fall through would
rotate *the page content selection*, which is the "working gesture aimed at
the wrong verb" this canvas has produced four times.

### `fn a_grip_outside_a_markups_box_still_resizes_it_in_review`

Driven at screen (532, 493) against a box declared
`[[443.3 493.1] - [531.0 578.8]]` — one point outside its right edge and a
tenth of a point above its top.

### `fn the_body_of_a_markup_still_means_move_not_resize`

A build that repaired the grip by widening `markup_body` would pass the
test above and turn every corner press into a translation — a working
gesture aimed at the wrong verb, which this canvas has produced four times
and which `Grip::Rotate` was given its own arm to prevent.

### `fn a_markup_grip_in_a_mode_that_authors_no_markup_resizes_nothing`

Read authors no markup, so a press that somehow arrived with `markup_grip`
set there must not become a resize. Without this, the repair above would
have widened the arm's reach past its own gate, which is the shape of every
mode-gating defect this project has shipped.

### `fn a_grip_outside_a_form_fields_box_still_resizes_it_in_edit`

`widget_grip` is `markup_grip`'s twin and was broken identically: a widget
is only selectable in Edit, so the press fell into the content branch,
which found no content selection and marqueed the page behind the field.
`a_resized_check_box_is_redrawn_not_stretched` SKIPPED rather than failing
on it, because the field it drives is too small to aim a corner grip at —
so this rung had **no driven witness at all**, which is why it gets one
here.
