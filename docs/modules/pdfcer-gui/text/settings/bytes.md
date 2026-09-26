# `pdfcer-gui/text/settings/bytes`

## Item notes

### `fn separations_silence`

Careful wording, and the care is the point. This is **not** a spec
ambiguity: §14.11.4 is perfectly clear about the invariant. What it does
not say is what an *editor* should do when an edit breaks it, and all three
answers are defensible for different workflows. Blurring the two shapes of
silence would make the window's whole framing dishonest.

### `fn missing_as_radius`

The only setting whose radius separately names **printing**, and it needs
to: an appearance chosen for the screen is the appearance that goes on
paper, and an operator checking a form before printing it is exactly the
person this setting is for.

### `fn missing_as_nothing_note`

The guess disclosure here is inverted from every other setting's, and
deliberately: what is disclosed is that *the other two are the guesses*.
Making either of them the default would be the "sneaky" failure the
disclosure rule forbids, because the operator would see a plausible
appearance with no indication pdfcer had chosen it.

### `fn xref_eol_radius`

Bytes and nothing else. Every value here is conforming, so unlike the
preview settings there is nothing for the operator to *see* and therefore
nothing to disclose beyond the fact itself.

### `fn xref_eol_match_note`

This default was changed on an operator ruling after the register pointed
out the shipped one was *"arguably wrong on pdfcer's own invariant"* — and
it was: objects pdfcer did not logically touch are re-emitted byte-identical,
and a full rewrite of a `CR LF` file under a fixed `SP LF` changes two bytes
in every entry. On a 5,000-object file that is a 10,000-byte diff in a
document nobody edited.

The note's *"below"* is only correct because the panel renders this option
**first**, which is not the order the functions are declared in. The
rendering order is the contract; see [`crate::dialogs::settings`].

### `fn xref_eol_space_cr_label`

No note: *"Space then carriage return"* describes itself completely, and
padding it would be noise. The `Option<&str>` in the option helper exists
for exactly these two entries.

### `fn trailing_eol_lf_note`

Both readings of the standard are self-consistent and it does not choose.
The note read as a plain recommendation; it now says which of the two
pdfcer picked and that it picked.

### `fn quad_order_silence`

It does NOT leave it open, and that is the honest and unusual thing to
have to say in a window whose every other silence line means *the standard
declines to choose*. Section 12.5.6.10 states an order and essentially no
producer follows it, so pdfcer is choosing between the clause and the world.
Saying "the standard is silent" here would be a comfortable sentence and a
false one.

### `fn style_policy_title`

Named for the ACT, not for the engine's type. `StylePolicy` means nothing
to an operator; *"faking bold and italic"* is what they will have seen
happen and the phrase they would search for.

### `fn style_policy_silence`

This is the one `*_silence` line in the window that is **not** about the
standard being silent. Every other setting here exists because ISO 32000-1
permits two readings; this one exists because the *page* may not carry what
the operator asked for, and there is no answer in any standard to what a
program should do then.

It says so outright rather than borrowing the shape of the others. A
sentence implying the standard is undecided about synthesised weights would
send an operator looking for a clause that does not exist.

### `fn style_policy_radius`

It changes **the bytes pdfcer writes**, and that is not obvious.

A faked weight is not a display trick: it is text rendering mode 2 plus a
stroke width written into the page's content stream, and a faked slant is a
shear term written into the text matrix. Both survive Save and both are what
every other viewer will show. An operator who read this as a preview setting
would hand on a drawing carrying artificial letterforms they thought were
only on their screen.

### `fn style_policy_auto_note`

It states what pdfcer does FIRST, because the thing operators get wrong
about this setting is assuming it decides whether a real face is used. It
does not — a real face is always preferred, under all three choices.

### `fn style_policy_refuse_note`

It says the button will appear not to work, in advance. This is the only
setting in this window that can make a control do nothing, and an operator
who chose it months earlier will otherwise read the silence as a defect.

### `fn style_policy_bound`

The fact that makes this setting narrower than it looks, and it is a
fact rather than a direction — so it is drawn under the group rather than
attached to one option, exactly as `actual_text_bound` is.

A real face is preferred under **every** choice here. pdfcer asks the engine
which real face is on offer before it asks for anything to be faked, and
takes the offer when there is one. None of the three options can turn that
off, and an operator who read *"Never fake it"* as *"never change my font"*
has misread it in the direction that matters.
