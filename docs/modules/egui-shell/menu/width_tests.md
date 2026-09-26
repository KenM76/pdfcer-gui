# `egui-shell/menu/width_tests`

## Item notes

### `fn every_row_is_justified_to_the_same_width`

`Atom::grow` right-aligns the chord *inside its own button*. If the
buttons were not all the same width, each chord would be flush with a
different right edge and the "column" would be a ragged diagonal —
which looks like a rendering fault and is the reason
`set_min_width` is called before a single row is drawn.

### `fn a_menu_with_chords_is_wider_than_the_same_menu_without`

The direct consequence of [`plan::COLUMN_GAP`] and of charging for the
chord's own text. With no font data this test is vacuous — both menus
measure the floor — which is precisely why [`testfont`] exists.

### `fn the_rendered_body_is_the_width_the_plan_computed`

The plan is pure arithmetic and the renderer is `egui`; they agree only
because the renderer measures with the plan's own inputs. This is the
tripwire on that: if a future edit changes the atom order, adds a
spacer, or measures the chord in a different `TextStyle`, the two
numbers separate and this fails — where otherwise the only symptom
would be a chord column that is slightly too tight, in one theme,
noticed by nobody.

### `fn a_pathological_label_clamps_and_says_so`

Past [`plan::MAX_BODY_WIDTH`] the label gives way, not the position —
the same trade the ribbon's overflow affordance makes, and for the same
reason: characters are recoverable (the tooltip has the full text),
position is not.

### `fn one_icon_costs_a_menu_the_same_column_as_four`

The rendered proof of the icon-column rule, against real font metrics.
Three menus, identical labels and chords, differing only in which rows
name a glyph:

| Menu | Every row's slot | Expected body |
|---|---|---|
| all four have icons | `Glyph` | the reference |
| one has an icon | `Glyph`, then three `Blank` | **the same** |
| none has an icon | `Absent` | strictly narrower |

The first two being **equal** is the whole rule: a blank slot costs
what a glyph costs, so the label column starts at one x whatever the
mix. Under a per-command rule the middle menu would come out narrower
than the first and its labels would zig-zag — and no width assertion
that looked at one menu at a time would notice.

The third being narrower is the other half, and it is what keeps the
rule cheap: a menu whose commands have no icons pays nothing for the
column — no slot, no indent, no extra width.

### `fn the_popup_path_justifies_every_row_to_the_same_width`

The harness above installs the popup's layout by hand for
determinism. This asserts the same property through an actual
right-click, so the shortcut cannot be hiding a difference between the
harness and the shell — the failure mode that makes a green geometry
suite worthless.
