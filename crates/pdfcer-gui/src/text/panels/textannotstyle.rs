//! # `text::panels::textannotstyle` — the words for restyling a mark that
//! carries WORDS
//!
//! Copy for `crate::panels::properties::markup::textannot`: the sticky note's
//! and the stamp's style rows, which reach `EditSession::set_text_annot_style`
//! rather than `set_markup_style`.
//!
//! ## ★★ Why a file of its own beside `text::panels::properties`
//!
//! **R2**, and a subject seam that survives it. `properties.rs` was at 1,487
//! lines — thirteen short of the ceiling — when `Pass 253.2` landed, so the
//! four strings below had nowhere to go in it. The cut is the same one the
//! code took one directory over: `properties::markup` draws what one verb
//! reaches and `properties::markup::textannot` draws what the other does, and
//! the copy follows the code rather than the file it happened to start in.
//! `text::panels::annotgeometry` is the precedent for a sibling here.
//!
//! ★ [`crate::text::panels::properties::markup_not_restylable`] stays where it
//! is, and its own doc carries the correction it took on 2026-09-06 — that
//! sentence and the string it superseded are one subject and must not end up in
//! two files where an author could soften either alone.//!
//! ## ★★★ THE CORRECTION THIS PASS OWES —
//! ## [`crate::text::panels::properties::markup_not_restylable`] WAS PARTLY
//! ## FALSE FOR AN AFTERNOON
//!
//! That function's own doc keeps the superseded sentence verbatim. What
//! belongs here is why it went wrong and what replaced it, because both are
//! about the words in this file.
//!
//! It said *"pdfcer does not redraw this kind of mark, so its colour, line
//! width and opacity cannot be changed here"*, and it was shown for the three
//! subtypes `set_markup_style` refuses: `/Text`, `/FreeText`, `/Stamp`. Every
//! word was true **of the only annotation-style verb that existed when it was
//! written**. `pdfcer-core` shipped a second one the same afternoon —
//! `set_text_annot_style` (`edit.rs:27124`), a different reader, a different
//! style struct — and it restyles a sticky note's icon and colour and a
//! stamp's colour.
//!
//! ⇒ From that moment the sentence made a **false claim about two of the three
//! subtypes it was shown for**, and false in the direction that matters: it
//! told the operator a capability did not exist on the very day it did, over a
//! row open since 2026-09-05 carrying his own *"check that these are fully
//! editable while you are at it."* That is the `set_button_action` shape
//! `check-verb-coverage.sh` exists because of — *"if your surface tells the
//! operator that pdfcer never authors an action, it is now saying something
//! untrue in the direction that matters."*
//!
//! ★ **The fix was not a reworded sentence. It was fewer marks reaching one.**
//!
//! | subtype | before | now |
//! |---|---|---|
//! | `/Text` | the refusal sentence | the icon and colour rows |
//! | `/Stamp` | the refusal sentence | the colour row |
//! | `/FreeText` | the refusal sentence | [`markup_text_box_not_restylable`] — a narrower claim, and still true |
//! | anything else | the refusal sentence | the refusal sentence, reworded to stop implying a family |
//!
//! ★★ Note which row is the interesting one. The `/FreeText` sentence is not
//! the old refusal kept for one subtype: the old one claimed the capability
//! was **missing**, and the new one says this shell **declines** to use a
//! capability that exists, for a measured reason. Reusing the string would
//! have kept a false premise alive under a true-looking conclusion.

/// ★★★ **Why a text box's appearance is not changed here — and it is NOT
/// because no verb exists.**
///
/// This is the sentence that replaced [`markup_not_restylable`] for a
/// `/FreeText` on 2026-09-06, and the distinction it draws is the whole reason
/// it is a second string rather than a reuse. `set_text_annot_style` restyles a
/// text box perfectly well. **This shell declines to call it**, and the
/// argument is measured rather than cautious:
///
/// * `annot_author::text_spec_from_dict` reports `multiline: false` for every
///   `/FreeText` there is, because §12.5.6.6 gives the subtype no such key, and
///   its own doc says a caller who needs the true value must **measure** it.
/// * `set_text_annot_style` re-bakes from that spec **without measuring**,
///   unlike `set_markup_note`, which bakes both ways and compares bytes.
/// * `crate::canvas::textannot::spec` authors `multiline: true` on every text
///   box this shell has ever placed.
///
/// ⇒ Changing the colour would silently re-lay the operator's callout as one
/// unwrapped line and push their second sentence off the box. **Visible
/// control, silently destructive** — one worse than the *visible control,
/// silently inert* case this panel already fixed once.
///
/// ★ The copy does not say *"pdfcer cannot"*, because it can and the claim
/// would be false the moment the engine measures the value. It says what would
/// happen, which stays true either way and is the thing the operator would
/// actually care about. Claim-bearing copy is verified, not softened.
///
/// ★ It names the note as still editable for [`markup_not_restylable`]'s
/// reason and on the same check: `set_markup_note` refuses a ce dimension and a
/// widget and nothing else, and on a `/FreeText` it is the verb that **does**
/// measure `multiline`, so the words can be corrected without the box being
/// re-laid.
#[must_use]
pub const fn markup_text_box_not_restylable() -> &'static str {
    "pdfcer does not change a text box's colour here: redrawing it would lay the words out as \
     one line and push anything past the first sentence outside the box. You can still move it, \
     resize it, delete it, and edit the words it carries."
}

