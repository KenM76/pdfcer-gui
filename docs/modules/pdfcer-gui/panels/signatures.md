# `panels::signatures` — three facts about each digital signature, reported
separately and never merged


## ★★★ THE RULE THIS PANEL EXISTS TO KEEP

**This is the one place in the product where a wrong answer is worse than no
answer.** A panel that said *"trusted"* about a chain it had not really
validated would be the exact failure mode the engine's design prevents — and
the engine prevents it by refusing to produce one verdict at all:
`SignatureVerdict` carries `integrity`, `coverage` and `trust` as **three
facts that never collapse into one bool**.

So this panel draws three labelled lines per signature, always all three, in
that order. There is no badge, no tick, no colour that means "fine", and no
arithmetic anywhere in this file that combines two of the facts into a
third. The order is also the order of increasing uncertainty: integrity is
arithmetic, coverage is arithmetic, trust is a judgement about the world.

★★ And [`pdfcer_core::signature::Trust::NotChecked`] renders **as itself**.
Not as a soft "no", not as a grey tick, not omitted. Hiding it would make
this panel indistinguishable, on screen, from one that had checked and found
nothing wrong — which is the inversion the whole feature exists to prevent.
[`crate::text::trust`] holds four separate sentences for the four reasons
trust can go unchecked, because *"you turned it off"* and *"your Acrobat
store is corrupt"* are opposite calls to action.

## What changed at the rewrite, and what deliberately did not

| | before | now |
|---|---|---|
| integrity | not computed; the panel said so in its first line | `verify_all_with_trust` |
| coverage | `byte_range_coverage`, per frame | unchanged — it is the same measurement, now printed beside the other two |
| trust | not computed | the operator's own Acrobat anchors, opt-in, off by default |
| the leading caveat | *"pdfcer does not check whether these signatures are valid — it cannot yet"* | replaced: that sentence became FALSE the moment this was wired |

★★★ That last row is this project's most expensive recorded failure shape —
a claim that was true when written and false within hours, with the prose
around it still true. The replacement
([`crate::text::trust::panel_intro`]) names the three facts instead of
denying one of them, so it cannot go stale the same way: it describes the
SHAPE of the report rather than the limits of a build.

## The file length, and the bytes, are read from DISK

`/ByteRange` is a claim about bytes, so it can only be checked against
bytes — which is why
[`pdfcer_core::signature::byte_range_coverage`] takes the length as a
parameter rather than reading it from the object graph: *"the object model
cannot check a claim about bytes against itself."* Verification needs the
same thing one step further: the real file, digested.

What is used is the file **on disk right now**, not a length captured when
the document was opened, and not the session's rendering of it. It answers
*"do these signatures hold over the file as it currently exists"*, which is
the question worth asking. An unsaved edit is not in the file and cannot be
covered by a signature; the panel says which state it measured rather than
leaving an operator to assume.

## ★★ Why verification is automatic, and what stops it being per-frame

The alternative considered was a *Check signatures* button. It was refused:
an operator who has opened a panel called Signatures has already asked, and
a button would leave the panel's resting state showing coverage numbers with
no integrity beside them — which is precisely the state this work exists to
end.

What makes that affordable is [`crate::trust::cached_report`], whose key
carries the file's identity, its length, its modification time and **both**
halves of the trust configuration. A cache that missed any of those would be
worse than none: the panel would show a verdict about a file, a setting or
an anchor set that is no longer in force, and it would look exactly like a
correct answer.

## This panel raises no actions, and cannot

Nothing about a signature is editable from anywhere in pdfcer — the engine
cannot sign, and this shell would have nothing to send it if it could. The
`actions` parameter is present because the dock calls every panel through
one signature; this one never pushes to it.
