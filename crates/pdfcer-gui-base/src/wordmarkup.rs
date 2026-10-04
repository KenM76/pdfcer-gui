//! # `wordmarkup` — the three markup kinds that carry WORDS
//!
//!
//! ## Why they were left out, and why that was right at the time
//!
//! `shell::commands::reach`'s register carries the reason verbatim, quoting
//! `canvas::markup`'s own table of kinds it deliberately does not handle:
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/wordmarkup.md`.

use pdfcer_core::annot_author::{
    AttachmentIcon, Color, MediaTempAccess, ScreenTrigger, SoundIcon, StampName, StampStyle,
    StickyIcon, TextAnnotSpec,
};
use pdfcer_core::fontdata::Std14;
use pdfcer_core::page_tree::Rect;
use pdfcer_core::vartext::{Quadding, TextColor};

/// Which text-bearing annotation is being placed.
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
    /// `/FileAttachment` — a marker carrying a file embedded in the PDF.
    /// Its text is an optional description; the file is the payload, so it
    /// never reaches [`spec`].
    Attachment,
    /// `/Caret` — a proofreading mark where words are to be inserted; the
    /// words travel as its note. Its own verb authors it, so it never
    /// reaches [`spec`].
    Caret,
    /// `/Sound` — a speaker or microphone icon carrying a recorded clip.
    /// Its text is an optional description; the clip is the payload, so it
    /// never reaches [`spec`].
    Sound,
    /// `/Screen` — a dragged region that plays an embedded video or audio
    /// clip. Its text is an optional description; the clip is the payload,
    /// so it never reaches [`spec`].
    Screen,
}

impl TextAnnotKind {
    /// Every kind, in the order the Markup tab offers them.
    pub const ALL: &'static [Self] = &[
        Self::TextBox,
        Self::Sticky,
        Self::Stamp,
        Self::Attachment,
        Self::Caret,
        Self::Sound,
        Self::Screen,
    ];

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
            // ui-text-exempt: command ids, never displayed
            Self::Attachment => "markup.attach_file",
            // ui-text-exempt: command ids, never displayed
            Self::Caret => "markup.insert_text",
            // ui-text-exempt: command ids, never displayed
            Self::Sound => "markup.sound",
            // ui-text-exempt: command ids, never displayed
            Self::Screen => "markup.screen",
        }
    }

    /// The kind `id` arms, or `None` if it names none.
    #[must_use]
    pub fn from_command(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|k| k.command() == id)
    }

    /// Whether placing this kind is a **drag** (rather than a single click).
    #[must_use]
    pub const fn is_dragged(self) -> bool {
        match self {
            Self::TextBox | Self::Stamp | Self::Screen => true,
            Self::Sticky | Self::Attachment | Self::Caret | Self::Sound => false,
        }
    }

    /// Whether this kind's text comes from a **gallery** rather than free
    /// typing.
    #[must_use]
    pub const fn uses_gallery(self) -> bool {
        matches!(self, Self::Stamp)
    }
}

/// The stamps offered, in the order the gallery lists them.
pub const STAMPS: &[StampName] = &[
    StampName::Approved,
    StampName::NotApproved,
    StampName::Draft,
    StampName::Final,
    StampName::ForComment,
    StampName::AsIs,
    StampName::Expired,
];

/// The sticky-note icons offered, in the order the dialog lists them.
pub const STICKY_ICONS: &[StickyIcon] = &[
    StickyIcon::Comment,
    StickyIcon::Key,
    StickyIcon::Note,
    StickyIcon::Help,
    StickyIcon::NewParagraph,
    StickyIcon::Paragraph,
    StickyIcon::Insert,
];

/// The file-attachment icons offered, in the order the dialog lists them
/// (§12.5.6.15 Table 184).
pub const ATTACHMENT_ICONS: &[AttachmentIcon] = &[
    AttachmentIcon::PushPin,
    AttachmentIcon::Paperclip,
    AttachmentIcon::Graph,
    AttachmentIcon::Tag,
];

/// The sound icons offered, in the order the dialog lists them
/// (§12.5.6.16 Table 185).
pub const SOUND_ICONS: &[SoundIcon] = &[SoundIcon::Speaker, SoundIcon::Mic];

/// What can start a media clip, in the order the dialog lists them.
pub const SCREEN_TRIGGERS: &[ScreenTrigger] = &[ScreenTrigger::Click, ScreenTrigger::PageOpen];

/// The temporary-file permissions offered, in the order the dialog lists
/// them (§13.2.4.3 Table 275).
pub const TEMP_ACCESS: &[MediaTempAccess] = &[
    MediaTempAccess::Never,
    MediaTempAccess::Access,
    MediaTempAccess::Always,
];

/// The icon a fresh sound comment carries: the standard's default.
pub const DEFAULT_SOUND_ICON: SoundIcon = SoundIcon::Speaker;

/// The icon a fresh file attachment carries: the standard's default.
pub const DEFAULT_ATTACHMENT_ICON: AttachmentIcon = AttachmentIcon::PushPin;

/// The icon a fresh sticky note carries.
pub const DEFAULT_STICKY_ICON: StickyIcon = StickyIcon::Comment;

/// The stamp a fresh gallery offers.
pub const DEFAULT_STAMP: StampName = StampName::Approved;

/// **How big a stamp's label is** — the operator's half of engine `Pass 287.0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StampSize {
    /// Size the label from the box the operator drew, as every build before
    /// `Pass 287.0` did — and widen the box if the word still does not fit.
    ///
    /// **Not the same as the engine's `StampStyle::legacy_derived()`**, and
    /// the difference is the whole point. That constructor pairs the derived
    /// size with `StampFit::ClipToBox`, which is the pre-Pass behaviour
    /// *including* the clipping the operator reported. This pairs the derived
    /// size with `GrowToText`, so the drag still chooses the size and the word
    /// is never cut off. It is the old behaviour with the defect removed,
    /// rather than the old behaviour reproduced.
    #[default]
    FitTheBox,
    /// A stated size in points; the box grows if the label is wider.
    Points(u32),
}

