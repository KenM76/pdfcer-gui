//! # `text::status::selection` — what is selected, said in words
//!
//! Every string [`crate::app::status::selected`] draws, and nothing else. One
//! subject, one consumer — the organising principle
//! [`crate::text`]'s header states for the whole catalog, applied inside an
//! area that had grown large enough to need it.
//!
//! ## Why this became its own file
//!
//! Because R2 said so, and R2 was right. `text::status` crossed 1,500 lines
//! when the form-containment clause landed, and the rule this project was
//! founded on is that the limit is the signal to find the seam rather than to
//! raise the limit — the GUI being replaced reached 25,005 lines in one
//! `main.rs`, and *"nothing could be reasoned about locally"* is the direct
//! cause of most of what is wrong with it.
//!
//! The seam was already there. Every function here is read by exactly one
//! widget, they are the only strings in the area that describe a *thing the
//! operator picked* rather than a control they can press, and they now include
//! the two sentences the form-XObject work turned on. The paths do not move:
//! `status`'s `mod.rs` re-exports this, so `t::selection_one` still resolves
//! and no call site changed. A catalog area is keyed by its consumer, and the
//! consumer did not change — only the file did.
//!
//! ## The four sentences, and the state each is for
//!
//! | state | line |
//! |---|---|
//! | one page object | `Selected: Path · 120.0 × 40.0 pt` |
//! | one object inside a form XObject | `… · inside a form` |
//! | several things | `3 objects selected` |
//! | anything, with more underneath | `… · 1 of 5 here` |
//!
//! ★ **The containment clause is [rule 4](R8b) disclosure and it is
//! off-canvas.** A form-interior object is drawn on the page exactly as it
//! will be drawn when saved — no badge, no tint, no dashed outline. What
//! pdfcer had to do to find it is reported here, in words, on a bar; never by
//! marking the drawing.

/// ★★★ **This page's colours are approximate at this zoom.**
///
/// ★★★ **What is selected**, for the status bar's left-hand readout.
///
/// # Why this line exists at all
///
/// The operator, 2026-08-26: *"when I click on one of the objects all I get is
/// the page selected."* He was right, and nothing on screen said so — the
/// selection outline round a page-sized object looks exactly like *"the page
/// is selected"*, which is a state this program does not have. This is the
/// sentence that turns that into a diagnosis.
///
/// # The wording
///
/// **The kind first**, because it is the word that answers his question:
/// *Form* is the one that explains a page-sized outline, and it is the word he
/// would have needed to ask the right next question.
///
/// **Size in points**, to one decimal, because the operator works in a CAD
/// world where a number is how you tell two similar things apart, and because
/// a page-sized object is obvious the moment its size is beside the page's.
/// Not millimetres: the rest of this bar and the geometry fields are in points,
/// and one surface in two units is worse than either unit.
#[must_use]
pub fn selection_one(kind: &str, width: f32, height: f32) -> String {
    format!("Selected: {kind} · {width:.1} × {height:.1} pt")
}

/// The same, for an object whose bounds the projection declined.
///
/// Rare — it needs a page transform that will not invert — and it says less
/// rather than saying something invented. A size derived from a failed
/// projection would be a number the operator could act on and could not trust.
#[must_use]
pub fn selection_one_unsized(kind: &str) -> String {
    format!("Selected: {kind}")
}

/// ★★★ **The same object, when it lives inside a form XObject.**
///
/// # The sentence this whole change exists to make sayable
///
/// The operator, 2026-08-26: *"when I click on one of the objects all I get is
/// the page selected."* He was clicking a real object; a page-sized form
/// XObject wrapped it, the form's `/BBox` won every hit test, and **nothing on
/// screen said the word "form" anywhere**. The selection outline round the page
/// edge looked exactly like a state this program does not have.
///
/// The engine now descends into forms, so the click lands on the object he
/// meant. This clause is what stops the *next* question — *"why can I select
/// it but not move it?"* — from being as unanswerable as the first one was.
///
/// # Why the suffix, and not a different sentence
///
/// Because it is the same selection, described more completely. Kind and size
/// are unchanged and still lead, because they are what the operator asked for
/// by clicking; the containment is the qualifier. A separate line would read as
/// a separate subject, which is `status::selected`'s standing rule about the
/// depth clause too.
///
/// # ★ Rule 4: this is DISCLOSURE, and it is off-canvas
///
/// Nothing is drawn differently on the page. A form-interior object renders
/// exactly as it will render when saved, with no badge, tint or dashed
/// outline — the operator's own finding that *"the nagging and red flagging in
/// the original GUI made for a lot of extra bugs in the visibility when
/// editing"*. The fact that pdfcer reached inside a form to find this object is
/// reported here, in the status bar, and nowhere on the drawing.
///
/// # The count
///
/// `nesting` is [`pdfcer_core::vector::FormLeaf::containment`]'s length — how
/// many forms enclose the object, outermost first. One is overwhelmingly the
/// common case and gets the article rather than the digit, because *"inside 1
/// form"* reads like a computer counting. Deeper nesting is worth the number:
/// it is the difference between "this is in the title block" and "this is
/// three wrappers down", which changes what the operator does next.
#[must_use]
pub fn selection_one_in_form(kind: &str, width: f32, height: f32, nesting: usize) -> String {
    format!(
        "Selected: {kind} · {width:.1} × {height:.1} pt · {}",
        inside_forms(nesting)
    )
}

