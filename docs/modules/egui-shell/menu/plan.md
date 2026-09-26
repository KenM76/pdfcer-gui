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

### `enum Slot`

Borrows from the registry and the [`Shortcuts`] index rather than
copying, because this is built once per menu open and dropped at the
end of the frame.

`PartialEq` is deliberately **not** derived: a `Slot` borrows a
[`Command`], and `Command` holds an `Enable` that may be a closure,
which has no meaningful equality. Tests match on shape instead, which
is what they actually want to assert.

### `fn resolve`

`context` is used only to name the menu in a disclosure, so a log line
says *which* menu referred to a command that is not there.

See rule 1 in the module header for the unregistered-versus-disabled
distinction, which is the whole reason this function exists rather than
the renderer walking `items` directly.

### `fn collapse`

Rule 4 in the module header. Separate from [`resolve`] so it can be
asserted on hand-built input, including the shapes a real document
would have to be perverse to produce but a *stale* one produces
routinely.

### `fn offers_anything`

Rule 2 in the module header. `true` iff at least one row is something
the operator could act on: an enabled command, or a custom item the
application owns.

### `enum IconSlot`

Three states rather than two, because "this row has no icon" is not one
answer — it is two, and they differ by **what the row beside it is
doing**. See [`icon_slot`] for the rule and the argument.

### `fn is_reserved`

[`Self::Blank`] and [`Self::Glyph`] cost exactly the same width;
that is the entire point of `Blank`, and it is why measurement and
drawing must both ask this question rather than each asking
"does the command have an icon?".

### `fn reserves_icon_column`

`true` iff at least one command row that survived [`resolve`] names an
icon key. Separators and [`Slot::Custom`] rows never decide it: a
separator has no columns, and a custom row is drawn by the application,
which is the only party that knows whether its widget has a glyph.

# Why the decision is per-menu and not per-row

A menu is a list of words and reads as one. The eye scans the left edge
of the labels, and the single thing that makes that scan cheap is that
the edge is *straight*. Deciding the slot per row breaks it: the rows
with icons indent and the rows without do not, so the label column
zig-zags and the reader loses the one alignment they were using.

The opposite extreme is worse still. Forcing a glyph onto every row —
inventing art for `Reflow block`, reusing something approximate for
`Close other documents` — buys a straight edge with a column of
pictures that do not mean anything, and a picture that does not mean
anything is read as one that does and then mis-read. That is the
wrong-picture refusal this project records at each command's own
registration, and it is not weakened by the column wanting to be full.

So: the **column** is a property of the menu, the **glyph** is a
property of the command, and a row whose command has no icon spends the
width and paints nothing ([`IconSlot::Blank`]). An indent is not a
hole — it is the same left margin every other row has.

# Why the empty menu is the common case and must stay free

A menu where *no* command has an icon reserves nothing
([`IconSlot::Absent`] everywhere): no slot, no indent, no extra width.
That matters more than it sounds — it means the column rule cannot make
a plain menu wider or move a single pixel of it. Only a menu that has
something to show pays for the column.

### `fn icon_slot`

`reserved` is [`reserves_icon_column`] for the whole menu; `has_key` is
whether *this* command names an icon. The pair is the entire rule, and
it is a free function over two `bool`s so it can be swept exhaustively
by a test rather than inspected inside a draw call.

| `reserved` | `has_key` | Result | Why |
|---|---|---|---|
| `false` | `false` | [`IconSlot::Absent`] | No row in this menu has a glyph; there is no column. |
| `false` | `true` | [`IconSlot::Absent`] | **Unreachable** — a row with a key is what makes `reserved` true. Answered rather than asserted: a panic here would turn a caller's bookkeeping slip into a crash in a popup, and `Absent` degrades to the pre-column layout, which is the safe direction. |
| `true` | `false` | [`IconSlot::Blank`] | A sibling has a glyph; indent to keep the label column straight, and paint nothing. |
| `true` | `true` | [`IconSlot::Glyph`] | Draw it. |

### `const COLUMN_GAP`

# Why this number exists at all, and why `egui` will not supply it

A menu row is drawn as an `egui::Button` whose atoms are
`[icon?] [label] [grow] [chord]`. The `grow` atom absorbs whatever
width is left over, which is what right-aligns the chord — and on the
**widest** row there is nothing left over, so the label and the chord
end up separated by one atom gap (4 pt) and read as a single run of
text: `Save a copy…Ctrl+Shift+S`.

`egui` cannot fix this for us because it sizes each button
independently; the fact that these buttons form two columns is a fact
about the menu, not about any one row. So the menu computes its own
width, adds this gap to the widest row, and every row — including that
one — then has at least this much clear space between the two columns.

24 pt is roughly two capital widths at the body size: enough that the
eye reads two columns rather than one sentence, and not so much that a
three-item menu becomes a banner.

### `const MIN_BODY_WIDTH`

A menu narrower than this reads as a tooltip that happens to be
clickable. It also gives the pointer somewhere to be that is not
on top of a row, which matters for dismissing without invoking.

### `const MAX_BODY_WIDTH`

A menu is a list of verbs, and past roughly this width it stops being
scannable and starts covering the thing that was right-clicked — which
for a canvas selection is the one piece of context the operator needs
while choosing. Beyond it the **label** gives way rather than the
position, for the same reason the ribbon's overflow affordance
truncates rather than moves: spending the shortfall on characters is
recoverable (the tooltip carries the full text), spending it on
position is not.

### `struct RowWidths`

Pure numbers, supplied by the renderer, which is the only party that
can ask `egui` how wide a string is. Keeping the *arithmetic* here
means it can be swept across hundreds of inputs by a unit test while
the measurement stays in one place next to the drawing.

### `fn atom_count`

Always the label; plus the icon slot if there is one; plus **two**
if there is a chord, because the chord is preceded by the zero-size
`grow` atom that does the right-aligning.

This matters because `AtomLayout` charges `gap × (atoms − 1)`
unconditionally — a zero-size atom still buys its gap
(`egui-0.35.0/src/atomics/atom_layout.rs`) — so a width estimate
that ignored the `grow` atom would under-estimate by one gap on
every row that has a chord. Under-estimating is the dangerous
direction: it is the one where the chord column runs off the right
edge of the menu.

### `fn total`

`atom_gap` is `egui`'s `spacing().icon_spacing`, which is what
`AtomLayout` puts between atoms; `padding` is the button's
horizontal padding, both sides. Both come from the live style
rather than from constants here, because both are theme-dependent
and an estimate that disagreed with the toolkit would be wrong in a
way no test of this module could see.

### `fn body_width`

The **widest** row decides, because the rows form two columns and a
column is as wide as its widest member. Then the result is clamped:
never thinner than [`MIN_BODY_WIDTH`], never wider than
[`MAX_BODY_WIDTH`].

An empty menu yields [`MIN_BODY_WIDTH`] rather than zero. It should
never be drawn at all (see [`offers_anything`]), and a zero-width popup
is a much worse thing to fall back to than a small one.