/// The label sizes the dialog offers, in the order it lists them.
pub const STAMP_SIZES: &[StampSize] = &[
    StampSize::FitTheBox,
    StampSize::Points(8),
    StampSize::Points(10),
    StampSize::Points(12),
    StampSize::Points(14),
    StampSize::Points(18),
    StampSize::Points(24),
    StampSize::Points(36),
    StampSize::Points(48),
    StampSize::Points(72),
];

/// The label size a fresh gallery offers. See [`StampSize`]'s header for why
/// this is the derived size and not the engine's 12 pt.
pub const DEFAULT_STAMP_SIZE: StampSize = StampSize::FitTheBox;

impl StampSize {
    /// The engine style this asks for.
    #[must_use]
    pub fn style(self) -> StampStyle {
        match self {
            Self::FitTheBox => StampStyle::default().with_font_size(None),
            Self::Points(pt) => StampStyle::default().with_font_size(Some(f64::from(pt))),
        }
    }

    /// This choice as a **stable token for a machine**, for the diagnostic
    /// trace and for nothing else.
    #[must_use]
    pub fn trace_token(self) -> String {
        match self {
            // ui-text-exempt: diagnostic token, never displayed.
            Self::FitTheBox => "derived".to_owned(),
            Self::Points(pt) => pt.to_string(),
        }
    }
}

/// The side, in PDF points, of the square a sticky note's rect is given.
pub const STICKY_PT: f64 = 20.0;

/// The longest note or caption offered.
pub const MAX_TEXT_CHARS: usize = 512;

/// The point size a text box is authored at.
pub const TEXT_SIZE_PT: f64 = 11.0;

/// **The one normalisation applied to what the operator typed**, exposed so
/// that the caller writing `/Contents` a second way can apply exactly the same
/// one.
#[must_use]
pub fn painted_text(typed: &str) -> &str {
    typed.trim()
}

