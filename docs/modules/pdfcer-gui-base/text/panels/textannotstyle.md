# `text::panels::textannotstyle` — the words for restyling a mark that
carries WORDS

Copy for `crate::panels::properties::markup::textannot`: the sticky note's
and the stamp's style rows, which reach `EditSession::set_text_annot_style`
rather than `set_markup_style`.

## Why a file of its own beside `text::panels::properties`

**R2**, and a subject seam that survives it. `properties.rs` was at 1,487
lines — thirteen short of the ceiling — when `Pass 253.2` landed, so the
four strings below had nowhere to go in it. The cut is the same one the
code took one directory over: `properties::markup` draws what one verb
reaches and `properties::markup::textannot` draws what the other does, and
the copy follows the code rather than the file it happened to start in.
`text::panels::annotgeometry` is the precedent for a sibling here.


That function's own doc keeps the superseded sentence verbatim. What
belongs here is why it went wrong and what replaced it, because both are
about the words in this file.

It said *"pdfcer does not redraw this kind of mark, so its colour, line
width and opacity cannot be changed here"*, and it was shown for the three
subtypes `set_markup_style` refuses: `/Text`, `/FreeText`, `/Stamp`. Every
word was true **of the only annotation-style verb that existed when it was
written**. `pdfcer-core` shipped a second one the same afternoon —
`set_text_annot_style`, a different reader, a different style struct — and
it restyles a sticky note's icon and colour and a stamp's colour.

⇒ From that moment the sentence made a **false claim about two of the three
subtypes it was shown for**, and false in the direction that matters: it
told the operator a capability did not exist on the very day it did, over a
row open since 2026-09-05 carrying his own *"check that these are fully
editable while you are at it."* That is the `set_button_action` shape
`check-verb-coverage.sh` exists because of — *"if your surface tells the
operator that pdfcer never authors an action, it is now saying something
untrue in the direction that matters."*

**The fix was not a reworded sentence. It was fewer marks reaching one.**

| subtype | before | now |
|---|---|---|
| `/Text` | the refusal sentence | the icon and colour rows |
| `/Stamp` | the refusal sentence | the colour row |
| `/FreeText` | the refusal sentence | [`markup_text_box_not_restylable`] — a narrower claim, and still true |
| anything else | the refusal sentence | the refusal sentence, reworded to stop implying a family |

Note which row is the interesting one. The `/FreeText` sentence is not
the old refusal kept for one subtype: the old one claimed the capability
was **missing**, and the new one says this shell **declines** to use a
capability that exists, for a measured reason. Reusing the string would
have kept a false premise alive under a true-looking conclusion.

## Item notes

### `fn markup_text_box_not_restylable`

* `annot_author::text_spec_from_dict` reports `multiline: false` for every
  `/FreeText` there is, because §12.5.6.6 gives the subtype no such key, and
  its own doc says a caller who needs the true value must **measure** it.
* `set_text_annot_style` re-bakes from that spec **without measuring**,
  unlike `set_markup_note`, which bakes both ways and compares bytes.
* `crate::canvas::textannot::spec` authors `multiline: true` on every text
  box this shell has ever placed.

⇒ Changing the colour would silently re-lay the operator's callout as one
unwrapped line and push their second sentence off the box. **Visible
control, silently destructive** — one worse than the *visible control,
silently inert* case this panel already fixed once.

The copy does not say *"pdfcer cannot"*, because it can and the claim
would be false the moment the engine measures the value. It says what would
happen, which stays true either way and is the thing the operator would
actually care about. Claim-bearing copy is verified, not softened.

It names the note as still editable for [`markup_not_restylable`]'s
reason and on the same check: `set_markup_note` refuses a ce dimension and a
widget and nothing else, and on a `/FreeText` it is the verb that **does**
measure `multiline`, so the words can be corrected without the box being
re-laid.

### `fn markup_text_annot_note`

It names the **two** things absent from those rows that are present a
few pixels away on every other mark — no way back to no colour, and no
opacity — because an operator who has just restyled a rectangle and then
selects a note will read their absence as a bug rather than as a limit. R9
makes them absent; this makes the absence legible.

*"the colour cannot be taken away again"* rather than *"there is no Clear
button"*: the operator is being told about the **file**, not about this
panel's furniture. `TextAnnotSpec`'s variants each carry a required colour,
so no-colour is not a state a sticky note can be in — which is a fact about
the format they would meet in any editor.

### `fn markup_icon_foreign`

