# `text::panels::dimension` — the words the ce-dimension properties section
shows

## What this surface is

The **bottom tier** of `pdfcer-core`'s style cascade, made reachable. Eleven
properties, each with an override checkbox and a label saying which tier
supplied the value it is currently showing — which is
`docs/ui_specs/tool-options-dock-and-ce-dimension-properties.md`
Amendment B §B.4's second bullet, and `FEATURES.md`'s

> ce-dimension style AND tolerance in the GUI — **one panel covering
> both**, showing which values are inherited and which are overridden and
> letting the override be set. The model and the CLI already do all of it;
> only the disclosure surface is missing.

## Rule 15, as everywhere in this feature

A **ce dimension** is one pdfcer authors; a **pdf dimension** is CAD-exported
page content pdfcer reads and must not alter. This catalog says *"this
dimension"* only where the operator has one selected and can see which, and
otherwise says *"the dimensions you draw"*, exactly as
[`crate::text::scale`] and [`crate::text::dimension_groups`] do.

## One vocabulary, shared with the CLI

`pdfcer dimension-style` names these eleven `unit`, `fraction`,
`decimal-marker`, `standard`, `text-height`, `line-width`, `arrow-length`,
`arrow-form`, `color`, `tolerance` and `tolerance-places`. Amendment B §B.5
states the hazard of diverging: *"a panel using different words for the same
nine things is how an operator ends up unable to script what he just
clicked."* These are the same words, in the operator's English.

## What this catalog must NOT do: build a label

`docs/core-api/03-capabilities.md` §1.6 trap (b) is explicit, and it cost
`pdfcer` a shipped defect: *"A panel that previews 'nominal + tolerance' by
concatenation **will disagree with the bytes in the page** for every limit
tolerance"*, because a limit tolerance **suppresses** the nominal rather
than printing beside it — and `Basic` prints no text at all, the box being
the notation.

So nothing here concatenates a nominal and a tolerance. The measurement is
rendered from the engine's own `MeasurementDisplay`, and what a tolerance
will *do* to the printed label is stated in a sentence
([`tolerance_suppresses_nominal`], [`tolerance_is_a_box`]) rather than
demonstrated by a preview this module would have to derive a second time.

## Item notes

### `fn the_three_style_sources_read_differently`

The property under test is that `Factory` and `Group` do **not**
collapse into one "inherited". They differ in what a `--clear` on the
group would do, and an operator reading "inherited" cannot tell which
tier to go and edit.
