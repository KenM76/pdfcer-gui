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

### `struct Palette`

Every field is a **role**, never a colour name: `accent`, not `blue`.
A theme that wants a green accent should not have to be read as "blue,
except it's green now" — which is exactly what a field called `blue`
forces on the next reader.

This struct is deliberately small and deliberately generic. Anything
that names a concept belonging to one application's domain goes in
[`Overlays`] instead; see this module's header.

### `enum Preset`

`#[non_exhaustive]` is deliberate: a preset added later must not be a
breaking change to anything matching on this, and every consumer
should be routing through [`Theme`] rather than branching on the name
anyway.

### `fn key`

Stable identifiers, never the display name: a display string is
operator-visible text and belongs in the application's string
catalogue, where it can be reworded without invalidating everyone's
saved settings.

### `fn from_key`

`None` rather than a default so the caller can say *"the settings
file asked for a theme this build does not have"* instead of
silently resetting the operator's choice — the difference between a
note and a mystery.

# An obligation on the consuming application

If the application persists a theme choice as a string somewhere
the shell cannot see — a settings file owned by an engine crate,
for instance — then nothing in the type system connects that
literal to this enum, and a rename on either side drifts them
apart. The symptom is a fresh install showing "this settings file
asks for a theme this version does not have" on its very first run:
a message about the operator's file that is really about ours.

The salvage source carried a test for exactly that seam. It could
not come across, because the seam is between the application and
its own storage, not between the application and the shell. **The
application must carry it**, asserting
`Preset::from_key(&its_default) == Some(Preset::default())`.

### `fn apply`

# Call this once per frame, or the palette governs nothing

Without it the appearance is `egui`'s defaults plus whatever each
call site draws on top: the palette reaches the overlays and never
the widgets, which is the two-thirds-of-a-theme failure this module
exists to prevent.

Applied every frame rather than once at startup so a theme change
takes effect immediately, with no restart and no cache to
invalidate. It is a handful of field writes against a struct
`egui` already owns.

# `all_styles_mut`, not `set_style`

`egui` keeps a SEPARATE `Style` per light/dark theme and picks
between them from the system setting. Writing only one of them
would make the application's appearance depend on the operator's
OS theme — so a machine set to dark would show a half-styled
application while the developer's light machine looked correct,
which is the worst kind of appearance bug because it is invisible
where it is being written. Both styles get the same palette, and
the preset alone decides whether the application is dark.

### `fn of`

Falls back to the default if nothing has been stashed — which
happens only before the first [`Theme::apply`], i.e. never during a
painted frame.

### `fn accent_pair`

Returns `(accent, on_accent)`. Use it for an affirmative dialog button,
an active tab, a selected mode chip — any chrome that must look
*emphatically enabled*.

# Why this exists as a function rather than two field reads

Because a fill and a foreground fetched separately are correct
separately and wrong together, and the result is a surface that looks
broken while every gate stays green. Two shapes of that, both real:

1. **`DEFECTS.md` D2** — an active ribbon tab taking `egui`'s
   *selection* visuals and a plate colour meant for content renders
   near-white on light grey. [`Palette::on_accent`]'s own doc comment
   states the root cause: two roles that must vary independently welded
   together.
2. **An affirmative button filled with [`Palette::selection_fill`]** —
   a **27 %-opacity** wash whose real job is tinting selected objects in
   the content area. Over a light panel it composites *paler than an
   ordinary button's opaque fill*, so the default action reads as
   **disabled** and the operator presses it repeatedly, queueing the
   action once per press.

Both values are correctly sourced from the theme, and neither is a
literal, so `tools/gates/check-theme-colors.sh` — which forbids raw
`Color32` outside this module — has nothing to say about either. **The
rule that gate enforces is "no invented colours"; it cannot enforce
"the right role".** A named pair is the mechanism that can: there is one
spelling of *"paint something as the emphasised action"*, and a preset
that changes its accent moves every such surface together.

