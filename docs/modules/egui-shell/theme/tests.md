# `egui-shell/theme/tests`

## Item notes

### `fn text_contrasts_with_its_background_in_every_preset`

A crude relative-luminance gap rather than a full WCAG contrast
ratio: the point is to catch a preset
where someone set a light text colour against a light panel — which
is what a `..quiet` spread does the moment a surface is darkened
and the text is not — and a coarse check that always fires beats a
precise one nobody runs.

**This test is kept even though the rendered-pair gate subsumes
most of it**, because it fails with a better message: it names the
palette role that is wrong, where the gate names the widget state
that renders wrong. Both are worth having when a preset is being
edited.

### `fn label_plates_stay_content_facing_not_chrome_facing`

Labels sit over CONTENT, not over chrome, and the content
is whatever colour the document says — overwhelmingly white. A dark
theme that darkened the label backdrop would put dark text on a
dark plate on a white page, which is unreadable in the one place it
matters most.

Worth a test because it is precisely the field a careless "make
everything dark" edit would flip.

**Note what this test does NOT do**, because it is half of why D2
shipped: it asserts `label_backdrop` is light *and stops there*. It
says nothing about what is behind `label_backdrop` when something
draws with it — and the active widget state's foreground is exactly
something that can. **A test that pins a colour without pinning its
pairing is a test that will agree with the bug.**

### `fn the_contrast_gate_catches_the_exact_defect_it_was_written_for`

Without this, `every_rendered_pair_is_readable_in_every_preset`
would pass identically if [`contrast::check`] returned `Ok` for
everything, and would be asserting nothing at all. So this
reconstructs D2 exactly — a light foreground on the active state
with its `bg_fill` left at `egui`'s default — and asserts the gate
catches it and *names the state*.

The discipline generalises past theming: a test that proves a typo
is rejected is worth nothing beside a test that proves the correct
spelling is accepted. A gate needs one of each or it cannot tell
"nothing is wrong" from "nothing is being asked".

### `fn on_accent_inverts_where_the_accent_is_light`

If every preset's `on_accent` were the same light colour, the field
would be a constant wearing a role's clothes and the next editor
would be right to inline it — reintroducing exactly the coupling
that made D2 invisible. The dark preset inverts it, and that is the
standing evidence the separation is load-bearing.

### `fn both_roles_the_selection_channel_serves_are_readable_in_every_preset`

# Why one channel has two backgrounds, and why that is the whole
difficulty

`egui` spends `selection.stroke.color` twice, on two different grounds,
and offers no way to separate them:

1. as the **ink on a selected widget's plate** — `egui-0.35.0/src/widget_style.rs:151-154`
   substitutes `selection.bg_fill` into both fills and
   `selection.stroke.color` into the text of anything `.selected(true)`;
2. as the **frame stroke of a focused, mutable `TextEdit`** —
   `egui-0.35.0/src/widgets/text_edit/builder.rs:699-706`, drawn over
   `text_edit_bg_color()`, which falls back to `extreme_bg_color`, which
   [`Theme::write_style`] points at [`Palette::panel`]. `TextEdit` has
   **no `.frame_stroke()`**: there is no per-widget escape hatch.

A test that measures only (1) is green about the wrong half of the
channel — the ring can be unreadable while the selected pair is
comfortable, and nothing says so. This measures both, from the `Style`
that actually ships.

# The numbers, measured at the floor of 90

| preset | selected pair (`accent` on `selected_plate`) | focus ring (`accent` on `panel`) |
|---|---:|---:|
| Quiet | 103.1 | 147.3 |
| Airy  | 118.9 | 170.2 |
| Dark  | 123.2 |  **96.0** |

⚠ The obvious alternative arrangement — `on_accent` on `accent` for the
plate, `on_accent` on `panel` for the ring — measures 165 / 165 / 125 on
the first column and **17.9 / 5.0 / 29.1** on the second. Airy's ring
lands white on white to within five levels of luminance. `on_accent` is
the ink for the accent FILL and is not a general foreground.