/// The same, for a form-interior object whose bounds the projection declined.
#[must_use]
pub fn selection_one_in_form_unsized(kind: &str, nesting: usize) -> String {
    format!("Selected: {kind} · {}", inside_forms(nesting))
}

/// ★★ **You are working inside a container and nothing is selected.**
///
/// `OPERATOR_REQUESTS.md` O70. The one state in the Smart-Selector arm with no
/// visible evidence anywhere else: no outline, no armed tool, nothing on the
/// page — just clicks that resolve differently from how they resolved a moment
/// ago.
///
/// ★ It names **Escape** for the reason `text::placing::armed_instruction`
/// does: this is the only statement of the way out that the operator can read
/// at the moment they need it, and a scope with no visible exit is exactly the
/// stranding the design exists to prevent.
#[must_use]
pub fn inside_container() -> &'static str {
    "Working inside a form — clicks select what is in it. Escape steps back out."
}

/// *"inside a form"* / *"inside 3 nested forms"* — the containment clause,
/// in one place so the sized and unsized sentences cannot word it differently.
fn inside_forms(nesting: usize) -> String {
    match nesting {
        0 | 1 => "inside a form".to_owned(),
        n => format!("inside {n} nested forms"),
    }
}

/// ★★ **Why a verb refused: the thing selected lives in a form XObject.**
///
/// # The two states this keeps apart
///
/// *"Nothing selected"* and *"the thing you selected cannot be moved by this
/// verb"* are the operator's mistake and the program's limit respectively, and
/// an interface that reports the second as the first sends them looking for
/// something they did not do wrong. `RESUME.md` records four occasions on this
/// project where a limit reported as an absence cost weeks.
///
/// # Every clause, and what it is answering
///
/// **"inside a form"** names the structure, because that is the fact the
/// operator can then act on — it explains the page-sized outline they used to
/// get, it explains why the Objects panel does not list this object, and it is
/// the word they need if they go looking in another tool.
///
/// **"pdfcer cannot edit inside one yet"** puts the limit on pdfcer rather than
/// on the document. The file is not malformed and there is nothing to fix in
/// it; a sentence that sounded like a complaint about the PDF would be a lie
/// about whose problem this is.
///
/// **"yet"** is load-bearing and is not optimism. `EditSession` writes a
/// paint-order edit to the page's content stream, and a form-interior object
/// lives in the form's — `FormLeaf::is_editable` is `false` for every leaf the
/// engine produces today. That is a boundary this shell reports, not a policy
/// it chose, and it is dated: `pdfcer-core` v0.14.0, 2026-08-27.
#[must_use]
pub const fn selection_inside_form_declined() -> &'static str {
    "That object is inside a form — pdfcer cannot edit inside one yet"
}

/// Several objects selected.
///
/// No kinds and no size: a mixed selection has neither, and picking the first
/// object's kind to stand for all of them would be a claim about the set that
/// is false the moment the set is mixed. The count is the honest whole of what
/// can be said until a multi-selection summary is built.
#[must_use]
pub fn selection_many(count: usize) -> String {
    match count {
        1 => "Selected: 1 object".to_owned(),
        n => format!("Selected: {n} objects"),
    }
}

/// ★★ **…and how many other things were under the same click.**
///
/// Appended to whichever line above applies, because it is a fact about the
/// same selection: *"this one, and there were others."*
///
/// This is the half that makes `Alt`+click discoverable. A cycling gesture
/// nobody knows about is a gesture nobody uses, and the operator has no way to
/// learn that four more objects were under his pointer unless something says
/// so. *"1 of 5 here"* says both that this is not the only answer and that
/// there is a question worth asking.
///
/// `here` rather than `under the pointer`: the bar has finite width and the
/// word is doing one job — locating the count at the click rather than in the
/// document.
#[must_use]
pub fn selection_with_depth(line: &str, taken: usize, of: usize) -> String {
    format!("{line} · {taken} of {of} here")
}

// ===========================================================================
// ★ Restyling existing text — `EditSession::format_text`, O37
// ===========================================================================

