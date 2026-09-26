# `app::settings` — the live configuration, and the funnel that makes it real

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
   `app/blank.rs` — see [`tests::no_call_site_builds_its_own_options`] for
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
