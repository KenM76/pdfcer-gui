# `icons::glyphs` — asking the font stack what it can actually draw

A test-only module holding two things and one gate:

| item | what it is |
|---|---|
| [`GlyphProbe`] | a **correct** "can this codepoint be drawn?" predicate |
| [`string_literals`] | a Rust source scanner that yields operator-visible literals |
| [`tests::every_glyph_the_catalog_draws_has_a_glyph`] | the widened glyph gate |

## Why this module exists at all: `egui`'s `has_glyph` lies

`DEFECTS.md` D12. A gate that asks [`epaint::Fonts::has_glyph`] whether a
catalog character is drawable gets **false** for characters that draw
perfectly — `⚠` U+26A0 among them — so a failure of such a gate is evidence
about the predicate, not about the font stack. `epaint 0.35`'s
`has_glyph` is:

```ignore
// epaint-0.35.0/src/text/font.rs:720
pub fn has_glyph(&mut self, c: char) -> bool {
    // TODO(emilk): this is a false negative if the user asks about the
    // replacement character itself 🤦‍♂️
    self.resolve_face(c) != self.cached_family.replacement_face_key
}
```

`resolve_face(c)` walks the family's fallback chain and returns the first
**face** whose charmap covers `c`. `replacement_face_key` is the face that
was found to contain `epaint`'s substitution character — `◻`, U+25FB
WHITE MEDIUM SQUARE (`epaint-0.35.0/src/text/fonts.rs:643`).

So `has_glyph` does not ask *"is this codepoint drawable?"*. It asks
*"is this codepoint drawable by a face other than whichever face happens
to own `◻`?"* — and returns **false** for every codepoint whose first
supporting face is that one. The upstream `TODO` names one instance of
the bug (the replacement character itself); the actual blast radius is
**every character that face is the first to supply.**

### The measurement, on this build's actual font files

`epaint 0.35`'s bundled proportional chain is
`[Ubuntu-Light, NotoEmoji-Regular, emoji-icon-font]`
(`epaint-0.35.0/src/text/fonts.rs:549-556`). Reading the four bundled
`.ttf` charmaps directly:

| codepoint | first face that supplies it |
|---|---|
| `◻` U+25FB (the substitution mark) | **NotoEmoji-Regular** |
| `⚠` U+26A0 | **NotoEmoji-Regular** |
| `ℹ` U+2139, `‼` U+203C, `❗` U+2757 | **NotoEmoji-Regular** |
| `⚑` U+2691, `★` U+2605, `○` U+25CB, `⏴⏵⏷` U+23F4-7 | emoji-icon-font |
| `—` `…` `·` `×` `“` `”` `−` `°` `◊` `•` `†` `‡` `№` `¶` `!` | Ubuntu-Light |
| `▲` `△` `●` `◆` `□` `✓` `✗` `ⓘ` `※` | *no face — genuinely absent* |

`replacement_face_key` for the proportional family is therefore
**NotoEmoji-Regular**, and `has_glyph` returns `false` for every row in
which NotoEmoji-Regular is the supplier — `⚠ ℹ ‼ ❗` — **although all
four draw perfectly.**

That one mechanism accounts for every measured answer, with no exceptions
across the 31 characters sampled: every character `has_glyph` calls
"present" is supplied by `Ubuntu-Light` or `emoji-icon-font`; every one it
calls "absent" is either genuinely absent **or** supplied by
`NotoEmoji-Regular`.

It also explains the otherwise absurd reading that
`has_glyph(Monospace, 'A')` is **false**: the monospace chain is
`[Hack, Ubuntu-Light, NotoEmoji-Regular, emoji-icon-font]`, `Hack` is the
first face there to supply `◻`, and `Hack` is also the first to supply
`A`. That is the false negative in its most obviously silly form, and it
is what makes the mechanism impossible to mistake for anything else.

## The predicate that is actually correct

Ask the renderer what it drew, rather than asking the font index a
question it answers wrongly.

When no face supplies `c`, `epaint` lays out the **substitution mark** in
its place (`epaint-0.35.0/src/text/font.rs:770-773`):

```ignore
let glyph_info = face.glyph_info(c, metrics).unwrap_or_else(|| {
    // `c` is in no face — render the replacement character instead.
    face.glyph_info(self.cached_family.replacement_char, metrics)
        .unwrap_or(GlyphInfo::INVISIBLE)
});
```

So: lay `c` out, take the single resulting [`epaint::text::Glyph`], and
compare its `uv_rect` — the glyph's actual rectangle in the font atlas —
against the `uv_rect` of a codepoint known to be unsupported. Equal means
`c` drew the substitution mark, i.e. it is **not** drawable. This asks
the question in the units the operator sees: *what pixels came out.*

### Why the sentinel is three codepoints and not one

The probe needs a `uv_rect` for "what an unsupported codepoint looks
like", and the obvious way to get one is to lay out a codepoint no font
could have. But if that assumption were ever wrong — a font grows
coverage, `epaint` swaps its bundled set — the sentinel would become a
real glyph, nothing else would match it, and **the gate would pass
everything.** That is the fail-open class `DEFECTS.md` D13 names: a check
that found nothing and a check that could not have found anything print
the same thing.

So [`GlyphProbe::new`] lays out **three** mutually unrelated unassigned
codepoints — U+0870, U+2FFFF and U+10FFFD, from three different planes —
and **panics unless all three produce the same rectangle.** Three
unrelated codepoints rendering identically is only explicable as the
substitution mark. If a future font set covers one of them the probe
fails loudly at construction instead of silently going blind, which is
the fail-**closed** direction.

## Why this module is `#[cfg(test)]`

Nothing in the shipped binary needs to ask this question: the catalog is
fixed at compile time, so the right moment to ask is the gate, not the
frame. Compiling it into the binary would add dead code and a `dead_code`
allowance to silence the warning about it.

Unit tests build the library with `cfg(test)` enabled for the whole
crate, so a `#[cfg(test)] pub mod` is reachable from every other module's
tests — which is what lets `app::status`'s bar gate share this predicate
rather than keep its own broken one.

## Why it lives under `icons`

`icons` already owns the question *"what happens when a mark cannot be
drawn?"* — that is [`crate::icons::paint_missing_mark`], the visible
stand-in for an icon whose art is missing. A catalog glyph like `⚠` is
the font-supplied sibling of an SVG icon: same job, same failure mode,
different pipeline. The substitution mark this module hunts for is
precisely `paint_missing_mark`'s counterpart on the text side.