/// Why a restyle of existing text did not happen.
///
/// # ★★ Why this is an enum here rather than a `String` from the engine
///
/// `FormatError` writes excellent prose about itself — the synthetic-italic
/// refusal explains the `Td` interaction, names §9.4.2 Table 108 and ends
/// *"Nothing was applied"* — and it is tempting to put it on the status bar
/// verbatim.
///
/// `check-ui-strings.sh` exclusion 3 says in as many words that an error type's
/// prose is **not** permission to route UI text through it, and the reason is
/// not tidiness. The engine's sentence is written for whoever is debugging: it
/// names the rule, the clause and the mechanism. An operator restyling a title
/// block needs the *remedy* first and does not need `Tm` at all. Two audiences,
/// two sentences; the engine's goes to the trace, where its audience is.
///
/// So this enum is the shell's own reading of which refusals an operator can
/// **act on**, and there are three. Everything else is either impossible from
/// this surface (a bad page index, an empty request) or is not improved by
/// being subdivided.
///
/// # ★★ It stopped being `Copy` on 2026-09-11, and the reason is a feature
///
/// One variant now owns a list of face names the engine computed, so the enum
/// holds a `Vec<String>` and cannot be `Copy`. Three doc comments in
/// [`crate::app::status::decline`] used to argue that this type is `Copy`
/// *"so that `Declined` stays `Copy` and `Declined::line` stays
/// `&'static str`"*, and both halves of that sentence have now been overtaken:
/// `Declined::line` became a [`std::borrow::Cow`] on 2026-09-10 for O141's
/// *"pdfcer cannot type a `q`"*, and the `Copy` half went here.
///
/// ★ What the argument was actually protecting is intact and is worth naming
/// so it is not lost with the derive: **the engine's prose must not reach the
/// status bar.** That is still true. What travels here is a list of
/// `/BaseFont` names — data pdfcer computed, not a sentence pdfcer wrote —
/// and the connective words around it are this catalog's own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextStyleRefusal {
    /// Nothing resolved to a run to restyle.
    NoRun,
    /// The run could not be pinned — the extraction carried no provenance for
    /// it, so pdfcer cannot be sure which show operator it would be editing.
    Unpinnable,
    /// The chosen face is not a font resource on this page (`FF-C`).
    FaceNotOnPage,
    /// A synthetic slant would move the line that follows this run.
    ItalicWouldMove,
    /// The chosen face cannot show every character in the run — and the faces
    /// that **could**, which is the payload.
    ///
    /// # ★★★ Why this variant carries data when none of its neighbours do
    ///
    /// Because it is the one refusal in this enum an operator can act on
    /// *immediately*, and until 2026-09-11 the shell knew the remedy and said
    /// nothing.
    ///
    /// `pdfcer-core`'s `Pass 274.0` made this refusal end on a **working**
    /// remedy instead of on *"choose a font that covers it"*, and `Pass 279.0`
    /// made that remedy **page-aware** — because the naive list was measured
    /// wrong in its most prominent position on `subset_missing.pdf`, where it
    /// named `Helvetica` and `Helvetica` resolved straight back into the
    /// `ABCDEF+Helvetica` that had just refused. Both improvements landed
    /// inside `Refusal::message`, a prose field.
    ///
    /// ★★ So this shell could not reach them. Splitting that message on
    /// `"these standard-14 faces have it: "` would have put a locator for
    /// another crate's sentence format inside a GUI, to break silently the
    /// first time the clause was reworded — and the public helper that looks
    /// like the answer, `std14_faces_covering`, is the **naive** one the engine
    /// measured as wrong, while the corrected `std14_faces_reachable` is
    /// `pub(crate)`. The two differ by one word. Reaching for the public one
    /// would have compiled, passed every gate, and sent the operator in a
    /// circle.
    ///
    /// ★ Filed rather than worked around (`Pass 296.1`, requested and shipped
    /// the same afternoon). `Refusal::remedy_faces` is now the same list the
    /// message's tail names, structured — one computation rendered twice — so
    /// what the operator reads here is exactly what pdfcer said, never more.
    ///
    /// **Empty is a real value and is not a failure.** It means nothing among
    /// the faces pdfcer can offer covers the character, which is also when the
    /// message's tail is absent; [`TextStyleRefusal::line`] falls back to the
    /// sentence that was there before this field existed.
    FaceLacksCharacters(Vec<String>),
    /// The operator's `style_policy` is `Refuse` and the only way to satisfy
    /// this request was to fake the weight or the slant.
    ///
    /// # ★★★ Raised by the ENGINE since 2026-09-11 — and it used to be ours
    ///
    /// This doc comment said the opposite until the automatic style ladder was
    /// wired, and the correction is worth keeping because the reasoning was
    /// right and the mechanism was temporary.
    ///
    /// The old account: the engine's `set_synthetic` gate refuses a synthesis
    /// only when a **real face was available** and would have been passed over.
    /// That is the right contract for a crate whose caller might genuinely mean
    /// "fake it". It is not what an operator who ticked *"never fake it"* asked
    /// for — on a page carrying no bold face at all, that gate has nothing to
    /// refuse in favour of, and it thickens the strokes. So this shell raised
    /// the variant itself, from a `preview_style_resolution` probe pinned to
    /// `StylePolicy::Refuse`, to get the wider reading.
    ///
    /// ★★ `FormatRequest::set_style` refuses on the wide reading natively.
    /// Its posture gate fires when the ladder reaches its **fourth** rung —
    /// after a real face on the page and after the standard-14 sibling have
    /// both been tried and neither was available — and returns
    /// [`FormatError::SynthesisRefusedByPosture`], naming the style, the run's
    /// font and the setting that caused the refusal. That is this variant's
    /// meaning exactly, decided by the side that knows what it tried.
    ///
    /// ⇒ The probe is deleted and `refusal_of` in
    /// `crate::app::actions::textstyle` maps the engine's error here. The
    /// variant is reached **less often** than before and that is the
    /// improvement: it now fires only when pdfcer genuinely had no real face to
    /// offer, where the shell's own probe also fired on pages where a
    /// standard-14 sibling was one resource away.
    ///
    /// [`FormatError::SynthesisRefusedByPosture`]: pdfcer_core::text_edit::FormatError::SynthesisRefusedByPosture
    FakingDeclined,
    /// Anything else the engine refused.
    Other,
    /// Some of the selection was restyled before something stopped the rest.
    PartOnly,
}

