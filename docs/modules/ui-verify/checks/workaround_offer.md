# `ui-verify/checks/workaround_offer`

`a_refused_edit_offers_its_workaround` — a text edit the engine refuses but
names a workaround for is offered in the Properties panel, nothing is applied
until the operator presses the offer, and the press makes the same edit the
named way.

The fixture is `fixtures/quote-operator.pdf`: one Helvetica text object whose
second line, `Quoted line` at y=680, is drawn with the `'` operator. The
engine's exact surgery refuses that run and names
`Workaround::RewriteQuoteOperator`, which is exact.

The window is placed off the desktop and driven only through
`ScriptedPointer`, with `PDFCER_DIAG_INVOKE=mode.edit,edit.text` arming the
Edit Text tool, so the check runs under `--no-input`.

## Steps

1. **Raise Properties.** No `workaround-offer` line may exist before an edit:
   an offer drawn without a refusal is not tied to one.
2. **Refuse.** A click inside `Quoted line`, `End`, `s`, `Escape`. No
   `edit-text-workaround` line may exist yet (a workaround applied without the
   press is the R8b failure); the last `edit-text-left-edge` must read
   `committed=no`, and the last `workaround-offer` must name
   `workaround=rewrite-quote-operator`.
3. **Press.** The `properties.workaround.apply` region must be declared; a
   click on it must leave `edit-text-workaround … used=rewrite-quote-operator
   exact=1` and a new `edit-text-left-edge … committed=yes`.

## Falsification

With the panel's button pushing `workarounds: false`, the retry is refused
again, no `edit-text-workaround` line is written, and the check fails at
step 3.
