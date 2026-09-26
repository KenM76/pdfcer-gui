# `egui-shell/theme/contrast`

## Item notes

### `fn grounds`

Deliberately not the full cross product. A gate is only worth
what its reader believes, and a pair nobody renders is a line in a
failure list that sends somebody to look for a surface that does
not exist. So each role names the grounds it has a renderer for:

- [`Self::Body`] and [`Self::Weak`] reach [`Ground::TextEditBg`]
  because a `TextEdit` draws its content and its hint there
  (`egui-0.35.0/src/widgets/text_edit/builder.rs:463-466` and `:591`).
- Nothing draws `strong`, a hyperlink, a warning or an error
  *inside* a text field, so those four stop at the two chrome
  grounds.

### `fn fmt`

**This is diagnostic text, not operator-visible copy.** It is
written for a failing test, a CI log or a verification harness. An
application that wants to surface a theme problem to a user should
render the structured fields itself, in its own string catalogue —
the shell has no business deciding how another project words a
message to its operator.

The line keeps the ten-pair version's wording word for word for a
widget pair, because that message was good and the widening had no
licence to spend it. Two things were added, both because the
widening produced a failure the old wording served badly:

- the trailing ` — {why}` clause, for every origin — a reader who
  knows `widgets.Active.bg_fill` failed is better off still for
  being told, on the same line, that it is what tab buttons paint;
- **one decimal place on the gap.** The first real failure the
  widening found measured 89.74 against a floor of 90, and at
  `{:.0}` it printed as *"luminance gap 90, needs 90"* — a
  diagnostic that reads as a bug in the gate. The threshold keeps
  `{:.0}` because it is a round number by construction.

### `fn the_pair_matrix_covers_every_origin_it_claims_to`

Worth its own test because the widget half of this module's value
is that its coverage is defined by `egui`'s matrix rather than by a
list — and the *other* half is a list, which is precisely the part
that can silently shrink. If a state were dropped from
[`WidgetState::ALL`], or a role from [`TextRole::ALL`], the gate
would still pass everything it looked at and nothing else would
notice.

### `fn a_translucent_foreground_is_composited_not_treated_as_opaque`

The specific number matters: `rgba(250,250,250,220)` is the plate
colour from D2, and treating it as opaque is the mistake that
would make this gate wrong about its own defect.

### `fn a_translucent_selection_plate_measures_differently_on_each_ground`

This is defect T2's arithmetic in miniature and the reason
[`Origin::SelectedWidget`] carries a ground at all. A 27 %-alpha
wash is not a dimmer plate; it is a different colour over every
background it meets, so the same theme values must produce two
different gaps over two different grounds. If they did not, the
ground field would be decoration and a regression to a wash could
hide behind whichever ground happened to be measured.

### `fn check_reports_every_failing_pair_not_only_the_first`

A gate that stops at the first failure turns a theme edit into a
sequence of rebuilds. `check`'s contract says all of them; this is
what holds it to that.

### `fn a_failure_names_the_line_to_change`

The message is the deliverable. A gate that says "contrast failed"
has told the reader to go and re-derive what this function already
knew.

The widget case's wording is asserted verbatim, because the
widening had no licence to spend a message that was already good.
The only deliberate change is the gap's decimal place; see
[`ContrastFailure`]'s `Display`.

### `const READABLE_LUMA_GAP`

90 on a 0–255 crude luminance scale. The figure is inherited from the
salvaged palette-level text test so that both gates agree about what
"readable" means; a theme that satisfies one and not the other would
produce two contradictory failures for one edit.

The shipped presets clear it comfortably. The tightest real pair is
the Dark preset's focus ring — `accent` on `panel` — at 96.0.

### `enum WidgetState`

Mirrors `egui::style::Widgets`' fields. A local enum rather than a
re-export because the point of it is to be *named in a failure
message* — "the Active state's bg_fill" is the sentence that points at
the line to change.

### `enum FillKind`

# Why both, and why this distinction is the whole defect

