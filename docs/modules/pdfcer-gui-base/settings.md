# `pdfcer-gui-base/settings`

## Why this module exists, and it is not "to hold a struct"

`pdfcer_core::settings::Settings` is the operator's answers to questions
the PDF standard declines to answer. Loading them is easy; **honouring**
them is where the old shell failed, and it failed silently.

⚠ That sentence read *"thirteen operator choices"* while the struct
carried twenty-three, and the drift was invisible because a number in
prose fails nothing. Every completeness claim in this module is now
phrased against the type, and `tools/gates/check-settings-funnel.py` is
what enforces it.


The mechanism is not a bug anyone wrote. It is what happens when option
structs are built at the call site:

```text
ExtractOptions::default()      →  word_gap_ratio, unmappable_code, actual_text
RenderOptions::default()       →  mask_resample, image_minify,
                                  cmyk_jpeg_polarity, missing_as
SaveOptions::identity()        →  xref_entry_eol, trailing_eol
```

Every one of those constructors is correct in isolation and every one of
them silently discards the operator's configuration. There were twelve such
call sites in the old crate and there are fifteen in this one.

The irony worth recording: `xref_entry_eol`'s whole *default* was changed on
an operator ruling, because a fixed `SP LF` produced a 10,000-byte diff on a
file nobody had edited — and the GUI could not honour anything but the
default anyway.

## The funnel

Three functions — [`Settings::extract_options`], [`Settings::render_options`]
and [`Settings::save_options`] — and a rule: **no code in this crate
constructs those three types itself.** The rule is not a convention; it is
checked, by [`tests::no_call_site_builds_its_own_options`], which parses
every `.rs` in the crate with `syn` and fails on a bare constructor outside
this module.

A grep would not do. `ExtractOptions::default()` appears in a dozen doc
comments in this crate — including several in this very header — and a grep
would count each one as a violation, or be loosened until it counted none of
the real ones. The same argument `shell::commands::reach` and
`redact::sealed` already make: the thing being counted is a *call*, and a
syntax tree contains no comments at all.

### The three exemptions, and why each is not a hole

1. **Tests and fixtures.** A test that pins the engine's own default
   behaviour must be able to say `ExtractOptions::default()`, or it is
   testing the operator's configuration instead of the engine's contract.
   The check skips `#[cfg(test)]` modules, `ocr/fixture.rs` and
   `pdfcer-gui-base/src/blank.rs` — see [`tests::no_call_site_builds_its_own_options`] for
   each one's argument.
2. **`with_provenance(true)`.** Text editing needs provenance, which no
   setting controls. It is a *modifier* on the funnel's output rather than
   a second construction: `settings.extract_options().with_provenance(true)`.
3. **Redaction's `SaveOptions::identity()`.** Deliberately NOT funnelled,
   and this is the interesting one — see [`Settings::save_options`].

## The funnel also guards a field it deliberately never SETS — O137

[`Settings::render_options`] does not mention `stroke_display`, and that
silence is the feature. `RenderOptions::default()` is
`StrokeDisplay::Actual` — faithful widths — so every path that builds its
options here (image export, DXF, print, print preview, the page thumbnails,
the clipboard) renders the document as it says it should be rendered,
without any of them having to remember to say so.

`view.line_weights` — the CAD *line weights off* display mode the operator
asked for by name — is therefore **not** routed through this funnel. It is
carried on the `RenderRequest` and assigned in exactly one function,
`render::worker::render_on_worker`, which draws the interactive canvas and
nothing else.

> **The one thing worse than not having that feature is having it follow him
> into a file he sends a client.**

⇒ So this module holds **two** rules rather than one, and they point in
opposite directions:

| rule | check |
|---|---|
| nobody but the funnel BUILDS these option structs | [`tests::no_call_site_builds_its_own_options`] |
| nobody but the canvas worker SETS `stroke_display` | [`tests::only_the_canvas_worker_sets_stroke_display`] |

Both are `syn` scans over every `.rs` in the crate, for the same reason: the
identifier appears in a dozen doc comments — including this paragraph — and
a syntax tree contains no comments at all.

## What is deliberately not here

**A watcher on the settings file.** `pdfcer-core` refuses one, and the
reason binds the shell too: live configuration that depends on when an
editor happened to flush is a source of irreproducible behaviour, not a
feature.

**A save on exit.** `save` is called deliberately, from the Save button, so
a crash cannot persist half a session's accidental state and an operator's
hand-edited file is never rewritten behind their back with pdfcer's own
formatting.

## Item notes

### `fn extract_options`

# The three fields, and the one that is a correctness knob

- `word_gap_ratio` decides where extracted text gets its spaces.
- `actual_text` decides how far a document's own replacement text is
  trusted over the glyphs drawn.
