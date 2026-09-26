# `ui-verify/checks/settings_theme`

`settings_theme_takes_effect` — the second half of `DEFECTS.md` **D10**,
proved in pixels.

# The defect

D10 was *"the theme system is built, tested, gated, and never installed"*.
Three presets, a palette, a role per colour, a rendered-pair contrast gate
over all five widget states, and its own self-test — compiled into every
shipped binary and never handed to the `Context`. Every colour an operator
had ever seen in this shell was `egui`'s stock light style.

It had two halves — `Theme::apply` never being called, and there being **no
way to choose a preset** — and either half alone leaves the operator looking
at `egui`'s stock style. Both are wired now; this is the check that says so,
in pixels, every run.

# ★ What this measures, and why nothing cheaper would do

**A rendered pixel, before and after, from the running program.**

Every cheaper oracle was already available and every one of them was green
throughout D10's shipped life, which is the whole lesson of that defect:

| oracle | why it passed anyway |
|---|---|
| the theme gate | asserts every colour is a *named role in the theme module*. True of the source whether or not the theme is installed. |
| the contrast gate | renders pairs **from the theme** and measures those. Never asks the application what it drew. |
| a unit test on the picker | would assert the token changes. A token is not a colour. |
| `FEATURES.md` | ticked the row. Three themes an operator could not reach were ticked for their whole shipped life. |

D10's own summary of that is the sentence this check exists to make false
next time: *"No test saw it."*

So the assertion is a **difference between two captures of one window**. The
window's own body region is sampled with the light preset in force, the Dark
radio is clicked, and it is sampled again. Two claims follow:

1. **The colour moved.** A picker that writes a token nothing reads leaves
   this identical.
2. **It moved the right way** — the second sample is substantially *darker*.
   A picker wired to the wrong preset, or a repaint that happened to change
   an unrelated pixel, moves it some other way.

Claim 2 is what stops claim 1 being satisfied by noise. Neither is stated as
an exact colour: the composited value depends on the window's opacity, the
platform's rounding and whatever the operator's own settings file says, and a
check that pinned `#2B2B2B` would fail on a machine where the theme worked.

# Why the sample is the window body and not a swatch

Because the failure D10 records is that the framework's chrome and `egui`'s
widgets painted from **two different palettes** — `apply` both writes the
styles and stashes the `Theme` where `Theme::of` retrieves it, and only the
second was missing. A swatch drawn by this dialog would prove the dialog
knows the preset. The window's background is drawn by `egui` from the style
`apply` wrote, so it proves the *installation* happened.

# What this check does NOT do

**It does not press Save.** The theme is deliberately the one setting that
takes effect on the draft, before Save, because a theme cannot be judged
from a radio label — and that is exactly the behaviour under test. Pressing
Save would additionally write the operator's real `settings.txt`, which a
harness has no business doing: this check runs on a developer's machine
against their own configuration, and a verification tool that silently
changes what it verifies is not a verification tool.

It closes the window with the ✕, which the application treats as a Cancel,
so the session ends on whatever the operator had.

# ★★★ The third assertion, and why it is in this file

Three holes sit in this file's subject. Two are about the presets
themselves and are answered next door in [`super::theme_page`], whose
header carries them; the third is here, because it is about this check's
own window. All three are the same species: **a thing the suite believes
and has never driven.**

## 3. Cancel really reverting a live theme change — a SILENT one-line coupling

The theme is the one setting in that window that takes effect **before
Save**, because a theme cannot be judged from a radio label. The mechanism is
one line in `app/frame.rs`:

```ignore
let theme_token = self.settings_draft.as_ref()
    .map_or(self.settings.theme.as_str(), |draft| draft.working.theme.as_str());
```

`Draft`'s own header states the consequence as the design: *"Cancel drops the
draft, so the look reverts with it — no separate undo path, and nothing that
can get out of step."*

Break that line the other way — read the draft and never fall back, or adopt
the token on close — and **nothing looks wrong**. The window shuts, the
application is dark, and the operator who was only *trying it on* has an
appearance they did not choose and no undo. Nothing on screen says a setting
was applied; nothing on disk changed either, so a restart silently undoes it
and the operator learns the program is unpredictable. The failure is
invisible in exactly the way `DEFECTS.md` D10 was.

[`SettingsThemeTakesEffect`] already opens the window, already measures it,
and now measures **three** pictures instead of two: light, dark, and light
again after the window is dismissed.

### ⚠ It dismisses with Escape, not with the Cancel BUTTON, and that is a gap

**`dialogs::settings` publishes no `ui-rect` for its Cancel button.** There
is nothing to aim at, and this check will not compute a coordinate for one:
the button beside it is **Save**, which writes the operator's real
`settings.txt`, and a harness that guesses at a button position and lands one
control to the left is a harness that silently rewrites the configuration of
the machine it is verifying. That trade is not close.

What is driven instead is the route `app::settings_window` documents as
**contractually identical**:

> If closing by ✕ did anything different from closing by Cancel — kept the
> draft, half-applied it, saved it — the window would have two exits with two
> meanings and no way to tell which one an operator took. **Both paths drop
> the draft.**

Escape is read from the child window's own input (`dialogs::host`) and sets
the same `frame.closed`, which the host turns into `settings_draft = None` —
the identical assignment `Outcome::Cancel` makes. So the coupling under test
is the one that ships.

★ What this does **not** cover, stated plainly so nobody reads more into a
green run than is there: a future edit that made `Outcome::Cancel` alone
behave differently — saving, or adopting the theme — would pass this check.
Closing that requires one line in the application:
`crate::diag::ui_rect("settings.cancel", <the Cancel button's rect>)` beside
the existing `REGION_BODY` declaration. This crate may not write it.
