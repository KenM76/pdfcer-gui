# `pdfcer-gui/text/settings/overprint`

## Item notes

### `fn exactly_one_scope_is_marked_as_the_engines_default`

Asserting **exactly one** rather than "the right one carries it"
catches the other half: a suffix added to a second label by hand, which
would leave two options both claiming to be what pdfcer does.

### `fn no_scope_label_spells_out_the_default_itself`

The suffix is derived; a label that spells it out would be a second,
unsynchronised claim about the same fact — which is exactly how the
original defect happened.

### `fn zero_tint_title`

**The title asks about GREY, not about "OPM 1's scope".** The engine's
own account of this setting is four screens on a genuine ambiguity in
§8.6.7; the operator's version of the same question is *"does a grey fill
wipe out the spot colour underneath it, or not?"* — which is what he would
have seen on paper and what would send him looking.

⇒ The window's rule is that a heading names the SYMPTOM. A heading reading
*"Overprint zero-tint scope"* is the field name, and a field name is
findable only by somebody who already knows the answer.

### `fn zero_tint_radius`

It names the same narrow reach the blend-space setting does, because it
has the same one: nothing happens on a file that never asked for overprint,
which is nearly every file that is not print-ready.

### `fn zero_tint_default_suffix`

DERIVED from `OverprintZeroTintScope::default()`, never written into a
label — and that is the whole point of it existing.


The label went on saying "(pdfcer's default)" about the option that was no
longer it — a sentence true when written and silently false afterwards,
which is the class this project has now met a dozen times. The engine's own
note is what caught it, and its advice was exactly this: *"if you read
`OverprintZeroTintScope::default()` to decide that, nothing to do."*

### `fn zero_tint_note`

Each note says what comes out on paper, with the measured numbers where
there are any and an admission where there are none. The third option is
unmeasured and its note says so in the operator's terms — *"nobody has
checked"* — rather than leaving him to infer it from the absence of a
figure.
