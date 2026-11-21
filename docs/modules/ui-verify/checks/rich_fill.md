# `ui-verify/checks/rich_fill`

`a_rich_text_field_fills_on_the_page` — **a rich-text form field is typed
into on the page, stays rich text, draws what was typed, and the status line
names the formatting the typing replaced.**

# The fixture

`fixtures/rich-field.pdf` (`rich-field.PROVENANCE.py`): one field `Note`
with `/Ff` RichText, `/V (Old bold)`, `/RV` holding `Old <b>bold</b>`,
`/DS (font: 14pt Helvetica)` and an empty `/AP /N`, so its box is blank
until it is filled.

# The assertions

The document opens in Review; pointer and keys are scripted.

1. The box holds no ink before typing (the control).
2. A click in the box opens the editor (`form-focus field=Note`).
3. Typing ` WWW` and clicking empty page commits one
   `form-fill-rich-text commands=1` line: the fill went through
   `fill_rich_text_field`, not the plain fill the engine refuses.
4. That line's `redrawn=` carries
   `replaced the formatting it held (bold)`.
5. The box holds ink after the commit.

# Falsification

- The canvas commit routed to `FillText` whatever the field: FAIL at 3.
- The fill's notes dropped: FAIL at 4.