/// The sentence under the sticky-note and stamp style rows.
///
/// ★★ It names the **two** things absent from those rows that are present a
/// few pixels away on every other mark — no way back to no colour, and no
/// opacity — because an operator who has just restyled a rectangle and then
/// selects a note will read their absence as a bug rather than as a limit. R9
/// makes them absent; this makes the absence legible.
///
/// ★ *"the colour cannot be taken away again"* rather than *"there is no Clear
/// button"*: the operator is being told about the **file**, not about this
/// panel's furniture. `TextAnnotSpec`'s variants each carry a required colour,
/// so no-colour is not a state a sticky note can be in — which is a fact about
/// the format they would meet in any editor.
#[must_use]
pub const fn markup_text_annot_note() -> &'static str {
    "A note or stamp always has a colour, so it can be changed but not taken away again. \
     Transparency is not offered for these two."
}

/// What the icon chooser shows for a note whose `/Name` pdfcer does not model.
///
/// ★ *"Not one of these"* rather than a blank or the first entry. §12.5.6.4's
/// seven names are a standard set and not a closed one, so a producer's own
/// icon name is **conforming** — the file is not broken and the word must not
/// suggest it is. What the chooser cannot do is show it as selected, because it
/// has no entry for it.
#[must_use]
pub const fn markup_icon_foreign() -> &'static str {
    "Not one of these"
}

/// **The file's own icon name, shown as the entry it is** — for a `/Name`
/// §12.5.6.4 permits and pdfcer does not model.
///
/// # ★★★ Why the name itself and not a category word
///
/// This replaced *"Not one of these"* on 2026-09-07. That phrase was the
/// honest answer while the name could not be carried: the engine's reader
/// flattened an unmodelled `/Name` to `Note`, so the shell knew only that
/// *something* had been lost and could not say what. `Pass 253.5` carries the
/// bytes (`StickyIcon::Other`), so the panel can name it — and a chooser that
/// says **"Sparkle"** tells the operator something a chooser saying **"Not one
/// of these"** cannot: which of his notes it is, and that pdfcer is going to
/// keep it.
///
/// ★ Quoted, because it is a value out of his file rather than a word this
/// program chose, and the quotes are what make a name like *Note 2* read as a
/// name rather than as an instruction.
///
/// ★★ `String::from_utf8_lossy` at the call site, not here: a `/Name` is a
/// sequence of bytes (§7.3.5) and there is no encoding declared for it, so a
/// producer may legally write one this program cannot decode. Replacement
/// characters in the chooser are the correct outcome — the operator sees that
/// something is there and unreadable, and the bytes still round-trip untouched.
#[must_use]
pub fn markup_icon_foreign_named(name: &str) -> String {
    format!("\"{name}\"")
}

/// ★★★ **The note under a foreign icon** — and it says something different
/// from what it said yesterday.
///
/// # It used to warn about destruction, and that warning is now FALSE
///
/// The superseded text, kept because the shape of the change is the useful
/// part:
///
/// > *"This note uses an icon pdfcer does not know. Changing its icon OR its
/// > colour here will replace that icon with one of the ones listed."*
///
/// That was exactly right against `pdfcer-core` v0.44.0.
/// `annot_author::text_spec_from_dict`'s `/Text` arm normalised an unmodelled
/// `/Name` to `Note` on the way past, and `set_text_annot_style` re-baked from
/// that normalised spec — so changing the **colour alone** on a note carrying
/// `/Sparkle` wrote `/Name /Note` into the file, silently.
///
/// This shell filed it
/// (`request_set_text_annot_style_rewrites_a_foreign_icon_name.md`) and
/// `Pass 253.5` fixed it: `StickyIcon::Other(Vec<u8>)` carries the bytes and
/// `from_name_lossless` reads them, so the round trip is exact and a colour
/// change touches nothing else.
///
/// ⇒ **A limitation sentence on this project has a shelf life measured in
/// hours**, and this one had a shelf life of one day. Spell every claim about
/// the engine as a dated assertion and re-read it before repeating it.
///
/// # What survives, and why it is still worth a sentence
///
/// The half that was never about loss. The engine's own note: *"`sticky_note`
/// paints the same glyph for all seven variants — the icon chooses the `/Name`
/// written, not the picture drawn."* So an operator comparing this window with
/// the program that made the note is looking at two different symbols for one
/// annotation, and is entitled to know that pdfcer chose the picture and the
/// file chose the name.
///
/// ★ Shown **before** he touches a control rather than as a hover. Nothing is
/// destroyed any more, so this is no longer a warning — but it is still an
/// explanation, and an explanation that arrives after the conclusion has been
/// drawn is not one.
#[must_use]
pub const fn markup_icon_foreign_note() -> &'static str {
    "This note uses an icon pdfcer does not draw. The name is kept exactly as it is in the file \
     — including when you change the colour — but pdfcer draws its own sticky-note symbol for \
     it, so it will not look the way it does in the program that made it."
}
