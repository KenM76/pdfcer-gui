# `pdfcer-gui/app/actions/forms/author`

## Item notes

### `fn author`

The single narrowing point between this shell's one `Draft` and
`pdfcer-core`'s five spec types — see [`crate::canvas::formfield::draft`]'s
header for why the shell holds one struct and the engine five. Every field
a kind does not have is simply not read here, which is what makes that
asymmetry cost one `match` instead of five dialogs.

## The tooltip is the whole reason this verb was thought impossible

`TooltipChoice` has three states and the default is `Undecided`, which every
one of these five verbs **refuses**: an interactive control owes a screen
reader a name, and the engine will not invent one silently. That refusal
reads like a structural gate blocking the feature; it is not one. It is a
required field of the dialog above.

So an empty tooltip becomes `TooltipChoice::Declined` rather than being left
`Undecided`. The two are not the same and the difference is the point:
`Declined` is the operator saying *"this control needs no name"*, which is a
decision and is sometimes correct — a decorative button beside a labelled
one. `Undecided` is nobody having been asked.

## Rule 4 — the outcome is disclosed, off-canvas, in full

`FieldAuthorOutcome` carries four things the operator **cannot see on the
page**, and the one that matters most is `merged`: a name that matches an
existing field makes this widget a second *view* of that field, so typing in
one changes the other. Nothing about the rendered page says so, and a
screenshot of it would be identical either way. That is precisely the half
of rule 4 that survives decision 059 — *render normally; report separately* —
so every flag the engine raises becomes a status line and none of them
becomes a mark on the canvas.
