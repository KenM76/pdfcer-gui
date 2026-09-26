# `canvas::markup::linestyle` — solid or dashed, on all three surfaces that
ask

`RIBBON_IA.md` §5.8's Markup row lists eight controls. Seven shipped on the
morning of 2026-09-06; **Line style** was the eighth, and it was the only
entry in the whole row with *"no engine verb at all"*. That stopped being
true the same afternoon, when `pdfcer-core` answered this shell's own
request with three halves rather than the one it asked for:

| half | engine | what it means here |
|---|---|---|
| **preserve** | a restyle that does not mention `dash` keeps one, *including a dash pdfcer never authored* | this module has **nothing to build** for it, and the fact that leaving the control alone is safe is what makes [`DashReading::Foreign`] legitimate |
| **author** | `MarkupOptions::dash` | [`super::pen::Pen::dash`] and the Markup ▸ Style chooser |
| **restyle** | `MarkupStyle::dash: Option<StyleEdit<BorderDash>>` | the Format ▸ Markup chooser and the Properties panel row |


## Why one module and not three controls

Because *what a dash is* has to be spelled once. Three surfaces offer this —
the pen that authors, the ribbon band that restyles, and the Properties panel
that restyles — and if each wrote its own list of patterns then an operator
could draw a mark at a pattern no restyle control could put back, or restyle
one to a pattern the pen could never have produced. The two width controls in
this shell already assert that they share a range for exactly that reason
(`app::markupband::tests::the_width_range_matches_the_pen_that_authors`);
this module makes the same property structural rather than asserted, because
there is only one list.

It also holds the **chooser widget** itself, so the three surfaces cannot
come to disagree about what the entries are *called* either.

## It is a new file rather than more of `pen.rs`, and that is R2

`pen.rs` stood at 1,010 lines. This subject is a type, a reading of a
dictionary key, a widget and their tests, and every one of those choices
needs its argument written beside it — comfortably past the headroom.
`tools/gates/check-file-size.sh` says in its own header that shaving prose to
fit a threshold is the behaviour it exists to refuse, so the subject moved
instead. The seam is the same one `text::commands::markupstyle` took: `pen`
is *what the pen is*, this is *what a line style is*.

## ⚠ THERE IS NO PHASE, AND NO CONTROL FOR ONE IS OFFERED

The content-stream `d` operator takes an array **and** a phase, but Table
166's `/D` carries the array alone and the standard says the phase *"shall be
assumed 0"* — the engine states it under the *"There is no phase"* heading
on `annot_author::BorderDash` itself, and emits `0`
when it bakes the appearance. A phase control here would be a value the file
cannot hold: the operator would set it, the writer would ignore it, and the
control would look like completeness while being a lie. It is written down
here so that nobody adds one later on the reasoning that a dash "obviously"
has an offset.