Dark's ring, at 96.0, is the tightest pair in the theme. It is also
the reason this is a loop over `Preset::ALL` and not a spot check: the
light presets clear both columns by fifty or more and would happily
bless an accent that Dark cannot use.

The ring is measured against `panel` and NOT against
`widgets.*.bg_fill`, because that is genuinely where `egui` draws it —
`text_edit_bg_color()`, not the widget state's own fill. Measuring the
convenient background instead of the real one is the error this whole
family of tests exists to avoid; see [`contrast`]'s module header.

### `fn the_selected_widget_accessors_agree_with_the_style_egui_will_paint`

[`Theme::selected_widget_ink`] exists so that
`icons::selected_image` can tint a glyph to match a plate it did not
paint, without reading `visuals.selection` (which the selection-channel
gate forbids). That is only safe while the accessor and `write_style`
agree — otherwise the glyph is coloured for a background that is no
longer behind it, which is D2 in miniature and would be invisible in
every other test.

Read through a real `Context` with the theme applied, because the
accessors go via [`Theme::of`] and a stash that never happened would
make them silently return the default preset's colours.

### `fn the_canvas_accessors_hand_back_the_values_the_canvas_used_to_read`

[`Theme::canvas_selection_ink`] and [`Theme::canvas_selection_fill`]
hand back `accent` and `selection_fill`; the widget channel carries
`selected_plate` and `accent` instead. The content area reads the
canvas pair through these accessors and never through
`visuals.selection`, which is `egui`'s channel for selected widgets.

Asserted here rather than argued in a comment, so a later edit that
"tidies" one of these into the chrome pair turns the overlay a different
colour **and goes red** instead of shipping.

Read through a real `Context` with the theme applied, not off the
struct, because the accessors go via [`Theme::of`] and a stash that
never happened would make every one of them silently return the
default preset's colours — the failure [`Theme::apply`]'s own doc
comment names.

### `fn an_out_of_range_panel_padding_saturates_rather_than_wrapping`

`egui::Margin` is `i8`. A plain cast turns 200 pt of padding into
−56, which paints content outside its own panel: a silent geometry
defect produced by a number that looked fine where it was typed.

### `fn a_foreign_fill_is_readable_under_every_preset`

[`Theme::foreign_fill_pair`] is the one accessor whose plate comes from
OUTSIDE the palette, so no preset author can have looked at it. Its ink is
therefore measured rather than named, and this walks the measurement: a
9×9×9 grid over the whole sRGB cube — 729 fills × 3 presets — asserting
each returned pair clears the same [`contrast::READABLE_LUMA_GAP`] floor
every other pair in this theme is held to.

# Why it asserts `is_some()` rather than tolerating `None`

`None` is the function's honest refusal — *this theme has no text colour
that reads on that fill, so do not tint at all* — and a caller that
obeys it is correct. But an operator meets that refusal as a field that
**silently keeps the theme's grey box while its neighbours are tinted**,
which reads as a defect, not as a decision. So the refusal must be
unreachable in a shipped preset, and this is what says so. A preset
re-tuned until some fill defeats both text roles fails here, on the
machine, rather than in front of him.

The grid includes the two corners that defeat a naive fixed ink: `0,0,0`
and `1,1,1`. A function that always answered [`Palette::text`] fails on
black under the light presets; one that always answered
[`Palette::on_accent`] fails on white.

### `fn a_foreign_fill_is_the_documents_own_colour_and_an_absurd_one_is_clamped`

The plate half of the pair is the one thing this function must not have an
opinion about: it is the file's own number, and a rounding that drifted
would make the editor box disagree with the raster beside it. The clamp
matters for the same reason `clamp_to_i8` does — a cast of `1.4 * 255.0`
into `u8` is undefined-adjacent nonsense in the one case where the input
came from a file somebody else wrote.