- `unmappable_code` decides what stands in for text pdfcer cannot read —
  and it is **not** a cosmetic choice. Downstream of extraction sit
  search, clipboard copy and **redaction-by-text**. Changing the
  sentinel changes character offsets, therefore changes which runs a
  redaction pattern matches. `pdfcer-core`'s R35 states it plainly: *a
  redaction built under one value is not equivalent under another.*

That last point is why both this and `actual_text` have radius lines in
the settings window that name redaction, which the old shell's did not.

# Why fields rather than builders

`ExtractOptions` exposes no `with_word_gap_ratio` / `with_unmappable_code`
/ `with_actual_text` — checked, not assumed. The fields are `pub` and
the struct is `#[non_exhaustive]`, so the only legal shape out of crate
is *start from `default()` and assign*. Assigning after `default()` is
what `clippy::field_reassign_with_default` complains about, which is why
the binding is `let mut options` on its own line rather than a struct
expression: the lint is about the pattern that *looks like* a struct
literal and is not, and `#[non_exhaustive]` makes the real literal
illegal here.

### `fn open_session`

# The finding this exists for, and it is the defect this module was
written to prevent — one channel later

This module's header enumerates the three **option structs** that
silently discard the operator's configuration, and
[`tests::no_call_site_builds_its_own_options`] parses every file in the
crate to keep them funnelled. All of that was correct and all of it was
blind to a fourth channel: a setting applied by a **method on the
session** rather than by a field on an options struct.

`Settings::quad_point_order` is one such. `EditSession::new(doc)` takes
the engine's default; nothing here called `set_quad_point_order`; so an
operator who chose *counterclockwise* in Settings > Saving got reading
order in every markup annotation this shell has ever authored. The
engine had already found the same defect on its own side and shipped
the setter to fix it, with the sentence this shell should have read:

> **A setting is a promise.** Storing one that does nothing breaks it
> silently, which is worse than not offering the choice.

⇒ The lesson is about the SHAPE of the guard, not about this field.
A funnel keyed on *constructors* cannot see a setting delivered by a
setter, and the check that enforced it reported green throughout. The
check now forbids `EditSession::new` outside this file for exactly that
reason — see its own doc comment.

# What it applies, and what it deliberately does not

**Every `Settings` member `EditSession` has a setter for**, which today
is `quad_point_order`, `widget_tab_tail` and `tab_row_tolerance`. The
rest of `EditSession`'s setters take their operand from a gesture, not
from the configuration, and do not belong to a session's opening.

⚠ This paragraph read *"`quad_point_order`, and nothing else, because
that is the only member of `Settings` with a session-level setter"*, and
backed it with a setter count. Both halves went stale together: the
engine grew the two `/Tabs` settings, the count went from fifteen to
twenty-nine, and the two controls in Settings › Forms were stored and
never reached a session — so an operator who chose a `/Tabs` tail rule or
widened the row tolerance got the engine's default in the very tab ring
he set them for. `check-settings-funnel.py` derives the pairs from the
pinned engine's own source rather than from a sentence here.

`set_tab_row_tolerance` clamps to a range and rejects a non-finite
value, so the setter is also the only honest way in: assigning the field
would skip a guard the engine wrote for a real failure, since a `NaN`
tolerance makes no two annotations ever share a row.

# What the setting actually changes, so the disclosure can be honest

Only the `/QuadPoints` **array**. The baked `/AP` appearance stream is
byte-identical under both orders, so no reader that honours the
appearance can tell — it changes what a consumer that re-derives
geometry from `/QuadPoints` sees, which is exactly the population
§12.5.6.10's ambiguity is about. Getting it wrong draws a bow-tie.

Existing annotations are **not** rewritten: this governs what the
session authors from now on. A preference change is not an edit, and
sweeping a document because a setting moved is the unrequested
normalisation `ARCHITECTURE.md` §5 forbids.

### `fn render_options`

# What it applies, and one deliberate absence

**Every member of `Settings` that `RenderOptions` has a builder for.**
Stated against the two types on purpose: this heading read *"Five
settings"* while the chain assigned six, and four more were offered in
Settings › Colour, written to the settings file, and discarded by
every rasterisation the shell had ever done. The count was wrong before
those four existed and nothing failed either time, which is why the
guarantee lives in `tools/gates/check-settings-funnel.py` and not in
this sentence.

**Annotation scope is NOT set here**, and that is the absence worth
stating. Whether annotations are drawn is a property of *what is being
rendered for* — the canvas draws them, a print job may not, an export
may be asked either way — and it is passed at the call site. Folding it
in here would give the canvas and the print preview one answer, which is
the opposite of what they need.

# `missing_as` reaches paper, not just the screen

