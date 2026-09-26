# `pdfcer-gui-base/text/scale`

## Item notes

### `fn the_intro_says_the_numbers_currently_measure_the_paper`

The sentence this whole window exists to deliver. An operator who has
placed dimensions and read numbers has been given plausible answers to
a question they did not ask, and this is where they find out. A test
rather than a convention, because the natural edit when an intro reads
long is to cut its first clause.

### `fn every_unit_is_named_distinctly`

A picker with two identically-labelled rows is a picker whose choice is
a coin toss.


⇒ **A completeness test that carries its own copy of the set is
testing the copy.** It reads `Unit::all()` now, which is the same list
the three unit dropdowns read, so the test and the product cannot
disagree about what the set is.

### `fn the_default_number_style_is_worded_as_a_choice`

"Whatever suits the unit" is a thing an operator can decide they want.
An empty string, or "Default", is a row that looks like a missing value
— and this one is the row most operators will leave selected.

### `fn the_fraction_styles_show_the_fraction`

The operator's ask was to be able to read `55 5/8"` rather than
`55.63"`. A picker offering "eighths" without showing `1/8` makes them
translate in their head at the moment they are trying to match a
drawing.

### `fn the_commit_button_names_what_it_does`

It re-propagates every member's appearance, so dimensions already on the
page change. "OK" would be a button whose blast radius the operator has
to have read the intro to know.

### `fn intro`

**The most important sentence in this window**, and it is a disclosure
rather than an instruction: it says what the numbers mean *right now*.

A fresh group's scale is the tri-state's *never-set* value, so every label
a dimension tool has placed reads in PDF points — a measurement of the
**paper**, not of the thing drawn on it. An operator who has placed
dimensions and read numbers has been given plausible answers to a question
they did not ask, which is worse than no answer, and this is where they find
that out.

### `fn ratio_only_note`

**This string used to say the other path could not be armed at all**, and
it was accurate until 2026-08-17: *"needs a reference line drawn on the
page, which this build cannot arm yet."* That was exactly the gap the
operator reported — *"still missing the feature where we set the scale by
selecting two lines or points and defining what that distance
represents"* — and the gesture exists now.

Kept as a sentence rather than deleted, because a cold dialog really does
have only one path that can produce a scale: no line is drawn yet. What
changed is that it points at the button instead of apologising for an
absence.

### `fn ratio_separator`

A catalog entry rather than a literal for the reason the settings window's
degree sign is one: the ui-strings gate looks for exactly this, and a
translator must be able to see that a separator exists — several languages
write a scale with something other than a colon.

### `fn group_label`

# Why the window needs one at all

The operator's report was *"there is no group dropdown"*, and the reason it
is a defect rather than a missing convenience is that this window changes
**one** group's scale and never said which. It was opened either from the
ribbon (the group being drawn into) or from the Manage-groups panel's *Set
scale...* button (the row that was selected), and those are frequently
different groups -- inspecting a detail group's settings while still
drawing into the plan group is an ordinary thing to want, and the panel
keeps the two deliberately separate for that reason. So a window with no
name on it could be, and sometimes was, aimed somewhere the operator was
not looking.

"Set the scale of" rather than "Group", because the row then reads as a
sentence with the picker as its object, and the one thing the operator must
not have to infer is which drawing this number lands on.

### `fn current_scale`

The operator's report was that the window *"does not show me the scale that
is already set"*. This is the half of the answer he can read; the other
half is that the entry controls are now seeded from it, so the live preview
under them reads the same number back (see
`crate::canvas::measure::scale::ScaleEntryFields::for_group`).

Deliberately phrased as a statement of fact about the document and not as
a warning. The scale that is set is not a problem, it is the context; a
window that greeted an operator with a caution every time they opened it
would be the nagging the ribbon spec forbids, and he has said as much about
the old GUI by name.

The `phrase` argument comes from
[`crate::text::dimension_groups::scale_phrase`] rather than being formatted
here, so this window and the Manage-groups panel cannot come to describe
the same group's scale in two different ways -- including the `NeverSet`
arm, which that function renders as the engine's own
`NO_SCALE_DISCLOSURE` verbatim because a shell is not permitted to
paraphrase it.

### `fn unit_name`

Full names rather than `Unit::abbrev`'s two letters. The abbreviation is
right on a *dimension label*, where space is scarce and the reader already
knows what they are looking at; it is wrong in a picker, where the reader is
choosing and `in` versus `ft` is two characters apart in a list they are
scanning once.

# Exhaustive, deliberately — and this is the one place that is safe

`pdfcer_core::dimension::Unit` is **not** `#[non_exhaustive]`, unlike most
of the engine's public enums, so a variant added there is a compile error
here rather than a silently unnamed row in a picker.


Contrast `crate::text::settings::theme_preset_label`, which *does* carry a
catch-all, because `egui_shell::theme::Preset` IS `#[non_exhaustive]` — the
whole point of the shell crate is that another application may ship presets
pdfcer has never heard of. The two are opposite situations and get opposite
treatment; neither is a style preference.

### `fn fraction_name`

The `None` entry is a **real choice**, not an absence, and its wording
says so. It is what an operator who never opens this control gets, and the
field stores `Option<FractionMode>` rather than re-deriving from the unit
precisely so that an explicit choice survives a unit change — somebody who
asked for eighths does not want them silently reverted by switching from
inches to feet.

The fraction entries say `1/8` rather than "eighths" because that is how a
drawing writes it, and matching the drawing's own notation is the whole
reason this control exists — the operator's ask was *"also want to be able
to choose the units and display type - rounding, fraction, etc."*, made
after a drawing dimensioned in inches always read `55.63"` and could never
read `55 5/8"`.

### `fn preview`

Takes the engine's own `ratio_label` rather than formatting the scale
here. `ScalePreview::ratio_label` is documented as the `/R`-style label —
`1:100`, or `25 ft = 42.3 pt` — and it is DISPLAY-ONLY, which is exactly
what this line is. Formatting a scale in the GUI would be a second
implementation of a string the engine already produces, and the two would
eventually disagree about rounding on a number the operator is checking.

### `fn degenerate`

Reachable by typing a zero into either side of the ratio. Worded as what is
wrong rather than as "invalid", because the operator can see the fields and
the useful half is which one pdfcer could not use.

### `fn accept`

**"Set scale", not "OK".** The button says what it does, because what it
does is larger than it looks: it re-propagates every member's appearance,
so dimensions already on the page change. An "OK" would be a button whose
blast radius the operator has to have read the intro to know.

### `fn calibrate_button`

Not "Calibrate". That is a word from our side of the fence — it names
the operation rather than the action — and an operator scanning this
window is looking for a way to avoid computing a ratio. This says what the
click does.

### `fn calibrate_note`

Names the case it saves the operator from — working the ratio out by hand —
because a button whose advantage is unstated reads as a longer route to the
same place.

### `fn calibrated_note`

The number is shown, and that is a disclosure rather than decoration.
It is the half of the equation pdfcer contributed, and an operator checking
their work needs to see that pdfcer measured the line they meant to pick —
a snap that landed on the wrong endpoint is visible here and nowhere else
once the dialog is up.

Points, because that is the unit the measurement is in and inventing a
friendlier one would mean picking a scale, which is the thing not yet
known.

### `fn real_length_hint_long`

The grammar lives in `pdfcer_core::dimension::parse_length` and is shared
with the CLI, so this describes it rather than defining it — two
descriptions of one grammar is how the GUI and the CLI come to disagree
about what `55 5/8"` means.
