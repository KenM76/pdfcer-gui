# `ui-verify/checks/markup_style`

`markup_style_group_is_drawn` — the ribbon group that was a caption over
nothing.

# The defect

`RIBBON_IA.md` §5.5 specifies a **Style** group on the Markup tab —
*"Colour · Line width · Fill · Opacity"* — and the manifest has declared it
since S2, as one `Item::custom("colour_swatch")`.

**No renderer ever matched that kind.** `egui-shell`'s custom-item extension
point works by the application supplying a closure that looks at
`item.kind`; the application's closure looked only for the Recent menu and
returned `None` for everything else. So the shell reserved the item's space,
the application declined to draw it, and the Style group rendered as a
**caption over an empty band** for the whole of v0.1.0.

Meanwhile the pen was two hard-coded constants — red, 2 pt — so every mark
this shell has ever authored is the same colour and the same width, with no
way to change either. §5.5 predicted the operator's report in advance:

> The `Style` group sets defaults for the next markup. … Both must exist;
> today only the first does, **which is why a placed markup feels final**.

# Why no test could see it, and this one can

This is a **third** shape of the invisible-wiring failure this harness
exists for, and it is worth naming beside the other two:

| shape | example | what was green |
|---|---|---|
| a command with no dispatch arm | `file.settings` | the registry, the manifest, the reachability check |
| a linked crate with a refusing adapter | `pdfcer-print` | the adapter's own tests, which asserted the refusal |
| **a declared item with no renderer** | `colour_swatch` | everything — the manifest test asserts the item is *declared* |

The third is the quietest. `shell::manifest::mod`'s own test asserts
`assert_eq!(style, vec![Item::custom(COLOUR_SWATCH)])` and passes, correctly,
for a build in which the item draws nothing: it is a claim about the
manifest, and the manifest was right. The reachability check cannot help
either — a `Custom` item carries no command id, which is the whole point of
it, so it is invisible to every check built on `command_references()`.

What is left is asking the running program whether it drew anything, which
is what `diag::ui_rect` is for.

# What this measures

Three regions, published by `canvas::markup::swatch`:

```text
markup.style.ink           the pen swatch
markup.style.highlighter   the highlighter swatch
markup.style.width         the width control
```

All three must be **declared and substantial**. Substantial matters as much
as declared: a control laid out with no usable area is the redaction panel's
apply button shipped below the bottom of its own pane, which this project
has already had once.

Then it **changes the width** and reads the `markup-pen` trace line back, so
the pass is not merely "three rectangles exist" but "a control was driven
and the pen moved".

# What it does NOT do, and what that costs

**It does not open the colour picker.** `egui`'s
`color_edit_button_srgba` opens a popup whose internals publish no regions,
so a harness cannot aim at a hue in it — and clicking blind inside a popup
is how a check starts passing for the wrong reason.

So the colour half is verified one step short: the swatches are on screen
and substantial, and the *pen* is proved mutable through the width control,
which shares the same trace line and the same state. What is not
machine-verified is that dragging in the picker lands on the annotation —
and the unit test `pen::tests::a_colour_round_trips_through_the_swatch`
covers the conversion either side of it. That gap is named here rather than
papered over: the honest description of this check is *"the Style group is
drawn and its state is live"*, not *"colour picking works"*.
