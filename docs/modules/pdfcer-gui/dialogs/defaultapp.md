# `dialogs::defaultapp` — the offer, made once

`OPERATOR_REQUESTS.md` **O173**: *"Ask once with a don't show me again check
box option."*

## Why an application may ask this at all, when it may not nag

R8b rule 4 is about **not marking a document with pdfcer's own
uncertainty**, and this window marks nothing: it is a question about the
machine, asked once, with a permanent way to stop it. That is the shape
every browser and every PDF viewer on Windows uses, and the standing rule
*"use the conventional interaction, never invent one"* makes that
convergence the specification rather than a precedent to improve on.

What would make it nagging is asking **again after the operator engaged**.
So it does not, and the rule is spelt out in [`DefaultAppDialog::settle`]:

- **Pressing the button** stops the offer. They answered.
- **Ticking the box** stops the offer. They answered a different way.
- **Not now**, box unticked, leaves it alone — and it is asked again next
  launch, exactly as Chrome and Acrobat do, because *"not this time"* is not
  *"never"*.

⇒ And it stops on its own the moment pdfcer **is** the default, because the
condition that opens it is a live reading of Windows rather than a
remembered fact. See [`should_offer`].

## What this window cannot promise

No program can make itself the default PDF viewer on Windows 10 or 11 —
[`crate::app::assoc`]'s header carries the mechanism. So the button performs
half an act and hands the other half to Windows, and every word here is
written to make that expected rather than surprising: an operator who is not
told a Windows dialog is coming will read that dialog as something going
wrong. See [`crate::text::assoc::body`].

## The buttons scroll nothing and are always reachable

Built on [`crate::dialogs::host::Host::scrolled`], which is the operator's
standing rule of 2026-09-10 — *"Those buttons should always be available,
and if there isn't size for all the features they get scrolled in their own
space"* — made structural. There is no size this window can be at which the
answer is off-screen.

## Rule 15

No dimension of either kind appears in this module.

## Item notes

### `fn settle`

The whole of O173's *"ask once"* lives in this function, and the
asymmetry is deliberate:

| What they did | Asked again next launch? |
|---|---|
| Pressed the button | **No.** They engaged; asking again is nagging. |
| Ticked the box | **No.** That is what the box says. |
| *Not now*, box clear | **Yes.** *Not this time* is not *never*. |

The third row is the conventional behaviour of every browser on the
platform, and the standing rule is that the convergence of the product
class is the specification.

⚠ **The offer is what stops, never the capability.** The button stays at
the top of Settings whatever is written here — `dialogs::settings` does
not read this preference at all, which is the mechanical form of that
promise rather than a comment claiming it.

# The save failure is swallowed, and that matches every other preference

`print::remembered::remember` states the rule: one discrete operator
decision is one write, and losing a preference across a restart does not
justify a modal in front of somebody who has just declined a dialog. The
worst case is being asked once more.

### `fn engaging_with_the_offer_silences_it`

The row of [`DefaultAppDialog::settle`]'s table most likely to be
removed by somebody tidying: it looks like the checkbox's job. It is
not — asking again after the operator engaged is the nagging this
project refuses, and the remedy for a change of mind is the Settings
button rather than a question that comes back.

### `fn the_box_is_not_pre_ticked`

A pre-ticked *don't ask me again* suppresses itself when somebody
dismisses the window without reading it, and there is no way to notice
that happened.

### `fn a_silenced_preference_is_never_asked`

This asserts only the cheap half — that the preference gates the
question — because the other half spawns processes and depends on the
machine the test runs on. The live half belongs to a driven check.
