//! # `canvas::textannot` — the three markup kinds that carry WORDS
//!
//! Text box, sticky note and stamp. The operator asked for the revisioning set
//! to be finished on 2026-08-18, and these are the three that were registered,
//! drawn on the Markup tab, and had no dispatch arm for the whole life of the
//! project.
//!
//! ## ★ Why they were left out, and why that was right at the time
//!
//! `shell::commands::reach`'s register carries the reason verbatim, quoting
//! `canvas::markup`'s own table of kinds it deliberately does not handle:
//!
//! > *Note · text box · sticky · stamp — Text-bearing, not geometric. A
//! > different gesture (place, then type) and a different spec type
//! > (`TextAnnotSpec`).*
//!
//! Both halves are true and neither is small. **Nothing about the
//! drag-and-release machinery the seven geometric kinds share applies here**:
//! those author on release, from geometry alone, with a pen. These cannot —
//! releasing the mouse produces an *empty box*, and an empty box is not an
//! annotation, it is a rectangle nobody asked for.
//!
//! ## The gesture: place, then type, then commit
//!
//! | kind | placing gesture | why |
//! |---|---|---|
//! | [`TextAnnotKind::TextBox`] | **drag a rectangle** | a `/FreeText` is painted *into* its rect and wraps to it, so the operator is choosing how wide the text is. A click would have to invent a width |
//! | [`TextAnnotKind::Sticky`] | **one click** | a `/Text` marker is fixed-size and `NoZoom` — its rect's width and height do not affect what is drawn, so asking the operator to drag one would be asking for a number that is discarded |
//! | [`TextAnnotKind::Stamp`] | **drag a rectangle** | pdfcer's stamp appearance is a framed label scaled into its rect, so the drag is choosing how big the stamp is |
//!
//! Then the dialog opens, and **nothing is authored until Accept**. That is
//! rule 4 applied to a gesture whose output is words: a half-typed note
//! committed on a stray click would be content the operator did not write.
//!
//! ## ★ Escape has two meanings here and they are ordered
//!
//! A placing drag in flight is abandoned by Escape, exactly as a markup band
//! is — that rung already exists and this kind rides it. Escape with the
//! **dialog** open is the dialog's, and closes it without authoring.
//!
//! The two cannot both be live: the dialog only opens once the drag is over.
//! Stating it because the ordering is the kind of thing that looks obvious
//! until a third claimant is added to `canvas::keys`' ladder.

use pdfcer_core::annot_author::{Color, StampName, StickyIcon, TextAnnotSpec};
use pdfcer_core::fontdata::Std14;
use pdfcer_core::page_tree::Rect;
use pdfcer_core::vartext::{Quadding, TextColor};

/// Which text-bearing annotation is being placed.
///
/// # ★ One enum carrying three kinds, not three tools
///
/// The same argument `MarkupKind` and `MeasureKind` both make, and for the
/// third time it is a statement about types rather than about tidiness: the
/// operator is placing exactly one annotation, so a type that could say
/// *text box* and *sticky* at once — which three booleans, or three tool
/// variants plus a "which is active" rule, both can — is a type whose illegal
/// states are prevented by discipline instead of by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAnnotKind {
    /// `/FreeText` — words painted onto the page, inside a box the operator
    /// drags. The callout of a revision markup set.
    #[default]
    TextBox,
    /// `/Text` — a sticky note: a marker on the page whose words live in a
    /// popup and are never painted.
    Sticky,
    /// `/Stamp` — a framed label: APPROVED, REVISED, and the rest.
    Stamp,
}

impl TextAnnotKind {
    /// Every kind, in the order the Markup tab offers them.
    pub const ALL: &'static [Self] = &[Self::TextBox, Self::Sticky, Self::Stamp];

