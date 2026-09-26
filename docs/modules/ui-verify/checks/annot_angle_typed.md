# `ui-verify/checks/annot_angle_typed`

`the_typed_angle_turns_a_mark` — **type a number into the Properties
panel's Angle field, press Apply, and the mark ends at that angle.**

# What this is for

The operator, 2026-09-07: *"also the angle should be editable from the
properties."* The **read** half of that has been driven since the day it
shipped — `rotating_a_markup_turns_it` asserts the field shows `270.85`
after a `-89.15` drag. The **write** half had not been driven at all, and by
this project's founding rule that means it was not done, however many unit
tests stood behind it.

## The two things only a driven run can see, and they are the whole risk

| # | link | its own test |
|---|---|---|
| 1 | the Angle field is laid out, published, and reachable | **nothing** — it is behind a dock tab, a mode, and a scroll |
| 2 | a scrub changes the draft rather than the document | `GeometryDraft` — the arithmetic, given values |
| 3 | Apply is enabled by an angle change alone | `differs_from` — the predicate, not the button |
| 4 | **the action raised is `SetRotation` and NOT `Rotate`** | **nothing** |
| 5 | **the number that travels is the ABSOLUTE angle, not a delta** | **nothing** |


⇒ So this check asserts the **trace's `asked=` field**, which carries the
number the panel handed the verb, against the number that was typed. A build
that passed a delta through would show `asked=` equal to the *change*
rather than to the destination.

## Why it turns the mark FIRST, before typing

Because an absolute setter and a delta setter are **indistinguishable on an
unturned mark**: from 0°, *set 30* and *turn by 30* are the same edit. The
check therefore drags the rotate handle a quarter turn before it types, so
the mark is at ~270.85° when the typed value arrives — and *set 30* and
*turn by 30* then differ by 270°, which no tolerance can absorb.

That is the same shape as the engine's own note about its A/B test: an
oracle that cannot distinguish the correct implementation from the plausible
wrong one is not measuring the thing its name claims.

## What it deliberately does NOT assert

**The final `/Rect`.** The engine reports the delta it worked out
(`deg=`), the rule it derived the rectangle from (`rect_derived=`) and the
new extent (`to=`), and all three are *reported* here rather than asserted:
the arithmetic has eight unit tests in `pdfcer-core` and a composability
test in this repository, and a second copy of the expected numbers here
would be a third place to maintain them. What this check owns is
**which verb the button reached and with what number**.
