# `ui-verify/checks/repeated_word`

`ctrl_b_bolds_the_second_copy_of_a_repeated_word` — **a restyle of a word
that occurs twice in one piece of text restyles the copy the caret is in.**

Off-screen, scripted pointer, no OS input. On a copy of
`fixtures/repeated-word.pdf`, whose one show operator reads `M10 x M10`: a
click inside the second `M10` opens a caret, the Format tab is raised, Ctrl+B
bolds the word at the caret, and Ctrl+S saves the copy in place.

Oracles:

- One `text-span-style-applied … applied=1` line and no `text-style-declined`.
- `save-in-place … outcome=ok`.
- The saved revision's appended bytes hold a string starting `(M10 x ` — the
  first copy still in the operator it started in — and no `( x M10)`, the
  remainder a restyle of the first copy leaves.

The shell addresses a cut inside an operator by its characters plus which of
their non-overlapping matches it is (`FormatRequest::occurrence`, counted by
`textstyle::span::occurrence`).

Falsified: dropping `.occurrence(n)` from the request restyles the first
copy; the saved tail holds `( x M10)` and not `(M10 x `, and the check fails.
