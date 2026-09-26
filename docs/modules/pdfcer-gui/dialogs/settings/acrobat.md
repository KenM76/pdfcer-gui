# `dialogs::settings::acrobat` — where Acrobat is, when the operator has
had to say

`OPERATOR_REQUESTS.md` **O122**: *"have a setting where people can change
it."* One control, and — like [`super::comments`] — a module for it because
the argument is about placement and about what the control has to explain.

## This group is visible whether or not discovery succeeded, and that
is the whole decision

O122's escape hatch, in the row's own words:

> **The escape hatch is that the path control lives in Settings and is
> visible there whether or not discovery succeeded**, so a non-standard
> install is fixable without the button ever having appeared.

The button beside Read / Review / Edit is **absent** on a machine where no
Acrobat was found — R9, an unavailable capability renders nothing. That is
right, and it has a consequence that has to be answered somewhere: a person
whose Acrobat lives on a volume Windows was never told about sees **no
evidence that the feature exists at all**. There is no greyed control to
hover, no menu entry to find, nothing to right-click.

So this group is the only place they can be told. It is always drawn, it
explains what the button would do, and it says out loud what state pdfcer
is currently in — see [`path`] and
[`crate::text::acrobat::resolved_note`].

Hiding the group when nothing was found would be the exact mistake R9
prevents in the other direction: the remedy for an absent capability must be
reachable, and an absent capability whose remedy is also absent is a dead
end.

## Why the resolved state is shown, and not just the field

Because without it a typo is invisible. A person who types
`D:\Apps\Acrobatt.exe` and a person who types the correct path see exactly
the same thing — a filled-in field — and both then look at a ribbon with no
button on it. The line underneath is what tells them apart, at the place the
mistake was made.

⚠ It reports the state as of the **last time pdfcer resolved**, which is
start-up or the last Settings save. So it does not update as the operator
types; it updates when they press Save, which is also when the button
appears. That is honest — the line and the button change together, so they
can never disagree — and it is stated here because a reader may otherwise
expect it to be live.

## Placement: last, with the program-level settings

[`super`]'s ordering rule runs from what the **program** looks like, through
what the **document** is made of, to what pdfcer **does with it**. This is
none of those either — it is a fact about *another program on this machine*,
which is a first for this window. It sits at the very end, after the shell's
own preferences, because it is the setting furthest from the document: every
group above changes something about a PDF, and this one changes nothing at
all except which program a single button starts.

## Item notes

### `const REGION_RESOLVED`

Named, because the whole value of that line is that it is **on screen and
legible**, and `ui-verify` can only assert that about a rect the
application published. A driven check that read the trace would learn what
pdfcer resolved and nothing about whether the operator can see it.

### `fn path`

`text_value` with an identity parse, exactly as [`super::comments`] uses
it and for its stated reason: the helper exists to hold a half-typed
*number* apart from a parsed value, and a path has no invalid intermediate
state. Every keystroke reaches the draft, so Save writes exactly what is on
screen.

**No validation as you type, and no red field.** A path that does not
exist is not a typing error — it is a path to something that is not there
yet, or on a drive that is not mounted, or typed from memory and about to
be corrected. Marking it wrong mid-word would be the field arguing with
somebody who has not finished. The resolved line below says what actually
happened, once, after Save, which is the moment the answer is knowable.

`resolved` is the application's live answer, passed in rather than computed
here: this module must not resolve, because resolving spawns processes and a
Settings pane redraws on every frame.
