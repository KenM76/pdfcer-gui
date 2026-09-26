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

## Item notes

### `fn every_offered_pattern_is_one_the_engine_accepts`

This is the assertion that makes *"refuse in the UI"* a real decision
rather than a hope. §8.4.3.6 refuses an empty, negative, non-finite or
all-zero array, and `BorderDash::new` answers `None` for each.
A `None` reaching [`LineStyle::dash`] would
collapse into the same answer as *solid* — so a fifth pattern typed with
a stray minus sign would produce a chooser entry that silently drew a
solid line, with no error anywhere.


Falsified by changing [`LineStyle::LongDash`]'s pattern to `[0.0, 0.0]`,
which turned the `is_some` assertion red, and by giving
[`LineStyle::Solid`] a pattern, which turned the control red.

### `fn solid_clears_the_dash_and_a_dash_sets_one`

`Clear` makes the border solid and `Set` makes it dashed. Getting this
backwards, or answering `None` for solid, would make the chooser's
first entry a control that does nothing: `MarkupStyle::dash: None`
means *leave whatever the annotation already has*, so a Solid that
raised `None` would silently keep the dash it was pressed to remove.

Falsified by returning `None` from the `Solid` arm of `style_edit`, which
turned the first assertion red.

### `fn the_four_styles_are_distinct_and_ordered`

Two entries with one pattern would be a list that looks like a choice
and is not; two with one label would be a combo an operator cannot read.
The order is asserted too, because it is argued in [`LineStyle::ALL`]'s
doc and a reordering is a change to the control's shape rather than to
its wording.

### `fn a_foreign_pattern_is_not_mistaken_for_one_this_shell_offers`

The property the [`DashReading::Foreign`] variant rests on: a producer's
`[6 2]` must not be reported as this shell's `Dashed`, because the
chooser would then show *Dashed* over a mark that is not, and a press on
any other entry would look like a change from a state the file never had.

Falsified by making `of_pattern` compare only the first element, which
turned the `[8.0, 9.0]` assertion red.

### `fn table_166_is_read_back_the_way_the_engine_reads_it`

# The graph is a **direct** one, and that is the right instrument

[`DirectGraph`] answers `None` for every id, so `resolve` returns each
value unchanged. That is not a weakened test: reference-following is
`ObjectGraph::resolve`'s **default method**, written and tested in the
engine, and this module contributes nothing to it. What this module
contributes is the decision about Table 166's key combinations, and those
are what the rows below vary. A session-backed graph would exercise the
engine's resolver a seventh time and this reader's table zero extra
times.

The rows, and what each one would cost if it were wrong:

| dictionary | expected | the failure it prevents |
|---|---|---|
| no `/BS` | solid | a chooser showing *Dashed* on every unbordered mark |
| `/S /S` | solid | the same |
| `/S /D`, no `/D` array | `Dashed` — Table 166's `[3]` | *Solid* shown over a mark Acrobat draws dashed |
| `/D [4 2]`, no `/S` | `Foreign` | the same, on the producer shape the engine's own doc calls out |
| `/S /D` + `[8 4]` | `LongDash` | a pattern of ours reported as the file's |
| `/S /B` | solid | a bevelled border offered as a dash |
| `/S /D` + `[0 0]` | `Dashed` | a §8.4.3.6-invalid array read as a pattern |

Falsified by deleting the `declared_dashed` fallback (the `/S /D` rows
went red) and by removing the `/S` early return (the `/S /B` row did).
