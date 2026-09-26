# `canvas::formfield::action` — what a push button does when it is pressed

The shell's model of `pdfcer_core::edit::ButtonAction`, and the one place it
is translated into the engine's type.

## ★★★ Why a second enum instead of using the engine's directly

Three reasons, and only the third is about types.

1. **A draft is edited, and an engine action is complete.**
   `ButtonAction::GoToPage { page_index, view }` cannot represent *"the
   operator has chosen Go to a page and has not said which one yet"*, and a
   dialog spends most of its life in exactly that state. The variants here
   carry every parameter for every kind at once — the way a dialog holds a
   text box's contents whether or not that box is showing — so switching
   the chooser to *Reset* and back does not lose the page number that was
   typed. That is [`ButtonDoes`]'s whole shape and it is why it is a struct
   with a `kind` rather than an enum with payloads.

2. **The engine's type is `#[non_exhaustive]` and its constructors are
   private-by-omission.** `SubmitSpec::new(url)` then assign fields; a
   struct literal from outside the crate does not compile. A dialog that
   held one would have to build it fresh on every frame, which is the same
   work as building it once at commit — so building it once at commit is
   what this does.

3. **Not every draft is authorable, and the refusal has to be shown before
   the press.** [`ButtonDoes::blocker`] is the predicate the dialog greys
   its Add button on, and it exists so the operator is not told *"that URL
   is relative"* by a dialog that has already closed.

## ★★ What is deliberately NOT here

**Reading an existing button's action** — because that is not a draft.
`pdfcer-core` `28b982c` could write one and not read one back, which is why
this module served only the placement path; the reader landed the same day
(`request_a_buttons_action_can_be_written_and_not_read.md`, answered by
`Pass 212.0`) and lives in `panels::forms::button`, which converts INTO this
type rather than the other way round.


★ The engine shipped **four** states where three were asked for, and the
fourth is the one that makes the row honest. `panels::forms::button`'s
header carries that argument; this module is unchanged by it, because a
DRAFT has no fourth state — an operator is always editing something this
shell can express, or it would not have offered to edit it.

## `/JavaScript` and `/Launch`

Absent, permanently, and not as an omission: `pdfcer-core` refuses both by
name and this shell agrees with the refusal. There is no *Run a script* row
to grey — R9's rule is that an unavailable capability renders **nothing**,
and a greyed *Run a script* would advertise a capability pdfcer has decided
not to have.
