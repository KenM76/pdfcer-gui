# `app::markupband` tests — the Format ▸ Markup band's own assertions

## Why they live in a file of their own

Module beside module, `#[cfg(test)] mod tests;` — the seam
`canvas::annotnodes` and `app::conditions` also take. It is a **subject**
seam rather than an arithmetic one: the parent draws controls, and this
file asserts what they decide.

## What these can and cannot prove, stated first

**They cannot prove an operator can restyle a mark.** Every test here calls
a function directly, and R1's whole point is that a passing unit test is not
a report of working software. Nothing below draws a pixel, opens a combo, or
reaches `set_markup_style`; `tools/ui-verify` is the instrument for that.

What they do prove:

1. **The engine's subtype list is *asked*, not restated.** Every visibility
   predicate is checked against `MarkupStyleSupport::for_subtype` rather
   than against a table kept here — see
   [`each_predicate_reads_the_engines_flag_and_nothing_else`], and
   `NO_SURFACE.md`'s *How to read a row* for why a table alone would be two
   copies of one constant that cannot disagree.
2. **A parked edit sets exactly one field**, which no operator could
   report going wrong: a width that silently reverts one frame after it is
   set reads as *the drag did not take*.
3. **`Clear` reaches the engine as `Clear`.** The fifth state is the one
   change whose whole effect is invisible on screen and visible only in the
   saved bytes, so a test is the only place it can be seen at all.

**Every negative assertion here is paired with a positive control.** A
"not X" assertion is vacuous when the thing that would produce X is absent:
it passes just as well on a build where the feature is switched off
entirely, and only a positive row beside it separates the two.