impl TextStyleRefusal {
    /// The sentence.
    ///
    /// Remedy first in every arm that has one, because the operator is looking
    /// at text that did not change and the useful half is *what to do now*.
    #[must_use]
    pub fn line(&self) -> std::borrow::Cow<'static, str> {
        // ★ Bound through a `&'static str` so only the arm that interpolates
        // carries machinery — the same shape, and for the same reason, as
        // `crate::app::status::decline::Declined::line`, which this feeds. The
        // bar redraws every frame; it allocates only on the frames reporting
        // the one refusal that names faces.
        let fixed: &'static str = match self {
            // Not "nothing is selected" — the operator may well have something
            // selected. What they do not have is TEXT selected, and naming the
            // wrong absence sends them to fix the wrong thing.
            Self::NoRun => {
                "Select some text on the page first — sweep across it with the Select tool, then change how it looks."
            }
            // ★ The honest half of this is "pdfcer cannot be sure", and the
            // sentence says so rather than blaming the file. A run that cannot
            // be pinned is one where an edit might land on a different piece of
            // text that reads the same, and doing it anyway is the one outcome
            // worse than declining.
            Self::Unpinnable => {
                "pdfcer could not tell exactly which piece of text that is, so it changed nothing rather than risk restyling a different one that reads the same."
            }
            // ★★★ Remedy first, and it names the list the operator is looking
            // at — and it is the SECOND wording, corrected on 2026-08-29.
            //
            // It read: *"pdfcer can only switch text to a font this page already
            // carries. Pick one of the faces in the list."* That was true and
            // `Pass 162.0` made it false: pdfcer now authors a standard-14 `/Font`
            // resource on demand, so a face this page does not carry is a change
            // that WORKS for fourteen of them.
            //
            // ⇒ A refusal sentence that states a limit the build no longer has
            // is worse than no sentence: it teaches the operator not to try
            // something the program can do, and it does so with the program's
            // own voice. This is the ★★ obligation in the engine's release
            // note — *"a face outside those fourteen still refuses by name"* —
            // discharged as a sentence rather than a silence, and it now says
            // WHICH boundary was crossed and why that boundary exists.
            //
            // ★ It names embedding as the reason rather than a deferral code.
            // `FF-C` means nothing to an operator; *"the font itself would have
            // to be copied into the file"* is the same fact in terms they can
            // weigh — and it is the honest account of why fourteen faces work
            // and a fifteenth does not.
            Self::FaceNotOnPage => {
                "pdfcer can switch text to a font this page already carries, or to one of the fourteen standard faces it can add itself. Any other face would have to be copied into the file, which pdfcer cannot do yet. Pick one of the faces in the list."
            }
            // ★ The refusal an operator would otherwise read as a bug. It says
            // what WOULD have happened, because "it moved my next line" is the
            // outcome they would have blamed pdfcer for.
            Self::ItalicWouldMove => {
                "Slanting this text would shift the line that follows it, because the two share a position in the file. pdfcer changed nothing rather than move text you did not select."
            }
            // ★★★ The only arm that returns early, because it is the only one
            // with a subject the operator can see. See the variant's own docs
            // for why the list could not be had until `Pass 296.1`.
            Self::FaceLacksCharacters(remedy) => return coverage_line(remedy),
            // ★ Remedy first, and the remedy is a SETTING, so the sentence
            // names where it lives. A refusal caused by the operator's own
            // choice that does not say which choice reads as a program defect.
            Self::FakingDeclined => {
                "No real bold or italic face on this page can show this text, and your settings tell pdfcer not to fake one. Nothing changed. Under Settings, the Fonts group has a \"Faking bold and italic\" choice that lets pdfcer thicken or slant the letters instead."
            }
            Self::Other => {
                "pdfcer could not make that change to this text and changed nothing. Text that was converted to outlines has no font to change; a face has to cover every character in the run."
            }
            // ★ The count is deliberately NOT in this sentence. The variant is
            // `Copy` and the catalog is `&'static str`, and adding an argument
            // to reach one number would make every sentence in this file a
            // `String`. What the operator needs is the fact that it is partial,
            // and that Ctrl+Z takes back what did happen.
            Self::PartOnly => {
                "Part of the selection was restyled before that happened. Ctrl+Z takes back what did change."
            }
        };
        std::borrow::Cow::Borrowed(fixed)
    }
}

