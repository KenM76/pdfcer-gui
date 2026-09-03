//! # `canvas::markup::text` — underline, strikeout and squiggly: markup whose
//! operand is a **selection**, not a drag
//!
//! The three kinds `FEATURES.md` has carried as `⬜ engine-ready` for the whole
//! project, behind one blocker stated in the operator's own terms — *"they mark
//! **text** and there is no text-selection gesture yet."* That gesture landed on
//! 2026-08-14 ([`crate::canvas::textsel`]), so this is that work's first payoff
//! rather than a new subsystem.
//!
//! ---
//!
//! ## 1. ★ THE INTERACTION DECISION, and it is the whole of this module
//!
//! [`super`]'s four kinds are **drag-shaped**: press, rubber-band, release,
//! commit. These three are not, and there were two honest ways to build them:
//!
//! | | model | what it costs |
//! |---|---|---|
//! | **(a)** | select text first, **then** press Underline — the selection is the operand | nothing new: no [`CanvasTool`](crate::canvas::tool::CanvasTool) variant, no gesture, no mode of `textsel` |
//! | **(b)** | arm Underline, then sweep, and the sweep both selects and marks | a tool variant, a gesture, a second mode of `textsel`, a second place a text range is resolved |
//!
//! **Shipped: (a), and it came from Acrobat**, which is the reference
//! application that decides this one under `HANDOFF.md` §3.4 — *"make your best
//! educated guesses to match what inkscape, acrobat, and SolidWorks do"* — for
//! the reason `textsel`'s own table gives: **Acrobat wins ties about reading and
//! about marking up what has been read, because Acrobat is what pdfcer
//! replaces.** Inkscape and SolidWorks have no vote here at all: neither has PDF
//! text markup, and neither has anything shaped like it (Inkscape's text tool
//! edits its own text objects; a SolidWorks note is content, not a comment on
//! content).
//!
//! ### What Acrobat actually does, and the honest complication
//!
//! Acrobat offers **both**. Select text with the Selection tool and a
//! context/hover menu appears carrying *Highlight text*, *Underline text*,
//! *Strikethrough text* — model (a) — and the Comment toolbar separately carries
//! the three as arm-then-sweep tools — model (b). Reader, the product this shell
//! is measured against, leads with (a): selecting is what its one tool does, and
//! marking is what you do to a selection.
//!
//! So (a) is *"what Acrobat does"* without qualification, and (b) is *"what
//! Acrobat also does"*. Given a tie broken toward the smaller change, three
//! further things decide it and each is specific to this shell rather than to
//! taste:
//!
//! 1. **(b) would need a second text-range derivation.** `textsel`'s §2 already
//!    refused Acrobat's `Alt`+drag rectangular selection on exactly this
//!    ground — *"one derivation, so what is shown and what is copied cannot
//!    diverge"*. An armed marking sweep is a third resolver beside `drag` and
//!    `click`, and the moment it exists the wash the operator sees and the
//!    `/QuadPoints` written to the file are produced by two functions that can
//!    disagree.
//! 2. **The operand is already visible.** A text selection paints a wash. Under
//!    (a) the operator can see precisely what will be marked *before* pressing
//!    anything, which is the pre-commit affordance rule 4 asks for — and it
//!    exists already, for free, drawn by the module that owns it (§4).
//! 3. **(a) is the shape `format.delete` already has.** A command that acts on
//!    a selection is not a new idea in this shell; a tool that arms and marks
//!    would be.
//!
//! What is deliberately **not** claimed: that (b) is wrong. It is the tool half
//! of Acrobat's answer and it is a reasonable thing to add later. What would
//! make it affordable is the `CanvasTool::Text` variant `textsel` §3 already
//! specifies for a different reason; until that exists, (b) is three new states
//! in the tool enum to reach one gesture.
//!
//! ★ **That variant now exists** ([`crate::canvas::tool::CanvasTool::Text`],
//! 2026-08-14), so the sentence above has come due and the answer is still (a).
//! The tool made (b) *cheaper* and did not make it right: (b)'s cost was never
//! mainly the tool enum, it was objection 1 — a marking sweep is a **second
//! text-range resolver** beside `drag` and `click`, and the moment it exists the
//! wash the operator sees and the `/QuadPoints` written to the file come from
//! two functions that can disagree. That objection is untouched.
//!
//! What the tool did do is remove the *reason a reader might have wanted* (b):
//! model (a) was unreachable in Edit, and (b) would have been a way to reach it
//! there. It is reachable now, by the route that costs no second derivation.
//!
//! ### 1.1 ★ The route is the ribbon, and Acrobat's is a menu on the selection
//!
//! Half of model (a) is *where the operator finds the verb*, and here the
//! shipped answer and Acrobat's differ: Acrobat pops a context menu — and, in
//! recent versions, a floating hover toolbar — **on the selection itself**,
//! while this ships the three commands on **Markup ▸ Text markup** and nothing
//! else. Named as a gap rather than passed over, because a reviewer who selects
//! a phrase and right-clicks it will expect the verbs to be there.
//!
//! What it would take, so the next hand does not re-derive it: a third canvas
//! menu context beside `CANVAS_OBJECT` and `CANVAS_EMPTY` — call it
//! `canvas.text` — chosen in [`crate::canvas::menus`] by the same
//! decide-at-the-click rule those two already use, with the decision being
//! *"was the pointer over a live text selection?"*. That is a real question with
//! a real answer (the selection's canvas quads are on the document and the
//! pointer position is in the same space), and it is a **menu taxonomy** change:
//! a new context id, its entry in the manifest's `menus`, and a rule for which
//! of three menus opens. Deferred on the same principle `textsel` deferred
//! `CanvasTool::Text` — the shape of the fix is written down; the fix is not
//! smuggled into the module that noticed it.
//!
//! **No chord is bound either**, deliberately and by the argument the operator's
//! own zoom-to-selection decision settled: this shell's manifest chords are
//! `Ctrl`-modified by construction, `Ctrl+U` is not an Acrobat binding to match,
//! and inventing one would match the muscle memory of nobody. The keymap stays
//! at nineteen bindings.
//!
//! ---
//!
//! ## 2. ★ THE MODE INTERSECTION, and it is narrower than either half
//!
//! This is the finding a reader most needs, because neither capability alone
//! predicts it. Marking text needs **both** halves, and they do not overlap in
//! the way anybody would guess:
//!
//! | mode | can select text? | `author_markup`? | can mark text? |
//! |---|:-:|:-:|:-:|
//! | `read` | ✓ | ✗ | **no** — nothing to mark *with* |
//! | `review` | ✓ | ✓ | **YES** |
//! | `edit` | ✓ **with the text tool armed** | ✓ | **YES**, since 2026-08-14 |
//!
//! ★ **The Edit row changed, and this section is now the record of how.** It
//! read *"`edit` | ✗ | ✓ | **no** — nothing to mark"*, and the paragraph under
//! it said the three controls were drawn there and **permanently greyed**. That
//! was true, it was a `RIBBON_IA.md` **P3** violation, and it is closed — by
//! exactly the fix this file predicted, in exactly the place it predicted, with
//! **no change to this file's rules**. The prediction is kept below rather than
//! deleted, because a fix that lands where its own design note said it would is
//! the strongest evidence a note is worth writing.
//!
//! > **Edit** shows the Markup tab, so the three controls are drawn there and
//! > are **permanently greyed**, because Edit's primary button is the content
//! > marquee and `textsel::takes_the_press` therefore refuses it a text
//! > selection. That is an inversion — an editor may not mark text a reviewer
//! > may — and it is the *same* inversion `textsel` §3 already records as a
//! > known gap with a known fix: a `CanvasTool::Text` armed by a `view.tool_*`
//! > command, at which point `takes_the_press` gains one disjunct and these
//! > three controls come alive in Edit with **no change to this file**.
//!
//! What actually landed: [`crate::canvas::tool::CanvasTool::Text`], armed by
//! **`view.tool_text`** in View ▸ Navigate beside the hand tool. An editor arms
//! it, sweeps a range, and presses Underline; `selection.text` is published from
//! the same live selection it always was, and the controls enable. The rule
//! `mark` implements is untouched, and so is every test below it — which is what
//! *"no change to this file"* was claiming.
//!
//! **What the two remaining zeros are, and why neither is a defect:**
//!
//! * **Read** is shown the File and View tabs only, so the Markup tab is not
//!   there and no `markup.*` command can be invoked; the dispatch arm declines
//!   anyway (`caps.author_markup`), which is the belt to that braces for a
//!   customized manifest that binds a chord. Read *not* authoring is the point
//!   of Read — `DEFECTS.md` D6. Note that Read *can* select text and now also
//!   carries the `view.tool_text` control (View is in every mode); arming it
//!   there changes nothing, because Read's select tool already swept text. What
//!   Read still cannot do is **mark** what it selected, which is the correct
//!   half of this table to be empty.
//! * **Edit without the tool armed** is still a zero, and deliberately: the
//!   controls are greyed exactly as long as there is no live text selection,
//!   which is *temporarily* unavailable in P3's own sense — the operator's next
//!   act can change it, from a control on a tab they are already being shown.
//!   That is the difference between the state before this change and the state
//!   after, and it is the whole of the difference: the pixels are identical, and
//!   the reachability is not.
//!
//! ### Greying, and the rule that now actually applies
//!
//! `RIBBON_IA.md` P3 forbids a control that is *always live and does nothing*,
//! and reserves greying for *temporarily unavailable, explained on hover*. Every
//! greyed state these three controls can now reach is temporary in that sense:
//! sweep some text and it ends. The paragraph that used to stand here argued
//! that Edit's unavailability was *"longer-lived than 'temporary' usually
//! means"* and that the alternative was a mechanism invented to hide a gap — a
//! defensible reading of a bad situation, and it is worth noticing that the
//! situation, not the reading, is what was wrong. **A rule being uncomfortable
//! to satisfy is evidence about the feature, not about the rule.**
//!
//! ---
//!
//! ## 3. The three kinds are NOT `MarkupKind` variants, deliberately
//!
//! [`super::MarkupKind`]'s contract is written down in its own docs and in
//! `shell::commands::mapping`: *a variant belongs in that enum when this rubber
//! band can draw it*, and every variant is required by test to have a command
//! that **arms a tool** and a `selected:` condition that lights while it is
//! armed. None of that is true of these three: their commands act immediately
//! and arm nothing, so a variant would be a tool that cannot be armed, a pressed
//! state that never lights, and a `CanvasTool` state no `GestureOutcome` can
//! reach — dead states in a type whose whole purpose is to say what the tool is
//! currently doing.
//!
//! So they are a separate enum, [`TextMarkKind`], with its own `ALL` and its own
//! id mapping (`shell::commands::text_mark_command`), and the two families stay
//! disjoint — which the mapping module asserts in both directions, because
//! `app::dispatch`'s guard arms are tried in order and an overlap would swallow
//! one silently.
//!
//! ### Highlight is the fourth `/QuadPoints` subtype and stays where it is
//!
//! [`super::MarkupKind::Highlight`] is engine-identical to these three —
//! `MarkupSpec::TextMarkup` with a different [`TextMarkupKind`] — and it remains
//! a **drag** across an area, because that is what it already is and because a
//! highlight over an image or a title-block cell is a thing operators want and
//! text markup cannot express. Acrobat has both there too (a text highlight from
//! the selection, an area highlight from the Comment toolbar).
//!
//! Making `markup.highlight` mean *"mark the selection if there is one, else arm
//! the band"* was considered and refused: one control with two behaviours
//! decided by invisible state is precisely the thing an operator cannot predict,
//! and the trace would not say which happened. If the selection-highlight is
//! wanted, it is a **fourth entry in [`TextMarkKind`]** and a fourth command —
//! at which point the Text markup group reads Highlight-text, Underline,
//! Strikeout, Squiggly and the area Highlight sits with the Shapes it behaves
//! like. That is a taxonomy change and belongs to the operator, so it is
//! recorded here and not taken.
//!
//! ---
//!
//! ## 4. ★ There is no second preview, and that is the decision
//!
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` rule 4 permits a
//! pre-commit affordance and requires that it describe *what will actually
//! commit*. [`super`]'s band satisfies that by being drawn in the shape and the
//! pen of the annotation it is about to author.
//!
//! Here the affordance already exists and is drawn by somebody else: **the
//! selection wash is the preview.** Its boxes are [`TextSelection::quads`], and
//! the quads this module authors are [`TextSelection::page_quads`] — literally
//! the same boxes from the same pass (`textsel` §5.1). A preview cannot describe
//! what will commit more exactly than by being it.
//!
//! The argument for adding a second one anyway, taken seriously and rejected: an
//! underline is not a wash, so a *shape* preview — a line at each quad's
//! baseline, in the pen colour — would show the operator the mark rather than
//! the range. Three things against, in order of weight:
//!
//! 1. **It would have to be drawn on hover of a ribbon button**, because there
//!    is no gesture in flight to hang it on. A canvas that changes while the
//!    pointer is in the ribbon is a canvas that flickers as the operator's hand
//!    passes over three adjacent controls.
//! 2. **It would be a second geometry** — the baseline offset, the squiggle's
//!    amplitude — approximating an appearance stream `pdfcer-core` generates. Two
//!    drawings of one annotation is how the preview comes to lie, and the lie
//!    would be invisible until the file was reopened.
//! 3. **The commit is instant and undoable.** There is no drag to abandon: press
//!    the button, see the annotation, undo it. The affordance a rubber band
//!    exists to provide — *aim before you commit* — is already provided by the
//!    selection.
//!
//! So: no hover preview, no ghost, no second colour. The wash, then the
//! annotation.
//!
//! ---
//!
//! ## 5. What this module does not do
//!
//! It does not touch a document, does not take an `EditSession`, and builds no
//! appearance stream. [`spec`] hands `pdfcer-core` a `MarkupSpec` and
//! `EditSession::add_markup` does the rest, which is the same route
//! `pdfcer`'s `markup-add` takes with the same value — the equivalence the
//! measure salvage's tests exist to protect, and the reason a canvas-authored
//! annotation is byte-identical to a CLI-authored one.
//!
//! [`mark`] is pure and is where every rule lives; the dispatch arm calls it and
//! pushes what it returns. That is the same division [`super::action`] has, for
//! the same reason: a rule with a test beats a rule inside a `match` arm.

use pdfcer_core::annot_author::{Color, MarkupSpec, Quad, TextMarkupKind};

use crate::app::actions::Action;
use crate::canvas::textsel::TextSelection;

/// Which of the three selection-marking subtypes a command authors.
///
/// Separate from [`super::MarkupKind`] because these are not drag-shaped and
/// arm no tool — see the module header §3. The order is the order the Markup
/// ribbon's Text markup group lists them, after Highlight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextMarkKind {
    /// ★★★ `/Highlight` — a translucent wash over each quad.
    ///
    /// **Added 2026-08-28** (`OPERATOR_REQUESTS.md` **O54**), and it is the one
    /// kind in this enum that is *also* a [`super::MarkupKind`] — the armed
    /// tool that draws an area highlight by dragging a box.
    ///
    /// ★★ That is not a duplication, and the reason is what these two enums
    /// actually encode: **not identity, but GEOMETRY.** `MarkupKind` is *"kinds
    /// whose operand is a shape the pointer draws"*; this one is *"kinds whose
    /// operand is a run of text"*. Highlight is the only kind that is honestly
    /// both, because a highlight over text follows the lines and a highlight
    /// over a scan is an area — and Acrobat's own tool does exactly that.
    ///
    /// ⇒ A kind reachable by two gestures needs an entry in both tables. The
    /// alternative — one enum with a geometry field — would put a branch in
    /// every arm that today cannot be wrong, to express a thing that is true of
    /// one variant.
    ///
    /// ★ It takes the **highlighter**, not the ink, unlike the other three —
    /// see [`Self::rgb`]. Same instrument, same swatch, whichever gesture
    /// reached it.
    Highlight,
    /// `/Underline` — a line near each quad's baseline.
    Underline,
    /// `/StrikeOut` — a line through each quad's vertical middle.
    StrikeOut,
    /// `/Squiggly` — a wavy line at each quad's baseline.
    ///
    /// pdfcer authors this natively **even though Acrobat's own UI does not** —
    /// a deliberate exceed-Acrobat choice recorded in `pdfcer-core`'s
    /// `TextMarkupKind::Squiggly`: the subtype is fully spec-legal (§12.5.6.10)
    /// and Acrobat displays it. It is offered here for that reason and not by
    /// oversight; the standing instruction is to *match* the reference
    /// applications, and matching does not mean declining something the engine
    /// already writes correctly.
    Squiggly,
}

impl TextMarkKind {
    /// Every variant, in the order the Markup ribbon lists them.
    ///
    /// Exists for the reason [`super::MarkupKind::ALL`] does and is the same
    /// shape deliberately: it lets the registry side map a command id to a kind
    /// and back through one pair of total functions, so a fourth kind added here
    /// fails a both-directions test rather than arriving with no command — or
    /// with a command that authors nothing.
    ///
    /// The mapping itself lives in `shell::commands::mapping`, not here:
    /// command ids are `shell/`'s vocabulary and `shell/` is a single-writer
    /// resource.
    pub const ALL: &'static [TextMarkKind] = &[
        TextMarkKind::Underline,
        TextMarkKind::StrikeOut,
        TextMarkKind::Squiggly,
    ];

    /// The `pdfcer-core` subtype this kind authors.
    ///
    /// The one place the shell's vocabulary meets the specification's, exactly
    /// as [`super::spec`] is for the geometric kinds. The two enums are
    /// deliberately not the same type even though three of the four names match:
    /// `TextMarkupKind` carries Highlight as well, which belongs to the *band*
    /// gesture here (§3), and a shared type would make that fourth value
    /// reachable from a control that cannot mean it.
    #[must_use]
    fn subtype(self) -> TextMarkupKind {
        match self {
            Self::Highlight => TextMarkupKind::Highlight,
            Self::Underline => TextMarkupKind::Underline,
            Self::StrikeOut => TextMarkupKind::StrikeOut,
            Self::Squiggly => TextMarkupKind::Squiggly,
        }
    }

    /// ★ **All three take the INK, and none of them takes the highlighter.**
    ///
    /// [`super::pen::Pen`] holds two colours because a stroke and a wash are
    /// different instruments — `pen.rs`' own words, *"an operator who sets the
    /// pen to green does not thereby want a green highlight, any more than
    /// picking a green biro changes the marker in their other hand."* This
    /// function is where that split is applied to the text kinds, and the answer
    /// is not a judgement call: **Underline, StrikeOut and Squiggly are lines**,
    /// so they are the biro. `Highlight` is the wash, and it is not in this enum
    /// at all — it is a *band* gesture handled by [`super::band`], which reaches
    /// the highlighter through [`super::pen::Pen::colour_for`].
    ///
    /// So the two modules partition [`super::pen::Pen`] exactly, with no kind
    /// reaching both colours and no kind reaching neither, and
    /// `pen::tests::every_geometric_kind_takes_the_ink_and_only_highlight_does_not`
    /// pins the half it can see.
    ///
    /// # ★ Why this used to be a hard-coded triple, and why that stopped being
    /// right
    ///
    /// Until 2026-08-17 this was `fn rgb(self) -> (f64, f64, f64)` returning
    /// `(0.85, 0.16, 0.16)` for all three, under a doc comment that said:
    ///
    /// > there is no pen control in this shell yet, so the default is stated
    /// > once, in the one place a spec is built, and **a real pen replaces
    /// > exactly this function**.
    ///
    /// That was correct when written. The real pen arrived in `4035b64` — two
    /// swatches and a width in Markup ▸ Style — and **did not replace this
    /// function**, because nothing connected the two: the constant compiled, the
    /// tests asserted the constant, and the swatch worked perfectly on every
    /// kind that went through [`super::spec`]. The observable result was a
    /// shipped inconsistency in the commit that answered *"I can't change a
    /// markup's colour"* — set the pen to blue, draw a rectangle, get blue;
    /// underline a word, get red.
    ///
    /// The generalisable part is not "remember to update duplicates". It is
    /// that **a doc comment naming its own seam is an asset only if something
    /// checks the seam when it is filled.** `NO_SURFACE.md` §1 praised exactly
    /// this style of comment for predicting `super::spec`'s refactor — and the
    /// same sweep listed this line as *"Underline / StrikeOut / Squiggly colour
    /// — surface: none"* without noticing that it was no longer a missing
    /// control but a **stale duplicate of one that now existed**. A prose seam
    /// marker is a note to a human; the thing that would have caught this is a
    /// test asserting the two paths agree, which is now
    /// `tests::the_ink_reaches_every_text_kind`.
    ///
    /// # Why the default is unchanged, and why that is not a coincidence
    ///
    /// [`super::pen::Pen::default`]'s ink is `(0.85, 0.16, 0.16)` — the same
    /// triple this function returned. So a build whose operator has never
    /// touched the swatch authors byte-identical annotations before and after
    /// this change, and the original argument for red still stands and still
    /// belongs somewhere: these three are lines, a line must be seen against the
    /// text it marks, a yellow underline under black glyphs on white paper is
    /// very nearly invisible, and red is what every other reader draws them in.
    /// That argument is now `pen.rs`' to make, because it is now `pen.rs`' value
    /// — which is the right home for it, since it is one default rather than
    /// two that must be kept equal.
    #[must_use]
    fn rgb(self, pen: super::pen::Pen) -> (f64, f64, f64) {
        match self {
            // ★★ The HIGHLIGHTER, not the ink, and it is the one arm that
            // differs. `pen.rs`: *"an operator who sets the pen to green does
            // not thereby want a green highlight, any more than picking a green
            // biro changes the marker in their other hand."* A highlight is a
            // wash whichever gesture drew it, so a text-following one and an
            // area one must come out of the same swatch — the alternative is
            // one feature that changes colour depending on how it was reached.
            Self::Highlight => pen.highlighter,
            Self::Underline | Self::StrikeOut | Self::Squiggly => pen.ink,
        }
    }
}

/// Why a text-markup command authored nothing.
///
/// Reported rather than silently absorbed, and with enough detail to act on,
/// for the reason [`super::Refusal`] carries: *"nothing happened"* has several
/// causes with opposite responses. A separate enum from that one because the
/// causes genuinely do not overlap — a drag can be degenerate and a selection
/// cannot; a selection can be stale and a drag cannot — and one enum covering
/// both would have every reader asking which half applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// There is no text selection to mark.
    ///
    /// **Unreachable from the ribbon**, because the commands are registered
    /// `enabled_when("selection.text")` and that condition is published from
    /// exactly this state's negation. Reachable from a chord in a customized
    /// manifest, which is why it exists rather than being an `expect`.
    NoSelection,
    /// A selection exists and describes a revision that has moved.
    ///
    /// The one refusal an operator can reach *by accident*: mark a selection,
    /// and the edit that lands bumps the epoch, so pressing a second text-markup
    /// command without re-sweeping asks to mark glyphs whose positions were
    /// recorded against the previous revision. Declining is the only honest
    /// answer — `canvas::textsel` §7 — and writing the annotation anyway would
    /// put a mark over *possibly* the wrong words into the file, which is the
    /// one thing rule 4 forbids outright.
    Stale,
    /// The selection resolved to no quads at all.
    ///
    /// Structurally unreachable — `textsel::resolve` returns `None` rather than
    /// a selection with an empty box list — and refused explicitly anyway,
    /// because the alternative is `EditError::EmptyGeometry` coming back from
    /// the engine for a shell that promised never to send it geometry that draws
    /// nothing. The guard is ours, upstream of theirs, exactly as
    /// [`super::Refusal::NoExtent`] is for the drag kinds.
    NoQuads,
}

/// Build the `pdfcer-core` spec one text-markup command authors.
///
/// Pure and unit-tested, for the reason [`super::spec`] is: the dispatch arm is
/// a routing line, and *which subtype*, *which colour* and *which quads* are
/// rules that deserve a test each.
///
/// Note what it does **not** do: normalise, order, merge or clip the quads. They
/// arrive from [`TextSelection::marks`] already grouped one per line of the
/// selection, in content order, in PDF user space — see `canvas::textsel` §5.1.
/// Touching them here would be the second geometry that section is about.
///
/// # The `pen` is a parameter, not a read
///
/// The exact signature change [`super::spec`] took when the Style group landed,
/// and for the same reason: this is a pure function whose job is to say what a
/// given request authors, and a colour it fetched for itself would make it a
/// function of application state that a test cannot vary. It takes the pen the
/// [`Action`] carried, and [`TextMarkKind::rgb`] decides which of the pen's two
/// colours a text kind is entitled to.
#[must_use]
pub fn spec(kind: TextMarkKind, quads: Vec<Quad>, pen: super::pen::Pen) -> MarkupSpec {
    let (r, g, b) = kind.rgb(pen);
    MarkupSpec::TextMarkup {
        kind: kind.subtype(),
        quads,
        color: Color::Rgb(r, g, b),
    }
}

/// ★ **The ONE action a text-markup command becomes** — the whole rule, pure.
///
/// Everything the command means is here: which selection is eligible, what a
/// stale one does, and what travels to the apply arm. `app::dispatch` calls this
/// and either pushes the `Ok` or traces the `Err`; it decides nothing, which is
/// the choke-point rule (`HANDOFF.md` §6) applied to a verb whose operand is not
/// the pointer.
///
/// # Why the quads travel rather than the selection
///
/// An [`Action`] is *a complete statement of intent, resolvable after the frame
/// that raised it* — the same property [`Action::CommitMarkup`] and
/// `VectorAction::DeleteSelection.into()` are built on. Carrying the selection instead would
/// mean the apply arm re-reading `doc.text_selection`, which by then may have
/// been cleared by the same frame's Escape, replaced by a click, or invalidated
/// by another action applied first. Carrying the quads makes the action a fact
/// about what the operator asked for at the moment they asked.
///
/// The **page** travels for the same reason and from the same place: it is
/// [`TextSelection::page`], not `doc.view.page_index`. A selection made on the
/// title-block sheet and marked after paging away must mark the sheet it was
/// made on; re-deriving the page in the apply would silently author it wherever
/// the operator happens to be looking.
///
/// # ★ The pen is sampled HERE, not read in the apply arm
///
/// [`Action::CommitMarkup`]'s `pen` field carries the argument in full and it
/// applies here without amendment: *"reading the live pen in the apply arm
/// would author a mark in whatever colour the operator happened to have
/// selected by the time the queue drained, which for a queue is a real gap and
/// not a theoretical one: the dispatcher raises actions during the frame and
/// `apply` runs at the end of it."*
///
/// It is the same rule this function's own docs already make about the quads
/// and the page, applied to the third thing that can change between the ask and
/// the apply. An action is a complete statement of intent; a statement of
/// intent that omits the colour is one the apply arm has to finish guessing.
pub fn mark(
    kind: TextMarkKind,
    selection: Option<&TextSelection>,
    epoch: u64,
    pen: super::pen::Pen,
) -> Result<Action, Refusal> {
    let selection = selection.ok_or(Refusal::NoSelection)?;
    let quads = selection.marks(epoch);
    if quads.is_empty() {
        // `marks` returns an empty slice for BOTH "stale" and "no quads", and
        // the two are told apart here rather than by two accessors: the
        // staleness rule belongs to `textsel` and asking it once is what keeps
        // this module from carrying a second copy of it.
        return Err(if selection.live(epoch) {
            Refusal::NoQuads
        } else {
            Refusal::Stale
        });
    }
    Ok(Action::CommitTextMarkup {
        page: selection.page,
        kind,
        quads: quads.to_vec(),
        pen,
    })
}

/// Report a text-markup command that authored nothing, with the reason.
///
/// One trace shape per refusal, so a harness reads `text-markup-declined` and
/// finds the cause on the same line rather than inferring it from an absence —
/// the contract `super::decline` and `canvas-move-declined` already honour.
pub fn decline(kind: TextMarkKind, reason: Refusal) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-markup-declined kind={kind:?} reason={reason:?}")
    });
}

/// Report a text markup that is about to be authored.
///
/// ★ Traced with its **quad count and its page**, not a success flag, for the
/// reason [`super::drag`]'s trace carries its coordinates: a line saying only
/// *"committed"* would be equally true before and after the defect anybody is
/// hunting. Here the two numbers that can be wrong are *how many boxes* (a
/// grouping that collapsed, or one that never merged) and *which page* (the
/// selection's, or the one currently on screen), and both are on the line.
///
/// Emitted from the dispatch arm rather than from [`mark`] so that a refusal and
/// a commit are traced from one place in one order, and so that [`mark`] stays a
/// pure function a test can call without a diagnostic channel.
pub fn trace_commit(kind: TextMarkKind, page: usize, quads: usize) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-markup-commit kind={kind:?} page={page} quads={quads}")
    });
}

/// **A markup drag that found text under it: the quads it would cover, and the
/// commit on release.** `OPERATOR_REQUESTS.md` **O54**.
///
/// Returns `Some(marks)` — canvas-space rectangles, one per line the drag
/// crosses — when the gesture is following text, and `None` when it is not, so
/// the caller falls through to the area band.
///
/// # ★★★ Why this is the DEFAULT for a highlight and the band is the fallback
///
/// The operator: *"we should be able to drag it along to just highlight text
/// too like it works in adobe."* Acrobat's Highlight follows text, and it is the
/// convergent behaviour of the class.
///
/// ★★ pdfcer's fallback is **better than the reference** and is kept for that
/// reason: over a scan with no text layer Acrobat's highlight draws nothing at
/// all, and an area highlight there is exactly what a drawing office wants. So
/// the rule is *follow text where there is text, box where there is not*, which
/// strictly dominates the behaviour being matched.
///
/// # ★★ Only Highlight, and the other seven band kinds are not offered this
///
/// A rectangle, an ellipse, an arrow or a cloud drawn over a paragraph means the
/// shape, not the words — nobody drags an arrow expecting it to follow a line of
/// text. Highlight is the one band kind whose *subject* is the text it covers,
/// which is why it is the one kind that appears in both geometry enums.
///
/// # ★ It commits nothing before the release
///
/// Same contract every preview in this crate is held to: the marks are handed
/// back on every frame so the operator can see what they are about to get, and
/// the action is raised once, on `Phase::Complete`. A preview that promised
/// quads and then committed a box would be the dishonesty rule 4 forbids.
pub struct Swept<'a> {
    /// The armed markup kind. Only `Highlight` is answered — see the docs.
    pub kind: super::MarkupKind,
    /// The pen, for the wash.
    pub pen: super::pen::Pen,
    /// The open document, for its text and its page.
    pub doc: &'a crate::app::state::OpenDoc,
    /// The page on screen.
    pub page_index: usize,
    /// The drag's two endpoints, in canvas space.
    pub from: egui::Pos2,
    /// See [`Self::from`].
    pub to: egui::Pos2,
    /// Where the gesture is.
    pub phase: crate::canvas::gesture::Phase,
}

pub fn swept(frame: Swept<'_>, actions: &mut Vec<Action>) -> Option<Vec<egui::Rect>> {
    let Swept {
        kind,
        pen,
        doc,
        page_index,
        from,
        to,
        phase,
    } = frame;
    if kind != super::MarkupKind::Highlight {
        return None;
    }
    let page_text = doc.page_text()?;
    let page = doc.pages.get(page_index)?;
    // ★ The SAME options the extraction ran with — `textsel::PageContext::opts`
    // — so the runs this drag sweeps are segmented exactly as the runs the
    // canvas paints and the find bar searches.
    let ctx = crate::canvas::textsel::PageContext {
        text: &page_text,
        page,
        index: page_index,
        epoch: doc.edit_epoch,
    };
    let selection = crate::canvas::textsel::drag(&ctx, from, to)?;
    let marks = selection.highlights(page_index, doc.edit_epoch);
    if marks.is_empty() {
        // ★★ No quads is NOT the same as no text: a drag that began and ended
        // inside one glyph selects nothing, and so does one over a page whose
        // text could not be extracted. Both mean *"this gesture is not
        // following text"*, and both fall through to the band — which is the
        // honest answer rather than a highlight of nothing.
        return None;
    }
    let marks = marks.to_vec();
    if phase == crate::canvas::gesture::Phase::Complete {
        match mark(
            TextMarkKind::Highlight,
            Some(&selection),
            doc.edit_epoch,
            pen,
        ) {
            Ok(raised) => {
                trace_commit(TextMarkKind::Highlight, page_index, marks.len());
                actions.push(raised);
            }
            Err(reason) => decline(TextMarkKind::Highlight, reason),
        }
        // Nothing is previewed on the frame that commits: the annotation is
        // about to be drawn for real, and a wash left over it would be a second
        // copy of the same colour, one frame stale.
        return None;
    }
    Some(marks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::page_tree::Rect as PageRect;

    /// One line's worth of quad, at a plausible page position.
    fn quad(y: f64) -> Quad {
        Quad::from_rect(PageRect::from_corners(72.0, y, 300.0, y + 10.0))
    }

    /// A selection over `lines` lines of the given page, stamped with `epoch`.
    fn selection(page: usize, epoch: u64, lines: usize) -> TextSelection {
        TextSelection::for_test(
            page,
            epoch,
            (0..lines).map(|i| quad(700.0 - 12.0 * i as f64)).collect(),
        )
    }

    /// The default pen, for the tests whose subject is not the colour.
    ///
    /// Named rather than spelled `Pen::default()` at nine call sites so that the
    /// tests which *are* about colour stand out by building their own — the
    /// reader can tell at a glance which assertions would move if the default
    /// moved.
    fn pen() -> crate::canvas::markup::pen::Pen {
        crate::canvas::markup::pen::Pen::default()
    }

    // -----------------------------------------------------------------
    // ★ The subtype, the colour and the quads
    // -----------------------------------------------------------------

    /// ★ **Each kind authors its own `/Subtype`, and none borrows another's.**
    ///
    /// The failure this catches is the copy-paste one: three arms built from one
    /// another, two of which say `Underline`. It would produce three ribbon
    /// controls that all draw an underline, and nothing else in the system would
    /// notice — the engine would author a perfectly valid annotation each time.
    #[test]
    fn each_kind_authors_its_own_subtype() {
        let expected = [
            (TextMarkKind::Underline, TextMarkupKind::Underline),
            (TextMarkKind::StrikeOut, TextMarkupKind::StrikeOut),
            (TextMarkKind::Squiggly, TextMarkupKind::Squiggly),
        ];
        for (kind, want) in expected {
            let MarkupSpec::TextMarkup { kind: got, .. } = spec(kind, vec![quad(700.0)], pen())
            else {
                panic!("{kind:?} must author a /QuadPoints text markup");
            };
            assert_eq!(got, want, "{kind:?}");
        }
        assert_eq!(
            TextMarkKind::ALL.len(),
            expected.len(),
            "a fourth kind must be given a subtype here, not left to inherit one"
        );
    }

    /// ★ **The quads are carried through untouched, in order and in number.**
    ///
    /// The one-derivation promise at this end of it: the boxes the operator saw
    /// washed are the boxes written into `/QuadPoints`. A build that merged,
    /// clipped, re-ordered or de-duplicated them here would mark a different set
    /// of glyphs from the one that was highlighted, and the difference would only
    /// be visible after saving.
    #[test]
    fn the_selections_quads_are_authored_unchanged() {
        let quads: Vec<Quad> = (0..4).map(|i| quad(700.0 - 12.0 * f64::from(i))).collect();
        let MarkupSpec::TextMarkup {
            quads: authored, ..
        } = spec(TextMarkKind::StrikeOut, quads.clone(), pen())
        else {
            panic!("a text mark must author a /QuadPoints text markup");
        };
        assert_eq!(authored, quads, "the boxes must arrive as they left");
    }

    /// ★★ **The operator's ink reaches every text kind** — the test that would
    /// have caught the defect this function was changed to fix.
    ///
    /// # What was here before, and why it passed through the whole bug
    ///
    /// This test used to be `the_pen_is_the_visible_one`, and it asserted the
    /// literal triple `(0.85, 0.16, 0.16)` against a function that returned the
    /// literal triple `(0.85, 0.16, 0.16)`. It was green for the entire life of
    /// the defect and would have stayed green forever, because **it and the code
    /// it tested were two copies of the same constant** — a test that restates
    /// its subject can only fail if someone edits one copy, which is the one
    /// thing nobody did.
    ///
    /// So the assertion is now a **relation, not a magnitude**: whatever colour
    /// the pen holds, that is the colour the spec authors. It is driven with a
    /// pen deliberately unlike the default in all three channels, so a build
    /// that went back to a hard-coded red fails on the first kind — and it would
    /// have failed at `4035b64`, which is the point of writing it this way.
    ///
    /// # Why it also checks the DEFAULT, in the same test
    ///
    /// Because the relation alone would be satisfied by a build that had
    /// silently changed what an untouched shell authors. Every annotation this
    /// project has ever written is `(0.85, 0.16, 0.16)`, and a change that
    /// quietly moved the default would alter the appearance of new marks in
    /// files sitting beside old ones. The second half pins that the *migration*
    /// was colour-preserving; the first half pins that the control now works.
    /// Neither implies the other.
    #[test]
    fn the_ink_reaches_every_text_kind() {
        // A pen unlike the default in all three channels, so no component can
        // agree by coincidence. Not the highlighter's yellow either — that is
        // the subject of the sibling assertion below.
        let chosen = crate::canvas::markup::pen::Pen {
            ink: (0.10, 0.35, 0.90),
            ..Default::default()
        };
        for &kind in TextMarkKind::ALL {
            let MarkupSpec::TextMarkup { color, .. } = spec(kind, vec![quad(700.0)], chosen) else {
                panic!("{kind:?} must author a /QuadPoints text markup");
            };
            let Color::Rgb(r, g, b) = color else {
                panic!("{kind:?} authored a non-RGB colour");
            };
            assert!(
                (r - chosen.ink.0).abs() < 1e-9
                    && (g - chosen.ink.1).abs() < 1e-9
                    && (b - chosen.ink.2).abs() < 1e-9,
                "{kind:?} ignored the operator's ink and authored ({r}, {g}, {b}) — the Markup ▸ \
                 Style swatch moves shapes and not text marks again"
            );

            // The default is colour-preserving: an operator who never touches
            // the swatch gets exactly what every earlier build authored.
            let MarkupSpec::TextMarkup { color, .. } = spec(
                kind,
                vec![quad(700.0)],
                crate::canvas::markup::pen::Pen::default(),
            ) else {
                panic!("{kind:?} must author a /QuadPoints text markup");
            };
            let Color::Rgb(r, g, b) = color else {
                panic!("{kind:?} authored a non-RGB colour");
            };
            assert!(
                (r - 0.85).abs() < 1e-9 && (g - 0.16).abs() < 1e-9 && (b - 0.16).abs() < 1e-9,
                "{kind:?} changed what an untouched shell authors: ({r}, {g}, {b})"
            );
        }
    }

    /// ★ **No text kind may reach the highlighter**, whatever the pen holds.
    ///
    /// [`TextMarkKind::rgb`]'s partition, asserted from this side: these three
    /// are lines and take the ink; Highlight is a wash, takes the highlighter,
    /// and is not in this enum. The failure it catches is a plausible one — a
    /// future hand "simplifying" `rgb` to `pen.colour_for(kind.into())` would
    /// route all three to whichever colour that mapping picked, and with the
    /// default pen that is **yellow**: a yellow underline under black glyphs on
    /// white paper marks nothing an operator can see, which is the one failure a
    /// mark whose entire job is to be noticed cannot afford.
    ///
    /// Driven with a highlighter that is *not* the default yellow, so the
    /// assertion catches the wiring rather than the hue.
    #[test]
    fn no_text_kind_takes_the_highlighter() {
        let chosen = crate::canvas::markup::pen::Pen {
            ink: (0.10, 0.35, 0.90),
            highlighter: (0.95, 0.90, 0.05),
            ..Default::default()
        };
        for &kind in TextMarkKind::ALL {
            let MarkupSpec::TextMarkup { color, .. } = spec(kind, vec![quad(700.0)], chosen) else {
                panic!("{kind:?} must author a /QuadPoints text markup");
            };
            let Color::Rgb(r, g, b) = color else {
                panic!("{kind:?} authored a non-RGB colour");
            };
            assert!(
                (r - chosen.highlighter.0).abs() > 1e-9
                    || (g - chosen.highlighter.1).abs() > 1e-9
                    || (b - chosen.highlighter.2).abs() > 1e-9,
                "{kind:?} took the highlighter — a wash colour on a line"
            );
        }
    }

    // -----------------------------------------------------------------
    // ★ The rule: what a command does with the selection it finds
    // -----------------------------------------------------------------

    /// ★ **A live selection becomes exactly one action, on ITS page.**
    ///
    /// The page assertion is the load-bearing half and it is written as a
    /// magnitude rather than a relation: the action must name page **7**, the
    /// page the selection was made on, not "a page". A build that read
    /// `doc.view.page_index` in the apply arm would author the mark on whatever
    /// sheet was on screen — the same class of defect as the markup that landed
    /// in the centre of the page, one axis over.
    #[test]
    fn a_live_selection_marks_its_own_page() {
        let sel = selection(7, 3, 2);
        let raised =
            mark(TextMarkKind::Underline, Some(&sel), 3, pen()).expect("a live selection marks");
        let Action::CommitTextMarkup {
            page, kind, quads, ..
        } = raised
        else {
            panic!("a text-markup command must raise CommitTextMarkup: {raised:?}");
        };
        assert_eq!(page, 7, "the selection's page, not the visible one");
        assert_eq!(kind, TextMarkKind::Underline);
        assert_eq!(quads.len(), 2, "one quad per line of the selection");
        assert_eq!(quads, sel.page_quads, "the selection's own boxes");
    }

    /// ★ **A selection made before an edit is refused, not marked.**
    ///
    /// `canvas::textsel` §7's rule at the authoring end: after an edit the
    /// recorded positions may name different glyphs, and writing a `/QuadPoints`
    /// annotation from them would put a mark over possibly-wrong words *into the
    /// file*. Distinguished from [`Refusal::NoSelection`] on the trace, because
    /// the two have different answers — sweep again, versus sweep at all.
    #[test]
    fn a_stale_selection_is_refused_and_says_so() {
        let sel = selection(0, 4, 1);
        assert_eq!(
            mark(TextMarkKind::Squiggly, Some(&sel), 5, pen()),
            Err(Refusal::Stale),
            "one edit later, the boxes may be over other glyphs"
        );
        assert_eq!(
            mark(TextMarkKind::Squiggly, None, 5, pen()),
            Err(Refusal::NoSelection),
            "…and no selection at all is a different fact with a different answer"
        );
    }

    /// A selection carrying no boxes authors nothing rather than handing the
    /// engine geometry that draws nothing.
    ///
    /// Structurally unreachable through `textsel::resolve`, which answers `None`
    /// instead — and guarded anyway, for the reason the geometric kinds guard
    /// their degenerate drag: the shell never sends the engine an empty
    /// `/QuadPoints`, so `validate_geometry` never has to refuse one and the
    /// operator never sees an engine error for a shell decision.
    #[test]
    fn a_selection_with_no_boxes_authors_nothing() {
        let empty = TextSelection::for_test(0, 1, Vec::new());
        assert_eq!(
            mark(TextMarkKind::Underline, Some(&empty), 1, pen()),
            Err(Refusal::NoQuads)
        );
    }

    /// ★ **Every kind behaves identically at the rule level.**
    ///
    /// Asserted over `ALL` rather than for one kind, because the plausible
    /// failure is per-kind: a fourth entry added to the enum, given a subtype and
    /// a command, and reaching a `mark` that quietly special-cases the three that
    /// were there first.
    #[test]
    fn every_kind_marks_and_refuses_alike() {
        let live = selection(2, 9, 3);
        for &kind in TextMarkKind::ALL {
            let raised = mark(kind, Some(&live), 9, pen()).unwrap_or_else(|e| {
                panic!("{kind:?} refused a live selection: {e:?}");
            });
            assert!(
                matches!(raised, Action::CommitTextMarkup { kind: k, .. } if k == kind),
                "{kind:?} raised {raised:?}"
            );
            assert_eq!(
                mark(kind, None, 9, pen()),
                Err(Refusal::NoSelection),
                "{kind:?}"
            );
            assert_eq!(
                mark(kind, Some(&live), 10, pen()),
                Err(Refusal::Stale),
                "{kind:?}"
            );
        }
    }
}
