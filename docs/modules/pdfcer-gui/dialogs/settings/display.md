# `dialogs::settings::display` — how pdfcer draws, as distinct from what it draws

The eighth group, and the only one whose settings are **not** about the PDF
standard. Every other group in this window exists because a clause declines
to have an opinion; these two exist because a machine has a speed.

## Two settings, out of seven commissioned

`RIBBON_IA.md` §5.2 specified a View ▸ Render group of five, plus two
behaviour settings on the same tab, and `shell::manifest::DIRECTED` carried
all seven as *"named individually, with their value sets and their defaults,
when this shell was commissioned"* — which is a stronger statement of intent
than a status mark, and is why they were emitted despite carrying no `G`.



`DIRECTED`'s own doc comment anticipated this outcome and named the remedy:
*"if it turns out to be wrong, the fix is deleting eight rows from one list
rather than re-deriving which entries were deliberate."* Six rows went; the
two that survived became these controls and left the ribbon, because a
setting belongs in the settings window and `RIBBON_IA.md` §6's own list of
what does not go on the ribbon now has a real destination to point at.

`crate::app::prefs`' header carries the full table with the evidence for
each verdict.

## Why these two are not in the engine's settings file

They are **preferences**, not answers to a silent standard, and this
window's own opening paragraph promises the latter. They live in
`userdata/preferences.txt` beside `settings.txt` — same roof, same
fail-soft parser, different file — for the reason `crate::app::prefs`
states. The group sits in this window because a *window* is where an
operator looks for a choice, and which file a choice is stored in is not
their concern.

## Item notes

### `fn the_shipped_quality_changes_no_raster`

The "a build that omits nothing behaves as it did before" rule, at the
one place it can be checked cheaply. `viewer::raster_scale` was
`zoom × pixels_per_point` exactly before this setting existed, so
`Normal` must multiply by one or every raster in the application
silently changed size the day the control landed.

### `fn the_qualities_ascend`

The control reads left to right as a scale, so a list whose middle
entry was not between its neighbours would be a scale that does not
scale.

### `fn the_shipped_settle_is_reachable_on_the_slider`

A default outside its control's bounds would be silently rewritten the
first time anybody opened this window, on every machine, without a
click. Third instance of this check in the window; third setting with a
range that must be the store's.

### `fn render_quality`

# Why this is a real setting on the drawings this shell is for

The benchmark sheet is 5.6 MB of dense vector site plan, and rasterising it
is the expensive thing this program does. The multiplier is the only control
an operator has over that cost, and both directions are wanted by real
people: someone panning a big sheet looking for a detail wants `Faster`, and
someone reading small text over a hairline grid wants `Sharper`.

The radius line says it affects speed as well as appearance, because that is
the trade being made and a control that mentioned only sharpness would be
describing half of itself.

### `fn page_cache`

# Why this is in *Drawing the page* and not in a group of its own