/// The coverage refusal's sentence, with or without the engine's remedy list.
///
/// # ★★ Remedy first, which reverses the sentence when there is one
///
/// This module's rule is *remedy first in every arm that has one*, because the
/// operator is looking at text that did not change and the useful half is what
/// to do now. Without a list there is no remedy to lead with and the sentence
/// opens on the diagnosis; with one it opens on the faces. That is two
/// sentences rather than one with a clause bolted on, and it is deliberate: a
/// sentence that opens *"That face has no shape…"* and ends *"… Times-Roman
/// can"* buries the actionable half behind the explanation.
///
/// ★ `"The face you picked"` rather than naming it. The name is in the face
/// chooser the operator is looking at, and repeating it costs width on a bar
/// that is already carrying up to fourteen face names in the first clause.
///
/// # ★ Why `const WITHOUT` and not a second catalog function
///
/// Because it is the *same* refusal. Two catalog entries would be two
/// sentences that must be kept consistent with each other by hand, and this
/// project has a gate (`check-ui-strings`) that would be content with both.
fn coverage_line(remedy: &[String]) -> std::borrow::Cow<'static, str> {
    const WITHOUT: &str = "That face has no shape for one or more characters in this text. pdfcer changed nothing rather than substitute a different letter or leave a blank.";
    if remedy.is_empty() {
        return std::borrow::Cow::Borrowed(WITHOUT);
    }
    std::borrow::Cow::Owned(format!(
        "{} can show this text. The face you picked has no shape for one or more characters in it, so pdfcer changed nothing rather than substitute a different letter or leave a blank.",
        join_or(remedy)
    ))
}

/// `["a", "b", "c"]` → `"a, b or c"`.
///
/// ★ `or`, not `and`: the faces are **alternatives**, and `join_and` in
/// `crate::text::page_size` — whose subject is edges a drawing runs past, all
/// of which are true at once — would read as though the operator needed all
/// three. Copied rather than shared for exactly that reason: the two differ in
/// the one word that carries the meaning, so a shared helper would need a
/// parameter that is really a choice about a sentence.
fn join_or(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} or {last}", rest.join(", ")),
    }
}

/// `bold` / `italic` / `bold italic` — the axes, in the operator's words, and
/// **the engine's** words.
///
/// # ★★ Why the shell has to choose this word at all now
///
/// Until 2026-09-11 every sentence in this group took a `style: &str` that came
/// straight off `FormatError::RealFaceAvailable`, and the shell never had to
/// own it. The automatic ladder ([`FormatRequest::set_style`]) reports a
/// [`StyleSynthesis`] instead, so the word is chosen on this side — and the
/// catalog is the only place in this crate where an operator-facing word may be
/// written at all (`check-ui-strings.sh`).
///
/// # ★★★ It DELEGATES, and the first draft of it did not
///
/// [`StyleSynthesis::axes`] is a public `const fn` on the engine's own type,
/// written for precisely this: its doc says *"for sentences about a style that
/// a REAL face may have supplied (`Pass 179.0`)"*, which is this whole group.
///
/// ★★ This function shipped for three hours with a hand-written
/// `match (bold, italic)` and a doc comment explaining that the engine's
/// version *"is private"* and so could not be called. It is not private, it was
/// never called `axes_label`, and **nothing was measured before that was
/// written** — the claim was inferred from the shape of the problem, in a file
/// whose own header says *"a limitation sentence is a citation with an
/// hours-long shelf life."* The cost of believing it would have been a second
/// copy of the engine's word list, drifting the first time a third axis
/// appeared, with a comment beside it explaining why the copy was correct.
///
/// ★ The one case that is NOT delegated: `StyleSynthesis::None` renders as
/// `"nothing"`, which is right for the engine's *"synthesised nothing"* and
/// wrong for *"this text is already nothing"*. It is unreachable from either
/// control — Bold and Italic each send exactly one axis — and it is answered
/// with a word that reads correctly rather than with `unreachable!`, because a
/// catalog function that can panic is a catalog function that can take the
/// program down over a word.
///
/// [`FormatRequest::set_style`]: pdfcer_core::text_edit::FormatRequest::set_style
/// [`StyleSynthesis`]: pdfcer_core::text_edit::StyleSynthesis
/// [`StyleSynthesis::axes`]: pdfcer_core::text_edit::StyleSynthesis::axes
fn axes(bold: bool, italic: bool) -> &'static str {
    let style = pdfcer_core::text_edit::StyleSynthesis::new(bold, italic);
    if style.is_none() {
        // ui-text-exempt: unreachable filler; see the doc comment above
        return "that way";
    }
    style.axes()
}

