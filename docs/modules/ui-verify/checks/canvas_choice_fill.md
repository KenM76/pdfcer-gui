# `ui-verify/checks/canvas_choice_fill`

`a_drop_down_can_be_answered_on_the_page` — **clicking a combo box or a
list box on the page opens its options and picking one writes the field** —
`FORMS_PARITY.md` §8.1 row 2.

# What was wrong, and why the wrongness was argued rather than forgotten

`canvas::forms` §5 used to carry a fifth reason a field is not offered on
the page: *"a choice field would need a dropdown anchored to the page,
which is a second popup surface with its own placement rules and no gesture
the panel does not already have."* Both clauses were wrong. The placement
rules are the ones the page-anchored context menu already has, and the
panel's gesture is **not** the same one: the panel is reached by finding a
row in a list of field names, and the operator reaching for a drop-down on
a drawing has the drop-down under the pointer already.

So a drop-down was selectable on the page and not answerable there, and the
selectable census said so in the trace on every run without any check
reading it.

# Why a unit test cannot close this row

`canvas::forms::choosing` has unit tests over the whole pure half: which
exports a pick produces, which row the list opens on, and where the
highlight goes. What they cannot see is the **chain in front of the verb** —
that the click reaches the form overlay at all, that the popup is drawn
somewhere a press can land, that the press lands on the row the operator
aimed at rather than on the page behind it, and that the arrow key reaches
the list instead of scrolling the canvas. Every one of those is a fact about
a laid-out frame and a real pointer.

The half-plane constraint the popup uses exists precisely because the naive
version fails here and nowhere else: a page-anchored popup handed
`constrain_to(screen)` slides back over its own anchor and then takes that
anchor's clicks, so the operator's pick re-opens the list instead of
answering it. A unit test of the pick arithmetic passes throughout.

# The sequence, and what each step rules out

| step | oracle | what a wrong build produces |
|---|---|---|
| click `ComboOne` | `form-choice-open row=1` | `row=0` — the list opens at the top rather than on the answer the document already holds |
| `ArrowDown` | **no** `form-choice-pick` | a pick — Windows changes a closed combo's value on an arrow press, and doing that here writes a document edit and an undo entry for a key pressed to *look* |
| `Enter` | `form-choice-pick row=2 selected=1 multi=false` | `row=1` — the arrow never reached the list; or no line at all — Enter did not answer |
| | `form-set-choice commands=1` | the command was built and the engine refused it, which is what a value outside `/Opt` produces |
| click `ListMulti`, three arrows, `Enter` | `selected=3 multi=true` | `selected=1` — the pick replaced the two values already in `/V` instead of adding to them |
| `Escape` | `left=false` | `left=true` — the list closed itself after the tick, so answering three options is three separate gestures |
| `Escape` | `left=true` | the ring is never given up and the field keeps the keyboard |

The `selected=` field is a **count**, never a value: which option an
operator picked is their answer to the form, so the trace carries how many
and the row index and not the text. That is why the multi-select assertion
reads `selected=3` rather than naming a weekday.

# Why this fixture and no other

`fixtures/all-field-kinds.pdf` is the only document in the corpus with a
choice field at all — which is why *"a drop-down cannot be answered on the
canvas"* was never going to be caught by a driven check before it existed.
It needs two of them and they must differ in the right way: `ComboOne` is
single-select with a `/V` that is **not** the first option, and `ListMulti`
is `MultiSelect` with **two** values already in `/V`. A single-select field
cannot tell "added to the selection" from "replaced it", and a field
answered with its own first option cannot tell "opens on the answer" from
"opens at the top".

# It sends real input, so it takes the screen

No `PDFCER_DIAG_VIEWPORT`: an off-screen window is where its sibling
`option_arrows` runs, and it can, because it presses nothing. Four clicks
and six keystrokes need the window where the OS will deliver them.

## Item notes

### `fn choice_boxes`

The application's numbers and not the fixture's, for `tab_navigation`'s
reason: a check that derived the rect from the PDF would be asserting that
two independent derivations agree, and would report a disagreement between
them as a broken drop-down.