Not `strong_text_color()` for the foreground. That follows
`override_text_color`, which is the **body text** colour — near-black
under the light presets. On a saturated accent that is poor contrast,
and under a preset whose accent is dark it would be black on black.
[`Palette::on_accent`] is the theme's own answer and inverts per preset.

Deliberately NOT `selection.bg_fill`. That channel carries
[`Palette::selected_plate`], a *diluted* accent chosen to be readable
under `accent` INK. An emphasised action wants the accent at full
strength with [`Palette::on_accent`] on it. Reading the channel gets you
whichever pair it happens to serve, which is the entire argument for
naming this one.

### `fn indeterminate_pair`

Returns `(plate, ink)`: `(`[`Palette::content_backdrop`]`,
`[`Palette::text_muted`]`)`.

# Why this is a named pair and not two field reads

Because **a correctly-sourced colour used for the wrong role passes
every gate this project has.** `tools/gates/check-theme-colors.sh`
forbids invented values; it cannot forbid a wrong role, and that gap is
what makes a wrong-role defect visible to the operator and invisible to
CI (see [`Self::accent_pair`]'s doc comment for the two shapes it
takes).

A caller that needs "a swatch that reads as no particular colour" has to
choose two roles, and the two roles that *look* right on the day are not
the two that stay right across three presets. Naming the pair means
there is one spelling of the question, one place the answer is decided,
and one thing for a preset author to re-tune.

# Why THESE two roles

* **[`Palette::content_backdrop`] as the plate.** Its own doc comment is
  the argument: it is *"the area behind the application's main content …
  the content must read as an object ON something"*, so it is the one
  role in the palette guaranteed to be distinguishable from
  [`Palette::panel`], which is what a Properties panel is drawn on. A
  plate equal to the panel would make the swatch's edge disappear and
  the control would read as *absent* rather than as *indeterminate* —
  and "absent" is the exact wrong reading, because the control works.
* **[`Palette::text_muted`] as the ink.** *"Secondary text: captions,
  hints, counts."* A mixed marker is a statement about the control, not
  a value in the document, so it must not be [`Palette::text`] — which
  is the weight the *values* in the panel are drawn at, and would claim
  the dash was one of them.

Deliberately **not** [`Palette::on_accent`], and
`tools/gates/check-plate-colour.sh` is the reason it would have been
caught: `on_accent` means *ink drawn ON `accent`*, and an indeterminate
control is the opposite of an emphasised one.

Deliberately **not** greyed-out widget visuals either. A greyed
control means *you cannot use this* (R9), and a mixed swatch is fully
usable — picking a colour applies it to the whole selection. Borrowing
the disabled look would tell the operator the opposite of the truth.

### `fn selected_widget_ink`

Returns [`Palette::accent`], which is what
`visuals.selection.stroke.color` carries. Use it only where a call site
hand-draws something *inside* a control `egui` has already styled as
selected, and therefore has to match a colour it did not choose: a
tinted glyph on a toggle, a hand-painted chevron, a custom check mark.

# What to use instead, almost always: nothing

`egui` styles a selected widget correctly on its own — it substitutes
both fills and the text colour out of this channel at paint time
(`egui-0.35.0/src/widget_style.rs:151-154`). A `ui.selectable_label(true, …)` needs no
colour from anyone. Reach for this **only** when you are drawing an
extra mark on top of a plate `egui` painted, because that mark is the
one thing `egui` cannot colour for you.

# What this is NOT

Not [`Theme::canvas_selection_ink`], although today both return
`accent`. That equality is a coincidence of the current palette and not
a contract: one is the ink on a chrome plate, the other is the outline
over a document page, and the whole point of defect T2's fix was that
those two must be able to move apart. Calling the wrong one is how the
canvas silently re-tunes when the chrome is re-tuned.

Not [`Theme::accent_pair`] either — that is the *emphasised action*
pair (`accent` + `on_accent`), a stronger surface than "selected".

### `fn selected_widget_pair`

