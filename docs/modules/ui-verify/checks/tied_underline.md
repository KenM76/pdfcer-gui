# `ui-verify/checks/tied_underline`

`ctrl_u_underlines_the_text_itself` — Ctrl+U marks the characters in the page
content so the line follows the text, Underline reads back pressed from that
mark, and a second Ctrl+U takes the line off.

# What it drives

`fixtures/repeated-word.pdf` (ignores `--pdf`), whose one line shows `M10`
twice; the caret goes into the second copy, at page point (118, 703).

1. Scripted click on `ribbon.tab.format`.
2. Ctrl+U. Owed: `text-decorate-applied ... on=true applied=1`, then the
   shell trace's last `ribbon-item-selected id=format.underline` after the
   press reads `selected=1`. Pressed is read from the decoration on the
   glyph, not from a pending style.
3. Ctrl+S, then `save-in-place outcome=ok`. The bytes appended to the file
   must hold both `/Line /Underline /Id 1>> BDC (M10) Tj EMC` (the second
   copy's show operator wrapped in the engine's decoration marker) and
   `/pdfc_Deco <</Rule 1>> BDC` (the rule the engine derives from it).
4. Ctrl+U again. Owed: `on=false applied=1`, and Underline released.

# Falsified

- With the line authored as markup instead of through
  `FormatRequest::decoration`, step 2 reads `on=None` with Underline not
  pressed, and the saved revision holds a stroked line
  (`0 G 0.7704 w ... m ... l S`) and neither needle of step 3.
- Both step-3 needles were tested against the saved files of the passing and
  the falsified run: present in the first, absent in the second. A needle
  naming what the markup route *lacks* (`/Subtype /Underline`) was rejected:
  the markup route writes content, not an annotation, so both runs satisfied
  its absence.

# Driven off-screen

Scripted pointer and keys, window at `-4200,-4200`; it runs under
`--no-input`.
