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
| `⚑` U+2691, `` U+2605, `○` U+25CB, `⏴⏵⏷` U+23F4-7 | emoji-icon-font |
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

## Item notes

### `const SENTINELS`

Chosen from three different Unicode planes so that no single font's
coverage decision can plausibly cover more than one of them:

- `U+0870` — Arabic Extended-B, unassigned at the time of writing.
- `U+2FFFF` — plane 2 (SIP), permanently unassigned (a noncharacter-
  adjacent tail position no CJK font allocates).
- `U+10FFFD` — plane 16, the last Private Use Area codepoint.

They are only ever *laid out*, never asserted to be absent — the assertion
is that they **agree with each other**, which is the property that makes
the fingerprint trustworthy. See the module header.

### `struct AtlasRect`

`epaint`'s own `UvRect` is not publicly re-exported (its module is
private, and `Glyph::uv_rect` is private-in-public), so its four fields
are copied out rather than named. `min`/`max` alone would identify the
glyph — the atlas allocates one rectangle per distinct glyph — but the
offset and size are carried too so that a future atlas that reused
rectangles could not collapse two glyphs into one answer.

### `fn uv_rect_of`

Control characters lay out to zero glyphs; they are reported as the
substitute so a stray `\u{7}` in a catalog string is caught rather
than panicking on an empty row.

### `fn char_literal`

Returns `None` for a lifetime, which is the ambiguity this has to resolve:
`'a` is a lifetime, `'a'` is a literal, and only the closing quote tells
them apart.

### `fn escape`

Returns `(resolved char, index after the escape)`. The char is `None` for
escapes that produce nothing renderable on their own (`\n`, `\t`, `\0`,
and a line continuation), which keeps them out of the glyph check without
dropping the rest of the literal.

### `fn the_probe_finds_the_warning_sign_that_has_glyph_denies`

This is D12's finding as an executable statement. `⚠` is asserted
**drawable** — which is the claim the defect entry denied — and
`epaint`'s own `has_glyph` is asserted to say the opposite, so the
day upstream fixes its false negative this test fails and says so
rather than quietly agreeing.

### `fn genuinely_absent_codepoints_are_still_reported_absent`

Without this the probe could be trivially satisfied by a predicate
that returns `true` for everything, which is the failure mode a
widened gate is most likely to acquire.

### `fn the_substitution_mark_is_the_one_case_the_probe_cannot_judge`

The substitution mark cannot be told apart from itself. Nothing in the
catalog uses `◻`, and this test is what makes that a known limit
instead of a surprise.

### `fn a_mid_file_test_module_does_not_blind_the_scanner`

The shape that once defeated `check-ui-strings.sh`: a test module in
the middle, with an operator-visible literal after it. This scanner is
a different implementation of the same job, so it is asserted here
independently — a literal after the module must be found, and nothing
inside the module may be.

### `fn a_line_continuation_still_counts_its_newline`

Found by this gate's first real run: the counter was incremented as
characters were *pushed*, and a continuation's newline is swallowed
inside `escape` without being pushed. Every line number after the
first wrapped string was therefore too low — 30 lines adrift by the
middle of `text/panels/objects.rs`. A wrong line number in a failure
message is worse than none: it sends the reader somewhere plausible.

### `fn a_cfg_test_use_does_not_swallow_the_next_function`

`#[cfg(test)] use super::*;` is the idiom that would do it: the
attribute arms the skip, the `use` never opens a brace, and the next
`{` in the file — an ordinary operator-visible function — would be
swallowed. A silent hole in coverage that prints clean.

### `fn a_byte_string_is_scanned_rather_than_refused`

`text/panels/objects.rs` builds a content stream from `b"…"` inside
its test module. Rust forbids non-ASCII in a byte string, so it can
never carry a codepoint this gate cares about — refusing the whole
file over one would have taken a real catalog module out of scope.

### `fn the_gate_catches_a_planted_unrenderable_codepoint`

This project requires it: a gate that has only ever passed is not
evidence of anything (`check-ui-strings.sh` PORT CHANGE 3, and the
D12 post-mortem). The gate above cannot be made to fail on demand
without editing the catalog, so its two halves — the scanner and the
probe — are driven here against a **planted** unrenderable codepoint
in a synthetic catalog file, through exactly the same code path.

The plant is `✓` U+2713, measured absent, placed in an
operator-visible position in a file that otherwise looks like a
catalog module — including a decoy `▸` inside a doc comment and a
decoy CJK string inside a test module, both of which must be ignored,
so this proves the gate fires on the real thing rather than on
anything that merely looks exotic.
