---
name: never-debug-format-a-field-a-machine-reads
description: A `{:?}` tuple in a key=value trace made a driven check report the opposite of the truth, while quoting the truth in its own failure message.
metadata:
  type: feedback
---

A trace field that a check parses must be **flat, scalar, and spelled the same
way everywhere it appears**. Never `{:?}` a tuple, `Option`, or any domain type
into it.

**Why:** 2026-09-05. `edit-text-classified` emitted `character={missing:?}` on
an `Option<(char, String)>`, rendering a debug tuple containing a space and a
comma. The sibling surface emitted `character='q'`. A `key=value` reader split
on whitespace, got a truncated fragment from one and `'q'` from the other,
compared them, and reported:

> **THE OFFER DOES NOT NAME THE CHARACTER**

…in a message that **quoted the offer naming the character**. The application
was correct throughout. That failure cost a driven run, a diagnosis and a
rebuild.

Three separate faults in one field:

1. **It contains delimiters** — a space and a comma — so any whitespace-splitting
   reader is wrong before it starts.
2. **It is a different shape from the field it must be compared with.** Two
   surfaces described the same character in two languages.
3. **`{:?}` makes the trace's vocabulary a consequence of a Rust derive**, so it
   changes silently when the type does. A check keyed on it is keyed on
   something nobody thinks of as an interface.

**How to apply:** when a check compares two trace fields, make the *emitters*
agree — do not loosen the comparison. Loosening leaves the two surfaces
speaking different languages and hides the next mismatch. If a value has two
parts, give it two fields. Related:
[[a-driven-failure-is-a-claim-about-the-check-too]],
[[a-long-green-check-can-be-aiming-at-nothing]].
