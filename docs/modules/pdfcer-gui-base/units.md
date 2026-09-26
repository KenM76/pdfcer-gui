# `pdfcer-gui-base/units`

**The one length-conversion table for this program.**

Every conversion between PDF points and a length an operator reads or types
goes through this module, and nowhere else declares a points-per-millimetre
constant, a `× 25.4 / 72.0` closure, or a `/ 72.0` DPI spelling.

# Why this module exists at all


* six private constants under two different names — `PTS_PER_MM` in
  `panels/pages`, `panels/docprops` and `dialogs/insert_image`, `PT_PER_MM`
  in `dialogs/new_document`, `dialogs/page_size` and `text/page_size` — two
  of them declared `f32` and the rest `f64`;
* seven inline closures re-declaring `|pt: f64| pt * 25.4 / 72.0`, five of
  them in one file;
* and `pdfcer_core`'s own [`Unit::baseline_per_point`], which is the value
  all thirteen were approximating.

That is not a tidiness complaint. Three concrete things went wrong:

**1. The divide form and the multiply form are not the same arithmetic —
and there is a third form neither of them is.**
`x / (72.0 / 25.4)` and `x * 25.4 / 72.0` differ in IEEE-754, because
`72.0 / 25.4` is itself inexact, so rounding it once and dividing is not the
same as multiplying by an exact numerator and dividing by an exact
denominator. Today that is masked by every consumer rounding hard, which is
precisely the kind of masking that stops working the moment somebody shows a
decimal.


```text
  engine  vs closure form    3_003 of 10_000 inputs disagree
  engine  vs divide  form      825 of 10_000 inputs disagree
  closure vs divide  form    3_441 of 10_000 inputs disagree
```

⇒ **"Disagree in the last ulp" was the wrong shape of claim.** Any two of
the three agree on most inputs and part on some. A4's 841.89 pt gives the
identical answer down the bit in all three spellings, so a program holding
all three cannot be shown to be inconsistent by checking a page size. The
disagreement waits for a sheet nobody thought to test — which is exactly
how the rounding half of this defect reached the operator.

**2. The rounding rule was inconsistent, and that one is visible.** See the
section below; it is the part that reached the operator.

**3. Two paths converted in `f32`.** On a 14,400 pt A0-class sheet an `f32`
millimetre loses roughly three decimal digits against the `f64` answer. It
never showed, because both `f32` sites round to whole millimetres — but a
module whose whole purpose is that two surfaces agree cannot be built on a
width that depends on which surface you asked.

# ★★★ The rounding rule, and why it is a decision rather than a default

A whole-number length shown to the operator rounds **half away from zero**:
`2.5 → 3`, `-2.5 → -3`. That is [`whole`], and it is the only function in
this program permitted to turn a length into a whole number for display.

The alternative was never chosen by anyone; it arrived by accident.
Rust's `{:.0}` format specifier rounds **half to even** — `2.5 → 2`,
`3.5 → 4` — which is the IEEE-754 default and an excellent rule for summing
long columns of numbers, because it does not accumulate an upward bias. It
is the wrong rule for a single sheet size, for two reasons: a CAD operator
reading a drawing expects the schoolroom rule, and, decisively, **it is not
what the other half of this program was already doing**.

The defect it produced, measured rather than reasoned:

```text
a sheet authored at exactly 210.5 mm
  converts to 596.6929133858 pt (f64) / 596.6929321289 pt (f32)
  both convert back to exactly 210.5000000000 mm

  page-thumbnail tooltip   {:.0}           ->  "210"
  print dialogue           .round() as i64 ->  "211"
```

One document, one sheet, two whole numbers, on two surfaces an operator can
have open at the same time.

⚠ **The precision difference is not what causes that**, and the first three
written accounts of this defect said it was. Both paths land on exactly
`210.5000000000`; the disagreement is entirely the rounding rule. The `f32`
problem is real and separate and is fixed here too, by this module being
`f64` throughout — but it contributed nothing to the number the operator
would have seen.

# What this module does NOT cover, deliberately

**Type size.** A font size printed as `12 pt` is the *typographic* point.
It is arithmetically the same 1/72 inch, and it must never be offered in
millimetres, metres or — the failure mode worth naming — kilometres. No
drawing or publishing program does this and Acrobat does not. Those
surfaces carry an in-source note pointing here; they are excluded, not
overlooked. See `UNIT_SURFACES.md` §4 for the list.

**Dimension-group text height, arrow size and gap.** `text/dimension_groups`
argues these are *paper* sizes — "10 pt tall whatever the drawing is scaled
at" — and that reasoning is sound. They are provisionally excluded on the
same grounds as type size, but the exclusion sits inside the dimensioning
feature and is flagged in `UNIT_SURFACES.md` §4 as the one worth putting in
front of the operator rather than deciding silently.

**Scale-aware conversion.** Everything here is the 1:1 *baseline*: it turns
a PDF point into a physical length on the sheet. Converting a sheet length
into a real-world length through a **ce dimension** group's scale is
`ScaleState::effective_scale` in the engine, and is a different question
with a different answer per group. (R8b rule 15: **ce dimensions** are the
ones this program authors; **pdf dimensions** are CAD-exported page content
it must not silently alter. Neither is what this module converts — this
converts *lengths*.)

# Contract

* Every function takes and returns `f64`. Callers holding `f32` (egui rects,
  mostly) convert in, not out: `mm_from_points(f64::from(rect.width()))`.
* Conversion is a single multiply by [`Unit::baseline_per_point`], so two
  call sites given the same input produce bit-identical results. That is the
  property the six private constants could not offer.
* No function here formats. They return numbers; the caller writes `{}` for
  a whole number and an explicit `{:.N}` for a decimal. **A length must not
  reach a format string as a bare `{:.0}`** — that is the half-to-even path
  this module exists to remove, and `tools/gates/check-unit-conversion.sh`
  fails the build on it.
