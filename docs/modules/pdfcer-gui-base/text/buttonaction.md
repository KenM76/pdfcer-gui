# `text::buttonaction` — every word the *What this button does* chooser says

One module for one control, because the control is where this project's
rule-4 obligation is heaviest: two of the seven choices write an address
into the document that some other program may act on, and **the operator
cannot see that by looking at the page**. Everything they can learn about it
has to be said here.

## The disclosure rule these strings implement


⇒ *"Design it from `SubmitDisclosure`, not from Acrobat"* — a straight copy
of Acrobat's dialog would be a **regression** against what pdfcer can now
say. So these strings state the whole address, and the six facts about the
payload that ISO 32000-1 §12.7.5.2 makes true and nobody can guess.

## Where the disclosure is NOT

**Not on the canvas.** Rule 4's clause that is most often got backwards:
applied content renders exactly as saved content will. A button carrying a
submit gets no badge, no tint and no dashed outline on the page — it is
drawn as the file will draw it. The disclosure lives in this dialog, and
afterwards in the status line, which is off-canvas by construction.

## Not a warning, and not a refusal

None of these say *"are you sure?"*. No scheme, host or port is refused
anywhere — destination policy is open by operator ruling — and `https`
appears **zero times** in ISO 32000-1, so blocking `http://` would be pdfcer
inventing a conformance requirement. [`submit_unencrypted`] therefore
**states** it and lets the operator decide. Nothing here may be phrased as
*"the standard requires"*, because none of it is.

## Item notes

### `fn every_choice_is_named_and_explained`

The reach clause is the load-bearing half — see [`does_note`]'s comment
on why the four inert ones carry one too. A new variant added without a
note would fail to compile (the `match` is exhaustive); a new variant
added with an empty one would not, so this asserts non-emptiness.

### `fn the_submit_disclosure_names_every_fact_it_owes`

Asserted by keyword rather than by exact text so a rewording does not
break it — but a rewording that DROPS one of the four will, which is the
point. These are the facts an operator cannot learn any other way.

### `fn current_none`

# The four states, and why there are four

`EditSession::button_action` answers with `ButtonActionState`, and this
shell asked for **three** — `None`, `Known`, `Foreign`. `pdfcer-core` shipped
four and explained why, and the explanation is worth carrying because the
distinction is entirely about what a control may OFFER:

| state | what it means | what this row offers |
|---|---|---|
| `None` | no `/A` at all | "Nothing" — set one |
| `Known` | modelled; writable back unchanged | show it, change it |
| `Unmodelled` | pdfcer **authors** this subtype and did not decode this instance | name it, offer to **replace**, never claim to show it |
| `Foreign` | pdfcer recognises it and **will not author** it | name it, offer nothing |

`Unmodelled` and `Foreign` differ in exactly one thing — whether
replacing is offered — and that is the decision the operator is actually
being asked to make. A three-state enum would have forced a wrong answer in
one direction or the other: `Foreign("SubmitForm")` on a submit pdfcer writes
happily would grey a row that should have been live.

Today `GoTo` and `SubmitForm` answer `Unmodelled` — authored, not yet
decoded. That will widen, and widening is additive: a state that becomes
`Known` gains a value to show and loses nothing.

### `fn current_unmodelled`

The sentence must do two things at once and neither may be dropped: say the
button **does** something, and refuse to say what. Claiming to show it would
be the sneaky half of rule 4; claiming it does nothing would be worse.

### `fn current_foreign`

The variant the request argued for by name. `None` and `Known` could both be
synthesised by a shell that guessed; this one cannot, and it is what lets
the row say *"this button runs a script"* instead of silently offering to
replace one with "Nothing".

### `fn changed`

`replaced` is what the engine says was destroyed, **including a script**.
`ButtonActionChange::replaced` carries it as a `String` rather than an
`Option<ButtonAction>` precisely so a removed script is expressible — a form
editor overwriting another tool's work should know it did.

### `fn does_label`

Phrased as a question about the button rather than as *"Action"*, which is
the format's word and not the operator's. Acrobat's tab is called *Actions*
and this project's standing rule is to use the conventional interaction —
but the conventional *interaction* is a chooser of behaviours, and the label
on it may be the plainer one.

### `fn does_choice`

Verb-first and in the operator's terms, never the `/S` subtype name. A
chooser reading *ResetForm / GoToPage / SubmitForm* would be pdfcer showing
its own internals to somebody who wants a button that clears the form.

### `fn does_note`

Every one of the seven says what the action **reaches**, because that is
the property none of them shows on screen and the property they differ on.
The four that reach nothing say so in as many words, so that the two that do
are not the only ones carrying a sentence — a disclosure that appears only
on the dangerous choice teaches an operator to skip reading it.

### `fn targets_note`

The terminal-name requirement, said before the engine refuses it. Table
210 states nothing about descendant expansion — the phrase *"all descendants
of the specified fields"* occurs twice per edition of ISO 32000 and never on
this row — so a grouping name is a button that hides a subtree in one reader
and nothing in another. `pdfcer-core` refuses one by name; this says why
while the box that holds the mistake is still on screen.

### `fn submit_disclosure`

Shown whole, before the button exists, and every clause is sourced:

1. **Hidden fields are sent.** `Hidden` is an *annotation* flag; every
   submit selector addresses *field* dictionaries. The only field-level
   withhold flag that exists is `NoExport`. Different objects.
2. **Masked fields are sent as plain text.** `Password`'s NOTE constrains
   storage, not transmission.
3. **A file-select field sends the contents of the local file it names.**
4. **The baseline payload already carries this document's own file path and
   its trailer `/ID`** — with nothing configured, `/Flags 0`.
5. Not stated here because pdfcer writes the baseline: `IncludeAppendSaves`
   would turn a submit into a save. Named in the module header so that a
   later option to set it arrives with its sentence already written.
6. Not stated here for the same reason: `SubmitPDF` ignores field selection
   entirely.

It does not say "are you sure". It is a statement of what the file will
declare, positioned where the operator is deciding whether to declare it.

### `fn submit_unencrypted`

The standard states no TLS rule; `https` appears zero times in ISO 32000-1.
pdfcer does not invent one, so this is the whole of the response: say it, and
let the operator decide. Refusing would be pdfcer enforcing a rule nobody
wrote, and doing so silently would be worse.

### `fn blocker`

One sentence per blocker, each naming **the box to fix** rather than the
rule that was broken. An operator reading *"the destination is not
absolute"* has to work out which of four boxes that refers to.

### `fn placed_with_action`

Names the action in the operator's words, not the `/S` subtype, and is
the off-canvas half of rule 4: the button on the page is drawn exactly as
the saved file will draw it, and what it now *does* is said here.

### `fn two_undo_entries`

`EditSession::coalesce_last` answers `false` when the undo stack was shorter
than the count asked for — every change is applied and only the **grouping**
failed. So the button exists and does what it was asked to do; the only
thing wrong is that taking it back needs two presses.

Worth a sentence rather than a shrug: an operator who presses Ctrl+Z once,
sees a button still sitting there, and is told nothing will conclude that
undo is broken — which is a far worse belief than the truth.

### `fn action_refused`

**Two commands, and the second one can fail on its own.** `pdfcer-core`
authors the button and sets the action as separate verbs, so a refusal on
the second leaves a correctly placed button with no behaviour. Silence there
would be the exact defect this whole feature exists to remove — a button
that looks right and does nothing — arriving by a different door.

The engine's own words are appended, because they name the specific
condition: a page past the end, a field that is not there, a target that is
a group.