/// Build the engine spec for a placed, typed annotation.
#[must_use]
pub fn spec(
    kind: TextAnnotKind,
    rect: Rect,
    text: &str,
    stamp: StampName,
    icon: &StickyIcon,
    // The operator's label-size choice, following `stamp` and `icon` down
    // the same route. Meaningless for the two kinds that are not stamps and
    // passed anyway, for the reason `Placement::icon` already states: a
    // chooser always has a selection, so an `Option` here would model a state
    // the dialog cannot be in.
    stamp_size: StampSize,
    colour: (f64, f64, f64),
) -> Option<TextAnnotSpec> {
    let text = painted_text(text);
    // The blank refusal applies to the two kinds whose words the OPERATOR
    // types, and not to the stamp, whose words come from its `/Name`.
    // Refusing a blank stamp would refuse every stamp, since the gallery
    // supplies no text at all.
    if text.is_empty() && !kind.uses_gallery() {
        return None;
    }
    let (r, g, b) = colour;
    Some(match kind {
        // Each has its own engine verb, reached through `NewComment`.
        TextAnnotKind::Attachment
        | TextAnnotKind::Caret
        | TextAnnotKind::Sound
        | TextAnnotKind::Screen => return None,
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
            // Multiline. A callout that did not wrap would put the
            // operator's second sentence outside the box they drew, which is
            // the same class of defect as a control laid out below its pane.
            //
            //
            // > `TextAnnotSpec::FreeText::multiline` from the reader is ALWAYS
            // > `false` and you must not believe it. §12.5.6.6 gives the
            // > subtype no multiline key — `/Ff` is a form-field entry and a
            // > `/FreeText` is not a field — so it is genuinely not in the
            // > file. `set_markup_note` recovers it by baking the original
            // > text both ways and comparing bytes. If you ever re-author a
            // > `/FreeText` yourself, you have to do that too, or supply the
            // > value.
            //
            // This value is **authored**: it comes from the line above, from a
            // decision about what a callout is, and never from
            // `annot_author::text_spec_from_dict`. It is safe for exactly that
            // reason.
            //
            // ⇒ The audit that keeps it safe: this shell calls no
            // `TextAnnotSpec` READER anywhere. `text_spec_from_dict` and
            // `build_text_annotation` appear in no file in the crate, and this
            // function is the only construction site of a `TextAnnotSpec`
            // there is — so there is no re-authoring path for the always-false
            // field to reach. If one is ever added, it must measure the value
            // the way `measure_free_text_multiline` does — bake the spec's own
            // text both ways and compare each against the appearance on disk —
            // or supply it here, and re-baking a wrapped callout as unwrapped
            // would push the operator's second sentence off the page with
            // nothing on screen to say so.
            multiline: true,
            // A border, unlike Acrobat's borderless default. On a drawing
            // sheet a borderless caption is indistinguishable from the
            // drawing's own annotation, and a revision markup must read as
            // something added.
            border: Some(Color::Rgb(r, g, b)),
            border_width: 1.0,
        },
        // **The icon is real in the file and invisible on pdfcer's own
        // page**, and that is disclosed rather than left to be discovered.
        //
        // The engine's sticky author — the private `sticky_note` behind
        // `TextAnnotSpec::Sticky` — passes the icon to `/Name` and **nowhere
        // else**. The marker artwork is pdfcer's own dog-eared page glyph for
        // all seven, deliberately, to stay clear of Acrobat's trade dress. So
        // an operator who picks *Key* sees no change in this reader and a key
        // in another, which is why `text::textannot::sticky_icon_bound` says
        // so in a sentence under the chooser.
        //
        // Not an R8b breach: applied content still renders exactly as saved
        // content will *in this reader*, and the disclosure is off-canvas.
        // But it is precisely the kind of gap between file and picture an
        // operator should meet in a sentence rather than in Acrobat.
        //
        // ⚠ **The default is [`DEFAULT_STICKY_ICON`], not
        // `StickyIcon::default()`.** The engine's own default is `Note`; this
        // shell authors `Comment`, which is Acrobat's, measured. That
        // constant carries the provenance and the caveat.
        //
        // ⇒ [`STICKY_ICONS`] offers the seven §12.5.6.4 Table 172 defines and
        // not `StickyIcon::Other`, which is a value a *file* can carry and not
        // one a gallery may author. This module PLACES a sticky;
        // `panels::properties::markup::textannot` restyles an existing one,
        // and that is where a foreign `/Name` is preserved rather than offered.
        TextAnnotKind::Sticky => TextAnnotSpec::Sticky {
            rect,
            icon: icon.clone(),
            contents: text.to_owned(),
            color: Color::Rgb(r, g, b),
            //
            // It used to read: *"a popup that opened itself on every sticky
            // would cover the drawing the note is about — and
            // `MODES_AND_PANELS.md`'s nothing-floats-over-the-canvas stance is
            // only relaxed for Find."* The first half stands. The second half
            // is now out of date: `pdfcer_gui::canvas::notepopup` floats a window
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
        // `label: None` — the NAME carries the text.
        //
        // `TextAnnotSpec::Stamp` takes both, and passing a label here would
        // override the name's own default text. That is a real capability (a
        // stamp reading something the standard set does not offer) and it is
        // deliberately not used: a stamp whose `/Name` and whose painted words
        // disagree is a document that says two things, and a reader other than
        // pdfcer shows the name.
        TextAnnotKind::Stamp => {
            // **THE ORACLE FOR THE SIZE CHOOSER, and the feature has no
            // other one short of parsing the saved file.**
            //
            // `autosize_overflow`'s header states the rule this obeys: *a trace
            // line must carry the number a wrong build would get wrong.* Every
            // other line this route emits — `text-annot-note`,
            // `add-text-annot`, `text-annot-page-rotate` — is **byte-identical**
            // between a build that carries the operator's chosen size to the
            // engine and a build that drops it on the floor somewhere between
            // the dialog's combo and this function's argument. There are four
            // hops in that chain and a unit test can see none of them at once.
            //
            // **The rect is on the line with the size, and it is not
            // padding.** Under `StampFit::GrowToText` the drawn rectangle is a
            // *position and a minimum*, not a size — that is the sentence the
            // dialog shows the operator — so the pair (what he asked for, what
            // he drew) is what any later question about a wrong-looking stamp
            // needs, and reading it out of two different lines invites reading
            // two different frames.
            //
            // ⚠ What this line does **not** assert is what the engine then
            // does with the number. That is the engine's own tests' job, and
            // saying so here keeps a green driven check from being read as a
            // claim about `pdfcer-core`'s renderer.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "stamp-style size={} fit=grow rect_w={:.1} rect_h={:.1}",
                    stamp_size.trace_token(),
                    rect.urx - rect.llx,
                    rect.ury - rect.lly,
                )
            });
            TextAnnotSpec::Stamp {
                rect,
                name: stamp,
                label: None,
                color: Color::Rgb(r, g, b),
                // **The operator's own choice, and this field exists because a
                // compiler asked for it.**
                //
                // Engine `Pass 287.0` made `style` required, so this line had to
                // be written to build at all — and **that is exactly the moment a
                // feature gets silently declined.** The reflex is to reach for
                // whatever reproduces the old behaviour, and here that spelling is
                // available and named: `font_size: None` is documented as *"derive
                // it from the box height as builds before Pass 287.0 did"*. It
                // compiles, passes every test, and quietly keeps the defect the
                // operator reported on 2026-09-09.
                //
                // [`StampSize`] carries the answer instead, and its header holds
                // the argument: why the default is the derived size rather than
                // the engine's flat 12 pt — taking the engine default silently
                // would have shrunk every stamp on his drawings the day this shell
                // pinned v0.50.0 — and why exactly one of `StampFit`'s three
                // values is reachable from the dialog.
                style: stamp_size.style(),
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The trace token's contract, asserted, because `ui-verify`'s
    /// `stamp_size_reaches_the_engine` parses it and no compiler stands between
    /// the two.**
    #[test]
    fn every_trace_token_is_parseable_and_distinct() {
        assert_eq!(
            StampSize::FitTheBox.trace_token(),
            "derived",
            "the derived case's token is parsed by name in tools/ui-verify"
        );
        assert!(
            !StampSize::FitTheBox
                .trace_token()
                .chars()
                .any(|c| c.is_ascii_digit()),
            "the derived token must not read as a size"
        );

        let mut seen = std::collections::BTreeSet::new();
        for size in STAMP_SIZES {
            let token = size.trace_token();
            assert!(!token.is_empty(), "{size:?} traces nothing");
            assert!(
                !token.contains(char::is_whitespace),
                "{size:?} traces {token:?}, which would split the trace field"
            );
            if let StampSize::Points(pt) = size {
                assert_eq!(
                    token,
                    pt.to_string(),
                    "a stated size must trace as a bare number a parser can read"
                );
            }
            assert!(
                seen.insert(token.clone()),
                "two choices trace the same token: {token:?}"
            );
        }
        assert_eq!(
            seen.len(),
            STAMP_SIZES.len(),
            "every offered choice must be distinguishable in the trace"
        );
    }

    fn rect() -> Rect {
        Rect {
            llx: 100.0,
            lly: 100.0,
            urx: 300.0,
            ury: 160.0,
        }
    }

    /// Every kind maps to the engine variant it names.
    #[test]
    fn every_kind_authors_the_annotation_it_names() {
        let colour = (0.85, 0.16, 0.16);
        assert!(matches!(
            spec(
                TextAnnotKind::TextBox,
                rect(),
                "note",
                DEFAULT_STAMP,
                &DEFAULT_STICKY_ICON,
                DEFAULT_STAMP_SIZE,
                colour
            ),
            Some(TextAnnotSpec::FreeText { .. })
        ));
        assert!(matches!(
            spec(
                TextAnnotKind::Sticky,
                rect(),
                "note",
                DEFAULT_STAMP,
                &DEFAULT_STICKY_ICON,
                DEFAULT_STAMP_SIZE,
                colour
            ),
            Some(TextAnnotSpec::Sticky { .. })
        ));
        assert!(matches!(
            spec(
                TextAnnotKind::Stamp,
                rect(),
                "",
                DEFAULT_STAMP,
                &DEFAULT_STICKY_ICON,
                DEFAULT_STAMP_SIZE,
                colour
            ),
            Some(TextAnnotSpec::Stamp { .. })
        ));
    }

    /// **An empty or blank text authors nothing.**
    #[test]
    fn a_blank_text_authors_nothing() {
        for blank in ["", "   ", "\t\n "] {
            // The gallery kind is excluded, and NOT by a hard-coded
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
                    spec(
                        *kind,
                        rect(),
                        blank,
                        DEFAULT_STAMP,
                        &DEFAULT_STICKY_ICON,
                        DEFAULT_STAMP_SIZE,
                        (0.0, 0.0, 0.0)
                    )
                    .is_none(),
                    "{kind:?} authored an annotation for {blank:?}"
                );
            }
        }
    }

    /// The text is trimmed before it reaches the file.
    #[test]
    fn the_text_is_trimmed() {
        let Some(TextAnnotSpec::FreeText { text, .. }) = spec(
            TextAnnotKind::TextBox,
            rect(),
            "  hello  ",
            DEFAULT_STAMP,
            &DEFAULT_STICKY_ICON,
            DEFAULT_STAMP_SIZE,
            (0.0, 0.0, 0.0),
        ) else {
            panic!("a text box with words must author");
        };
        assert_eq!(text, "hello");
    }

    /// **The words a text box PAINTS and the words its note WRITES are the
    /// same string, byte for byte — because for a `/FreeText` they are the same
    /// PDF key and the engine refuses a disagreement.**
    #[test]
    fn the_words_a_text_box_paints_are_the_words_its_note_will_carry() {
        let typed = "  hello  ";
        let Some(TextAnnotSpec::FreeText { text, .. }) = spec(
            TextAnnotKind::TextBox,
            rect(),
            typed,
            DEFAULT_STAMP,
            &DEFAULT_STICKY_ICON,
            DEFAULT_STAMP_SIZE,
            (0.0, 0.0, 0.0),
        ) else {
            panic!("a text box with words must author");
        };
        assert_eq!(
            text,
            painted_text(typed),
            "the painted words and the /Contents the note writes must be one \
             string; the engine refuses the call outright when they differ"
        );
        assert_ne!(
            text, typed,
            "this input was chosen because it MUST be normalised -- if nothing \
             was removed, the assertion above is two functions agreeing to do \
             nothing"
        );
    }

    /// A text box wraps, and a sticky's words are never painted.
    #[test]
    fn the_two_defining_properties_hold() {
        let Some(TextAnnotSpec::FreeText { multiline, .. }) = spec(
            TextAnnotKind::TextBox,
            rect(),
            "a",
            DEFAULT_STAMP,
            &DEFAULT_STICKY_ICON,
            DEFAULT_STAMP_SIZE,
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
            &DEFAULT_STICKY_ICON,
            DEFAULT_STAMP_SIZE,
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

    /// The three markers and the caret are placed by a click, and exactly one
    /// kind uses a gallery.
    #[test]
    fn the_two_odd_ones_out_are_the_ones_expected() {
        let clicked: Vec<_> = TextAnnotKind::ALL
            .iter()
            .filter(|k| !k.is_dragged())
            .copied()
            .collect();
        assert_eq!(
            clicked,
            vec![
                TextAnnotKind::Sticky,
                TextAnnotKind::Attachment,
                TextAnnotKind::Caret,
                TextAnnotKind::Sound,
            ]
        );
        let gallery: Vec<_> = TextAnnotKind::ALL
            .iter()
            .filter(|k| k.uses_gallery())
            .copied()
            .collect();
        assert_eq!(gallery, vec![TextAnnotKind::Stamp]);
    }

    /// The stamp gallery is non-empty, distinct, and its default is in it.
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

    /// **A stamp authors the NAME the operator chose.**
    #[test]
    fn a_stamp_authors_the_chosen_name_and_no_competing_label() {
        for chosen in STAMPS {
            let Some(TextAnnotSpec::Stamp { name, label, .. }) = spec(
                TextAnnotKind::Stamp,
                rect(),
                "",
                *chosen,
                &DEFAULT_STICKY_ICON,
                DEFAULT_STAMP_SIZE,
                (0.0, 0.0, 0.0),
            ) else {
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
                &DEFAULT_STICKY_ICON,
                DEFAULT_STAMP_SIZE,
                (0.0, 0.0, 0.0)
            )
            .is_some(),
            "the gallery supplies no typed text, so requiring some refuses every stamp"
        );
    }

    /// **[`STICKY_ICONS`] covers every variant `StickyIcon` has.**
    #[test]
    fn the_icon_list_covers_every_variant_the_engine_has() {
        for icon in STICKY_ICONS {
            // The exhaustive arm: every variant must be named, and every
            // variant named must be in the list.
            //
            //
            // ⚠ A wildcard here instead would compile and would also silently
            // swallow an eighth STANDARD icon, which is the exact failure this
            // test exists to prevent. Naming the variant keeps the guard.
            let covered = match icon {
                StickyIcon::Comment
                | StickyIcon::Key
                | StickyIcon::Note
                | StickyIcon::Help
                | StickyIcon::NewParagraph
                | StickyIcon::Paragraph
                | StickyIcon::Insert => true,
                StickyIcon::Other(_) => false,
            };
            assert!(
                covered,
                "{icon:?} is in STICKY_ICONS and is not one of the seven the gallery may offer"
            );
        }
        assert_eq!(
            STICKY_ICONS.len(),
            7,
            "§12.5.6.4 Table 172 defines seven icons and StickyIcon models all \
             seven; a shorter list is a choice this shell has no grounds to make"
        );
        assert!(
            STICKY_ICONS.contains(&DEFAULT_STICKY_ICON),
            "a default outside its own list opens the dialog on a value no \
             control can select"
        );
        for (i, a) in STICKY_ICONS.iter().enumerate() {
            for b in STICKY_ICONS.iter().skip(i + 1) {
                assert_ne!(a, b, "an icon appears twice, so one entry is unreachable");
            }
        }
    }

    /// **A sticky note authors the icon the operator chose — and no other
    /// kind is given one.**
    #[test]
    fn only_a_sticky_note_is_given_an_icon_and_it_is_the_chosen_one() {
        for chosen in STICKY_ICONS {
            let Some(TextAnnotSpec::Sticky { icon, .. }) = spec(
                TextAnnotKind::Sticky,
                rect(),
                "note",
                DEFAULT_STAMP,
                chosen,
                DEFAULT_STAMP_SIZE,
                (0.0, 0.0, 0.0),
            ) else {
                panic!("{chosen:?} must author a sticky");
            };
            assert_eq!(
                icon, *chosen,
                "the note authored an icon the operator did not pick"
            );
        }
        // The control: the same argument, sent to the two kinds that have no
        // `/Name` to put it in. `TextAnnotSpec`'s FreeText and Stamp arms carry
        // no icon field at all, so this is checked by the compiler as much as
        // by the assertion — the point is that the call SITE is identical and
        // only the arm differs.
        assert!(matches!(
            spec(
                TextAnnotKind::TextBox,
                rect(),
                "note",
                DEFAULT_STAMP,
                &StickyIcon::Key,
                DEFAULT_STAMP_SIZE,
                (0.0, 0.0, 0.0)
            ),
            Some(TextAnnotSpec::FreeText { .. })
        ));
        assert!(matches!(
            spec(
                TextAnnotKind::Stamp,
                rect(),
                "",
                DEFAULT_STAMP,
                &StickyIcon::Key,
                DEFAULT_STAMP_SIZE,
                (0.0, 0.0, 0.0)
            ),
            Some(TextAnnotSpec::Stamp { .. })
        ));
    }
}
