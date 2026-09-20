---
name: a-request-for-something-shipped-is-a-discoverability-report
description: When Ken asks for a feature that already exists, the request is the bug report — find the route that failed him, don't just tell him where it is
metadata:
  type: feedback
---

When Ken asks for something that has already shipped, **do not answer by telling
him where it is.** The request is a defect report about the route, and the route
is what to fix.

**Why:** on 2026-08-28 he asked for *"a permanent setting where we can add font
locations"* — which had shipped the previous day, complete with a Tools command
literally named **Font folders** whose only purpose was to be findable. That
command opened the Settings window at the top of ten collapsed headings. Driven
with the landing removed, only three heading regions publish at all; the Fonts
group is not among them. He was one click from it, through a control named after
his own question, and it did not answer him.

⇒ **A setting he cannot find is a setting that does not exist to him.** Shipping
a capability and shipping a way to reach it are two pieces of work, and only the
second is finished when he can answer his own question.

**How to apply:** when a request names something that exists —

1. **Say so plainly in the row**, and say it is a finding about the surface, not
   a deflection. Do not quietly close it as already-done.
2. **Find the route that failed.** There usually is one, built for exactly that
   question, and it usually stops short: opens the right window, wrong place.
3. **Fix the landing**, and drive it. The check invokes the command and asserts
   the control published a *visible* region — never drive the scroll, which was
   tried, cost four fixes, and was reverted.
4. **Ship the literal ask too.** He asked for a checkbox; he gets a checkbox.
   Being right about the underlying cause is not a reason to skip what he said.

Related: [[scope-a-request-to-the-whole-expected-behaviour]] — he expects what
surrounds a request, and *being able to find it* is part of what surrounds it.