This window files by the **symptom that brings an operator looking**
(`super`'s header), and the symptom here is *"scrolling back to a sheet
makes me wait"* — which is a fact about how a frame is drawn, exactly like
the two controls above it. A "Memory" group would file it by what it spends
rather than by what it does, and nobody arrives with the symptom *"pdfcer is
using the wrong amount of RAM"*.

# Third rather than first in the group, and it is the newest

Quality and settle are read on every frame by an operator who is *looking at
the page*; this one is read when they are annoyed by a wait they have
already had. That is the same ordering argument the group makes about the
two opening-view controls below: settings that affect what you are looking
at now, then settings that affect what happens next.

### `fn zoom_settle`

# A slider, and linear

Linear because the useful resolution is even: 50 ms against 150 ms matters
about as much as 500 against 600, since both answer *how long am I willing
to look at a soft page*. That is the same argument the parallel-tolerance
slider makes and the opposite of the word-gap one, whose useful range is all
at the low end.

The range is the store's own `MIN_SETTLE_MS..=MAX_SETTLE_MS`, not a local
pair of literals — the third instance of that rule in this window, and it
exists for the same reason each time: a control narrower than what the file
accepts silently rewrites a hand-edited value on open, and the operator
never touched the control.

### `fn opening_fit`

# Why this is offered at all, when a fit command already exists


# A radio group, not a dropdown

Three values, each needing a sentence about what it costs on a large sheet.
A dropdown shows one at a time and hides the comparison, which is the only
thing that makes the choice decidable.

### `fn paste_chords`

# Here as well as on the status bar, and that is not duplication

The status-bar control is the one the operator will use: it sits beside the
page buttons, which is where they are already looking when they are
thinking about pages. This one is how they *find* the choice — and how they
read the two sentences that make it decidable, which a one-word toggle in a
24-point bar has no room for. Both write the same field, so neither can
**Which chord pastes a form field as a new one, and which as a duplicate.**

`OPERATOR_REQUESTS.md` **O58**. Ken, 2026-08-29: *"let's make it an option to
have it swap to match Acrobat or work the way we have it now."*

# Here rather than on a keyboard-shortcuts page, and there is no keyboard page

The shortcuts dialog *lists* bindings; it does not edit them. And this is not
really a question about keys — it is a question about **which of two pastes
is the ordinary one**, which is why the labels name the behaviour and the
chords are the parenthetical rather than the other way round.

# It is in Display because that is where input-gesture preferences already live

Beside `wheel_paging`, which is the same shape of question: an operator
deciding what a familiar input should mean in this program. Neither is about
what the document *is*, which is what keeps them out of Saving and Pages.

### `fn field_shade`

# One setting with three switches, not three settings

[`widgets::toggle`]'s own documentation carries the control-shape argument.
The reason they are **one setting** is different and is about the operator
rather than the widget: the three interlock. A guide is dragged out of a
ruler gutter, so `guides` without `rulers` is a switch that appears to do
nothing. That relationship needs saying once, in a place all three readers
will be looking — which is what a single header and a shared disclosure buy.

# The disclosure is not a note under the guides switch

It is true whichever way that switch is set — a document with remembered
guides opens with them showing either way — so it belongs to the setting
rather than to one of its parts. Putting it under the switch would make it
read as an argument for turning guides on, which it is not; it is a fact
about what pdfcer will do regardless. Same distinction the replacement-text
bound makes, and `widgets::disclosure` exists for exactly this.
**Shade the fillable fields** — `OPERATOR_REQUESTS.md` O96.

In *Display* rather than in a Forms group, which is where he asked for it
(*"in our display section"*) and is also right: it changes nothing about the
form and everything about what the page looks like. An operator turning it
off is tidying their view, not altering how fields behave.

[`crate::app::prefs::Prefs::shade_form_fields`] carries why a wash over part
of a page is an affordance rather than the content marking rule 4 forbids.

### `const OCR_RESET_REGION`

The Reset button's rect, declared only while Reset is drawn, so a check can
read its absence as "the colour is the default".

### `fn ocr_colour`

In *Display* beside the field wash, because it is the same kind of
answer: a colour pdfcer paints over the page that never reaches the file.
Not in *Appearance*, which is about how the program itself looks — a
preset and a theme, neither of which knows anything about a particular
scan. `crate::app::prefs::Prefs::ocr_layer_colour` carries why choosing a
colour for invisible text is not the content marking R8b forbids.

# The reset is ABSENT rather than greyed when there is nothing to reset

R9's rule, and here it also carries information: a swatch with no button
beside it *is* the default, so the control answers "have I changed this?"
without a second sentence. A permanently greyed button would answer it
only on hover, and would suggest the colour could not be put back.

### `fn auto_hide`

One header over both, because they are one decision the operator makes
twice: *"how much of the window do I want the drawing to have?"* Two
separate sections would ask it twice and would put the sentence that makes
the feature safe — the drawing does not move — under only one of them.

In *Display* beside the page chrome, and not in *Appearance*: Appearance
is about how pdfcer LOOKS (its preset, its colours), and this is about how
much of the window the drawing gets. `field_shade` was filed here by the
same test and the operator put it here himself.

The Settings window is also where the **state** of these two lives, which
is why they are checkboxes here and plain commands on View ▸ Window: see
`shell::commands::catalog::view`'s note on why neither ribbon control
renders pressed, and on what Office does.