    /// The command id that arms this kind.
    #[must_use]
    pub const fn command(self) -> &'static str {
        match self {
            // ui-text-exempt: command ids, never displayed
            Self::TextBox => "markup.text_box",
            // ui-text-exempt: command ids, never displayed
            Self::Sticky => "markup.sticky_note",
            // ui-text-exempt: command ids, never displayed
            Self::Stamp => "markup.stamp",
        }
    }

    /// The kind `id` arms, or `None` if it names none.
    ///
    /// Derived from [`Self::command`] rather than written out a second time,
    /// exactly as `markup_for_command` and `measure_for_command` are — so the
    /// two directions cannot disagree even in principle.
    #[must_use]
    pub fn from_command(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|k| k.command() == id)
    }

    /// Whether placing this kind is a **drag** (rather than a single click).
    ///
    /// See the module header's table. The sticky note is the exception and the
    /// reason is that its rect is not read: a `/Text` marker is fixed-size and
    /// `NoZoom`, so a dragged width would be a number the operator chose and
    /// the format discards.
    #[must_use]
    pub const fn is_dragged(self) -> bool {
        match self {
            Self::TextBox | Self::Stamp => true,
            Self::Sticky => false,
        }
    }

    /// Whether this kind's text comes from a **gallery** rather than free
    /// typing.
    ///
    /// Only the stamp. `manifest/markup.rs` recorded the blocker as *"the
    /// stamp control exists and needs a GALLERY … a stamp with no chooser has
    /// no operand"*, and that is what this predicate drives: the dialog offers
    /// [`STAMP_LABELS`] instead of an empty field.
    #[must_use]
    pub const fn uses_gallery(self) -> bool {
        matches!(self, Self::Stamp)
    }
}

/// The stamps offered, in the order the gallery lists them.
///
/// # ★ The ENGINE's names, not a list of my own
///
/// The first draft of this module invented seven upper-case strings —
/// `APPROVED`, `REVISED`, `VOID` and so on — and it was wrong in a way worth
/// recording, because it looked entirely reasonable. `TextAnnotSpec::Stamp`
/// does not take a free label: it takes a [`StampName`], which is **ISO
/// 32000-1 Table 181's standard stamp set**, plus an *optional* label that
/// overrides the name's default text.
///
/// Inventing strings would have authored `/Name /Draft` — the enum's default —
/// on every stamp regardless of what it said, so a reader other than pdfcer
/// would show *Draft* under a stamp reading `APPROVED`. The annotation would
/// have disagreed with its own appearance, which is the quietest possible way
/// to be wrong about a document.
///
/// # Which of the fourteen, and why not all of them
///
/// The engine offers fourteen. These are the ones a **drawing revision**
/// workflow uses; the rest (`TopSecret`, `Sold`, `Departmental`,
/// `NotForPublicRelease`) belong to document control rather than to drafting,
/// and a gallery of fourteen is a list an operator scans instead of a set they
/// know. Adding one is a line here — the constraint is the enum, not this
/// list.
pub const STAMPS: &[StampName] = &[
    StampName::Approved,
    StampName::NotApproved,
    StampName::Draft,
    StampName::Final,
    StampName::ForComment,
    StampName::AsIs,
    StampName::Expired,
];

/// The stamp a fresh gallery offers.
///
/// `Approved` rather than the engine's `Draft` default, and the difference is
/// deliberate: `Draft` is the right default for a *format* that must pick
/// something, and the wrong one for an *operator* who has just pressed a stamp
/// control on a drawing they are reviewing. The commonest first stamp in a
/// review is the one that says the review passed.
pub const DEFAULT_STAMP: StampName = StampName::Approved;

/// The side, in PDF points, of the square a sticky note's rect is given.
///
/// # ★ It is not a size the operator sees
///
/// A `/Text` annotation's marker is drawn at a **fixed size** and carries
/// `NoZoom`/`NoRotate`, so the reader paints the same icon however big the
/// rect is. `TextAnnotSpec::Sticky`'s own documentation says as much: *"the
/// marker is fixed-size … so only its lower-left corner matters in practice."*
///
/// So this number decides nothing about the picture. What it must be is
/// **non-degenerate** — a zero-area rect is refused by the engine's geometry
/// validation and would turn a placed note into a silent refusal — and roughly
/// icon-sized, so that anything reading the rect for a hit test or a bounding
/// box gets an answer near the truth rather than a point.
///
/// 20 pt is about the size Acrobat draws its note icon at, which makes it the
/// least surprising answer to a question the format says is not being asked.
pub const STICKY_PT: f64 = 20.0;

/// The longest note or caption offered.
///
/// # ★ It bounds the FIELD, not the format
///
/// `/Contents` is a PDF string and has no length worth naming. What is bounded
/// is what an operator can usefully put on a drawing: a `/FreeText` is painted
/// into a box the operator dragged, and at some length it either shrinks below
/// legibility or is clipped. 512 characters is a long paragraph — comfortably
/// more than any callout on a drawing sheet — and short enough that the
/// operator meets the bound while typing rather than in the saved file.
pub const MAX_TEXT_CHARS: usize = 512;