It decides what a form control with no stated appearance state looks
like, and the print path renders through this same function. An
operator checking a form before printing it is exactly who that setting
is for, which is why its radius line is the only one that separately
names printing.

### `fn a_fresh_install_matches_other_viewers`

`OPERATOR_REQUESTS.md` **O52**, and it is the assertion that says the
operator got what he asked for rather than that a function exists.

It asserts on the value a fresh install actually receives — the
engine's default put through `colour_default` — which is the only claim
worth making while two crates disagree about what the default is. A test
that checked `Settings::default()` would be testing `pdfcer-core`, and a
**This test outlived the function it was written for, and that is
the point rather than an accident.**

It was written on 2026-08-28 against `app::settings::colour_default`, a
three-line seed this shell carried because `pdfcer-core`'s default was
still `NeutralBlack` and O52 had reversed the operator's earlier ruling.
That function shipped with a `debug_assert_ne!` tripwire whose message
said *"delete it and its call site"*.

**`Pass 153.0` landed the same day and the tripwire fired.** The seed is
gone, its call site is gone, and this assertion now reads the engine
directly — which is what it was always about. What the operator asked
for was *"a fresh install opens on Match other PDF viewers"*, and that
claim is worth a test whichever crate is responsible for making it true.

⇒ A test written against a temporary mechanism should assert the
**outcome**, not the mechanism. This one did, so removing the mechanism
cost one line.

### `fn every_field_moved`

`Settings` is `#[non_exhaustive]`, so a struct expression is illegal out
of crate and this is the only shape available: start from the default
and assign. That is also exactly what the funnel's own implementations
have to do, so the awkwardness is shared rather than incidental.

### `fn the_session_funnel_applies_the_operators_quad_point_order`

It asserts **both** values, and that is not symmetry for its own
sake. Asserting only `Counterclockwise` would pass on an implementation
that hard-coded it, which is the same defect wearing the other value;
asserting only the default would pass on the broken build this replaced.
The pair is what makes it a test of the *wire* rather than of a value.

The document is the blank template rather than a fixture from disk,
because the subject is the session's configuration and not its content —
and a test that read a file would fail for reasons that have nothing to
do with what it asserts.

### `fn every_setting_reaches_the_options_it_configures`

Nine of thirteen settings in the old shell were persisted, shown, edited
and never read. This asserts that every field the funnel is responsible
for actually reaches the option struct it belongs to — for all ten of
them at once, from one non-default `Settings`.

It compares against the value **set**, not against a hard-coded
expectation, so it cannot go stale if an engine default moves. And it
asserts each field individually rather than comparing whole structs,
because a whole-struct comparison would need a second construction and
would then be asserting that two copies of the same code agree.

### `fn default_settings_change_nothing_about_the_engines_own_defaults`

The other half of the property, and not a tautology: a funnel that
accidentally *forced* a value — say by writing `MaskResample::Nearest`
as a literal instead of reading the field — would pass the test above
whenever the operator happened to want that value, and would pin the
application to one answer forever. This catches it by asserting the
funnel is transparent when it has nothing to say.

### `fn visit_item_fn`

A test-gated free function compiles to nothing in a release
build, exactly as a test-gated module does, and this crate has
two of them — `app::state::open_fixture` and its sibling — which
exist so a dozen test modules share one way of opening a
fixture. They were found by this check the moment
`EditSession::new` joined the forbidden list, which is the check
working: the *reason* they are allowed is the one already
written for modules, and it had simply never been reachable
before, because no forbidden constructor had ever appeared
outside a `mod tests`.

### `fn every_export_path_renders_real_widths_with_line_weights_off`

# The vacuous shape this deliberately avoids

A test that "exports are unaffected" passes trivially if it never turns
the mode on. So this turns it on — through the real
[`crate::viewer::ViewState`], on a real opened document — and then asks
the **funnel** what an export would be given. The funnel is what
`app::actions::export`, `dialogs::print`, `clipboard::place` and
`panels::pages::thumbnails` all build from, so one assertion covers
every one of them without four copies that could each be got wrong.

The companion assertion is the load-bearing one: the same document's
**canvas** request must carry `Hairline` at the same moment, and its
render key must agree with it. Without those this test would also pass
on a build where the feature does nothing at all.

It also runs the two builders the print and export paths actually
chain onto the funnel's output — `with_annotation_scope` and
`with_backdrop` — because a builder that reset the field would defeat
everything above and is invisible from this side otherwise.

### `trait SettingsExt`

A **trait on the engine's type** rather than a wrapper struct. The engine's
`Settings` is `#[non_exhaustive]`, so a wrapper would have to re-expose
thirteen fields by hand and would go stale the day a fourteenth arrived —
whereas an extension trait grows only where it must, which is in the three
option builders below.
