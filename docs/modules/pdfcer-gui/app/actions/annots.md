# `pdfcer-gui/app/actions/annots`

## Item notes

### `fn rect_rule_token`

# Why not `{:?}`

Because a `Debug` rendering is a formatting detail of somebody else's enum
and a driven check that greps for `rect_derived=Artwork` would go quiet the
day `RectDerivation` gains a field, gets renamed, or has its derive removed
— quietly, and in the direction that reads as *the feature stopped
happening*. A `{:?}` on a tuple is how a check comes to report the
opposite of the truth while quoting the truth in its own message.

`RectDerivation` is `#[non_exhaustive]`, so the match **must** carry a
wildcard and a rule this build does not know cannot be a compile error
here. The wildcard says `other` — an honest *"this build does not know
that one"* rather than a guess at which rule ran.

The tokens match what the engine's own CLI prints (`rect_derived=`), so a
trace here and a `pdfcer` command line can be compared without a lookup
table.

### `fn refusal_for`

# One function over every rotation verb, and it is the compiler's job to
keep it complete

[`rotate`], [`set_rotation`] and [`rotate_dimension`] all refuse from the
same short list, so a second copy of this mapping would be a second place
for a variant to be forgotten — and a forgotten variant here is a grip that
is dragged, released, and does nothing with no explanation.

# Why the fallback is a sentence rather than the error's `Display`

`tools/gates/check-ui-strings.sh`' exclusion 3 names the failure in as many
words: a `format!` of an `EditError` routes **diagnostic prose into the UI**.
[`crate::text::rotating::RotateRefusal::Other`] is a hand-written sentence
that says the one thing an operator needs about an unrecognised refusal —
*the page is exactly as it was* — and names no cause it does not know.