*"Not one of these"* rather than a blank or the first entry. §12.5.6.4's
seven names are a standard set and not a closed one, so a producer's own
icon name is **conforming** — the file is not broken and the word must not
suggest it is. What the chooser cannot do is show it as selected, because it
has no entry for it.

### `fn markup_icon_foreign_named`

# Why the name itself and not a category word

This replaced *"Not one of these"* on 2026-09-07. That phrase was the
honest answer while the name could not be carried: the engine's reader
flattened an unmodelled `/Name` to `Note`, so the shell knew only that
*something* had been lost and could not say what. `Pass 253.5` carries the
bytes (`StickyIcon::Other`), so the panel can name it — and a chooser that
says **"Sparkle"** tells the operator something a chooser saying **"Not one
of these"** cannot: which of his notes it is, and that pdfcer is going to
keep it.

Quoted, because it is a value out of his file rather than a word this
program chose, and the quotes are what make a name like *Note 2* read as a
name rather than as an instruction.

`String::from_utf8_lossy` at the call site, not here: a `/Name` is a
sequence of bytes (§7.3.5) and there is no encoding declared for it, so a
producer may legally write one this program cannot decode. Replacement
characters in the chooser are the correct outcome — the operator sees that
something is there and unreadable, and the bytes still round-trip untouched.

### `fn markup_icon_foreign_note`

The superseded text, kept because the shape of the change is the useful
part:

> *"This note uses an icon pdfcer does not know. Changing its icon OR its
> colour here will replace that icon with one of the ones listed."*

That was exactly right against `pdfcer-core` v0.44.0.
`annot_author::text_spec_from_dict`'s `/Text` arm normalised an unmodelled
`/Name` to `Note` on the way past, and `set_text_annot_style` re-baked from
that normalised spec — so changing the **colour alone** on a note carrying
`/Sparkle` wrote `/Name /Note` into the file, silently.

This shell filed it
(`request_set_text_annot_style_rewrites_a_foreign_icon_name.md`) and
`Pass 253.5` fixed it: `StickyIcon::Other(Vec<u8>)` carries the bytes and
`from_name_lossless` reads them, so the round trip is exact and a colour
change touches nothing else.

⇒ **A limitation sentence on this project has a shelf life measured in
hours**, and this one had a shelf life of one day. Spell every claim about
the engine as a dated assertion and re-read it before repeating it.

# What survives, and why it is still worth a sentence

The half that was never about loss. The engine's own note: *"`sticky_note`
paints the same glyph for all seven variants — the icon chooses the `/Name`
written, not the picture drawn."* So an operator comparing this window with
the program that made the note is looking at two different symbols for one
annotation, and is entitled to know that pdfcer chose the picture and the
file chose the name.

Shown **before** he touches a control rather than as a hover. Nothing is
destroyed any more, so this is no longer a warning — but it is still an
explanation, and an explanation that arrives after the conclusion has been
drawn is not one.

### `fn stamp_text_size_label`

# Why *"Text size"* and not *"Font size"*

