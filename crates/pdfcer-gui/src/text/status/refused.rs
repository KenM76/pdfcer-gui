//! # `text::status::refused` — **the one sentence for an edit the ENGINE
//! refused and could not be asked why**
//!
//! One string, in a file of its own, for [`super`]'s stated reason and
//! [`super::formdelete`]'s exact precedent: a catalog area is keyed by the
//! consumer it serves, this one's consumer is
//! [`crate::app::status::decline`]'s [`Declined::EditRefused`] arm alone, and
//! **R2** (no `.rs` file over 1,500 lines) is what forced the split while the
//! subject boundary is what decided where it fell. `super`'s `mod.rs` stands at
//! 1,479 lines, so a paragraph added there would be a paragraph added to a file
//! two dozen lines from the ceiling.
//!
//! ## ★★★ Why the sentence lives HERE and not with a surface
//!
//! [`crate::app::status::decline::Declined::line`] reaches out of
//! [`crate::text::status`] five times — to [`crate::text::tool`], to
//! `text::forms::groups`, to `text::panels::bookmarks` — and every one of those
//! reaches is the same rule applied: **a string lives with the surface that
//! owns its subject.** The Points tool's refusal belongs to the tool; a
//! bookmark drag's belongs to the bookmarks panel.
//!
//! This sentence has no such surface. Its subject is not text editing, not
//! rotation, not form fields: it is *an edit — any edit — that
//! `crate::app::actions::funnel` asked for and the engine declined*, arriving
//! from any of ~78 call sites, about whichever verb the operator happened to
//! invoke. The only surface that owns it is the status bar's `⊗` slot itself,
//! and the catalog area whose consumer is `crate::app::status` is this one.
//!
//! ⇒ So the reach-across precedent argues **against** reaching across here. It
//! stays in `text::status`, in a file of its own, exactly as `formdelete` did.
//!
//! [`Declined::EditRefused`]: crate::app::status::decline

