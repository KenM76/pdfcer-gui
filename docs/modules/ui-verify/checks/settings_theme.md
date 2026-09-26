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

# What this measures, and why nothing cheaper would do

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

# The third assertion, and why it is in this file

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

What this does **not** cover, stated plainly so nobody reads more into a
green run than is there: a future edit that made `Outcome::Cancel` alone
behave differently — saving, or adopting the theme — would pass this check.
Closing that requires one line in the application:
`crate::diag::ui_rect("settings.cancel", <the Cancel button's rect>)` beside
the existing `REGION_BODY` declaration. This crate may not write it.

## Item notes

### `const CENTRAL_PANEL`

Sampled by the Cancel half of [`SettingsThemeTakesEffect`], which runs with
**no document**: with nothing open the central panel is the application's
own surface and takes the preset's colour directly, so it is the cheapest
honest witness that a live theme change reached the application window
rather than only the dialog that chose it.

### `const APPEARANCE_HEADING`

**It has to be clicked, because that group is CLOSED when the window
opens** — and that is a deliberate design decision rather than an oversight,
so the check accommodates it rather than the application being changed to
suit the check.

The window collapses thirteen settings into seven subject groups because an
operator arrives with a *symptom* and the headings are how a symptom finds
its setting. Exactly one starts expanded, and it is **Colour** — it holds
the setting most likely to have brought someone here and the only default
that knowingly differs from other PDF viewers.

This check found the consequence on its second run: the window opened, the
body published its rect, and no theme radio existed, because a collapsed
`CollapsingHeader` does not run its body closure at all. The failure message
said *"the window has no theme picker in it"*, which was a true statement
about the frame and a false one about the build.

### `const MIN_DARKENING`

The `quiet` preset's panel is a light grey around `#E8E8EA`; the `dark`
preset's is in the forties. That is a gap of roughly 190 per channel, so a
floor of **60** is comfortably inside it while being far outside anything a
selection highlight, a focus ring or an antialiasing difference produces.

Stated as a *minimum drop* rather than as a target colour deliberately — see
the module header on why pinning an exact value would fail on a machine
where the feature works.

### `const LIGHT_START_FLOOR`

A guard against a vacuous run, and against the worse thing a vacuous run
does here: a **false defect report**.

This check launches the operator's own binary against the operator's own
`settings.txt`, and if that file already says `theme = dark` then the window
opens dark, clicking Dark changes nothing, and every assertion below reads
as *"the picker writes a token nothing installs"* — `DEFECTS.md` D10, filed
against a build in which the feature works perfectly. A fixture cannot
defeat this, because the condition is not a default but a **starting
state**.

140 sits above the darkest light preset (`quiet`'s surface measures ≈ 241)
by a hundred levels and above the lightest dark one (`dark`'s surface
measures ≈ 38) by the same again, so no plausible palette lands on it.

### `const MAX_REVERT_DRIFT`

**Zero is the expected reading.** The theme token after Cancel is the same
`String` it was before the window opened, so the same style is written, the
same fill is painted, and a lossless BGRA capture of it differs by nothing
at all. This is not a tolerance for a real difference; it is a tolerance for
the sampler, which reports the *mean of a quantised bucket* and can therefore
move by a level or two if a scrollbar, a focus ring or a hover highlight
falls inside the region on one capture and not the other.

**6** is one quantisation bucket ([`crate::pixels`] quantises to 5 bits, so a
bucket is 8 levels wide) and is two orders below the ≈ 200 that separates the
light presets from Dark — so a Cancel that failed to revert cannot hide under
it, and a repaint artefact cannot trip it.

### `fn mean_channel`

Every luminance claim in this file is stated as a mean channel rather than as
a perceptual luminance, and deliberately: the question here is *"did this
surface change colour"*, not *"can this be read"*. [`crate::pixels`] answers
the second, weights the channels for the eye, and would report a green and a
blue of equal brightness as the same — which is exactly wrong for detecting
a theme that tinted something.

### `fn open_the_theme_picker`

`Ok(None)` means the theme radios are on screen and clickable. `Ok(Some(_))`
is a FAILURE message — a control that should be there and is not. `Err` is a
SKIP: the harness could not deliver a click, or the application never said
anything, and neither is a verdict on the feature.

# Why this is shared and `markup_rectangle`'s equivalent is not

`checks::driving`'s header records the rule: a move is shared when the two
callers would otherwise **drift apart while both passing**. These two are the
same three clicks at the same three regions, and every failure message names
a specific application constant — `file.settings`, `dialog:settings`,
`settings.heading.appearance`. Two copies would mean two places to update
when one of those is renamed, and the copy nobody updated would go on
reporting *"the window has no theme picker in it"* about a window that has
one. That is the failure this suite has already recorded twice.

It stays inside this module rather than moving to `driving` because nothing
outside this file opens the Settings window, and a helper hoisted before it
has a second caller is a helper whose shape is guessed.
