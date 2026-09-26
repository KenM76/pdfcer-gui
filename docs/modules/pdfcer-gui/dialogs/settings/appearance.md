# `dialogs::settings::appearance` — the theme picker

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
