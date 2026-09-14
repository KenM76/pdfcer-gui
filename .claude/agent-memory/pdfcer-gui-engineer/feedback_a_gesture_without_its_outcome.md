---
name: a-gesture-without-its-outcome
description: "drag pages between them" was true of the gesture and wrong about the result — it copies, Shift moves — and the module header implementing it had been wrong for 25 days, falsified 4h44m after it was written
metadata:
  type: feedback
---

**Naming a gesture without naming its outcome publishes the outcome the reader
already expects.** `README.md` said *"Open several documents at once and drag
pages between them."* Every verb around it was reachable and the sentence never
claimed a move — it did not have to. Anyone who has dragged a file in Explorer
reads a drag between two containers as a move. It is a **copy**; Shift moves.

**Why:** this is a sibling of the engine's R257 (*a citation that does not name
its repository is not a citation*) and of its claim-3 finding (*in a paragraph
about the application, a verb that exists only in the CLI reads as a promise
about the window*). All three fail the same way — **by resolving**, to
something real, rather than by erroring. The reader's habits are as much a
resolution context as a repository is. And the cost profile is the bad one: an
operator who assumes a move finds out tomorrow, on the drawing they did not
have open, or by assuming the source is short and deleting the wrong copy.

**How to apply:** when product copy names an interaction, ask what the reader's
*previous* application taught them it does. If the answer differs from what
ours does, the sentence needs the outcome in it, not a footnote. One-line test:
would a competent operator bet money on what happens after the mouse is
released? Related: [[a-capability-claim-in-product-copy-needs-the-same-citation-as-a-limitation-claim]].

---

## ★★★ The same fact was wrong in the code's own module header, and had been for 25 days

The README was the **second** place. `pagedrag.rs` — by its own words *"the
module every reader of the feature reaches first"* — opened with the heading
**"A drag between documents is a COPY"**, and that header has been cited from
other documents in this project.

The timeline is the finding:

| commit | time | what it did |
|---|---|---|
| `d829ae2` | 2026-08-20 00:31 | shipped the cross-document drag, copy-only, and wrote that header |
| `b28e320` | 2026-08-20 **05:15** | added Shift-to-move, in the same file, and did not touch the header |

**Falsified four hours and forty-four minutes after it was written, by the next
commit, and it then stood for twenty-five days.** Nothing a compiler, a test or
a gate can see was ever wrong: the modifier's own doc comment further down the
same file is correct, complete, and explains the Windows convention it follows.
The file disagreed with itself and every instrument stayed green.

⇒ **A module header is a summary of code that keeps moving, and the commit that
moves it edits the code, not the summary.** When a commit changes what a module
*does* — not how it does it — the header is part of the change set. The tell is
a commit whose subject line would make a good heading: if the subject describes
behaviour, some header already claims the old behaviour.

Same family as [[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]]
(hours, not months, is the real shelf life) and
[[a-contract-you-write-for-someone-elses-function-is-a-claim-to-measure]].
