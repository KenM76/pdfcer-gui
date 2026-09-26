# `dialogs::settings::saving` — three settings nobody can see

All three change the **bytes pdfcer writes** and none of them changes
anything visible. That is stated in all three radius lines in nearly the same
words, and it is the whole reason they are grouped together rather than filed
with the settings whose effects an operator can look at.

## The third one is stronger than "nothing visible", and its wording says so

[`xref_entry_eol`] and [`trailing_eol`] are invisible *in a viewer*.
[`quad_point_order`] is invisible **in pdfcer specifically**, and for a
structural reason: pdfcer bakes a full appearance stream for every markup
annotation (R44), so its own renderer never reads `/QuadPoints` back. The
order it writes there matters only to a third-party consumer that re-derives
the marked geometry — and a wrong order draws a bow-tie rather than a
rectangle, in somebody else's program, after the file has left.

That is the failure mode a settings window is genuinely for. An operator can
mark up a document, look at it, save, reopen, and be entirely satisfied while
the file is wrong for the recipient. Nothing on this side of the handover can
tell them.

## Why a setting nobody can see is worth having

Because pdfcer's round-trip discipline is a promise about bytes: **objects
pdfcer did not logically touch are re-emitted byte-identical.** A save that
rewrites two bytes on every line of a file's index is a diff of ten thousand
bytes in a document nobody edited — invisible in a viewer, and immediately
visible to version control, to a checksum, and to anyone diffing a
before-and-after.

That is a real consequence for the people who use this program, and the
window says so rather than treating "nothing visible" as "nothing".

## Item notes

### `fn only_the_legal_entry_forms_are_offered`

§7.5.4 permits exactly three, and the temptation a future hand will feel
is to add the others "for completeness" — bare `LF` in particular, since
it is what a text editor produces. Every one of them makes the entry the
wrong length and the file non-conforming.

Asserted by round-tripping each offered value through the engine's own
byte encoding: a form that is not two bytes cannot be legal, and a form
the engine does not know would not compile.

### `fn matching_nothing_falls_back_to_the_documented_form`

The case the default's own note promises — *"files that have no index of
this kind get a space then a newline"* — and one an operator will hit
without knowing it, because a cross-reference **stream** file has no
entry EOL at all.
