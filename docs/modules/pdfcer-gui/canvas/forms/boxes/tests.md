# `pdfcer-gui/canvas/forms/boxes/tests`

## Item notes

### `fn text_field`

By hand rather than from a fixture because the predicate under test is
about combinations no single real document carries — a rich-text
read-only field on a rotated page is not a document anybody shipped.

### `fn drawn_widget`

# `page: None` and `rect: None` are the point of this fixture

Both keys are Optional in the spec and **both are frequently absent in
real files** — and `pdfcer-core` additionally reads `/P` without
resolving through the graph, so a direct rather than indirect `/P` also
reads as absent. Every one of the ten form fixtures in
`D:\Dev\pdfcer\fixtures\synthetic\forms\` writes `/P` on every widget, so
a test built from a fixture cannot reach the case; the engine team hit
exactly that when a deliberate sabotage of their own `/P` handling
passed against their whole corpus.

So the fixture omits both, and every assertion in this module is
therefore also an assertion that neither is consulted. If someone
reintroduces a `/P` lookup or a `Widget::rect` read, these tests stop
passing here rather than stopping working in the field.

### `fn an_undrawn_widget_is_not_offered_on_the_canvas`

The decision the module header §5.1 argues for, pinned. `demo-form.pdf`
carries this case, and the failure if it regressed is the worst kind:
an invisible click target over blank paper, which an operator can only
find by accident and cannot find again.

### `fn a_rotated_page_withholds_a_text_editor_but_not_a_button`

Both halves, because the interesting content of the decision is the
asymmetry: a text field cannot be edited in place on a `/Rotate 90`
page (egui cannot rotate a `TextEdit`), while a check box has no text
direction and is offered exactly as it is anywhere else. A build that
refused both would be over-cautious in a way no operator could
understand.

### `fn the_canvas_declines_every_field_the_panel_blocks`

Asserted against the panel's own function rather than a re-derivation,
so the test cannot pass by agreeing with a third copy of the rule. The
silent failure it guards is two surfaces disagreeing about which fields
are fillable — an operator clicking a field on the page that the panel
says is read-only, or the reverse.

### `fn each_radio_widget_selects_its_own_state`

The defect this prevents is a radio group in which every button selects
the first option: the field's `/V` would be set to the same name
whichever kid was clicked, and the group would look broken in a way
that reads as an engine fault rather than a shell one.

### `fn a_widget_with_no_p_entry_is_still_placed`

`/P` is Optional (§12.5.2 Table 164) and frequently absent, and
`pdfcer-core` reads it without resolving through the graph, so a direct
`/P` reads as absent too. The obvious implementation of *"which page is
this widget on?"* — look up `Widget::page` — therefore returns **nothing
at all** on such a form: no error, no refusal, no trace, just a form on
which clicking a field silently does not work.

Every one of the ten form fixtures in
`D:\Dev\pdfcer\fixtures\synthetic\forms\` writes `/P` on every widget, so
a test opening a fixture cannot reach this. The engine team hit exactly
that: a deliberate sabotage of their own `/P` handling passed against
their whole corpus, and they had to build a form in memory that omits the
key. This is that form, in this shell — [`drawn_widget`] omits `/P`, and
the assertion is that the box is produced anyway.

It is the general rule in a new place: **a test that cannot reach the
case is satisfied by any implementation.**

### `fn two_adjacent_fields_never_claim_each_others_clicks`

The property the no-tolerance decision buys, asserted as the thing an
operator would notice: two fields one point apart — the ordinary shape
of a form table — resolve to exactly one answer each, and the gutter
between them resolves to neither. A six-point catch radius would make
all three of these ambiguous.

### `fn a_field_too_small_to_read_is_grown_about_its_own_centre`

A 12 pt field at 25 % zoom is three screen points tall. Without the
minimum the operator cannot read what they typed; without the centring,
growing it would slide the box off the field it belongs to and the
editor would appear to jump as the zoom changed.

### `fn a_fields_quadding_reaches_the_box_a_click_makes`

The half of the quadding contract that lives in [`classify`]. A
classification that carries no alignment leaves [`super::editor`] nothing
to read, and every field — left, centred or right — is then typed into
left-aligned. The value is asserted for all three codes rather than for
one, because a `field.quadding` hard-wired to `Left` passes a
single-value test perfectly.

### `fn each_quadding_code_anchors_the_editor_at_its_own_end`

A silent transposition is the failure this guards: swapping `Center`
and `Right` compiles, draws a caret, passes every other test in this
file, and is visible only as a right-aligned form typed into centred.
Asserted as three separate, distinct answers so a mapping that
collapsed two codes into one cannot pass either.

### `fn only_the_select_tool_fills_a_form`

The whole of the "no `CanvasTool` variant" decision, expressed as the
one line it is. The markup rows matter most: a pen that also filled a
field would make one press mean two things, and the operator would
discover it by finding text in a box they were drawing a rectangle
over.

### `fn a_real_form_produces_boxes_inside_its_own_pages`

The end-to-end shape of the read path — parse, place, project — on the
document the panel's own disclosures were written against. It asserts
what a screenshot cannot: that boxes exist at all, that each one names a
field the form really has, and that each lands inside the page it
claims.

Note what it deliberately does **not** prove:
[`a_widget_with_no_p_entry_is_still_placed`] exists because this test
cannot reach the `/P`-absent case — every widget in this fixture carries
`/P`, so a `/P`-keyed implementation would pass here.

### `fn the_three_field_fixture_offers_three_clickable_text_boxes`

Its own test, so that a fixture which stopped having them fails here with a
sentence about the fixture rather than turning a driven tab-navigation run
into a confusing report about keyboard focus.

The generator below records why the engine corpus could not supply this:
every text field in it is `/AP`-less, and an `/AP`-less field is not drawn
on the canvas at all, so there is nothing to click.

### `fn a_choice_field_with_no_options_is_not_fillable_but_is_still_selectable`

The fixture is a `/Ch` field with an **empty `/Opt`**: there is nothing to
pick, so `classify` refuses it. Were selection taken from the same list,
that refusal would also remove it from the only list the canvas hit-tests,
and a field the operator can plainly see would not be clickable at all.

The two assertions are deliberately opposite, because a test that only
checked the target would pass against a change that made every widget
fillable — which is a different bug with the same symptom on this test.

### `fn an_undrawn_widget_is_still_selectable`

`NoAppearance` routes a field to the panel for FILLING because the page
draws nothing there — but pdfcer authors widgets, and a widget it made
and then failed to draw is exactly the one an operator needs to reach in
order to delete it. The rectangle is real even when the appearance is
not.

### `fn a_background_is_read_and_stating_none_is_not_the_same_as_stating_nothing`

The subject is [`editor_fill`], which decides what colour the in-place
editor tints itself. Its whole job is a mapping, so the test is the
mapping — stated per variant, because the match is deliberately exhaustive
with no wildcard and a future variant should arrive here as a compile
error in the module and a missing case in this list, not as a silent
`None`.

**The case worth reading twice** is the pair at the top. Table 189 lets
a file state `/BG []` — an EMPTY array, meaning *explicitly no colour* —
and that is a different fact from `/BG` being absent. The engine keeps
them apart ([`MkColor::None`] versus the enclosing `Option` being `None`),
and this function is the one place they are allowed to merge, because the
question it answers — *do I tint?* — has the same answer for both. Both
are asserted so that a reader can see the merge is intentional rather
than a missing arm.

### `fn a_cmyk_background_uses_the_engines_own_table_and_not_one_minus_k`

This shell refuses to convert DeviceCMYK in two other places on purpose
(`app::markupband::rgb_of`, `app::fontband`) because those are swatches
whose readback would write an invented colour back into the file.
[`editor_fill`] writes nothing and sits on a raster the engine itself
produced, so the right answer there is to use the engine's own table —
which makes the box AGREE with the pixels beside it.

The assertion is chosen to be falsifiable by the failure it guards
against: solid K ink alone is a **warm near-black**, around 0.13 red, and
the naive `1.0 - k` an implementer reaches for first gives exactly 0.0. A
hand-rolled conversion therefore fails here rather than shipping as a
half-shade of wrong on every CAD form in the building. The bounds are the
engine's own documented ones for this input, quoted rather than invented.

### `fn the_combo_flag_reaches_the_box_census`

A combo box drops its list below the widget; a list box draws its options
inside the widget's own rectangle. `canvas::forms::choosing` cannot make
that choice unless the census carries the flag, and before it did, every
`/Ch` field got the drop — which is the half of O209 that reads *"the list
option is somehow hidden from view in Acrobat until I click on it."*

Both polarities are asserted from one fixture, so a census that hard-coded
either answer fails.

### `fn the_edit_flag_is_a_capability_only_alongside_the_combo_flag`

Table 230 states it outright — *"used only with Combo"* — and the engine
enforces the same conjunction: `set_choice_value`'s free-text branch is
gated on `COMBO && EDIT`, so a census that honoured bit 19 alone would
offer typing into a field the engine then refuses with
`ChoiceValueNotInOptions`. The operator would see a box they could type in
and an error naming a value they chose deliberately.

All three interesting polarities are asserted from one fixture, so a census
that read either bit alone fails.