`egui` gives each widget state two background colours and lets each
widget choose. `Button` and `SelectableLabel` paint `weak_bg_fill`;
`CollapsingHeader` headers, `egui_tiles` tab buttons and several
others paint `bg_fill`. A theme that assigns one and not the other has
themed an arbitrary subset of its own widgets, and which subset is
decided by `egui`'s internals rather than by the theme's author.

D2 is exactly that: `weak_bg_fill` was assigned the accent, `bg_fill`
was not, and the widgets that lost were the ones nobody happened to
look at.

### `enum Ground`

# Why this exists at all

[`FillKind`] answers "which of a widget's two fills". This answers a
different question — "what is behind a piece of text that is not
inside a widget" — and the widget matrix has no way to express it,
which is half of why ten pairs missed three defects.

The three grounds below are `egui`'s own, and each is reachable from a
`Visuals` alone. They are not palette roles: a theme may point all
three at one colour (this one points two of them at `Palette::panel`)
and the gate must still name them separately, because a *later* theme
may not, and because the failure message has to say which surface the
reader should go and look at.

### `enum TextRole`

# Why these are invisible to the widget matrix

Every one of them is computed at paint time from something other than
the state's own `fg_stroke`. Reading the five `WidgetVisuals` back
therefore cannot reach any of them — the same structural reason the
selected pair was missed. Each variant's doc names the `egui` source
line that renders it, because a role nobody can point at a renderer
for should not be in this list.

### `enum Origin`

# Why this replaced two plain fields


The alternative — a parallel list of "other" pairs with their own
failure type — was rejected because it splits the one thing this module
is for. [`check`] returning *every* failure in one run is what makes a
theme edit one rebuild rather than five, and two lists reintroduce the
sequencing this module's `check` doc argues against.

So the origin became a sum type, and each variant answers the three
questions a 2 a.m. reader has: **what colour**, **on what**, and
**what renders it**. The widget variant's [`Self::fg_path`] and
[`Self::bg_path`] reproduce the old message's wording exactly, so no
existing failure text lost a word.

### `fn why`

# Why a failure carries this and not just the two field paths

Because the field path says which line to edit and says nothing
about whether editing it is the right move. A reader who is told
`weak_text_color() on panel_fill` still has to go and find out
what draws with it before they can judge whether the theme is
wrong or the call site is. This is that sentence, written once,
beside the measurement.

### `struct ContrastFailure`

Carries the whole [`Pair`] plus the threshold it was measured against,
because a failure message that says "gap 41" without saying "needed
90" makes the reader go and find the threshold.

### `fn luma`

The Rec. 709 coefficients applied directly to sRGB bytes, with no
linearization. This is not photometrically correct and is not trying
to be — see the module header on why a coarse measure is the right
tool for the failure being guarded against, and on the one bias
(saturated reds score low) that has actually changed a decision.

The alpha channel is ignored. Composite first with [`over`] if it
matters; [`pairs`] does.

### `fn over`

# Why this is necessary rather than fussy

The colour at the heart of D2 was `rgba(250,250,250,220)` — a
*translucent* near-white. Measuring its luminance as if it were opaque
overstates its contrast against a dark background and understates it
against a light one, and a plate colour used as a foreground is
exactly the case where that error is largest. A gate that got this
wrong would be wrong specifically about the defect it exists to catch.

The widening added two more translucent things to measure and it did
not have to add any arithmetic for either: `weak_text_color()` is a
premultiplied 60 % of the body colour, and `selection.bg_fill` may be a
wash. Both go through this function.

`Color32` in `egui` is premultiplied, so the source channels are
already scaled by alpha and the composite is `src + dst·(1−a)`.

### `fn check`

Pairs covered by an [`Exemption`] are measured and then skipped; see
[`EXEMPTIONS`] for why they are not simply absent from [`pairs`].

# Errors

Returns **every** failing pair rather than the first, so one run names
the whole problem. A gate that reports one failure at a time turns a
theme edit into a sequence of rebuilds, and the second failure is
often the one that explains the first.
