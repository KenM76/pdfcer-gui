# `dialogs::settings` — where a spec ambiguity becomes a choice

## What this surface is for


> Where standards are ambiguous those should become settings that the user
> can choose direction one, with the initial installed default as the best
> guess of what is usually followed.

`pdfcer_core::settings` is the store that makes such a choice survive a
restart. This module is the only place an operator can *make* one without
opening a text editor — and the no-affordance-without-capability rule cuts
both ways here: **a setting the program honours but offers nowhere is just
as much a gap as a control that does nothing.**

The engine honours its settings whether or not anything offers them, which
is how a program reaches the state the operator reported: features added
with *"no surface for changing or editing the settings for them."*

## Why a window and not a dock panel

The dock's own test is **"selection state is watched, workflows are
entered."** Watched things — the page rail, the object tree, the armed
tool's options — earn a permanent compartment because they are consulted
continuously *while doing something else*. Settings are the opposite:
consulted in bursts, deliberately, when there is something to change.
Properties is out of the dock for precisely this reason; putting Settings
*in* would be the same mistake with a different noun.

So it is a window, opened from the File tab's pdfcer group, in the shape an
operator arriving from Office or PDF-XChange already expects of *File →
Options*.

## Groups, and why a heading is navigation rather than tidiness

**An operator opens this window with a *symptom*** — "my black lines look
grey", "copied text has no spaces", "my dimension came out as an angle" —
and the group headings are how a symptom finds its setting. A flat list
makes the reader scan every row.

Which means a setting filed under the wrong heading is not untidy, it is
**unreachable**. `parallel_epsilon_degrees` governs whether two lines are
dimensioned as a distance or an angle, so it belongs in [`measuring`] and
not under *Copying and extracting text* — which is where being a slider
like the word-gap one would put it.

**The group list and its order live in [`show`]'s body, which says so and
is the contract.** They are deliberately not restated here: a second copy
of an order that changes whenever a setting is added is a copy that goes
quietly wrong, and a wrong map is worse than no map.

### Colour is the one group that starts expanded

It holds the setting most likely to have brought someone here — and the
only one whose default knowingly differs from other PDF viewers, which is
the "my black lines look grey" symptom. Every other group starts collapsed.

## The three obligations, enforced by a function signature

A settings screen that listed keys and radio buttons would satisfy nobody
here. Three things must be visible that a conventional one omits:

1. **What the default rests on.** Most of these defaults are *reasoned
   inference* — a guess — and a guess must say it is a guess. Exactly one
   is well-sourced and says that too.
2. **That a choice was made at all.** These are settings *because the
   standard declines to have an opinion*. An operator who does not know
   that reads a difference between pdfcer and Acrobat as a pdfcer bug.
3. **Which way costs what.** A setting whose blast radius is the SAVED
   BYTES is a different kind of decision from one that only changes the
   preview.

Obligations 2 and 3 are not left to discipline: [`widgets::header`] takes
`title`, `silence` and `radius` as **required arguments**, so a setting
cannot be added without answering all three. Obligation 1 lives in the
option notes and is pinned by tests in [`crate::text::settings`].

## Cancel is real

The window edits a **working copy**. Nothing reaches the live configuration
or the disk until *Save*, and *Cancel* discards the lot. This is not
ceremony: several of these settings change **saved bytes**, so a radio click
that took effect immediately would be an edit the operator never intended
and cannot see.

**Theme is the single exception**, and it is deliberate — see [`Draft`].