/// The point size a text box is authored at.
///
/// **Not zero.** `TextAnnotSpec::FreeText` documents `0.0` as auto-size, and
/// auto-size is the right default for a box whose content is unknown — but it
/// is the engine's heuristic rather than the operator's choice, and this shell
/// has no surface to override it. 11 pt is a legible caption on a drawing
/// sheet at the sizes this shell is for, and choosing it explicitly means the
/// operator gets the same size on every sheet rather than one that varies with
/// how big a box they happened to drag.
pub const TEXT_SIZE_PT: f64 = 11.0;

/// Build the engine spec for a placed, typed annotation.
///
/// # ★ Pure, and separate from the action arm for the standing reason
///
/// It is the part that could be wrong in a way an operator would notice — a
/// stamp authored with the wrong quadding, a sticky whose words went into the
/// wrong field — and a `&mut EditSession` is not available to a test that only
/// wants to ask what was built. Every geometry rule in this crate is split
/// this way.
///
/// Returns `None` for an empty text, which is the one refusal this function
/// makes: an annotation carrying no words is not a thing the operator asked
/// for, and authoring one would put an empty box on their drawing that they
/// then have to find and delete.
#[must_use]
pub fn spec(
    kind: TextAnnotKind,
    rect: Rect,
    text: &str,
    stamp: StampName,
    colour: (f64, f64, f64),
) -> Option<TextAnnotSpec> {
    let text = text.trim();
    // ★ The blank refusal applies to the two kinds whose words the OPERATOR
    // types, and not to the stamp, whose words come from its `/Name`.
    // Refusing a blank stamp would refuse every stamp, since the gallery
    // supplies no text at all.
    if text.is_empty() && !kind.uses_gallery() {
        return None;
    }
    let (r, g, b) = colour;
    Some(match kind {
        TextAnnotKind::TextBox => TextAnnotSpec::FreeText {
            rect,
            text: text.to_owned(),
            // Helvetica: the face every reader has and the one a drawing
            // callout is set in. `vartext` refuses symbolic faces, so this is
            // also the safe end of what the engine will author.
            font: Std14::Helvetica,
            font_size: TEXT_SIZE_PT,
            color: TextColor::Rgb(r, g, b),
            // Left, because a callout is read as prose and prose is
            // left-aligned. Centring is a stamp's property, not a note's.
            quadding: Quadding::Left,
            // ★ Multiline. A callout that did not wrap would put the
            // operator's second sentence outside the box they drew, which is
            // the same class of defect as a control laid out below its pane.
            multiline: true,
            // A border, unlike Acrobat's borderless default. On a drawing
            // sheet a borderless caption is indistinguishable from the
            // drawing's own annotation, and a revision markup must read as
            // something added.
            border: Some(Color::Rgb(r, g, b)),
            border_width: 1.0,
        },
        // ★★★ **THE ICON IS HARDCODED, and as of 2026-09-05 that is a
        // measured gap rather than an unexamined default.**
        //
        // §12.5.6.4 Table 172 defines **seven** — `/Comment`, `/Key`,
        // `/Note`, `/Help`, `/NewParagraph`, `/Paragraph`, `/Insert` — and
        // `pdfcer_core::annot_author::StickyIcon` has modelled all seven since
        // sticky notes shipped (`annot_author.rs:2311-2327`), written straight
        // into `/Name` at `annot_author.rs:3206`. Acrobat offers the same
        // seven on its note tool. **This shell has never asked for any but the
        // default.**
        //
        // # Why it is still hardcoded after being found
        //
        // Not an engine gap — the capability is there and free. It is blocked
        // on **where the operator's choice would travel**, and all three
        // routes are outside this work's reach:
        //
        // 1. a field on `Action::CommitTextAnnot` — `app/actions/action.rs`
        //    sits at **exactly 1,500 lines**, R2's ceiling, so a field with
        //    its doc comment means splitting a file seven concurrent tracks
        //    are editing;
        // 2. a payload on `TextAnnotKind` — it is also the **tool identity**
        //    (`canvas::tool::CanvasTool::TextAnnot`), so two stickies with
        //    different icons would become two different armed tools;
        // 3. a field on the markup `Pen` — the natural home, since the pen
        //    already carries the ink and the opacity this same call reads live
        //    — but the control for it belongs on Markup ▸ Style, and the
        //    ribbon manifest is a concurrent track's.
        //
        // ⇒ Route 3 is the right one and it is one field plus one ribbon
        // control. Recorded here, in the code, rather than in a document
        // nobody re-reads — and recorded as a **shell** gap so nobody files it
        // at the engine, which has already done its half.
        //
        // R9 is satisfied meanwhile: no chooser is drawn, no greyed control,
        // no placeholder. The note gets `/Note`, which is what every sticky
        // this program has ever authored carries.
        //
        // ---------------------------------------------------------------
        // ★★ **RE-MEASURED 2026-09-06: route 1's blocker is GONE, and route
        // 3 is no longer the recommendation.**
        //
        // The paragraph above is kept verbatim rather than rewritten, because
        // what changed is a *measurement* and the correction is the useful
        // part. `wc -l app/actions/action.rs` is **1,479** — twenty-one lines
        // of R2 headroom, where the note above recorded exactly 1,500. A
        // concurrent track split that file after this was written.
        //
        // ⇒ Route 1 now costs a field and its doc comment, and it is the
        // RIGHT route, not merely the newly-affordable one. The reason is the
        // one the stamp already proves: **`StampName` travels this exact path
        // today.** `dialogs/textannot.rs:77` holds the gallery's choice,
        // `:262` puts it on `Action::CommitTextAnnot`, `action.rs:1367`
        // carries it, `app/actions/textannot.rs:63` lands it on `Placement`,
        // and `:148` hands it to this function as the `stamp` argument. An
        // icon is the same shape of operand as a stamp name and belongs in
        // the same carrier — route 3 would put two answers to *"what did the
        // operator pick?"* in two different places, one on the pen and one on
        // the action.
        //
        // # The whole remaining change, so it is one sitting for whoever owns
        // # these files
        //
        // 1. `canvas/textannot.rs` (here): `STICKY_ICONS` + `DEFAULT_STICKY_ICON`
        //    beside `STAMPS`/`DEFAULT_STAMP`, and an `icon: StickyIcon`
        //    parameter on `spec` replacing `StickyIcon::default()` below;
        // 2. `text/textannot.rs`: `sticky_icon_label`, beside `stamp_label`;
        // 3. `app/actions/action.rs`: one field on `CommitTextAnnot`;
        // 4. `app/actions/textannot.rs`: one field on `Placement`, threaded
        //    at `apply.rs:751`;
        // 5. `dialogs/textannot.rs`: the radio group at `:399-405` again, over
        //    `STICKY_ICONS`, gated on `Sticky` as the stamp's is on `Stamp`.
        //
        // **Not done here, and the reason is ownership rather than
        // difficulty.** This was measured by the 2026-09-06 note-editing track,
        // which owns items 1 and 2 and neither of items 3 and 5;
        // `dialogs/textannot.rs` was outside its grant entirely, and the tree
        // was red from six concurrent tracks at the time. Landing a
        // five-file signature change across three other tracks' files while
        // the build is broken is how a reconciliation becomes expensive.
        //
        // ★ R9 still holds and that has not changed: nothing is drawn. The
        // seven icons are `pdfcer_core::annot_author::StickyIcon`'s, which is
        // §12.5.6.4 Table 172's complete set and exactly the seven Acrobat's
        // note tool offers — so the gallery needs no list of ours, the way
        // `STAMPS` needed one.
        //
        // The **editing** half of this — changing a placed note's icon or
        // colour — is the engine's and is filed:
        // `request_a_sticky_notes_icon_and_colour_cannot_be_changed.md`,
        // re-measured against v0.42.0 on 2026-09-06 and **still unanswered**
        // (no `/C` read in `annot.rs`, no icon read, no set verb).
        TextAnnotKind::Sticky => TextAnnotSpec::Sticky {
            rect,
            icon: StickyIcon::default(),
            contents: text.to_owned(),
            color: Color::Rgb(r, g, b),
            // Closed, and ★ the REASON changed on 2026-09-05 even though the
            // value did not.
            //
            // It used to read: *"a popup that opened itself on every sticky
            // would cover the drawing the note is about — and
            // `MODES_AND_PANELS.md`'s nothing-floats-over-the-canvas stance is
            // only relaxed for Find."* The first half stands. The second half
            // is now out of date: `crate::canvas::notepopup` floats a window
            // over the canvas, deliberately, and its header carries the
            // argument — a pop-up is **chrome**, the same class of thing as a
            // selection handle, and nothing about it reaches the page.
            //
            // So the value survives on the first half alone, which is the
            // stronger half anyway: pdfcer collects the note's words in a
            // dialog **before** authoring, so by the time the annotation
            // exists the operator has already read and typed what it says. A
            // window opening to show them their own sentence back would be
            // covering the drawing to tell them nothing. Acrobat authors a
            // sticky open because Acrobat has no dialog — the pop-up *is* the
            // text field — and copying the value without the mechanism would
            // be copying the wrong half.
            open: false,
        },
        // ★ `label: None` — the NAME carries the text.
        //
        // `TextAnnotSpec::Stamp` takes both, and passing a label here would
        // override the name's own default text. That is a real capability (a
        // stamp reading something the standard set does not offer) and it is
        // deliberately not used: a stamp whose `/Name` and whose painted words
        // disagree is a document that says two things, and a reader other than
        // pdfcer shows the name.
        TextAnnotKind::Stamp => TextAnnotSpec::Stamp {
            rect,
            name: stamp,
            label: None,
            color: Color::Rgb(r, g, b),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> Rect {
        Rect {
            llx: 100.0,
            lly: 100.0,
            urx: 300.0,
            ury: 160.0,
        }
    }

    /// ★ Every kind maps to the engine variant it names.
    ///
    /// The failure this catches is a copy-paste between arms — a sticky
    /// authored as a `FreeText` would paint the operator's private note onto
    /// the page, which is the opposite of what a sticky is for and is not
    /// recoverable by anything but noticing.
    #[test]
    fn every_kind_authors_the_annotation_it_names() {
        let colour = (0.85, 0.16, 0.16);
        assert!(matches!(
            spec(
                TextAnnotKind::TextBox,
                rect(),
                "note",
                DEFAULT_STAMP,
                colour
            ),
            Some(TextAnnotSpec::FreeText { .. })
        ));
        assert!(matches!(
            spec(TextAnnotKind::Sticky, rect(), "note", DEFAULT_STAMP, colour),
            Some(TextAnnotSpec::Sticky { .. })
        ));
        assert!(matches!(
            spec(TextAnnotKind::Stamp, rect(), "", DEFAULT_STAMP, colour),
            Some(TextAnnotSpec::Stamp { .. })
        ));
    }

    /// ★ **An empty or blank text authors nothing.**
    ///
    /// The one refusal this module makes, and it is worth a test rather than a
    /// comment: an annotation with no words is an empty box on the operator's
    /// drawing that they then have to find and delete. Whitespace counts as
    /// empty, or a stray space bar would author one.
    #[test]
    fn a_blank_text_authors_nothing() {
        for blank in ["", "   ", "\t\n "] {
            // ★ The gallery kind is excluded, and NOT by a hard-coded
            // `!= Stamp`. It is excluded by the same predicate the production
            // code branches on, so the exception cannot drift: if a second
            // kind ever takes its words from a gallery this test follows it
            // without an edit, and if the stamp stops using one it is covered
            // here immediately.
            //
            // `a_stamp_authors_without_typed_text` asserts the other side, so
            // the exception is tested rather than merely skipped.
            for kind in TextAnnotKind::ALL.iter().filter(|k| !k.uses_gallery()) {
                assert!(
                    spec(*kind, rect(), blank, DEFAULT_STAMP, (0.0, 0.0, 0.0)).is_none(),
                    "{kind:?} authored an annotation for {blank:?}"
                );
            }
        }
    }

    /// The text is trimmed before it reaches the file.
    ///
    /// Leading space in a `/FreeText` is painted, so an operator who typed a
    /// space before their sentence would get an indented callout they did not
    /// ask for and cannot see the cause of.
    #[test]
    fn the_text_is_trimmed() {
        let Some(TextAnnotSpec::FreeText { text, .. }) = spec(
            TextAnnotKind::TextBox,
            rect(),
            "  hello  ",
            DEFAULT_STAMP,
            (0.0, 0.0, 0.0),
        ) else {
            panic!("a text box with words must author");
        };
        assert_eq!(text, "hello");
    }

    /// ★ A text box wraps, and a sticky's words are never painted.
    ///
    /// The two properties that make each kind the thing it is. A `/FreeText`
    /// that did not wrap puts the operator's second sentence outside the box
    /// they drew; a sticky whose contents were painted would publish a private
    /// note onto the drawing.
    #[test]
    fn the_two_defining_properties_hold() {
        let Some(TextAnnotSpec::FreeText { multiline, .. }) = spec(
            TextAnnotKind::TextBox,
            rect(),
            "a",
            DEFAULT_STAMP,
            (0.0, 0.0, 0.0),
        ) else {
            panic!("a text box must author");
        };
        assert!(multiline, "a callout that cannot wrap is a clipped callout");

        let Some(TextAnnotSpec::Sticky { open, .. }) = spec(
            TextAnnotKind::Sticky,
            rect(),
            "a",
            DEFAULT_STAMP,
            (0.0, 0.0, 0.0),
        ) else {
            panic!("a sticky must author");
        };
        assert!(
            !open,
            "a popup that opens itself covers the drawing the note is about"
        );
    }

    /// Every kind's command round-trips, and no two share an id.
    #[test]
    fn every_kind_has_a_distinct_command() {
        for k in TextAnnotKind::ALL {
            assert_eq!(TextAnnotKind::from_command(k.command()), Some(*k));
        }
        let ids: Vec<&str> = TextAnnotKind::ALL.iter().map(|k| k.command()).collect();
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                assert_ne!(ids[i], ids[j]);
            }
        }
        assert!(TextAnnotKind::from_command("markup.rectangle").is_none());
    }

    /// Exactly one kind is placed by a click, and exactly one uses a gallery.
    ///
    /// Both are properties of the FORMAT rather than of taste — a `/Text`
    /// marker discards its rect, and a stamp with a free-text label is a text
    /// box with a border. A second kind acquiring either predicate would mean
    /// one of those two facts had changed, which is worth failing over.
    #[test]
    fn the_two_odd_ones_out_are_the_ones_expected() {
        let clicked: Vec<_> = TextAnnotKind::ALL
            .iter()
            .filter(|k| !k.is_dragged())
            .copied()
            .collect();
        assert_eq!(clicked, vec![TextAnnotKind::Sticky]);
        let gallery: Vec<_> = TextAnnotKind::ALL
            .iter()
            .filter(|k| k.uses_gallery())
            .copied()
            .collect();
        assert_eq!(gallery, vec![TextAnnotKind::Stamp]);
    }

    /// The stamp gallery is non-empty, distinct, and its default is in it.
    ///
    /// A default outside its own list would leave the dialog opening on a
    /// value no control can select — the same defect the settings window's
    /// range checks exist for.
    #[test]
    fn the_stamp_gallery_is_usable() {
        assert!(!STAMPS.is_empty());
        assert!(STAMPS.contains(&DEFAULT_STAMP));
        // Distinctness by pairwise comparison over ITERATORS rather than over
        // indices. `StampName` is `PartialEq` and not `Hash`, so the set trick
        // is unavailable — and an index loop is what clippy objects to, with
        // reason: it is the shape that goes out of bounds when someone edits
        // the range.
        for (i, a) in STAMPS.iter().enumerate() {
            for b in STAMPS.iter().skip(i + 1) {
                assert_ne!(
                    a, b,
                    "a stamp appears twice in the gallery, so one entry is unreachable"
                );
            }
        }
    }

    /// ★ **A stamp authors the NAME the operator chose.**
    ///
    /// The regression test for the mistake this module made in its first
    /// draft: inventing label strings and leaving `/Name` at the enum's
    /// `Draft` default, so a stamp reading APPROVED would carry `/Name /Draft`
    /// and any reader but pdfcer would show *Draft*. An annotation that
    /// disagrees with its own appearance is the quietest way to be wrong about
    /// a document.
    ///
    /// `label: None` is asserted with it, because a label and a name that both
    /// carry text is the same disagreement arrived at from the other side.
    #[test]
    fn a_stamp_authors_the_chosen_name_and_no_competing_label() {
        for chosen in STAMPS {
            let Some(TextAnnotSpec::Stamp { name, label, .. }) =
                spec(TextAnnotKind::Stamp, rect(), "", *chosen, (0.0, 0.0, 0.0))
            else {
                panic!("{chosen:?} must author");
            };
            assert_eq!(name, *chosen, "the stamp authored a different name");
            assert!(
                label.is_none(),
                "a label beside the name is a document that says two things"
            );
        }
    }

    /// …and a stamp is the one kind a blank text does not refuse.
    ///
    /// Its words come from its `/Name`, so the blank guard that protects the
    /// other two would refuse every stamp there is.
    #[test]
    fn a_stamp_authors_without_typed_text() {
        assert!(
            spec(
                TextAnnotKind::Stamp,
                rect(),
                "",
                DEFAULT_STAMP,
                (0.0, 0.0, 0.0)
            )
            .is_some(),
            "the gallery supplies no typed text, so requiring some refuses every stamp"
        );
    }
}
