---
name: a-correctly-sourced-value-used-for-the-wrong-role-passes-every-gate
description: check-theme-colors enforces "no invented colours", not "the right role" — a theme value used for the wrong purpose ships looking broken with every gate green.
metadata:
  type: feedback
---

**A value read from the right place and used for the wrong purpose passes every
check this project has.** When two theme roles could plausibly serve a surface,
the fix is a **named accessor for the pairing**, not care.

## The incident, 2026-09-03

Ken, about the print dialog: *"it looks greyed out as though it doesn't do
anything even when I hit print — but it is working, so after many clicks I
checked the printer and of course there was a dozen jobs there."*

`dialogs::host::Host::buttons` filled the affirmative button with

```rust
let fill = ui.visuals().selection.bg_fill;
let text = ui.visuals().strong_text_color();
```

with a comment explicitly congratulating itself: *"Drawn from the theme's own
selection fill and the strong text colour it is guaranteed to contrast with —
never a literal, which `check-theme-colors.sh` enforces and which defect D2 is
about."*

Every clause of that is true and the result was broken. `egui-shell`'s theme
sets `v.selection.bg_fill = Color32::from_rgba_unmultiplied(90, 140, 220, 70)` —
**alpha 70/255, a 27 % wash** — because that role's real job is tinting selected
objects on the *canvas*, where seeing through it is the point. Composited over a
light panel it becomes about `rgb(193, 207, 230)`: **paler than
`widgets.inactive.weak_bg_fill`**, the opaque fill every ordinary button gets.

So the default action rendered **less solid than the Cancel button beside it**,
which is exactly what a disabled control looks like.

## Why nothing caught it

- `check-theme-colors.sh` forbids raw `Color32` outside the theme module. This
  used a theme role. Green.
- The rendered-pair contrast gate measures **widget-state pairs**
  (`inactive.bg_fill` against `inactive.fg_stroke`, etc.). This was an ad-hoc
  pairing of a *selection* fill with a *text* colour, which is not a widget
  state and is not in its enumeration.
- **This is defect D2 for the third time.** The active ribbon tab had the
  identical bug and was fixed by moving to `accent` + `on_accent`;
  `FEATURES.md` records that fix and even notes the mode selector "always did"
  paint the accent. `Host::buttons` was written from the same wrong role and
  nobody connected them, because a dialog button is not a ribbon tab and no test
  compares the two.

## The fix, and the general form

`egui_shell::Theme::accent_pair(ctx) -> (accent, on_accent)`. **One spelling of
*"paint this as the emphasised action"***, so a preset that changes its accent
moves every such surface together, and there is nothing for a future author to
choose between.

> **A gate that forbids invented values cannot enforce correct roles.** When a
> surface needs two theme values that must agree, expose them as a **pair**
> behind a purpose-named function. The name is the mechanism: `accent_pair` has
> no plausible wrong use, where `visuals.selection.bg_fill` has an obvious one.

★ Also: `strong_text_color()` is not the foreground for an accent. It follows
`override_text_color`, i.e. the **body text** colour — near-black under a light
preset. On a saturated accent that is poor, and under a preset with a dark
accent it is black on black. `Palette::on_accent` exists precisely so that
pairing is the theme's decision and inverts per preset; its own doc comment says
the two must never be welded together.

**How to apply:** when reaching for a `visuals.*` colour for chrome, ask *what
is this role FOR* — not *does it look right in the preset I have open*. A
translucent role is the loudest tell: translucency means the role expects
content behind it, and chrome has a panel behind it, so the colour changes with
whatever it is over.

Related: [[feedback_a_measurement_of_the_wrong_surface_looks_exactly_like_a_broken_one]],
[[feedback_a_proxy_condition_survives_one_correction]],
[[feedback_a_long_green_check_can_be_aiming_at_nothing]].