/// Disclosure, **ladder rung 1, SAME family**: the page already carried the
/// bold or italic form of this text's own typeface, and pdfcer bound it.
///
/// ★ Worded as a **better** outcome rather than as a substitution, because it
/// is one. The operator asked for bold; the page turned out to carry a genuine
/// bold face, so they got a genuine bold face. Wording it as "pdfcer did
/// something other than what you asked" would train them to distrust a control
/// that just did its best possible job.
///
/// ★★ It ends by saying the letterforms are **unchanged**, which is the whole
/// reason this sentence and [`text_style_used_other_family`] are two sentences
/// and not one. To a draughtsman those are different events: taking the same
/// family's bold face is invisible on the plot, and taking another family's
/// changes the look of a title block. Only one of the two is worth his
/// attention, and a single sentence covering both would either alarm him about
/// the harmless case or fail to warn him about the visible one.
///
/// # ★★★ Why this is a PAIR again, and what it cost to be one sentence
///
/// It was a pair before 2026-09-11, keyed on
/// `FormatError::RealFaceAvailable { same_family, .. }`. Adopting the automatic
/// ladder deleted that error — rung 1 binds the face directly — and
/// [`StyleLadder`] as shipped on 2026-08-30 carried `requested`, `bound`,
/// `rung`, `synthesised` and `passed_over` and **no `same_family`**. The
/// engine's matching rule (`family_stem`) is private and engine invariant R74
/// forbids `pdfcer-gui` re-deriving it by name, so the shell could not recover
/// the answer: **the better verb carried less information than the two it
/// replaced.**
///
/// What stood here instead named both `/BaseFont`s — *"was set in Calibri and
/// is now set in Times-Bold"* — a fact the shell was entitled to state, needing
/// no family rule, and legible as a family change to whoever read it carefully.
/// Weaker than the sentence it replaced, and not a guess.
///
/// ⇒ It was filed as
/// `request_style_ladder_does_not_say_whether_the_face_it_bound_is_the_same_family.md`
/// and `Pass 295.0` shipped [`StyleLadder::same_family`], so the workaround is
/// **deleted** rather than left dormant: the engine's reply says in as many
/// words that naming both faces was the right call and can now go. The old
/// sentence's `from` argument went with it, which is why nothing here reads
/// `FormatReport::font_change` any more.
///
/// ★ `same_family` is `Option<bool>` and `None` — *nothing was bound* — must
/// **not** be flattened into `Some(false)`. Neither of this pair fires for it;
/// see `crate::app::actions::textstyle::ladder_note`, which traces it as an
/// engine invariant breaking rather than inventing a third sentence.
///
/// [`StyleLadder`]: pdfcer_core::text_edit::StyleLadder
/// [`StyleLadder::same_family`]: pdfcer_core::text_edit::StyleLadder::same_family
#[must_use]
pub fn text_style_used_sibling_face(bold: bool, italic: bool, to: &str) -> String {
    let style = axes(bold, italic);
    format!(
        "This page already carried {to}, the {style} form of this text's own typeface, so pdfcer used it rather than thickening or slanting the letters artificially. The letterforms are unchanged."
    )
}

/// Disclosure, **ladder rung 1, DIFFERENT family**: the only real face that
/// could show this text belonged to another typeface, and pdfcer used it.
///
/// # ★★★ This is the half of a restyle the operator can SEE
///
/// The engine's own word for a cross-family fallback is that it is *"a bigger
/// change than a weight swap"*, and on a drawing that is literally true: a
/// title block set in `Calibri` and restyled into `Times-Bold` does not merely
/// get heavier, it changes shape, and it changes shape at 1:1 on a plotter
/// where nobody is looking at a status line any more.
///
/// ★★ So the sentence leads with the **constraint** — no bold form of the
/// text's own typeface was available — before naming what pdfcer did instead.
/// That order matters: *"pdfcer used Times-Bold"* read cold sounds like a
/// choice somebody made carelessly, where *"nothing in your own typeface could
/// do it, so pdfcer used Times-Bold"* is the same fact with its reason
/// attached, and the reason is what tells the operator whether to accept it or
/// to add a face to the drawing.
///
/// ★ It does **not** name the old face. It used to, because naming both was
/// the shell's substitute for knowing the family relationship; now that
/// [`StyleLadder::same_family`] answers the question directly, the old name
/// adds a second `/BaseFont` to read and no information. See
/// [`text_style_used_sibling_face`] for the whole history.
///
/// [`StyleLadder::same_family`]: pdfcer_core::text_edit::StyleLadder::same_family
#[must_use]
pub fn text_style_used_other_family(bold: bool, italic: bool, to: &str) -> String {
    let style = axes(bold, italic);
    format!(
        "No {style} form of this text's own typeface could show it, so pdfcer used {to} — a real {style} face from a different typeface. The letterforms themselves will look different, not just heavier or slanted."
    )
}

/// Disclosure, **ladder rung 2**: the standard-14 sibling of this text's own
/// family was bound, and nothing was embedded.
///
/// # ★★★ The rung that was unreachable from this shell until 2026-09-11
///
/// There was no sentence for this because there was no outcome for it. This
/// shell asked for bold with `set_synthetic`, whose gate only ever looks at
/// faces **already on the page**; a page carrying nothing but `Helvetica` —
/// which is most CAD-exported title blocks, and pdfcer's own `rotated-text.pdf`
/// — has no bold resource, so the gate passed and the strokes were thickened.
/// `Helvetica-Bold` was a standard-14 name the whole time: every conforming
/// reader carries it, ISO 32000-1 §9.6.2.2 says it needs no font file, and
/// binding it is one new `/Font` resource of about sixty bytes.
///
/// ★★ So the operator was getting a **faked** weight on the commonest page in
/// their working set, five days after the engine shipped the rung that binds a
/// real one. The sentence says which of the two happened, because after this
/// change "pdfcer made it bold" has two very different meanings and only one of
/// them survives being printed at 1:1 on a plotter.
///
/// ★ It names the growth explicitly. An operator whose file must stay small —
/// a drawing going to a portal with an upload cap — is entitled to know that
/// this route did not embed a typeface, and the alternative reading ("pdfcer
/// added a font to my file") is the one they would otherwise assume.
#[must_use]
pub fn text_style_used_standard_face(bold: bool, italic: bool, to: &str) -> String {
    let style = axes(bold, italic);
    format!(
        "No {style} face on this page could show this text, so pdfcer set it in {to} — one of the fourteen faces every PDF reader is required to carry. The letters are genuinely {style} rather than thickened or slanted artificially, and nothing was embedded, so the file is essentially no larger."
    )
}

