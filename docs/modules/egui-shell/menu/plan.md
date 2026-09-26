# `egui-shell/menu/plan`

## Item notes

### `fn a_hidden_item_leaves_no_row_while_a_disabled_one_leaves_a_greyed_one`

Rule 3. The two mechanisms are easy to confuse and produce menus that
both look plausible, so three rows are resolved against one condition
set for three outcomes:

* a hidden row leaves **no slot at all** — not a disabled one;
* an unhidden but disabled row leaves a slot with `enabled: false`;
* a row with no condition is unaffected.

Falsified by deleting the `visible` check in [`resolve`]: the first
assertion fails with three slots instead of two.

### `fn hiding_a_group_collapses_the_rule_that_introduced_it`

The interaction the two rules have to get right together: hiding the only
item in a group leaves a rule with nothing above it, which is the stale
document shape [`collapse`] already handles — asserted rather than
assumed, because it is the way rule 3 goes wrong visibly.

Falsified by running `resolve` without `collapse`: the leading
separator survives and the menu opens with a horizontal line at the top.

### `fn an_unknown_command_is_absent_and_a_disabled_one_is_greyed`

The two halves of rule 1, asserted together because the whole
difficulty is telling them apart. Getting either one wrong produces
a menu that looks plausible: drop the disabled row and the menu
silently changes shape as the selection changes; keep the
unregistered row and the build ships a placeholder for a command it
does not have.

### `fn a_menu_of_only_disabled_commands_offers_nothing`

The decision behind "right-clicking something with nothing to offer
does nothing". Asserted here, before any drawing, because after
drawing there is by definition nothing to observe.

### `fn the_atom_count_charges_for_the_invisible_grow_atom`

`AtomLayout` bills `gap × (atoms − 1)` whether or not an atom has
any size, so the zero-width atom that right-aligns the chord costs
a real gap. Forgetting it under-estimates every row that has a
chord — and under-estimating is the direction in which the chord
column runs off the right edge.

### `fn one_row_with_a_glyph_gives_the_whole_menu_a_column`

The two halves of the rule, asserted together because the whole
difficulty is that they are the same question asked of different
scopes. Get the first wrong and a menu with a single icon draws it
against a ragged label column; get the second wrong and every plain
menu in the application silently gains an indent it has no use for.

The third case is the one a per-row implementation would never
think to check: punctuation and application-drawn rows must not
vote. A separator has no columns, and a `Custom` row is drawn by the
application, which is the only party that knows whether its widget
has a glyph — so a menu of nothing but those reserves nothing.

### `fn an_icon_less_row_in_a_reserving_menu_spends_the_width_and_paints_nothing`

The per-row half of the rule, swept over every input pair, because
three of the four answers are load-bearing in a different direction:

* `Blank` must be *reserved* (or the label column zig-zags) and must
  not *draw* (or R9 is broken with a placeholder mark);
* `Glyph` must be both;
* `Absent` must be neither, which is what keeps a plain menu free.

The fourth pair — no column, but this row has a key — is
unreachable, since a row with a key is exactly what makes a menu
reserve. It is *answered* rather than asserted against, and the
answer degrades to the pre-column layout rather than panicking
inside a popup.

### `fn a_blank_icon_slot_costs_exactly_what_a_glyph_costs`

The geometry the whole rule rests on. If a blank row were measured
as though it had no slot, the arithmetic would under-estimate every
icon-less row in a menu that has icons — and under-estimating is the
dangerous direction, because the widest row sets the body width and
an under-measured widest row truncates its own label. So the
assertion is *equality*, not "close enough".

The second half asserts the other direction: a menu with no icons
must be unchanged, so its rows are narrower by exactly the slot plus
the one atom gap the slot buys — and the body follows the rows
uniformly, which is what keeps the label column straight rather than
letting the widest row grow alone.
