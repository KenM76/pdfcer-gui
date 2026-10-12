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

## What each mark is offered

| subtype | rows |
|---|---|
| `/Text` | icon, colour, opacity |
| `/Stamp` | colour, text size and fit, words, opacity |
| `/FreeText` | border colour, width and line style, background, text colour, font, size, opacity |
| `/Caret`, `/FileAttachment`, `/Sound`, `/Screen` | icon colour, opacity |
| anything else `set_markup_style` cannot reach | opacity, and [`markup_not_restylable`] |

## Item notes

### `fn markup_text_annot_note`

Says that a note or stamp's colour cannot be removed: each `TextAnnotSpec`
variant carries a required colour, so no-colour is not a state the mark can
be in. Told as a fact about the file, not about this panel's buttons.

### `fn text_box_text_colour_cmyk`

A `/DA` colour in CMYK has no faithful sRGB swatch; the sentence replaces
the swatch's silent narrowing with a statement of what a pick will do.

### `fn redraw_offer_text_box_foreign`, `fn redraw_offer_text_box_rich`, `fn redraw_offer_marker_foreign`

The three refusals `pdfcer_gui_base::redrawoffer::Why` answers with a
retry: `EditError::FreeTextAppearanceForeign`, `FreeTextIsRichText` and
`MarkerAppearanceForeign`. Each says the mark was left unchanged, why, and
what the redraw will cost, because the redraw is lossy and the operator
must choose it.

### `fn appearance_replaced`, `fn rich_text_dropped`

The disclosures after a restyle that replaced a look pdfcer did not draw
(`appearance_was_foreign`) or dropped `/RC` formatting
(`rich_text_dropped`, which names the keys removed). Off-canvas, per the
"fuzzy, never sneaky" rule.

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
