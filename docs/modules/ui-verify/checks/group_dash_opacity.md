# `ui-verify/checks/group_dash_opacity`

`a_groups_line_style_and_opacity_reach_its_members`.

## What it drives

`fixtures/dimension-scaled.pdf` (one ce dimension in the default group),
Review mode with Manage groups open. It presses the group row, opens the
folded Appearance section, ticks the line-style row's *Set by group* box,
opens the chooser and picks Dashed; then ticks the opacity row's box, clicks
the percentage, selects it, types `50` and presses Enter. Every press is a
region the panel publishes (`dimension-groups.dash.set`, `.dash.combo`,
`.dash.option.hidden`, `.opacity.set`, `.opacity`). The two rows sit below
the dock's fold once Appearance opens, so `dimdrive::press_in_pane` wheels
inside `dock.right.frame` until each lies wholly within it before pressing;
the panel's own `panel:dimension-groups` rect is its scrolled content and
cannot serve as the clip.

## Verdict

After each gesture, `SetGroupStyle` writes one `dimension-member-style` line
per member: the dash and opacity `resolve_style` gives it, and the tier
`style_provenance` names for each. The newest line per member must carry
`dash=group solid=false` after Dashed, and additionally `opacity=0.5
opacity_from=group` after 50 %. FAIL when no line follows a gesture or any
member reads otherwise.

## Falsified

With `stroke_rows`' dash editor planted to write `DimDash::SOLID` whatever
was picked, the check FAILs: picking Dashed changes nothing, so no
`dimension-member-style` line follows it. With the source restored it
PASSes.

## What it does not prove

How the dashed, half-opaque dimension looks on the page; the trace reads the
style the engine resolves for regeneration, not the pixels. A member that
overrides dash or opacity itself (it must not move) is covered by the
panel's own `will_move` unit tests, not here.
