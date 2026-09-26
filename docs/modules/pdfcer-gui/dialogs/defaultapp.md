# `dialogs::defaultapp` — the offer, made once

`OPERATOR_REQUESTS.md` **O173**: *"Ask once with a don't show me again check
box option."*

## ★★★ Why an application may ask this at all, when it may not nag

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

## ★★ What this window cannot promise

No program can make itself the default PDF viewer on Windows 10 or 11 —
[`crate::app::assoc`]'s header carries the mechanism. So the button performs
half an act and hands the other half to Windows, and every word here is
written to make that expected rather than surprising: an operator who is not
told a Windows dialog is coming will read that dialog as something going
wrong. See [`crate::text::assoc::body`].

## ★ The buttons scroll nothing and are always reachable

Built on [`crate::dialogs::host::Host::scrolled`], which is the operator's
standing rule of 2026-09-10 — *"Those buttons should always be available,
and if there isn't size for all the features they get scrolled in their own
space"* — made structural. There is no size this window can be at which the
answer is off-screen.

## Rule 15

No dimension of either kind appears in this module.
