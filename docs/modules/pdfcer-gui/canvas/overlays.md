# `canvas::overlays` — the application's own colour roles, published per
frame

## The gap this closes

`FEATURES.md` has carried it as a ⬜ row since Phase 7:

> **The snap indicator has no colour yet** — `Overlays` is never installed
> by `pdfcer-gui` (`grep Overlays crates/pdfcer-gui/src` → zero hits), so
> `snap_indicator_tint` returns `None` on every frame and the marker falls
> back to the selection stroke. The theme wiring still owes an `Overlays`
> set beside `Theme::apply`, plus `assert_distinct` over `preview` vs
> `dimension_selected` per preset.

Both halves are here: the set, and the test.

## ★ Why the roles are defined HERE and not in `egui-shell`

**R7.** `egui-shell` never learns what a PDF is, and `"dimension_selected"`
is a pdfcer concept wearing a colour. `egui_shell::theme::Overlays` is
therefore a *generic role map* — a `BTreeMap<String, Color32>` with an
install/read pair and a collision check — and the roles in it are the
application's, exactly as the ribbon manifest's command ids are.

The shell's own docs name `preview` and `dimension_selected` as an example.
That is prose in a doc comment, not a dependency, and
`tools/gates/check-shell-purity.sh` scopes to `pdfcer-*` crate and module
names for that reason.

## ★ Why every colour comes from the palette, and none is a literal

`tools/gates/check-theme-colors.sh` forbids a raw `Color32` outside
`crates/egui-shell/src/theme/`, and the reason is the one this module would
otherwise re-create: a colour chosen here would be right on the preset it
was chosen under and wrong on the other two, silently, because nothing
renders all three at once.

So each role names a **palette entry whose meaning already matches**, and
the mapping is the argument:

| role | palette entry | why that one |
|---|---|---|
| `preview` | `notice` | *"Something is worth knowing and nothing is broken"* — which is precisely what a snap marker is. It is a **proposal**: pdfcer saying *here is where I think you are pointing*, before any click has committed anything |
| `dimension_selected` | `accent` | what every other selection in this application is drawn in. A committed ce dimension that is selected is selected, and inventing a second selection colour for one object kind would be a cue that means nothing anywhere else |

## ★ The pair must stay DISTINCT, and that is the test this module owes

`egui-shell`'s own `overlays.rs` states it and cannot enforce it:

> the measurement preview and the committed dimension differ because one is
> a proposal and one is document state […] a theme that merges two roles
> removes a cue that was doing work, and it would do so silently.

[`tests::the_preview_and_committed_roles_are_distinct_on_every_preset`] is
that check, run over `Preset::ALL` rather than over the default — a preset
is exactly where two roles collapse into one, and the two that are not the
current default are the two nobody looks at.
