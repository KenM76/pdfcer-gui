# `dialogs::settings::defaultapp` — the button at the top of Settings

`OPERATOR_REQUESTS.md` **O173**: *"Then it should be in the top of our
settings as a button to execute the changeover."*

## ★★★ Why this is FIRST, above the presets row

[`super`]'s ordering rule runs from what the **program** looks like, through
what the **document** is made of, to what pdfcer **does with it** — and the
presets row sits above all three because it sets all three. This group is
above even that, and it is the only thing in the window that is not a
setting at all: it is an **act**, performed once, whose effect is outside
pdfcer entirely.

The operator asked for it there by name, and the reason it is the right
place survives independently of the ask: it is the one control here that
somebody arrives at Settings *specifically* to press, having just
double-clicked a drawing and watched the wrong program open. A control found
by scrolling is a control found by people who already knew it existed.

⇒ It is **not** a collapsible [`super::widgets::group`], for the same
reason. A collapsed heading is a heading whose button is not on screen, and
`super`'s own comment records what that cost the last time: *"Opening
Settings showed nothing but a list of standards."* Four short lines is a
price worth paying at the top of a scroll area; a hidden button is not.

## ★★ It is drawn whether or not it would do anything

R9's escape hatch, the same shape [`super::acrobat`] argues at length. There
is no ribbon command for this and there never will be — it is a machine
setting, not a document one — so this group is the **only** evidence in the
program that the capability exists. An operator who ticked *Don't ask me
again* on the startup offer, and later changed their mind, has exactly one
place to look, and it must be here whatever state the machine is in.

## ★ Rule 4 — fuzzy, never sneaky

No canvas is involved, so the marking clause is not in play. The disclosure
clause is, and it is the whole of [`state_line`]: pdfcer performs half an
act and cannot perform the other half, so it says which half it did and
reads back what Windows actually thinks rather than reporting its own
intention. See [`crate::app::assoc`] for why the second half is impossible
and [`crate::text::assoc`] for the wording that carries it.

## Rule 15

No dimension of either kind appears in this module.