/// **An edit reached the engine, the engine refused it, and this shell cannot
/// say why** — `OPERATOR_REQUESTS.md` **O116**, 2026-09-04.
///
/// # ★★★ What this sentence is for, and what it replaces
///
/// It replaces **silence**, which is this project's founding defect class:
/// *"I did the thing and nothing happened and nothing said why."* The state it
/// ends was reproduced on an ordinary CAD drawing with an ordinary embedded
/// font — Edit ▸ Edit text arms, a click places a caret, characters are typed,
/// Enter commits, `EditSession::edit_text` refuses, and the operator is told
/// nothing at all because `crate::app::actions::funnel`'s error arm wrote one
/// line to `PDFCER_DIAG` and stopped.
///
/// The engine's verdict in that case was *correct* — a symbolic font whose
/// code↔glyph relation lives inside an embedded program `pdfcer-core` does not
/// parse genuinely cannot be edited safely — which is the point. **A right
/// answer delivered as a silence is indistinguishable from a broken feature.**
///
/// # ★★★ Three deliberate properties, each of which will look like an omission
///
/// ## 1. It names NO cause, and cannot
///
/// Not because a cause would be unwelcome — because there is no honest way to
/// obtain one. `pdfcer_core::edit::EditError` (and its two text-editing
/// siblings) expose **no coarse discriminant a front end may switch on**, and
/// the two shortcuts available are both worse than saying less:
///
/// - **Match on the engine's variants** — a second copy of their taxonomy,
///   living in this crate, that **drifts**. The first time they split a
///   variant this either stops compiling (best case) or falls silently into a
///   wrong arm (likely case), and the symptom of the second is *the operator
///   being told the wrong reason for a refusal* — which is strictly worse than
///   the silence it replaced.
/// - **Grep the `Display` string** — prose, theirs to reword, and a front end
///   that greps a diagnostic for `"font"` is a front end that breaks on a
///   comma.
///
/// ⇒ `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`
/// `request_can_edit_errors_expose_a_coarse_kind_a_front_end_may_switch_on.md`
/// asks the engine for that discriminant — three or four stable buckets
/// (*unsupported font*, *structure frozen*, *not found*, *other*) — and is
/// unanswered. Its "What we will do meanwhile" section is this entry's whole
/// charter: **ship ONE un-categorised sentence rather than leave it silent, and
/// replace it when the discriminant lands.**
///
/// ★ So this function is written to be **deleted**. The day `EditError` gains a
/// `kind()`, this becomes four sentences that name the four buckets, and
/// `Declined::EditRefused` becomes four variants. Nothing else about the
/// mechanism changes; the wiring, the retirement rule and the slot are already
/// right. Whoever does that work should read this paragraph as the brief.
///
/// ## 2. It points NOWHERE, and that is the uncomfortable half
///
/// The request's own draft said *"see Render diagnostics for the reason"*, and
/// **that pointer would be a false promise**, verified rather than assumed:
///
/// - the Render-notes disclosure and the Render-diagnostics dialog both read
///   [`crate::app::status::notes::findings`], which is a census of
///   `pdfcer_render::Diagnostics` — the **rasteriser's** report on one page's
///   interpretation (unresolved `/Contents`, skipped fonts and images,
///   substituted glyphs, hidden `/OC` sections, unimplemented operators);
/// - an `EditError` is `pdfcer_core::edit`'s, produced by a **verb**, and
///   reaches no field of that struct. There is no path from one to the other;
/// - worse, on the very document that produced O116 the page rasterises
///   perfectly — the font is embedded and drawn correctly, it merely cannot be
///   *re-encoded* — so Render diagnostics would report the page **clean**. The
///   pointer would send the operator to a surface that says nothing is wrong.
///
/// ⇒ **A sentence that sends the operator somewhere the answer is not is worse
/// than one that admits it has no more to say.** The pointer is omitted. If the
/// day comes that a refusal genuinely does reach a second surface, add it then
/// and delete this paragraph.
///
/// ## 3. It carries none of the error's OWN words
///
/// `EditError`'s `Display` is diagnostic prose — *"R-INV-2: font
/// 'AAAAAA+JetBrainsMono-Regular' is symbolic with a built-in/custom cmap and
/// no usable /Encoding (§9.6.6.4 Branch B ignores /Encoding)…"* — and
/// `tools/gates/check-ui-strings.sh`'s exclusion 3 says in as many words that
/// being a `Display` impl *"is not permission to route UI text through an error
/// type"*. No `format!("{error}")` reaches a label. The trace line keeps that
/// text, **unchanged**, because the two audiences are different: whoever is
/// reading `PDFCER_DIAG` wants §9.6.6.4, and the operator wants to know their
/// drawing is intact.
///
/// # ★★ Why "and the document is unchanged" is half the sentence
///
/// Because the failure this ends is not *"I was not told why"* — the operator
/// can live with that — it is *"I do not know whether it took."* A refused edit
/// and a completed one look identical on a dense CAD sheet: the text is small,
/// the change is four characters, and the page under the caret is a thicket. An
/// operator who suspects the edit half-landed will press again, or undo
/// something that never happened, or save a copy to compare. The clause is
/// there to stop all three, and it is **true by construction**: the funnel's
/// error arm bumps no epoch, drops no texture, writes no undo entry and never
/// reaches `pages::resync`.
///
/// # ★ Why it does not apologise, name a remedy, or suggest a workaround
///
/// It cannot name a remedy without knowing the cause (property 1), and an
/// invented one — *"try a different font"* — would be advice about a document
/// this shell has not examined. `crate::text`'s own conventions forbid both the
/// apology and the guess: *"Name the thing that went wrong and what the operator
/// can do"*, and where nothing can be done, saying so plainly is the honest
/// remainder.
#[must_use]
pub const fn edit_declined_by_engine() -> &'static str {
    "That change was refused, and the document is unchanged."
}
