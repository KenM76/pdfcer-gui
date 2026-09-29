# `settingspages::appearance` — the theme picker

One setting, and the only one in the window that is not about the PDF
standard at all.

## This closes the second half of `DEFECTS.md` D10

D10 was *"the theme system is built, tested, gated, and never installed"*:
three presets, a palette, a role per colour, a rendered-pair contrast gate
over all five widget states and its own self-test — compiled into the binary
and never handed to the `Context`. Every colour an operator had ever seen in
this shell was `egui`'s stock light style.


> There is also **no way to choose a preset**: the settings dialog is one of
> the unsalvaged Class-B surfaces, so even once `apply` is wired, the preset
> is whatever the code picks until that dialog lands.

`app/mod.rs`'s install site said the same thing from the other end, and
called its hard-coded preset *"a placeholder in the honest sense: the
mechanism is real and reachable, and only the chooser is missing."* This is
the chooser.

## Why the token is a `String` and not an enum

`Settings::theme` is an **opaque token** in `pdfcer-core`, deliberately.
The set of themes is a shell concern and `pdfcer-core` must never gain a GUI
dependency — the invariant that keeps a future WASM fork a shell swap rather
than a rewrite. Core stores and round-trips the token and takes no view on
it; `egui_shell::theme::Preset::from_key` is what turns it into a theme, and
it returns `Option` rather than a default **so the caller can say** that the
file asked for something this build does not have.

That `Option` is the entire reason [`theme`] is not three lines long.

## Item notes

### `fn every_offered_preset_round_trips_through_its_token`

The bridge this module is: a preset whose key `from_key` does not
recognise would render as a radio nobody can select — click it, the
token is written, `from_key` returns `None` next frame, and the window
shows the unknown-theme sentence about a theme it has just offered.

### `fn the_shipped_default_token_is_a_theme_this_build_has`

`pdfcer-core` writes `"quiet"` as a **literal** in `Settings::default`,
for the layering reason in this module's header — core may not name a
shell type. A literal is exactly what can drift, and this is the test
that catches it: a fresh profile must not open showing the
unknown-theme disclosure.

### `fn an_unknown_token_is_not_silently_replaced`

The property the disclosure promises. A draft carrying a token from a
newer pdfcer must still carry it after the window has drawn — this test
covers the data half; the sentence is covered in the catalog.

### `fn theme`

# Why this cannot use [`widgets::option`]

That helper compares a `&mut T` against a `T` by `PartialEq`, which requires
the stored type and the offered type to be the same. Here they are not: the
store holds a `String` token and the window offers a `Preset`. Mapping in
both directions inside a generic helper would mean a second type parameter
and a pair of closures at every call site, to serve one setting.

So it is hand-rolled, and kept honest by being *short*: read the current
token once, draw a radio per preset, write the token back on a click.

## The dead store the source carried, removed

The original wrote `selected = true;` inside the click arm and then
`let _ = selected;` to silence the unused-assignment lint — a write nothing
reads, plus a second statement to hide that fact from the compiler. Two
lines whose combined effect is nothing. The radio's selected state is a
pure function of the token, so it is computed inline and there is no local
to leave stale.

### `fn ui_scale`

# Why it is in this group rather than with the other preferences

The other four preferences live in *Drawing the page*, and this one does
not, because the window's groups are a **navigation model**: an operator
arrives with a symptom and the heading is how the symptom finds its
setting. The symptom here is *"the program's text is too small to read"*,
which is a question about the window, not about the page — and Appearance
is the group of settings that change **the program's own appearance and
nothing about the document.** Theme is the other one.

It also means the group's two members are the only two settings in the
window that preview live, which makes that exception legible in one place
instead of scattered.

# A slider, not a radio group

Unlike every enum in this window there are no named alternatives to
compare: the values are a continuum and the right one is *the one at which
this operator can read this screen*. There is nothing to explain per value
and everything to try. Twenty-five steps is far too many radios and exactly
the right number for a drag.

The range and the step are the store's constants rather than local
literals, for the reason repeated at every slider in this window: a control
narrower than what the file accepts silently rewrites a hand-edited value
on open, and the operator never touched the control.

# Why the value is shown as a percentage

Because "125 %" is a quantity an operator can compare against the Windows
display setting they already know, and "1.25" is a number they have to
interpret. It is the same value; the suffix is doing the explaining.
