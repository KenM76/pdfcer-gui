# `egui-shell/theme/mod`

## Item notes

### `fn dark`

`label_backdrop` and `label_text` deliberately stay
dark-text-on-light-plate, because they sit over CONTENT, whose
colour the document decides and the theme does not.

`on_accent` inverts rather than being inherited, and that is the
clearest demonstration of why it is a role of its own: this
preset's accent is a *light* blue, so the readable foreground on
it is near-black. A single shared "light plate" colour — which is
what the salvage source used here — could not express that, and
the contrast gate in [`contrast`] would refuse the preset if it
tried.

### `fn write_style`

# Three invariants this function must keep — `DEFECTS.md` D2

1. **Both fills are assigned for every widget state.** `bg_fill` and
   `weak_bg_fill` are two different backgrounds that different
   widgets choose between: `egui_tiles` tab buttons and
   `CollapsingHeader` headers paint with `bg_fill`, ordinary buttons
   with `weak_bg_fill`. Assigning the accent to only one of them
   leaves a near-white `active.fg_stroke` sitting on `egui`'s stock
   light background — unreadable dock tab labels and section
   headings, at 1×, with nothing in the palette wrong.
2. **All ten fills come from the palette.** A field this function
   does not write keeps `Style::default()`'s value, which is a
   *light-theme* grey — so under the dark preset every unassigned
   fill is an invisible-text site waiting for the right widget. The
   contrast gate can only refuse a theme whose fills it can read;
   what is never assigned is what it cannot see.
3. **The foreground on the accent is [`Palette::on_accent`]**, never
   [`Palette::label_backdrop`]. See that field's doc comment:
   reaching for a content-facing plate colour to serve as chrome
   text is the category error behind D2.

The regression test is
`every_rendered_pair_is_readable_in_every_preset`; its doc comment
explains why a test that reads the palette cannot catch this class.

### `fn clamp_to_i8`

`egui` stores margins as `i8`. A cast alone would wrap a 200 pt
padding to a negative margin, which paints content outside its own
panel — a silent geometry defect from a plausible-looking number. This
saturates instead, so an absurd metric produces an absurd-but-sane
margin that is visible on screen and traceable to its cause.
