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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// The chosen face cannot show every character in the run.
    FaceLacksCharacters,
    /// The operator's `style_policy` is `Refuse` and the only way to satisfy
    /// this request was to fake the weight or the slant.
    ///
    /// ★★ NOT the same thing the ENGINE's `StylePolicy::Refuse` refuses.
    ///
    /// The engine's gate refuses a synthesis only when a **real face was
    /// available** and would have been passed over. That is the right contract
    /// for a crate whose caller might genuinely mean "fake it". It is not what
    /// an operator who ticked *"never fake it"* asked for: on a page carrying
    /// no bold face at all, the engine's gate has nothing to refuse in favour
    /// of and thickens the strokes.
    ///
    /// ⇒ So this variant is raised by the SHELL, from
    /// `EditSession::preview_style_resolution`, and it is the wider reading —
    /// see `crate::app::actions::textstyle`. Recorded here because a future
    /// session reading only the engine's docs would conclude this variant is
    /// unreachable, and it is reached on the commonest page of all.
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
    pub const fn line(self) -> &'static str {
        match self {
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
            Self::FaceLacksCharacters => {
                "That face has no shape for one or more characters in this text. pdfcer changed nothing rather than substitute a different letter or leave a blank."
            }
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
        }
    }
}

/// Disclosure: a real face was used instead of a synthetic weight.
///
/// ★ Worded as a **better** outcome rather than as a substitution, because it
/// is one. The operator asked for bold; the page turned out to carry a genuine
/// bold face, so they got a genuine bold face. Wording it as "pdfcer did
/// something other than what you asked" would train them to distrust a control
/// that just did its best possible job.
#[must_use]
pub fn text_style_used_real_face(style: &str, face: &str) -> String {
    format!(
        "This page carries a real {style} face, so pdfcer used it: the text is now set in {face} rather than being thickened or slanted artificially."
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

/// Disclosure: a real face was offered, tried, and could not show the text, so
/// pdfcer faked it instead.
///
/// # ★★★ The third rung, and the sentence is the whole point of having it
///
/// This is the outcome that used to be a **refusal**. The engine's gate names
/// a real face of the run's own family; `set_font` then rejects it because it
/// has no shape for one of the characters — `Times-Bold` remaps `o` to a
/// bullet, so it cannot show `hello world` on a page where `Calibri-Bold` can.
///
/// The old behaviour said *"there is a real bold face, use it"* about a face
/// that had just failed, and changed nothing. This says what actually
/// happened, **and names the face**, because "pdfcer faked it" without the
/// reason invites the operator to go looking for a bold face that is right
/// there and does not work.
#[must_use]
pub fn text_style_faked_instead(face: &str) -> String {
    format!(
        "This page carries {face}, but it has no shape for one or more characters in this text, so pdfcer thickened or slanted the letters artificially instead of using it."
    )
}

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

/// Disclosure: bold or italic was applied by switching to a face from a
/// **different family**.
///
/// ★★★ A separate sentence from [`text_style_used_real_face`], because the
/// engine draws the distinction itself and says why: a fallback to another
/// family is *"a bigger change than a weight swap"*, offered only when no face
/// of the run's own family on that page can show the run.
///
/// ★★ It is also the one substitution the operator **will** see. A weight swap
/// within a family looks like bold; a family change looks like different
/// letters. Reporting it in the same words as an ordinary real-face
/// substitution would be true and would bury the half they can notice — which
/// is Rule 4 read backwards, disclosing the invisible and hiding the visible.
#[must_use]
pub fn text_style_used_other_family(style: &str, face: &str) -> String {
    format!(
        "No {style} face of this text's own family is on the page, so pdfcer used {face} instead. The letterforms will look different, not just heavier or slanted."
    )
}

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
