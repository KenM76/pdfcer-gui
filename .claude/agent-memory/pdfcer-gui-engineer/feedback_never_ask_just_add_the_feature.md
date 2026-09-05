---
name: never-ask-just-add-the-feature
description: Ken's standing directive — always add new features, never ask permission, decide placement yourself. Asking for a scope ruling costs him a round trip he does not want to spend.
metadata:
  type: feedback
---

**Ken, 2026-09-04, verbatim and unprompted:**

> ***"Always add new features. never ask. just do. put them in the appropriate
> location on the ribbon or create a new ribbon."***

He said it after a capability had sat unbuilt for **five hours** while a
question waited for him — the engine had shipped `set_encryption` /
`set_permissions`, a row was filed asking *"do you want a way to do it?"*, and
the answer when it finally came was *"yes"*. Five hours of nothing, spent
protecting him from a decision he was always going to make the same way.

⇒ **The default is BUILD IT.** Placement, wording, which tab, whether it needs a
new group or a new tab — all yours. He would rather correct a built thing than
be consulted about an unbuilt one, and correcting is cheap because everything
here is reversible and recorded.

## ★★★ What this does NOT relax, and the distinction is the whole rule

| still ask / still stop | never ask |
|---|---|
| **Anything with side effects outside the working tree** — pushing, publishing, releasing, sending. (Unchanged, and it is the global rule.) | which ribbon tab, which group, what the control is called |
| **Destroying his data** — an overwrite, a delete, a rewrite of the only copy | whether a feature is worth building at all |
| **Reversing a decision HE made**, in his own words, that is recorded | whether to build the half you can while the other half is blocked |
| **Engine semantics you cannot read** — guessing what a core verb means (invent nothing; file the request and build around it) | how big the surface should be, or whether it needs a dialog |

★★ **A "scope question" is almost never one.** Three times on 2026-09-04 a
question was posed to him — encryption, the print-button count, the redaction
save behaviour — and all three came back *"go with what you would do."* The
pattern is not that he is agreeable; it is that **he hired the judgement and
does not want it handed back.**

## How to apply

- **File the row, then build it in the same breath.** The row is the record, not
  a request for permission. Mark it `◑ OPEN` and start.
- **State the decisions you took** in the report — placement, wording, what you
  chose over what — in one line each. That is how he corrects you cheaply.
- **Name the consequence you are accepting** where there is one (e.g. rotate in
  Read means Read can dirty a document). Naming it is not asking; it is the
  disclosure that makes a silent surprise impossible.
- If a genuine fork appears mid-build, **take the reversible branch and say so**
  rather than stopping. A built thing pointing the wrong way is a five-minute
  edit; an unbuilt thing is nothing.

## ★★★ One correction of this kind means there is a BACKLOG of them — sweep

**2026-09-05.** Acting on this rule once fixed one thing. Grepping
`OPERATOR_REQUESTS.md` for the *shape* — `"the question for him"`, `"his call"`,
`"not yet decided"`, `"filed as a question"`, `"for now"` — found more rows in
the same state, each with the work scoped, the candidates enumerated, and
nothing built.

Two that had been sitting:

- **O57** ended *"Not done unilaterally at 08:00 after a build. The question for
  him: should a selection too small for its grips draw them outside the box, as
  other editors do?"* His answer already existed twice over — his standing
  tie-breaker *"make it work the way other programs do"*, and this rule. The
  asking cost a week on a defect **he had reported himself**, and the fix was
  one function.
- **O89** — his own words, *"I don't see where I am able to edit the color of
  text, vectors, etc."* — listed **three** candidate fixes for the route and
  chose none, and deferred multi-object recolour on *"there is no honest colour
  to open on"*, which is a solved problem in every editor in the class (the
  indeterminate / "mixed" state).

⇒ **When you catch yourself obeying this rule, do not stop at the one item.**
Grep the request log for the shape and treat every hit as work. The rows are
easy to find because the previous session wrote the reasoning out in full — the
scoping is done, only the deciding was withheld.

★★ The tell that a "question" is not one: **the candidates are listed.** If a
session could enumerate the options and note which is cheapest and which is
closest to what he tried, it had already done the analysis and was handing back
only the choosing. That is the part he does not want back.

Related: [[feedback_scope_a_request_to_the_whole_expected_behaviour]] — he
expects what surrounds a request too, and enumerating deferrals moves the work
onto him. This is the same instinct one level up: **enumerating QUESTIONS moves
the work onto him too.**