/// Disclosure, **ladder rung `AlreadyStyled`**: the text was already that way.
///
/// # ★★ Not a refusal, and the distinction is the whole point
///
/// Bold and Italic are buttons that APPLY, not switches that reflect — there is
/// no "is this run bold" bit in a PDF, so a pressed-in toggle would be claiming
/// to have read a fact that is not recorded. Pressing Bold on a run already set
/// in `Times-Bold` is therefore an ordinary, expected gesture, and the engine
/// answers it by taking the ladder's zeroth rung and changing nothing.
///
/// ★ Reported rather than silent, because "I pressed it and nothing happened"
/// is indistinguishable from a broken button. This sentence is the difference
/// between a control that did nothing and a control that had nothing to do.
#[must_use]
pub fn text_style_already_that_way(bold: bool, italic: bool) -> String {
    let style = axes(bold, italic);
    format!("This text is already {style}, so pdfcer left its weight and slant alone.")
}

/// Disclosure, **ladder rung 4**: nothing real was available, so the letters
/// were thickened or slanted artificially.
///
/// ★★ Both halves of the ladder's failure are named, because they send the
/// operator to different remedies. *"Nothing on this page"* is answered by
/// adding a face to the drawing; *"no standard face for this family"* is
/// answered by choosing a different family. A sentence saying only "pdfcer
/// faked it" leaves them with neither.
///
/// # ★★★ It still does not name the faces pdfcer tried, and the reason CHANGED
///
/// It used to be unable to. `StyleLadder::passed_over` was a `Vec<String>` of
/// pre-formatted `"BaseFont (the whole refusal message)"` pairs, and splitting
/// on the space before the bracket would have been a locator for the engine's
/// message format living in a GUI — filed as
/// `request_style_ladder_passed_over_is_prose_a_shell_has_to_parse.md`, and
/// `Pass 295.0` shipped `Vec<PassedOver>` with `base_font`, `reason` and a
/// structured `refusal` carrying the offending character.
///
/// ★★ **The structure landed and this sentence still does not use it, which is
/// a decision and not an oversight.** `FormatReport::disclosures` already
/// carries the engine's own *"Passed over (could not show these characters):
/// …"* clause, verbatim, into the very same status line, one line below this
/// one. Adding a shell sentence naming the same faces would be the same fact
/// twice, and this module's own rule — stated in
/// `crate::app::actions::textstyle::ladder_note` — is that **a disclosure
/// repeated is a disclosure skipped**.
///
/// ⇒ Where the structure went instead is the surface that had nothing:
/// **the hover, before the press.** `crate::panels::properties::text` reads
/// `EditSession::preview_style_ladder`'s `passed_over` and says which faces
/// pdfcer will pass over and which character defeats each one — in plain words
/// (*"no 'o'"*) rather than the engine's `R-INV-1: character U+006F`. After the
/// press the engine speaks; before it, nothing did.
///
/// [`PassedOver`]: pdfcer_core::text_edit::PassedOver
#[must_use]
pub fn text_style_faked(bold: bool, italic: bool) -> String {
    let style = axes(bold, italic);
    format!(
        "No real {style} face was available for this text — nothing on this page can show these characters that way, and this text's family has no standard {style} face to fall back on — so pdfcer thickened or slanted the letters artificially instead."
    )
}

/// Disclosure, **`StylePolicy::Warn` only**: the weight or slant was faked.
///
/// # ★★ Why this is a separate sentence rather than louder formatting
///
/// The engine already reports a synthesis in `FormatReport::disclosures`, and
/// under `Auto` that quiet report is the whole obligation. `Warn` exists for
/// the operator for whom *"a faked weight in the output is a problem worth
/// noticing at the moment it is created"* — a drawing that will be printed, a
/// document that will be handed on — and a disclosure they have to go looking
/// for does not serve them.
///
/// ★ It is prose rather than an alarm colour because the edit **happened**.
/// Rule 4's shape holds: the text renders exactly as it will render when
/// saved, and the fact about it is said off-canvas.
#[must_use]
pub const fn text_style_faked_warning() -> &'static str {
    "pdfcer faked that weight or slant — no real face on this page could show this text that way, so the letters are thickened or shaped artificially rather than set in a genuine bold or italic face."
}