The operator's own words, twice: *"still can't adjust the size of a stamp
on the canvas, or by entering a different size in the properties box."* He
is describing the words on the stamp getting bigger, not choosing a
typeface's metrics — and pdfcer does not let him choose the face at all
here (a stamp's label is Helvetica Bold, always). A control called *Font
size* sitting where no font can be picked invites the next question, which
is where the font control is, and the answer is that there is not one.

### `fn stamp_text_size_suffix`

A suffix rather than a second word, because a point size is a number an
operator already reads with its unit attached, and the row lives in a
narrow column shared with every other properties section.

### `fn stamp_fit_label`

*"If it does not fit"* rather than *"Fit policy"*. The operator meets
this control at the moment he has typed a larger number, so the words that
help are the ones naming the situation he is about to be in — not the
engine's term for the family of answers.

### `fn stamp_fit_grow`

*"Make the stamp wider"* names what the operator will SEE. The engine's
own doc for this variant makes the same point from the other side — the
drawn box becomes *"a position and a minimum size rather than a cage"* —
and it is the default here for the reason that doc gives: a wider stamp is
visible as itself and therefore cannot be quietly wrong (R8b rule 4).

### `fn stamp_fit_shrink`

It says *"shrink"* in the option itself, because this is the variant
that changes the number the operator just typed. The disclosure after the
fact ([`stamp_label_shrunk`]) says by how much; this says that it can
happen at all, before he chooses it.

### `fn stamp_fit_clip`

# Why the option that hides characters is offered at all

Because it is the behaviour every build before `Pass 287.0` had, it is the
one that produced the operator's original complaint, and — in the engine's
own words — *"a behaviour that can only be obtained by accident is worse
than one that can be requested"*. Somebody reproducing an existing
document's appearance needs it.

The wording carries the consequence rather than the mechanism. *"Cut the
words off"* is what happens; *"clip to the bounding box"* is how. An
operator who picks this one has been told what he is picking.

### `fn stamp_fit_option`

# Why a dispatcher and not three call sites picking their own string

Because there are **two** surfaces that ask this question — the placing
dialog and the properties panel — and a policy labelled *"Make the stamp
wider"* in one and *"Grow the box"* in the other is two policies as far as
the operator is concerned. The shell would then own a private, diverging
vocabulary for somebody else's enum, which is the same failure mode as a
second reader of somebody else's format.

⚠ `StampFit` is `#[non_exhaustive]`, so this cannot be an exhaustive
`match` and the fallback matters. A fourth policy the engine adds and this
build does not know gets [`stamp_fit_unknown`] — a sentence that says the
build does not know it, rather than a plausible label invented from the
variant's name. `crate::canvas::stampfit::FITS` is what a control iterates,
so an unknown policy is never *offered*; this arm exists for the day one is
*read back* from somewhere.

### `fn stamp_fit_unknown`

It names the situation rather than guessing: an operator who sees this
is looking at a build older than the file or older than the engine it was
linked against, and *"this build does not know"* is the only true thing
that can be said about it.

### `fn stamp_size_da_unreadable`

# Why this is the one size source that owes a sentence

The three sources are not three degrees of confidence, and it is worth
being exact about which one is anomalous:

| source | what it means | owed |
|---|---|---|
| `DeclaredInDa` | the author stated it | nothing — it is their number |
| `RecoveredFromAppearance` | no `/DA` at all; read off the baked `Tf` | **nothing** |
| `DaUnreadable` | a `/DA` is there and yields no size | this sentence |

The middle row is the one a shell gets wrong. It is tempting to warn
that a recovered number is *"less certain"*, and it is not: the engine's
own doc says it is *"not an anomaly and owes no warning — every stamp
authored before `Pass 287.0` is in this state, and so is anything another
producer wrote. The number is exactly what is on the page."* A warning
there would fire on the majority of stamps in the world and teach the
operator to ignore the one that matters.

The last clause is the actionable half. The operator is about to
overwrite a `/DA` string pdfcer could not parse, and that is a thing he is
entitled to know **before** he presses, not after.

### `fn stamp_label_shrunk`

Off-canvas, in the status line: R8b rule 4's surviving half. The stamp
itself renders exactly as a saved-and-reopened copy will render — nothing
is tinted, badged or outlined — and the fact that a size was decided for
the operator reaches him in words instead.

Both numbers, because the size alone cannot answer the question the
disclosure exists to answer. *"12 pt"* is the same sentence whether he
asked for 12 and got it or asked for 24 and the box took half of it away;
the pair is what makes it an inference report rather than a readout.

### `fn stamp_label_clipped`

The most serious of the four outcomes, and the only one where the file no
longer shows something the operator wrote. It states the count, because
*"some of the words"* is a sentence somebody can look at a stamp and
disagree with.

*"the words are centred, so the loss is split between both ends"* is not
padding: the engine counts a character as hidden when its advance is not
**entirely** inside the box, and the label is drawn centred. An operator
counting the missing letters at the right-hand edge alone would otherwise
make the number look wrong.

### `fn stamp_label_box_grown`

Reported, but as the mildest of the three, and the engine says why:
growing *"is disclosed by the canvas itself — the operator drew a rectangle
and got a wider one, which is visible as itself and cannot be quietly
wrong."* So this sentence is a courtesy rather than an obligation, and it
exists because the operator who has just typed a number into a properties
field is **not** watching the canvas — he is watching the field.

### `fn stamp_label_fit_unknown`

# Why an unknown inference gets a sentence rather than silence

Because the three named outcomes are disclosed, and an operator who has
learnt that pdfcer says when it changed something will read silence as
*nothing changed*. `StampLabelFit` is `#[non_exhaustive]` precisely so the
engine can add a fourth, and `is_inference()` already answers `true` for
it — so the fact that a decision was made is known, and only its shape is
not.

⇒ Say the known half and name the unknown half as unknown. A sentence that
says *"pdfcer changed this and this build cannot say how"* sends the
operator to look at the stamp; a silence sends them nowhere. This is the
same posture `stamp_fit_unknown` takes one file along, and both are
tripwires: seeing either in the wild means the pin has moved past this
shell's vocabulary and the three sentences above owe a fourth.
