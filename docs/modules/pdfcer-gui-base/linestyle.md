# `linestyle` — solid or dashed, on all three surfaces that
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

### `enum LineStyle`

# Why this type exists rather than a `BorderDash` on the pen

`annot_author::BorderDash` owns a `Vec<f64>`, so it is
`Clone` and **not** `Copy`. [`super::pen::Pen`] is `Copy` and is passed by
value through every gesture commit in `canvas::markup`; both restyle
surfaces pass a `Copy` `Current` by value through half a dozen control
functions. Putting the engine's type into any of them would have turned a
field addition into a borrow-checker sweep of files three other tracks were
writing in on the day this landed — for no gain, because **this shell offers
four patterns and not an arbitrary array**.

So the choice is modelled as the choice, and the engine's value is built at
the boundary by [`Self::dash`]. One `Copy` enum, three surfaces, one list.

# `BorderDash::new` returns `Option`, and this shell REFUSES IN THE UI

`pdfcer_core::BorderDash::new` refuses a pattern §8.4.3.6 does not admit: empty
(which *is* the standard's solid line, so it is not a dash), negative,
non-finite, or every element zero. The reply that shipped the field said to
*"refuse in your own UI or let the `None` refuse for you"*. This shell does
the **first**, and here is the decision written down:

* **Every offered pattern is a compile-time constant** satisfying §8.4.3.6,
  so the refusal is not reachable by any operator action. There is no numeric
  entry anywhere in this control, and therefore no operator-supplied array to
  validate — which is the whole reason refusing in the UI is available at
  all. A control that let an operator type `0 0` would have to do the other
  thing.
* [`tests::every_offered_pattern_is_one_the_engine_accepts`] asserts it for
  every variant, so a fifth pattern added carelessly — a typo'd negative, a
  `[0.0, 0.0]` — fails the build rather than silently becoming *solid* in the
  operator's file.
* Where the `Option` still has to be handled at run time, [`Self::dash`] and
  [`Self::style_edit`] are the only places, and their callers **park
  nothing** on a `None`: no substitution, no `unwrap`, no undo entry for a
  refusal. Substituting [`pdfcer_core::annot_author::BorderDash::table_166_default`]
  would be this shell writing a pattern the operator did not pick, which is
  the sneaky half of R8b.

# Where the four patterns come from, and where they DELIBERATELY do not

[`Self::Dashed`] is **sourced**: `[3]` is Table 166's own default for `/D` —
the pattern the standard gives an annotation that declares `/S /D` and no
array, and the value `annot_author::BorderDash::table_166_default` builds.
It is the one entry here that is not this
shell's choice, and it is deliberately the first dash in the list.

The other two are this shell's, argued rather than measured, and the
distinction is stated because [`super::pen::Pen::default`]'s colours *are*
measured out of Acrobat's own store and these are not. **Acrobat's `cAnnots`
tree holds no dash key**: the search that established there is no Acrobat
line-width to match (see `Pen::default`'s own account of it) found no dash,
thickness or border entry either. So there is no Adobe pattern to transcribe,
and this project's claim-bearing-copy rule forbids inventing one and calling
it Adobe's. What decided them instead:

* they must be **unmistakable from one another** at 100 % over dense CAD
  linework, which rules out two patterns differing by a fraction of a point;
* they should span the two broken-line conventions a draughtsman already
  reads — a long even dash for hidden geometry, a dash-dot for a centre or
  reference line. That is a claim about what an operator will **recognise**,
  not a citation of a line-type standard: ISO 128's run lengths are
  scale-dependent and this shell has not measured them, so it does not cite
  them.

⇒ If a measurement of Acrobat's own patterns ever turns up, this list should
be revisited **with that measurement**, exactly as `Pen::default` says of the
2 pt width.

### `const ALL`

Solid first because it is the state every mark starts in and the one an
operator reaches for to undo an experiment, and because Table 166 lists
`/S` first. The three dashes then run **shortest to longest**, so the
list reads as increasing distance from a solid line rather than in an
order somebody happened to type.

A `const` rather than a literal at each call site, for
`panels::properties::markup::ALL_ENDINGS`' reason: three choosers writing
their own lists would come to offer different sets, and the one that went
stale would be the one nobody opened.

### `fn pattern`

Exhaustive with no `_` arm, deliberately: a fifth style must be taught
its pattern here or fail to compile, rather than falling into a catch-all
and being authored as somebody else's dash.

### `fn dash`

What goes into `MarkupOptions::dash`. `None` writes no dash at all,
which is byte-for-byte what this shell authored before the control
existed — so a build whose operator never touches the chooser produces
the same file it did before, which is this project's standing rule for
a capability becoming choosable.

⚠ A `None` from `BorderDash::new` would collapse into the same answer as
*solid*. That is why the refusal lives in the choice of constants rather
than here — see this type's header, and
[`tests::every_offered_pattern_is_one_the_engine_accepts`], which is what
makes the collapse unreachable rather than merely unlikely.

### `fn style_edit`

`StyleEdit::Clear` for [`Self::Solid`] — the engine's own spelling of
*make it solid*, rather than the removal of a control's value — and
`StyleEdit::Set` for a dash.

The `None` is what the two restyle surfaces decline on: they park
nothing and raise nothing, so an unbuildable pattern produces no write
and no undo entry. It is unreachable for the four constants above, and it
is *expressed* rather than `expect`ed because a paint-loop panic on a
state that is merely unexpected is a worse failure than a control that
declines.

### `fn of_pattern`

Compared element by element with an exact `==` on `f64`, and that is
correct rather than sloppy: both sides came from the same four literals —
this shell wrote the array, the file stored it, and a small decimal
round-trips through `Object::Real` exactly. A tolerance would make
`[3.0001]` read as *Dashed*, and the next press would silently rewrite it
to `[3]` — the file changing under an operator who touched a different
control.

### `enum DashReading`

# Why a third variant exists, and why it is not selectable

A producer's dash is not required to be one of the four this shell offers,
and the engine now **preserves** it: a restyle that does not mention `dash`
keeps it, *including a dash pdfcer never authored*. So a chooser that could
only show four states would have to show one of them for a mark that is
none of them — and whichever it picked would be a claim about the
operator's file that the file does not make.

[`Self::Foreign`] is that state, named as what it is. It is **displayed and
never offered**: picking an entry replaces it, and there is no entry meaning
*"put it back"*, because putting it back would need this shell to hold the
old array across an undo boundary it does not own. Leaving the chooser alone
is what keeps it — which is exactly what the engine's preservation
guarantees, and is why *doing nothing* is a real answer here rather than an
omission.

# ⚠ It carries no pattern, and that is a trade rather than an oversight

Showing the file's own run lengths — *"Dashed 6, 2"* — would be a little more
informative and would cost this type its `Copy`, which both restyle surfaces'
`Current` structs depend on and pass by value through every control function
they have. The state is what the chooser needs: **this mark is dashed, in a
pattern that is the file's and not one of ours.** The array itself is `/BS`
`/D`'s bytes, and a ribbon combo is not where a file's bytes are read out.

⇒ If a surface ever genuinely needs the numbers — a Properties panel line
that reports rather than offers — this variant gains a payload and both
`Current`s stop being `Copy`. That is a real cost and it should be paid for a
real need, not for a nicety.

### `fn label`

For [`Self::Foreign`] it is a sentence about the **file**, not the name
of an entry — see [`t::line_style_foreign`]. A combo whose closed state
showed *Dashed* for a pattern that is not this shell's *Dashed* would be
the quiet lie the swatch's CMYK arm was rewritten to stop telling.

### `fn read`

# Why this is a SECOND reader of a key the engine already reads

Because the engine's reader is not public. `annot_author::read_border_dash`
is `pub(crate)`, and `spec_from_dict` does **not** carry the dash: a dash
cuts across `MarkupSpec`'s variants rather than belonging to any one of them,
so it travels in `annot_author::AppearanceOptions` beside the spec instead
of inside it. There is therefore no public route from an
annotation dictionary to *"is this mark dashed, and how"* — and a control
that cannot show the current value is a control that shows an invented one,
which is the `fontband::size` defect this project has already paid for.

⇒ So this is the copy, and it is **written down as a copy** rather than
presented as a reading. The table below is transcribed from
`read_border_dash`'s own doc comment; it was not
derived independently:

| `/S` | `/D` | read as |
|---|---|---|
| `/D` | present and usable | that pattern |
| `/D` | absent, or unusable | [`LineStyle::Dashed`] — Table 166's `[3]` default |
| absent | present and usable | that pattern |
| `/S`, `/B`, `/I`, `/U` | either | solid |

The third row is the one a literal reading gets wrong. `/S` defaults to `/S`,
so `/BS << /D [4 2] >>` is *technically* a solid border with a meaningless
array — and producers write exactly that, and Acrobat draws it dashed. The
engine honours it; so does this, because a chooser that read it as solid
would offer *Solid* as the current state of a mark the operator can see is
dashed.

# ⚠ What a divergence between the two readers can and cannot cost

It can only ever cost the **display**. Nothing is written unless the operator
picks an entry, and picking an entry sends an absolute value — `Set(pattern)`
or `Clear` — that does not depend on what was read here. So the worst outcome
of this copy drifting from the engine's is a chooser showing the wrong
current style until it is touched; it cannot silently rewrite a pattern.

That bound is the reason the copy is acceptable at all, and it is the thing
to re-check if anybody ever makes this function's result decide what gets
**written**. **If the engine publishes `read_border_dash`, delete this and
call it** — recorded on `NO_SURFACE.md`'s boundary table.

### `fn chooser`

Returns the style the operator picked, or `None` for a frame in which they
merely looked at it — the same *nothing was invoked* contract
`app::markupband`'s controls have.

# It never reports the style that is already showing

Selecting the entry a mark already has must raise nothing: on the restyle
surfaces that would be an undo entry the operator did not earn and a re-bake
of an appearance that is already right, and on the pen it would be a trace
line for a change that did not happen. The comparison is against
[`DashReading::selected`], so a [`DashReading::Foreign`] mark reports on
**every** pick — which is correct: none of the four is what it currently is.

# `width` is the caller's, because the three surfaces have different room

The ribbon band budgets `CUSTOM_ITEM_WIDTH` per custom item and a Properties
panel row does not; passing it in is what lets one widget serve both without
the band's constraint leaking into the panel or vice versa.
