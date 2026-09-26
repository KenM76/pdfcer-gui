# `pdfcer-gui/text/settings/extract`

## Item notes

### `fn unmappable_silence`

*"The standard's own sentence about what to do here is incomplete"* is not
a figure of speech: §9.10.2's failure clause is grammatically broken — it
says a reader *"may choose a character code of their choosing"* where a
Unicode value is what is produced — and specifies no sentinel anywhere.

### `fn unmappable_radius`

Names **redaction**, which the source's radius line did not. R35 is
explicit that a redaction built under one value is not equivalent under
another: the sentinel changes character offsets, which changes which runs a
pattern matches. An operator who redacts by pattern needs to know that
changing this setting invalidates the reasoning behind a redaction they
have already reviewed.

### `fn unmappable_omit_note`

The source warned that extracted text reads as complete when characters are
missing. True, and the smaller half. The larger one is documented in
`pdfcer-core` and was shown nowhere: the layout pass drops a run with no
characters, so a run whose codes are *all* unmappable **vanishes entirely,
glyph records included** — a page of `Identity-H` text with no `/ToUnicode`
yields *zero runs* rather than runs of sentinels.

That is the more surprising failure and the one that breaks anything
needing per-glyph positions.

### `fn actual_text_silence`

Three ISO 32000-1 statements disagree and none dislodges the others. The
only sentence addressing precedence is a *may*, and it sits in an
informative note — which is why neither reading can be eliminated and why
this is a setting rather than a decision.

### `fn actual_text_glyphs_note`

The second sentence is not in the source: `Glyphs` **loses genuinely
unrecoverable text**, because a ligature whose only Unicode identity was
its replacement text extracts as whatever can be made of the glyph.

### `fn actual_text_bound`

New in this port; the old window disclosed it nowhere despite `pdfcer-core`
documenting it and calling it *"a fact to disclose, not a direction to
pick"*.

There is **no length correspondence** between replacement text and the
content it replaces — the standard's own example maps two shown characters
to one — so character-level mapping back to glyph positions is *impossible*
across such a run. That bounds search highlighting, selection and
redaction-by-text to **sequence** granularity **whichever of the three
options is chosen**, which is exactly why it belongs under the group rather
than inside one option's note: an operator who reads it as an argument for
picking *Ignore it* has been misled.

### `fn parallel_silence`

Not a spec silence — the PDF standard has no view on dimensioning at all —
but the same shape of silence, and worth saying because the operator would
otherwise reasonably assume CAD practice had settled it. A search of the
SolidWorks dimension corpus for a threshold found none.

### `fn parallel_radius`

A **third** radius category, which neither the preview settings nor the
byte-changing ones cover: it affects *new authoring only*. Dimensions
already placed do not move, and nothing in the file changes.

### `fn degree_suffix`

# Why one character has a function

`check-ui-strings.sh` requires every operator-visible string to come from
this catalog, and a bare `"°"` in a `Slider::suffix` call is exactly what
that gate looks for. The old shell solved it with a private constant in the
panel, which satisfies the gate's letter; a catalog entry satisfies its
reason, since a translator localising this window would otherwise never see
that the suffix exists.

### `fn parallel_note`

The last sentence is deliberate and was in the source for a stated reason:
an operator reading this control needs to know that a wrong global value is
a one-click per-dimension fix, not something they must come back here to
adjust. Without it, the natural response to one bad classification is to
change a global default on the strength of a single drawing.
