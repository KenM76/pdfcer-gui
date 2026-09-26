# `egui-shell/ribbon/rhythm`

## Item notes

### `const BAND_ROW_SPACING`

**The band's only row pitch** — the ordinary case and the re-wrapped case
share it, because [`band_row_height`] is unconditional. One, because the
mockup's arithmetic needs it: `3 × 22 + 2 × 1 = 68`, which is
[`crate::theme::Metrics::ribbon_rows`] exactly.

Tighter than the theme's `item_spacing.y`, and it has to be: the band's
height is fixed (R128) and three rows must fit an area sized for them.

### `fn compressed_control_height`

# Why this exists at all — Word's third row is shorter

The obvious implementation of S5 is "allow three rows", and it fails
immediately: three rows of `control_height` are half again as tall as two,
so the band grows, so the canvas beneath it moves on every tab click, which
is R128 and is the defect this whole module is arranged around.

Word does not grow its band. Measured in `evidence/word-ribbon/`, its band
is the same height at 1900 pt and at 1000 pt while the Font group goes from
two rows to three — because **its rows are not uniform**. The tall row is
the one with combo boxes in it; the icon rows are shorter. The band is a
fixed budget and rows are packed into it, rather than the band being two
rows tall by definition.

So a re-wrapped group here divides the **same** row area into
[`plan::MAX_GROUP_ROWS`], and its controls are drawn shorter to suit.
`Theme::apply` pins `spacing.interact_size.y` to `control_height`, which is
what makes every control exactly one row tall; overriding it on the group's
own `Ui` is what makes an extra row possible without touching the band.

While [`plan::MAX_GROUP_ROWS`] equals [`plan::GROUP_ROWS`] this returns the
same number as [`band_row_height`], so a re-wrapped group draws at the
ordinary row height. The two are kept separate because they answer different
questions — the ladder's ceiling and the band's natural depth — and raising
the ceiling must not change the ordinary case.

# The guard, and why the eligibility test is a measurement

A control still has to show its icon, so the result is compared against
[`crate::theme::Metrics::icon_pts`] by [`rewrap_is_legible`] rather than
assumed to clear it. A theme whose numbers did not clear would make this
return less than `icon_pts`, `measure_group_rows` would then report the
re-wrapped width as no better than the natural one, the ladder would decline
to spend a rung on it, and the group would simply never re-wrap. **The
feature turns itself off rather than clipping.**

### `fn caption_font`

One accessor rather than two reads of the metric, because the number is
used in three places that must not drift — the caption's own
`RichText::size`, [`band_height`]'s prediction of how tall that caption
will be, and [`super::sizing`]'s Large label. A band whose height
prediction and whose caption disagree is R128 by a fraction of a line,
which is precisely the class of drift `group_body`'s closing
`allocate_space` exists to absorb and would rather not have to.

### `fn rows_height`

# It is a budget stated by the theme, not a multiple of a row

The alternative spelling is `GROUP_ROWS × (control_height + item_spacing)`,
meaning *"exactly as tall as the rows"*, and it has a property that reads as
a defect once named: a group that uses every row fills the area edge to
edge, so its caption is drawn immediately beneath its last control, while a
one-row group's caption sits a whole row lower. The captions still share a
baseline, but the band has no headroom anywhere and reads as cramped.

The mockup's band is a **budget** instead: `.grp .items { align-items:
flex-start }` lays the rows into the top of a 68 px area and `.grp .cap
{ margin-top: auto }` hangs the caption off the bottom of it, whatever the
rows did. So the area is stated by the theme
([`crate::theme::Metrics::ribbon_rows`]) and the rows are laid into it.

# The trailing `item_spacing`, which is the term that bites

`rows × height + (rows − 1) × spacing` is the right answer for the *ink* and
the wrong one for the **cursor**: `egui` advances the cursor past every
laid-out rect by `item_spacing`, after the last row as much as after the
first. A group that used every row therefore leaves its cursor one gap
beyond that figure, its padding computes as zero, and it ends up exactly
`item_spacing` taller than a one-row neighbour whose padding *was* applied.

**A fixture can hide this entirely, so the fixture has to be checked too.**
`super::width_tests`' context installs a font but applies no
[`crate::theme::Theme`], so `egui`'s default `interact_size.y` (18 pt) sits
well under the theme's `control_height` (24 pt) and every row carries 6 pt
of slack for a stray gap to hide in. In the running application the two are
equal by construction — `Theme::apply` sets `spacing.interact_size.y =
control_height` — so there is no slack and the discrepancy is visible in the
band's own trace. `super::height_tests::context` applies the theme for
exactly this reason: a fixture that is more forgiving than the program
flatters the thing it measures.

# What the collapse ladder does with this area

`RIBBON_SCALING.md`'s three rungs are re-wrap → collapse → scroll, and rung
one divides *this* area into [`plan::MAX_GROUP_ROWS`] rows rather than
[`plan::GROUP_ROWS`]. Both row counts are constants, so the ladder changes
the divisor and never the area — the band cannot grow a rung. See
[`rewrap_is_legible`] for the self-disabling behaviour that protects.

### `fn band_height`

Five terms, in the order they are drawn:
[`crate::theme::Metrics::ribbon_pad_top`], the control rows,
[`CAPTION_GAP`], one line of [`caption_font`], and
[`BAND_PADDING_BOTTOM`]. Derived from the theme, the font and two
constants, and from nothing the manifest can vary — see the module header
on R128 for why that independence is the whole point.

The top padding (`.ribbon { padding: 6px 8px 0 }`) is a term *here* rather
than an `add_space` before the first group, for the reason
[`BAND_PADDING_BOTTOM`] gives at length: space emitted only when there is a
group to emit it before would be absent on a tab whose groups all went into
the overflow menu, and a band six points shorter on one tab than on its
neighbour moves the canvas on a tab click. `group_body` spends it, out of
the group box's top padding, on every group and on none.

The bottom padding belongs in this derivation for the same reason: space
emitted after the last group would be absent on a tab that drew no group,
which is a reachable state, and would make the height content-derived
through the back door.

With the shipped `Quiet` theme the sum is `6 + 68 + 3 + 12.7 + 5 ≈ 94.7` pt,
against the mockup's own `grid-template-rows` figure of 96 px for the ribbon
row (which includes its 1 px bottom border).

`pub(crate)` so a test can state the claim in the same terms the
renderer does rather than by re-deriving it.
