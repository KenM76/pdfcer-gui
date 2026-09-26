# `text::new_document` — the copy of the sized-New dialog

Every operator-facing string `crate::dialogs::new_document` draws. One
function per string, per `crate::text`'s contract: the gate
`tools/gates/check-ui-strings.sh` fails the build for a literal that
reaches a widget from anywhere else.

## What this surface is a size chooser FOR, which decides the wording

Not for drafting. Nobody drafts a sheet in pdfcer — documents arrive from
SolidWorks — and `crate::app::blank`'s header says so plainly while
arguing the A4 default. What this is for is the case the header also
names: *"A4 is very plausibly not the right size for this operator's next
new sheet"*, whose own drawings are **A3 and A1**.

So the copy is short and assumes competence. An operator opening this
knows what A1 is; what they need is to find it quickly, see it confirmed,
and get out. There is no explanation of what a page size is, no advice
about which to pick, and no reassurance.

## Millimetres, and why there is no unit toggle

The size list is ISO-first because the operator's corpus is, and every
entry states its size in **millimetres**. There is no inches toggle, and
that is a decision rather than an omission:

- The A series is *defined* in millimetres. A1 in inches is 23.39 × 33.11,
  which is a number nobody recognises.
- The US and ANSI entries are in the same list and would want inches, so a
  toggle would be right for four entries of sixteen and wrong for twelve.
- This shell already has a units answer elsewhere and it is **not global**:
  the measure tools express a length in whatever the *dimension group's*
  own `NumberFormat` says, which is a per-document drafting convention, not
  an application preference. A unit switch here would be a second,
  unrelated units concept for a dialog that is open for four seconds.

The custom fields are therefore millimetres, stated in the label rather
than in a suffix an operator can miss, and the resulting sheet is echoed in
**both** units by [`sheet_summary`] so a Letter-minded reader is not left
converting.