// ★★★ `text_style_faked_instead(face)` WAS HERE until 2026-09-11, and it is
// deleted rather than kept, because **its subject is gone**.
//
// It said: *"This page carries {face}, but it has no shape for one or more
// characters in this text, so pdfcer thickened or slanted the letters
// artificially instead of using it."* That was the third rung of a dance this
// shell rolled by hand — ask for synthesis, catch
// `FormatError::RealFaceAvailable`, retry with the face it names, and when THAT
// refuses for coverage, fake it and say whose fault it was. The face in the
// sentence was `real_font`, straight off the refusal.
//
// The engine's automatic ladder (`Pass 179.0`) does that walk itself, better:
// it skips a face the coverage gate would refuse instead of trying and failing
// (`find_styled_face` filters on `accepted.is_ok()`), tries the standard-14
// sibling that this shell never reached at all, and only then synthesises. It
// reports the refused faces on `StyleLadder::passed_over` — as prose, not as
// names — and discloses them in its own words in the same note list.
//
// ⇒ There is no longer a route on which this shell holds a bare face name at
// the moment it fakes a weight. `text_style_faked` above is the replacement;
// it names the two REASONS, which is what the operator can act on, and the
// faces arrive in the engine's own verbatim sentence beside it.
//
// ★ The standing rule is the reason this is a tombstone and not a retained
// function: a mechanism with no caller rots, and the next reader cannot tell a
// deliberate fallback from a forgotten one. What is worth keeping is the
// ARGUMENT — that "pdfcer faked it" without the reason invites the operator to
// go looking for a bold face that is right there and does not work — and that
// argument is now carried by `text_style_faked`'s doc comment.

/// Disclosure: how many separate pieces of text one gesture restyled.
///
/// # ★ Why this sentence exists at all
///
/// `EditSession` has no undo-grouping verb, so restyling N runs is N entries in
/// the undo log and N presses of Ctrl+Z. That is a limit of the engine that the
/// operator meets through this shell, and an operator who presses Ctrl+Z once,
/// sees two thirds of their change still there and concludes undo is broken is
/// the exact outcome this sentence prevents.
///
/// Filed with the engine rather than worked around here — a shell-side coalesce
/// would work and would leave every other consumer with the same defect.
#[must_use]
pub fn text_style_multi(count: usize) -> String {
    format!(
        "That selection covered {count} separate pieces of text on the page, so pdfcer restyled each one. Ctrl+Z takes them back one at a time."
    )
}

// ★★★ `text_style_used_other_family` WAS DELETED AND RESTORED ON THE SAME DAY,
// 2026-09-11, and the round trip is recorded because the reasoning on the way
// down is what got it back.
//
// It was deleted in the morning because **the flag it was chosen by did not
// exist on the automatic route** — not because the distinction stopped
// mattering. Rung 1 reported `StyleLadder { requested, bound, rung, synthesised,
// passed_over }`, which did not say whether `bound` belonged to the run's own
// family. The engine KNEW — `plan_style_ladder` searches same-family first and
// then any family, so the answer is a branch it had already taken — and did not
// publish it.
//
// The argument for wanting it back was rule 4 read forwards: a fallback to
// another family is *"a bigger change than a weight swap"*, and it is **the one
// substitution the operator will SEE**. Reporting it in the same words as an
// ordinary same-family swap is rule 4 read backwards — disclosing the invisible
// and hiding the visible. Filed as
// `request_style_ladder_does_not_say_whether_the_face_it_bound_is_the_same_family.md`.
//
// ⇒ Answered in `Pass 295.0` the same day: `StyleLadder::same_family:
// Option<bool>`. The sentence is back above, as
// [`text_style_used_other_family`], paired with
// [`text_style_used_sibling_face`], and the pair is now chosen by an engine
// verdict rather than by a `FormatError` variant that the automatic route never
// returns.
//
// ★ NOT re-derived here, then or now. `family_stem` is private, and a shell
// that re-implements pdfcer's font-family matching is decision 058's exact
// case — it would agree on every fixture, disagree on the first real drawing,
// and be the workaround every other consumer then has to write for itself. The
// four hours between the deletion and the restoration were spent asking, which
// is the whole point of the request channel.

/// ★★★ **The cap fired on a PART, and nothing was said** —
/// `OPERATOR_REQUESTS.md` O69: *"the nodes are hard to see and click on."*
///
/// The sibling of [`crate::text::status::too_many_anchors`], and it exists
/// because that one's guard excluded the exact route the operator reported.
///
/// # What he saw, and why it read as broken rather than as limited
///
/// The Points tool puts the selection at the **Part** rung, so
/// `entered_object()` is `Some` — and the disclosure was gated on it being
/// `None`. A subpath with more than four hundred anchors therefore drew no
/// dots and said nothing: he armed the tool, clicked a shape, watched the
/// selection box change, and the program went quiet. A limit reported as an
/// absence is the failure `RESUME.md` records four separate occasions of.
///
/// # ★★ Why it is not the same sentence
///
/// [`crate::text::status::too_many_anchors`] ends *"Double-click into a part
/// of it, or use the Points tool, to see that part's"* — advice that is
/// correct at the Object rung and **wrong here**, because there is nothing
/// below a subpath to descend into. Reusing it would send him looking for a
/// rung that does not exist.
///
/// The remedy this one names is the one that now works: **zoom in**. Since
/// 2026-08-31 the cap counts what is on screen rather than what the path
/// contains, so magnifying the area genuinely makes the dots appear — which it
/// did not before, and which is why this sentence could not have been written
/// honestly until the cull shipped.
///
/// ★ It lives here rather than beside its sibling in `text::status` because
/// that module is at 1,482 lines against R2's 1,500. The seam is noticed
/// rather than trimmed, which is that file's own standing note.
#[must_use]
pub fn too_many_anchors_in_part(count: usize, cap: usize) -> String {
    format!(
        "This part has {count} points and pdfcer draws at most {cap} at a time, so none are \
         shown here. Zoom in to see the ones you are looking at."
    )
}