Returns `(selected_plate, accent)` — bit-for-bit what
[`Theme::write_style`] puts into `visuals.selection`. The pair form
exists for the same reason [`Theme::accent_pair`] does: the two values
are only correct *together*, and a call site that paints its own
selected surface should state both in one breath rather than fetch a
fill here and a foreground there — which is the shape of `DEFECTS.md`
D2.

### `fn canvas_selection_ink`

Outlines, node marks, grips, rubber-band borders, drop carets, ruler
span markers, the selected form field's box — everything drawn *over
the document* to say "this is what you have picked".

# Why this is a named function and not `visuals().selection.stroke`

Because `egui::Visuals::selection` is `egui`'s styling channel for
**selected widgets** — see [`Theme::write_style`], which quotes the four
lines of `egui-0.35.0/src/widget_style.rs` that substitute it into every
`Button::selected(true)`. A theme that points that channel at the
content area paints every selected chrome control with content ink:
accent text on a 27 % wash, luminance gap 72.5 in the Dark preset
against a floor of 90. The channel serves chrome, and only one of the
two claimants can have it.

The standing rule: a correctly-sourced value used for the wrong role
passes every gate, so expose the PAIR behind a purpose-named function.
[`Theme::accent_pair`] is that mechanism for chrome; this and
[`Theme::canvas_selection_fill`] are it for content. A call site asking
for `canvas_selection_ink` cannot accidentally be asking for the chrome
role, because the two questions have different spellings.

"Canvas" here means the application's content area — the region
[`Palette::content_backdrop`] sits behind. The shell has no opinion
about what is drawn there.

# What this is NOT for

**Never for chrome.** A selected dock tab, a toggled ribbon button, a
pressed mode chip, a highlighted menu row: those are widgets, `egui`
already styles them from `visuals.selection`, and if one needs to state
its own colours the pair is [`Theme::accent_pair`]. Using canvas ink on
chrome is the *inverse* of the defect above and produces the same
class of failure — a colour chosen against a background nobody paired
it with.

It also may not be written into a document. See this module's header:
if a colour can end up in a saved file it is not a theme colour.

### `fn canvas_selection_fill`

Returns [`Palette::selection_fill`], which is **27 % alpha on
purpose**: seeing the object through the tint is the entire point of a
selection wash over a drawing. That is also precisely why it is unfit
for chrome — a translucent fill is not a dimmer accent, it is a
different colour over every background it meets — which is how a
dialog's default button renders paler than the Cancel beside it (see
[`Theme::accent_pair`], shape 2).

# What this is NOT for

Not a widget fill, not a button, not a tab, not a plate to draw
[`Palette::on_accent`] on. See [`Theme::canvas_selection_ink`] for the
full argument and for why reaching into `visuals().selection.bg_fill`
is now a gate failure (`tools/gates/check-selection-channel.sh`).

### `fn canvas_selection_pair`

The pair form exists for the same reason [`Theme::accent_pair`] does:
the two values are only correct *together*, and a call site that draws
a washed rectangle with an outline around it should state them in one
breath rather than fetch two unrelated-looking colours. Use it wherever
both are needed; use the singles where only one is.

### `fn rendered_style`

# Why this is public

It is the oracle the contrast gate needs. A test that reads the
*palette* can only check the colours somebody chose; a test that
reads the *style* checks the colours `egui` will actually paint,
including every field the theme forgot to assign — which is the
class of defect `DEFECTS.md` D2 belongs to. Anything that wants to
ask "what will this look like" should ask here, not of
[`Self::palette`].

Built from `Style::default()`, so it deliberately does not carry
an application's font definitions or text styles. It answers a
question about colour and spacing, which is all this module sets.

### `fn check_contrast`

A convenience over [`contrast::check`] and [`Self::rendered_style`]
so an application — or a verification harness — can assert the same
property this module's own tests assert, against a theme it built
itself. See [`contrast`] for what is measured and why.

# Errors

Returns every failing pair, not just the first, so one run names
the whole problem rather than making the caller iterate.
