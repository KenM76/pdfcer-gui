# `canvas::formfield::action` — what a push button does when it is pressed

The shell's model of `pdfcer_core::edit::ButtonAction`, and the one place it
is translated into the engine's type.

## Why a second enum instead of using the engine's directly

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

## What is deliberately NOT here

**Reading an existing button's action** — because that is not a draft.
`pdfcer-core` `28b982c` could write one and not read one back, which is why
this module served only the placement path; the reader landed the same day
(`request_a_buttons_action_can_be_written_and_not_read.md`, answered by
`Pass 212.0`) and lives in `panels::forms::button`, which converts INTO this
type rather than the other way round.


The engine shipped **four** states where three were asked for, and the
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

## Item notes

### `fn url_blocker`

# The two refusals are the engine's, restated, and each has a reason
that is about READERS rather than about safety

- **Relative** — §7.11.2.2 resolves it against the document's own location,
  and ISO issue #256 records readers disagreeing about §12.6.4.8's `/Base`
  concatenation badly enough that *"only the host portion gets used"* in
  some of them. Two readers, two destinations, one file.
- **Non-ASCII** — §7.11.5 requires RFC 1738 encoding; ISO 32000-2 then types
  `/URI` as an `ASCII string` in one column and *"encoded in UTF-8"* in the
  next.

**`http://` is allowed and is not a refusal.** Destination policy is open
by operator ruling — *"we'll allow a submit to send filled data wherever the
document's author said"* — and `https` appears **zero times** in ISO 32000-1.
Blocking it would be pdfcer inventing a conformance requirement. It is
disclosed instead, which is the honest half.

### `fn the_reader_landed_and_this_is_what_was_owed`

It read *"a tripwire that names its own deletion"* and asserted that
`pdfcer-core` could write a button's action and not read one — so this
module served the PLACEMENT path only, and the Forms panel had no row
for a button already in the document.


Kept as a headstone rather than deleted outright, because the shape
paid out for the fifth time in three days and the count is the
argument: a test that ASSERTS a limitation goes red on the first build
after `cargo update` that lifts it, and names the code to change while
somebody is still looking.

What survives as an assertion is the half that is still true: `Nothing`
clears, and every other kind writes.

### `enum ButtonDoesKind`

[`Self::ALL`] is the exhaustive list the picker is built from, so a new
variant reaches the operator by existing. No count is stated here: the
array's own length is the claim, and it is one the compiler checks.

`Nothing` is first and is the default, because that is what
`add_push_button` authors and this shell does not change a document's
meaning by having a dialog open. Choosing anything else is a deliberate act.

### `const ALL`

Ordered by **reach**, not by the standard's section numbers: the four
that cannot leave the document come first, then the two that write an
address into the file. An operator scanning the list meets the safe ones
first and the two that need a sentence of disclosure last, which is the
order a chooser should be read in.

### `fn reaches_outside`

The predicate the dialog uses to decide whether a disclosure block is
drawn. **Not** a predicate about danger — a `Uri` is inert until a human
clicks it in a viewer — but about whether the file gains a statement
pointing off the machine, which is the thing an operator cannot see by
looking at the page.

### `struct ButtonDoes`

Every parameter for every kind, held at once. See the module header for why
this is a struct with a discriminant rather than an enum with payloads: a
dialog that lost the page number when the chooser moved to *Reset* and back
would be punishing the operator for looking.

### `enum PageViewChoice`

A shell mirror of `pdfcer_core::edit::PageView`, for the same reason
[`ButtonDoes`] mirrors `ButtonAction`: this one is `Default` and `Copy` and
sits in a draft that is cloned every frame.

### `enum ActionBlocker`

Returned rather than rendered, so the caller decides where the sentence
goes — the dialog puts it under the chooser and greys Add; a driven check
reads the discriminant. A function that drew the message itself would make
the condition untestable except by screenshot.

### `fn blocker`

Checks only what can be checked without the document. Everything that
needs one — does that page exist, is that field name terminal, is it
even a push button — is the engine's, and its refusals are reported when
they arrive.

### `fn target_names`

The same treatment a choice field's options get, for the same reason: a
text box has a trailing newline after the last line typed, and without
discarding empties every button would carry a final target that is the
empty string — which the engine would refuse as a field that does not
exist, naming a name the operator never typed.

### `fn to_core`

# Returns `None` for two different reasons, and the caller must not care

*Nothing* and *a draft that [`Self::blocker`] refuses* both answer
`None`. That is safe **only** because the one caller checks `blocker`
first and does not reach here otherwise — which the dialog enforces by
greying Add. Stated here because a second caller written later would not
know, and the failure would be a button silently authored inert.
