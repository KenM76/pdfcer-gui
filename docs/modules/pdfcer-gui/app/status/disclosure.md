# `app::status::disclosure` — the three rule-4 lines in the status bar


> The left half carries four things, and only the first is the narrator. The
> others look similar and are governed by different rules.

Everything here is **rule 4**: pdfcer did something the operator did not ask
for and cannot see, so it says so — off-canvas, never on the page.

## Three lines, and they are INDEPENDENT

| line | answers |
|---|---|
| [`fill_disclosure`] | what a form fill had to **infer** — an auto-size chosen, characters that could not be encoded |
| [`edit_disclosure`] | what a move or delete had to change about an object's **form** to express the request |
| [`recovered_disclosure`] | how this **file** was assembled, before anything was drawn |
| [`load_anomalies_disclosure`] | what this **file** said twice, and which reading pdfcer used |

⚠ The table names the four rule-4 lines. Two further tenants have since
joined the module and are **states rather than disclosures** —
[`blend_space_disclosure`] and [`line_weights_disclosure`] — each arguing its
own obligation in its own header; and [`catching_up`] is narration. The
module is now "the bar's left-hand sentences", and only the four above are
governed by the rule this header opens with.

The obvious mistake, adding a third beside two, is an `else if` chain that
shows whichever fires first. A document opened from a damaged index, then
edited, with a form filled, owes the operator all three —
`disclosure_independence` in the parent asserts they cannot collide.

The last two are the odd ones out and the reason for this module's
header: the first two are about **something the operator just did**, and
those two are about **what the file was before they touched it**. They are
also the only ones that persist for the life of the document rather than
until the next edit — see [`load_anomalies_disclosure`]'s lifetime section
for why an `edit_epoch` key would be actively wrong for them.
