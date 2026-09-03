//! # icons::catalog — which glyphs exist, and what each one means
//!
//! The [`Icon`] enum is the whole vocabulary: one variant per drawn glyph,
//! named for the **role** the icon plays rather than for the artwork, so a
//! future re-draw changes one constant in [`super::assets`] and touches no
//! call site.
//!
//! Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\icons.rs` (Class A,
//! `SALVAGE.md`). Every variant's doc comment is carried across, because
//! several of them are not descriptions at all — they are *rulings*. Three
//! kinds recur and each one is a decision somebody paid for:
//!
//! * **"This glyph was authored because a text character had no face."**
//!   [`Icon::Back`], [`Icon::Close`], [`Icon::ChevronUp`],
//!   [`Icon::ChevronDown`] each replace a Unicode character that was
//!   VERIFIED to render as a tofu box in the shipped font stack. The
//!   operator's standing ruling (2026-08-06) is that a missing glyph is
//!   **authored**, not worked around by rewording the control.
//! * **"This glyph must not be that other glyph."** [`Icon::Back`] vs
//!   [`Icon::ChevronLeft`], [`Icon::ShowPoints`] vs [`Icon::EditObjects`],
//!   [`Icon::Layers`] vs [`Icon::Combine`]. Each pair states the shape cue
//!   that keeps them apart at 16 px, and losing that note is how the pair
//!   quietly converges in a later "consistency" pass.
//! * **"An icon is a claim."** [`Icon::Signatures`] must not be a seal,
//!   badge, shield or checkmark, because pdfcer performs no cryptographic
//!   verification and those shapes read as VALIDATED. [`Icon::Fonts`] must
//!   not be a pencil or an I-beam, because the Fonts panel writes nothing.
//!   A glyph reaches the operator's eye before the panel's first line does.
//!
//! ## The one key namespace
//!
//! [`Icon::name`] is the string an `egui_shell::Command` names with
//! `.with_icon("…")`, and [`Icon::from_key`] is the reverse. There is
//! exactly one spelling of each key and it lives in `name`; `from_key`
//! searches [`Icon::ALL`] rather than carrying a second `match`, so the two
//! cannot drift. `every_name_round_trips_through_from_key` pins it anyway,
//! because "cannot drift" is a property of today's implementation and the
//! test is a property of the contract.

use super::assets;

/// Every icon pdfcer ships, one variant per drawn glyph.
///
/// Two roles deliberately share one asset: [`Icon::Open`] and
/// [`Icon::FontFolders`] are both the plain folder glyph — Open is a
/// top-level action and Font Folders is a labelled row three levels into a
/// dock, and they are never on screen together.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Icon {
    /// Open a file. ScripTree `icon-folder.svg`.
    Open,
    /// Save a copy.
    Save,
    /// Thumbnail-rail visibility toggle ("sidebar").
    Sidebar,
    /// Annotation-visibility toggle ("comment-bubble").
    Comment,
    /// Previous page ("chevron").
    ChevronLeft,
    /// Leave a ribbon-opened surface and return to the armed tools' own
    /// options ("back-arrow").
    ///
    /// Authored 2026-08-06 under the operator's standing ruling that a
    /// missing glyph is AUTHORED, not worked around: the control wanted `←`
    /// (U+2190), the coverage gate correctly rejected it as having no glyph
    /// in the shipped stack, and the first fix reworded the button to plain
    /// text. Rewording spends the operator's affordance to protect the font
    /// stack; an icon costs one asset and keeps both.
    ///
    /// Distinct from [`Icon::ChevronLeft`] by its SHAFT — see `back.svg`'s
    /// own embedded note. Same reasoning that made [`Icon::ChevronUp`] and
    /// [`Icon::ChevronDown`] exist: those two were also authored precisely
    /// because their text glyphs were tofu.
    Back,
    /// Next page.
    ChevronRight,
    /// "Move selection up" in the page rail and the Combine-files list,
    /// drawn instead of the text glyph `▲` (U+25B2) — VERIFIED tofu in the
    /// running build 2026-08-03, same Geometric Shapes block as `▾`. Those
    /// buttons are glyph-ONLY, so a missing glyph left them with no visible
    /// identity at all.
    ChevronUp,
    /// Menu-disclosure marker on a dropdown button, drawn instead of the
    /// text glyph `▾` (U+25BE) — which is absent from every font in egui's
    /// default Proportional chain and rendered as a tofu box on four shipped
    /// toolbar controls.
    ChevronDown,
    /// A magnifier — Find. Empty lens: [`Icon::ZoomIn`]/[`Icon::ZoomOut`]
    /// are the same shape carrying a `+`/`-`, so the unmarked lens is what
    /// says "search" rather than "magnify by".
    Search,
    /// Dismiss / remove — drawn instead of the text glyph `✕` (U+2715),
    /// which is absent from every font of the shipped stack.
    ///
    /// Authored rather than reworded, per the operator's 2026-08-06 ruling
    /// that a missing glyph for a real control gets an icon created for it.
    Close,
    /// Zoom out ("magnifier±").
    ZoomOut,
    /// Zoom in.
    ZoomIn,
    /// Fit whole page ("frame-fit").
    FitPage,
    /// Fit page width ("frame-fit-width").
    FitWidth,
    /// Fit page height ("frame-fit-height") - O29, the Acrobat parity mode.
    FitHeight,
    /// Rotate page counter-clockwise ("rotate-page").
    RotateCcw,
    /// Rotate page clockwise.
    RotateCw,
    /// Document properties. ScripTree `icon-document.svg`.
    Properties,
    /// Markup menu ("shapes").
    Markup,
    /// Text menu ("note").
    Text,
    /// Edit page text. ScripTree `icon-edit.svg`.
    EditText,
    /// Add page text ("text-cursor-plus").
    AddText,
    /// Edit vector objects — the "Obj" tool. Not in the ui-spec (that tool
    /// shipped after the spec was written); authored in the same style
    /// contract, see [`super::assets`] §5.
    EditObjects,
    /// Create an interactive form field — the "Create Field" tool. Authored
    /// 2026-08-07; not in the ui-spec, same style contract as
    /// [`Icon::EditObjects`].
    ///
    /// Deliberately NOT reusing any existing asset. The nearest candidates
    /// each would have said something false: `edit-objects.svg` promises
    /// vector editing, and a plain box would read as the FILL surface, which
    /// is a different capability in a different ribbon group.
    FormField,
    /// Measure/dimension menu. ScripTree `icon-ruler.svg`.
    Measure,
    /// Undo ("history-arrow").
    Undo,
    /// Redo.
    Redo,
    /// Copy-text menu ("copy").
    Copy,
    /// Tools dock toggle. ScripTree `icon-tool.svg`.
    Tools,
    /// Keyboard-shortcuts window ("keyboard").
    Keyboard,
    /// About window ("info") — version, licence, and the attribution surface.
    ///
    /// The circled "i" is the one glyph in this set with no plausible
    /// alternative: every application that has an About item draws it, which
    /// is exactly the "industry convention with no single author" that header
    /// §2 permits and encourages. Distinct from [`Icon::Properties`] on
    /// purpose — that one is the document glyph and means *this file*, and
    /// About is about the program rather than about anything open in it.
    Info,
    /// "Show points" view toggle — draws every anchor of the object being
    /// worked inside, so the points can be aimed at BEFORE one is selected.
    ///
    /// Deliberately close to, and deliberately distinct from,
    /// [`Icon::EditObjects`]: both show square node marks, because both are
    /// about the same points. That one puts two on a Bézier (shape editing);
    /// this one puts three on a straight run with the middle one offset (the
    /// points themselves, and the canvas's selected-node vocabulary).
    ShowPoints,
    /// Bookmarks panel toggle — the document's outline.
    ///
    /// A ribbon with a notch, which is the one shape read as "bookmark"
    /// without a label. Deliberately not a page-with-lines: that is
    /// [`Icon::Properties`]/document territory, and this panel is about
    /// places IN a document rather than the document itself.
    Bookmarks,
    /// Layers panel toggle — optional content.
    ///
    /// Three stacked sheets. Three rather than two so it does not read as
    /// [`Icon::Combine`]'s linked pair at 16 px.
    Layers,
    /// Signatures panel toggle.
    ///
    /// A written flourish over a signing rule, and emphatically **not** a
    /// seal, badge, shield or checkmark: each of those reads as VALIDATED,
    /// and pdfcer performs no cryptographic verification. The panel's first
    /// line says so; the glyph must not contradict it before the panel is
    /// open. An icon is a claim too.
    Signatures,
    /// Fonts panel toggle — the document's font inventory.
    ///
    /// A capital A on a baseline rule. The letterform reads as "type"; the
    /// rule under it is what stops it reading as "a text tool", which
    /// matters because this panel writes nothing and an icon borrowed from
    /// an editing tool would suggest otherwise. Deliberately not
    /// [`Icon::AddText`]'s I-beam-plus or [`Icon::EditText`]'s pencil for
    /// that reason.
    Fonts,
    /// The Tool panel — an arrow cursor with two option rules beside it.
    ///
    /// ★ The one panel glyph that draws **the operator's hand rather than the
    /// document**. Bookmarks, Layers, Pages and Objects all picture content;
    /// this panel's subject is what the pointer does, so its icon is the
    /// pointer.
    ///
    /// Deliberately **not** [`Icon::Tools`]'s wrench, which is already spoken
    /// for by the Tools ribbon tab — and which means *settings* in every other
    /// application on this machine, a surface pdfcer has separately and must not
    /// be confused with.
    Pointer,
    /// Markup → Rectangle.
    ShapeRect,
    /// Markup → Ellipse.
    ShapeEllipse,
    /// Markup → Arrow line.
    ShapeArrow,
    /// Markup → PolyLine — an **open** run of clicked segments.
    ///
    /// Drawn as [`Icon::ShapePolygon`] with its closing segment removed, because
    /// that is exactly how the two annotations differ (§12.5.6.13: `/PolyLine`
    /// is `/Polygon` that does not close). The pair therefore teaches itself —
    /// an operator who learns one has learned the other — where two unrelated
    /// drawings would have to be memorised separately.
    ShapePolyline,
    /// Markup → Polygon — the same run of clicks, closed.
    ///
    /// An **irregular** pentagon rather than a regular one, and the irregularity
    /// carries the meaning: regularity would read as *a shape primitive*, beside
    /// [`Icon::ShapeRect`] and [`Icon::ShapeEllipse`], where what this control
    /// means is *a shape you build out of clicked corners*. Five corners rather
    /// than four, because a four-corner glyph at 16 px reads as a rotated
    /// `shape-rect`, which is the one collision this band cannot afford (icon
    /// ui-spec §2.1's ❌-grade risk: two members of one group differing only by a
    /// small feature).
    ShapePolygon,
    /// Markup → Revision cloud — the same closed run of clicks with a cloudy
    /// border.
    ///
    /// ★ The one glyph in this band whose meaning is carried by its **edge**
    /// rather than by its outline, and that is the annotation's own doing: a
    /// revision cloud is a `/Polygon` with `/BE << /S /C >>` on it (Table 181),
    /// so the scallop *is* the difference. Drawn as the same closed loop as
    /// [`Icon::ShapePolygon`] with its straight edges replaced by outward arcs
    /// — which is exactly how the two annotations differ in the file, so the
    /// pair teaches itself the way `ShapePolyline`/`ShapePolygon` already do.
    ///
    /// Nine arcs, an odd count. An even one is symmetric about both axes and
    /// reads as a decorative rosette; a revision cloud on a real drawing is a
    /// hand-made mark. Nine is also the floor at which the scallop survives
    /// 16 px — fewer and the arcs flatten toward [`Icon::ShapeEllipse`], which
    /// is the collision this band cannot afford.
    ShapeCloud,
    /// Markup → Freehand — the `/Ink` annotation.
    ///
    /// One irregular flowing stroke spanning the whole tile, with round caps
    /// because `pdfcer-core`'s `ink()` builder sets `LineCap::Round` and a glyph
    /// that promised square ends would be describing a different annotation.
    ///
    /// Deliberately **not** [`Icon::TextSquiggly`], which is a *periodic*
    /// four-lobe wave sitting in a band under two text lines and means "mark
    /// these words". The two never share a band — Ink is in Shapes and Squiggly
    /// is in Text markup — but an operator carries the vocabulary between them,
    /// so the difference is made structural (aperiodic, no baseline, full-tile)
    /// rather than positional.
    ShapeInk,
    /// Markup → Highlight band.
    ShapeHighlight,
    /// View ▸ Navigate → the **text tool**, which makes the primary drag sweep
    /// a range of text instead of marqueeing objects.
    ///
    /// A bare I-beam — the shape every operating system, word processor and PDF
    /// reader draws over selectable text, and literally the `CursorIcon` the
    /// tool installs, which is the strongest thing an icon can be: a picture of
    /// what the control does to the pointer.
    ///
    /// Deliberately not [`Icon::AddText`], which is this glyph plus a small
    /// badge. The badge is the difference and it is the right one: a plus
    /// **creates** text, and this tool creates nothing — it selects what the
    /// page already carries. The two live on different tabs and never share a
    /// band, so what has to survive is an operator carrying the vocabulary
    /// between them; the beam here is therefore centred and full-width where
    /// `AddText`'s is pushed left to clear its badge.
    ///
    /// Nor [`Icon::Fonts`]'s A-on-a-baseline, which reads as "type" — a property
    /// of the document — where this must read as "cursor", a property of the
    /// pointer. That is the mirror of the constraint `Fonts` records against
    /// borrowing from an editing tool.
    TextSelect,
    /// Markup → Underline the selected text.
    ///
    /// The first of a family of three that differ only in the third stroke —
    /// under, through, wavy — which is exactly how the three commands differ.
    /// See the asset for why the "text" strokes are inset and the mark is not.
    TextUnderline,
    /// Markup → Strike out the selected text.
    TextStrikeout,
    /// Markup → Squiggly-underline the selected text.
    TextSquiggly,
    /// Text → FreeText box.
    TextFreeText,
    /// Text → Sticky note.
    TextSticky,
    /// Text → Stamp, and the reserved Bates-numbering glyph.
    Stamp,
    /// Combine files…. ScripTree `icon-link.svg`.
    Combine,
    /// Split this document…. ScripTree `icon-scissors.svg`.
    Split,
    /// Insert pages from a file…. ScripTree `icon-upload.svg`.
    InsertPages,
    /// Import form data… — the same upload art as [`Icon::InsertPages`].
    ///
    /// ★★ A **distinct key over shared art**, which is [`Icon::FontFolders`]'s
    /// arrangement and not the shared-key convention `format.properties` uses.
    /// The difference matters: a shared *key* says *two controls about one
    /// thing*, and this is not that — inserting pages and importing form data
    /// have nothing in common but a direction. What they share is that the only
    /// honest picture for either is *"something comes in from a file"*, which
    /// is what the upload arrow draws.
    ///
    /// ★ Keying it to `insert-pages` would have been the near-miss reuse this
    /// catalog's refusal table exists to prevent: a pages-named key on a form
    /// command reads as a mistake to anyone grepping either.
    ImportFormData,
    /// Font folders… — the same folder art as [`Icon::Open`].
    FontFolders,
    /// Redaction.
    ///
    /// It is the one intentionally solid-FILLED glyph in an otherwise
    /// all-outline set, which is also why it is the pipeline's only coverage
    /// of the fill path (see `redaction_is_the_only_filled_icon`). The fill
    /// is not decoration: every other tool in this app draws or measures,
    /// and this one obliterates, so its glyph reads as a solid bar rather
    /// than an outline of one.
    Redact,

    // =======================================================================
    // ★ The 2026-08-14 pass — twenty-five glyphs for the ribbon's remaining
    // text buttons.
    //
    // They are kept together, in one dated block, rather than interleaved
    // with their subject neighbours above. The block IS the record: of 88
    // registered commands, 47 named an icon and 41 did not, so a ribbon band
    // mixed glyphs and bare words with no rule behind which was which.
    // Thirty of the 41 are answered here. The remaining eleven are recorded
    // refusals, stated at their registrations in `crate::shell::commands`
    // and summarised in [`super::assets`] §5 deviation #8 — that list is the
    // other half of this one and neither is complete without it.
    //
    // Every variant below is an addition the ui-spec does not cover, for the
    // single reason [`super::assets`] §5 deviation #7 gives: the spec's §0
    // audited the OLD shell's toolbar, and these are controls that toolbar
    // did not have. Where a spec row DID reserve something, the asset cites
    // it; where this pass overrode one, the asset carries the reason.
    // =======================================================================
    /// Print. ScripTree `icon-printer.svg` (ui-spec §8.12).
    ///
    /// Explicitly **not** [`Icon::Stamp`], which is what the salvage source
    /// drew Print with. That was a mis-assignment rather than a convention
    /// worth carrying: stamp art means "a mark applied with a stamp"
    /// (ui-spec §3.4) and is shared with the reserved Bates-numbering glyph.
    Print,
    /// Export — **both** Export DXF and Export form data.
    ///
    /// ScripTree `icon-download.svg`, which ui-spec §3.1 reserved for
    /// exactly this ("reserve `icon-download` for a future Export-data /
    /// Extract-pages action, so the download/upload pair stays meaningful as
    /// an 'in/out of this document' pair"). The mirror of
    /// [`Icon::InsertPages`]'s upload art, and it only carries that meaning
    /// while both halves stay reserved for it.
    ///
    /// One key on two commands is the deliberate convention
    /// `crate::shell::commands`' header states: a family sharing a glyph is
    /// how a ribbon reads as grouped, and uniqueness is a property of ids.
    Export,
    /// Settings. ScripTree `icon-settings.svg` — three sliders.
    ///
    /// Deliberately not a cogwheel: at 16 px a cog's teeth close into a
    /// disc, and "machinery" is the wrong reading for a preferences dialog.
    Settings,
    /// Insert an image. ScripTree `icon-image.svg`.
    ///
    /// ui-spec §8.5 reserved the picture metaphor for OCR. This is the
    /// earlier and the primary claim: it places an actual raster on the
    /// page, where OCR merely reads one. See the asset for why the eventual
    /// share is available but must be conscious.
    InsertImage,
    /// Set the dimension group's scale. ScripTree `icon-convert.svg`
    /// (ui-spec §8.2).
    ///
    /// Deliberately not a second ruler: [`Icon::Measure`] already means
    /// "measure something", and this command measures nothing — it changes
    /// what measurements are read against.
    SetScale,

    /// Page display → one page at a time.
    ///
    /// ★ The four page-display glyphs are ONE control and are drawn as one:
    /// bare page silhouettes with no interior detail, because the
    /// arrangement is the information. Two axes carry all four positions —
    /// **left-to-right** says how many pages are across, and a **cut bottom
    /// edge** says whether they keep coming. An operator who learns one axis
    /// has learned the radio.
    ///
    /// That is also why all four exist or none would: a radio with three
    /// glyphs and one bare word does not read as a radio at all.
    PageSingle,
    /// Page display → one column, scrolling.
    PageContinuous,
    /// Page display → two pages side by side.
    ///
    /// Distinct from [`Icon::ReadMode`]'s open book, the other two-page
    /// glyph and two groups away on the same tab: these are straight-edged
    /// sheets with a gutter, the book's leaves curve and meet at a spine.
    PageFacing,
    /// Page display → two pages side by side, scrolling.
    PageFacingContinuous,

    /// Marquee zoom — drag a box, magnify it.
    ///
    /// The fourth member of ui-spec §3.1's magnifier family, whose grammar
    /// is that the LENS names the member: empty is [`Icon::Search`], a minus
    /// is [`Icon::ZoomOut`], a plus is [`Icon::ZoomIn`], a **box** is this.
    ///
    /// Corner brackets would have been the other obvious marquee glyph and
    /// are refused: [`Icon::FitPage`] *is* four corner brackets and sits two
    /// buttons away in the same group.
    ZoomRegion,
    /// Zoom to the selection.
    ///
    /// ui-spec §3.1's corner-bracket family, reduced to a **diagonal pair**
    /// closing on an object. Four brackets would differ from
    /// [`Icon::FitPage`] only by the small rect inside — a same-group
    /// collision of the ❌ grade ui-spec §2.1 names.
    ZoomSelection,
    /// The Hand (pan) tool.
    ///
    /// Deliberately not a four-way arrow cross, which is simpler and reads
    /// at a smaller size but says MOVE THIS OBJECT. This tool moves the
    /// viewport and nothing else, and markup on the same page *can* be
    /// dragged — so the cheaper glyph would have been a lie.
    /// Scissors — `edit.cut`.
    Cut,
    /// A clipboard — `edit.paste`.
    Paste,
    /// The **Select** tool's filled arrow.
    Cursor,
    /// The **Points** tool's hollow arrow, with the anchors it reveals.
    ///
    /// Its outline is byte-identical to [`Self::Cursor`]'s; see the SVG's own
    /// comment for why that must stay true.
    CursorNode,
    Hand,

    /// Rulers toggle — the canvas's ruled edges.
    ///
    /// Two graduated bands meeting at a corner, and the L is the whole
    /// distinction from [`Icon::Measure`]'s single band. One ruler is a tool
    /// you measure with; two ruled edges framing a corner are furniture the
    /// window wears. The two live on different tabs, but an operator carries
    /// the vocabulary between them.
    Rulers,
    /// Grid toggle.
    ///
    /// Three cells a side, not four or five: at four the ladder closes to
    /// about 2.5 px in a 16 pt slot and the glyph becomes the wash
    /// `DEFECTS.md` #8 records — a grid so fine it is a tint rather than a
    /// grid. The icon must not repeat the feature's own defect.
    Grid,
    /// Guides toggle.
    ///
    /// Two lines, off-centre, **overshooting** the page on every side. Both
    /// properties separate it from [`Icon::Grid`] and both are true of a
    /// real guide: it is dragged out of a ruler and belongs to the window,
    /// so it does not stop where the paper does. Centring them would draw a
    /// crosshair, which means "target" everywhere else.
    Guides,

    /// Pages panel toggle.
    ///
    /// ★ This variant **retires a recorded decision**. `view.panel_pages`
    /// carried a note reading "No icon, and that is a decision rather than
    /// an omission — there is no `document` (or `pages`) key in
    /// `crate::icons::catalog`, and naming one would draw the catalogue's
    /// deliberate visible slashed mark". That was true and is now spent: the
    /// key exists. The note has been rewritten at the registration rather
    /// than left standing as though its premise still held.
    ///
    /// Three sheets, front one whole and the two behind showing only the
    /// edges that clear it — both choices borrowed from [`Icon::Layers`],
    /// which faced the same two risks: two complete offset rects is
    /// [`Icon::Copy`], and three fully-drawn sheets is a solid mass at 16 px.
    /// Distinct from `Layers` itself, which is isometric because a layer is
    /// a plane you look *through*; these are square-on because a page is a
    /// thing you look *at*.
    Pages,
    /// Forms panel toggle.
    ///
    /// A page carrying two input boxes. Boxes rather than [`Icon::Properties`]'s
    /// ruled lines is the distinction: lines are text somebody wrote, boxes
    /// are places left empty for you.
    ///
    /// Carries no tick or check mark, for [`Icon::Signatures`]' reason — a
    /// check reads as VALIDATED and nothing here validates anything. And it
    /// does not contradict ui-spec §8.14's "no dedicated toolbar icon" for
    /// form *filling*: that ruling is about there being no fill TOOL to arm,
    /// and this is a panel toggle, a control §8.14 never contemplated.
    Forms,

    /// Read mode — the chrome gets out of the way.
    ///
    /// An open book. `RIBBON_IA.md` §3 named this and [`Icon::Fullscreen`]
    /// as the two commands with "no ribbon control at all", on a tab
    /// literally called View — "the single most confusing thing in the
    /// current ribbon". They have controls now, so they have glyphs.
    ReadMode,
    /// Full screen.
    ///
    /// Four arrows fanning outward. The **shafts** are what keep it apart
    /// from [`Icon::FitPage`]'s four bare brackets on the same tab: brackets
    /// alone say "bring the content inside this frame", brackets with a
    /// shaft running out to each say "push the frame out to the screen", and
    /// those are opposite motions.
    Fullscreen,
    /// Floating panels — whether the operator may tear a panel out.
    ///
    /// A docked frame with a smaller window straddling its corner. The
    /// **title bar** on that window is load-bearing: without it the glyph is
    /// two overlapping rounded rects, which is [`Icon::Copy`] exactly.
    FloatingPanels,
    /// Reset layout.
    ///
    /// A two-pane window with an arrow sweeping back over it — the
    /// object-plus-arrow construction ui-spec §3.1 "rotate-page"
    /// established and [`Icon::RotateCcw`] uses. That paragraph is also why
    /// the pane divider is not optional: an arrow alone is history
    /// ([`Icon::Undo`]), an arrow around a thing is an operation on that
    /// thing.
    ResetLayout,

    /// Delete — **both** Delete pages and the Format tab's Delete.
    ///
    /// A lidded bin. Boring on purpose: this sits on the two commands that
    /// remove something an operator can see, and a clever picture on a
    /// destructive verb is one somebody has to stop and decode.
    ///
    /// One key on two commands, per `crate::shell::commands`' shared-key
    /// convention. The verb is genuinely the same; what differs is the
    /// target, which the label beside the glyph names, and the two are never
    /// drawn together (Format is contextual, and one tab's band shows at a
    /// time).
    ///
    /// Deliberately not [`Icon::Close`]'s bare cross, which means DISMISS,
    /// and not scissors — ui-spec §2.1 keeps scissors on Split and off
    /// anything that removes content.
    Delete,
    /// Extract pages to a new file.
    ///
    /// The Extract-pages half of the download direction ui-spec §3.1
    /// reserved; [`Icon::Export`] is the other half. The **page** is what
    /// separates the two: both say "out", this one says what is going out.
    PageExtract,

    /// Flatten form fields.
    ///
    /// Drawn to ui-spec §8.14's own construction — "a form-field rectangle
    /// with a small downward chevron pressing onto it (burn-in metaphor)".
    /// The caret inside the rectangle is borrowed from [`Icon::FormField`],
    /// so the thing being pressed is unmistakably a field.
    FormFlatten,
    /// Manage a list — **both** Manage fields and Manage dimension groups.
    ///
    /// ★ A recorded **deviation**: ui-spec §8.2 assigns `icon-ring.svg` to
    /// Manage Dimension Groups, and two concentric circles read as a target
    /// or a radio button at 16 px, not as a list of named things. That row
    /// was written at reservation depth before the Measure surface existed
    /// and states no reasoning to weigh against.
    ///
    /// The family here is one of **action, not of subject**: the two
    /// commands have nothing to do with each other, but both answer a click
    /// by opening a managed list, and that is the only thing an icon can
    /// honestly promise where "fields" and "dimension groups" are words only
    /// the label can say. They sit on different tabs.
    ManageList,

    /// Selection-filter row: **text as a whole object**.
    ///
    /// One of five glyphs added for `crate::canvas::pick`'s popup (O17). Its
    /// pair is [`Icon::TextSelect`], one row below it, which means the
    /// *characters* you sweep rather than the *object* you click — the two
    /// rows ask genuinely different questions about the same ink, so the two
    /// glyphs share nothing.
    PickText,
    /// Selection-filter row: **path objects** — the line work of a drawing.
    ///
    /// The operator's word for the row is "lines". Carries no node marks, and
    /// that absence is the entire thing separating it from
    /// [`Icon::EditObjects`] and [`Icon::ShowPoints`], both of which are also
    /// "a stroke across the box".
    PickPath,
    /// Selection-filter row: **the Part rung** — one subpath of a path, or one
    /// show-operator run of a text object.
    ///
    /// A chain with a bracket under one segment. The hardest of the five,
    /// because what it has to depict is a *relationship* rather than a thing.
    PickPart,
    /// Selection-filter row: **form XObjects** — a whole nested drawing that
    /// the page treats as one opaque object.
    ///
    /// Usually the title block or the border on a CAD sheet, and the usual
    /// answer to "why is the selection box so big?".
    PickFormXObject,
    /// Selection-filter row: **`/Link` annotations**.
    ///
    /// ★ Deliberately not [`Icon::Combine`]'s chain, which means "join these
    /// files" and is a metaphor the operator has already learned for something
    /// else. This is the box-with-escaping-arrow every browser uses.
    PickLink,
}

impl Icon {
    /// Every icon, in catalogue order.
    ///
    /// This is the list the catalogue-wide tests walk, and it is what makes
    /// "every shipped asset is valid" an enforced property rather than a
    /// hope — so a new [`Icon`] variant MUST be added here or it ships
    /// unverified. `all_is_exhaustive` guards the omission that would
    /// otherwise be invisible.
    pub const ALL: &'static [Icon] = &[
        Icon::Open,
        Icon::Save,
        Icon::Sidebar,
        Icon::Comment,
        Icon::ChevronLeft,
        Icon::Back,
        Icon::ChevronRight,
        Icon::ChevronDown,
        Icon::Search,
        Icon::ChevronUp,
        Icon::Close,
        Icon::ZoomOut,
        Icon::ZoomIn,
        Icon::FitPage,
        Icon::FitWidth,
        Icon::FitHeight,
        Icon::RotateCcw,
        Icon::RotateCw,
        Icon::Properties,
        Icon::Markup,
        Icon::Text,
        Icon::EditText,
        Icon::AddText,
        Icon::EditObjects,
        Icon::FormField,
        Icon::Measure,
        Icon::Undo,
        Icon::Redo,
        Icon::Copy,
        Icon::Tools,
        Icon::Keyboard,
        Icon::Info,
        Icon::ShowPoints,
        Icon::Bookmarks,
        Icon::Layers,
        Icon::Signatures,
        Icon::Fonts,
        Icon::Pointer,
        Icon::ShapeRect,
        Icon::ShapeEllipse,
        Icon::ShapeArrow,
        Icon::ShapePolyline,
        Icon::ShapePolygon,
        Icon::ShapeCloud,
        Icon::ShapeInk,
        Icon::ShapeHighlight,
        Icon::TextSelect,
        Icon::TextUnderline,
        Icon::TextStrikeout,
        Icon::TextSquiggly,
        Icon::TextFreeText,
        Icon::TextSticky,
        Icon::Stamp,
        Icon::ImportFormData,
        Icon::Combine,
        Icon::Split,
        Icon::InsertPages,
        Icon::FontFolders,
        Icon::Redact,
        // The 2026-08-14 pass, in the enum's own order.
        Icon::Print,
        Icon::Export,
        Icon::Settings,
        Icon::InsertImage,
        Icon::SetScale,
        Icon::PageSingle,
        Icon::PageContinuous,
        Icon::PageFacing,
        Icon::PageFacingContinuous,
        Icon::ZoomRegion,
        Icon::ZoomSelection,
        Icon::Cut,
        Icon::Paste,
        Icon::Cursor,
        Icon::CursorNode,
        Icon::Hand,
        Icon::Rulers,
        Icon::Grid,
        Icon::Guides,
        Icon::Pages,
        Icon::Forms,
        Icon::ReadMode,
        Icon::Fullscreen,
        Icon::FloatingPanels,
        Icon::ResetLayout,
        Icon::Delete,
        Icon::PageExtract,
        Icon::FormFlatten,
        Icon::ManageList,
        // The 2026-08-21 pass — the selection filter's rows (O17).
        Icon::PickText,
        Icon::PickPath,
        Icon::PickPart,
        Icon::PickFormXObject,
        Icon::PickLink,
    ];

    /// The asset's SVG source.
    ///
    /// `include_str!` at compile time rather than a runtime file read,
    /// because pdfcer ships single-folder portable: the executable must not
    /// depend on an `assets/` directory travelling beside it, and an icon
    /// that fails to load at startup is not a failure mode worth having when
    /// the whole set is ~79 KB of text. See [`super::assets`] for why the
    /// `.svg` files live inside `src/icons/`.
    #[must_use]
    pub const fn source(self) -> &'static str {
        match self {
            Icon::Open | Icon::FontFolders => assets::FOLDER,
            Icon::Save => assets::SAVE,
            Icon::Sidebar => assets::SIDEBAR,
            Icon::Comment => assets::COMMENT,
            Icon::ChevronLeft => assets::CHEVRON_LEFT,
            Icon::Back => assets::BACK,
            Icon::ChevronRight => assets::CHEVRON_RIGHT,
            Icon::ChevronDown => assets::CHEVRON_DOWN,
            Icon::Search => assets::SEARCH,
            Icon::ChevronUp => assets::CHEVRON_UP,
            Icon::Close => assets::CLOSE,
            Icon::ZoomOut => assets::ZOOM_OUT,
            Icon::ZoomIn => assets::ZOOM_IN,
            Icon::FitPage => assets::FIT_PAGE,
            Icon::FitWidth => assets::FIT_WIDTH,
            Icon::FitHeight => assets::FIT_HEIGHT,
            Icon::RotateCcw => assets::ROTATE_CCW,
            Icon::RotateCw => assets::ROTATE_CW,
            Icon::Properties => assets::DOCUMENT,
            Icon::Markup => assets::MARKUP,
            Icon::Text => assets::TEXT,
            Icon::EditText => assets::EDIT,
            Icon::AddText => assets::ADD_TEXT,
            Icon::FormField => assets::FORM_FIELD,
            Icon::EditObjects => assets::EDIT_OBJECTS,
            Icon::ShowPoints => assets::SHOW_POINTS,
            Icon::Bookmarks => assets::BOOKMARKS,
            Icon::Layers => assets::LAYERS,
            Icon::Signatures => assets::SIGNATURES,
            Icon::Fonts => assets::FONTS,
            Icon::Measure => assets::RULER,
            Icon::Undo => assets::UNDO,
            Icon::Redo => assets::REDO,
            Icon::Copy => assets::COPY,
            Icon::Tools => assets::TOOL,
            Icon::Keyboard => assets::KEYBOARD,
            Icon::Info => assets::INFO,
            Icon::Pointer => assets::POINTER,
            Icon::ShapeRect => assets::SHAPE_RECT,
            Icon::ShapeEllipse => assets::SHAPE_ELLIPSE,
            Icon::ShapeArrow => assets::SHAPE_ARROW,
            Icon::ShapePolyline => assets::SHAPE_POLYLINE,
            Icon::ShapePolygon => assets::SHAPE_POLYGON,
            Icon::ShapeCloud => assets::SHAPE_CLOUD,
            Icon::ShapeInk => assets::SHAPE_INK,
            Icon::ShapeHighlight => assets::SHAPE_HIGHLIGHT,
            Icon::TextSelect => assets::TEXT_SELECT,
            Icon::TextUnderline => assets::TEXT_UNDERLINE,
            Icon::TextStrikeout => assets::TEXT_STRIKEOUT,
            Icon::TextSquiggly => assets::TEXT_SQUIGGLY,
            Icon::TextFreeText => assets::TEXT_FREETEXT,
            Icon::TextSticky => assets::TEXT_STICKY,
            Icon::Stamp => assets::STAMP,
            Icon::Combine => assets::LINK,
            Icon::Split => assets::SCISSORS,
            Icon::InsertPages | Icon::ImportFormData => assets::UPLOAD,
            Icon::Redact => assets::REDACT,
            Icon::Print => assets::PRINTER,
            Icon::Export => assets::DOWNLOAD,
            Icon::Settings => assets::SETTINGS,
            Icon::InsertImage => assets::IMAGE,
            Icon::SetScale => assets::CONVERT,
            Icon::PageSingle => assets::PAGE_SINGLE,
            Icon::PageContinuous => assets::PAGE_CONTINUOUS,
            Icon::PageFacing => assets::PAGE_FACING,
            Icon::PageFacingContinuous => assets::PAGE_FACING_CONTINUOUS,
            Icon::ZoomRegion => assets::ZOOM_REGION,
            Icon::ZoomSelection => assets::ZOOM_SELECTION,
            Icon::Cut => assets::CUT,
            Icon::Paste => assets::PASTE,
            Icon::Cursor => assets::CURSOR,
            Icon::CursorNode => assets::CURSOR_NODE,
            Icon::Hand => assets::HAND,
            Icon::Rulers => assets::RULERS,
            Icon::Grid => assets::GRID,
            Icon::Guides => assets::GUIDES,
            Icon::Pages => assets::PAGES,
            Icon::Forms => assets::FORMS,
            Icon::ReadMode => assets::READ_MODE,
            Icon::Fullscreen => assets::FULLSCREEN,
            Icon::FloatingPanels => assets::FLOATING_PANELS,
            Icon::ResetLayout => assets::RESET_LAYOUT,
            Icon::Delete => assets::DELETE,
            Icon::PageExtract => assets::PAGE_EXTRACT,
            Icon::FormFlatten => assets::FORM_FLATTEN,
            Icon::ManageList => assets::LIST,
            Icon::PickText => assets::PICK_TEXT,
            Icon::PickPath => assets::PICK_PATH,
            Icon::PickPart => assets::PICK_PART,
            Icon::PickFormXObject => assets::PICK_FORM_XOBJECT,
            Icon::PickLink => assets::PICK_LINK,
        }
    }

    /// The stable key this icon answers to.
    ///
    /// Two jobs, and they are the same string on purpose:
    ///
    /// 1. **It is the application's icon key**, the thing a command names
    ///    with `.with_icon("…")` and the thing `egui-shell` hands back in
    ///    `IconRequest::key`. The shell never interprets it — an icon set is
    ///    a licensing and rasterization decision, which is the application's
    ///    business — so this is the only place the vocabulary is defined.
    /// 2. **It is the texture's debug name.** egui keys textures by handle,
    ///    not by name, so that part is purely for debuggers and texture
    ///    inspectors — but a texture list full of "icon" tells you nothing,
    ///    and one full of `icon:rotate-ccw@32:Bold` tells you everything.
    ///
    /// Kebab-case throughout, matching the command ids and the asset
    /// filenames it was salvaged from.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Icon::Open => "open",
            Icon::Save => "save",
            Icon::Sidebar => "sidebar",
            Icon::Comment => "comment",
            Icon::Close => "close",
            Icon::ChevronLeft => "chevron-left",
            Icon::Back => "back",
            Icon::ChevronRight => "chevron-right",
            Icon::ChevronDown => "chevron-down",
            Icon::Search => "search",
            Icon::ChevronUp => "chevron-up",
            Icon::ZoomOut => "zoom-out",
            Icon::ZoomIn => "zoom-in",
            Icon::FitPage => "fit-page",
            Icon::FitWidth => "fit-width",
            // ui-text-exempt: icon key, never displayed
            Icon::FitHeight => "fit-height",
            Icon::RotateCcw => "rotate-ccw",
            Icon::RotateCw => "rotate-cw",
            Icon::Properties => "properties",
            Icon::Markup => "markup",
            Icon::Text => "text",
            Icon::EditText => "edit-text",
            Icon::AddText => "add-text",
            Icon::FormField => "form-field",
            Icon::EditObjects => "edit-objects",
            Icon::ShowPoints => "show-points",
            Icon::Bookmarks => "bookmarks",
            Icon::Layers => "layers",
            Icon::Signatures => "signatures",
            Icon::Fonts => "fonts",
            Icon::Measure => "measure",
            Icon::Undo => "undo",
            Icon::Redo => "redo",
            Icon::Copy => "copy",
            Icon::Tools => "tools",
            Icon::Keyboard => "keyboard",
            Icon::Info => "info",
            // ui-text-exempt: icon asset key, never displayed
            Icon::Pointer => "pointer",
            Icon::ShapeRect => "shape-rect",
            Icon::ShapeEllipse => "shape-ellipse",
            Icon::ShapeArrow => "shape-arrow",
            Icon::ShapePolyline => "shape-polyline",
            Icon::ShapePolygon => "shape-polygon",
            // ui-text-exempt: icon asset key, never displayed
            Icon::ShapeCloud => "shape-cloud",
            Icon::ShapeInk => "shape-ink",
            Icon::ShapeHighlight => "shape-highlight",
            Icon::TextSelect => "text-select",
            Icon::TextUnderline => "text-underline",
            Icon::TextStrikeout => "text-strikeout",
            Icon::TextSquiggly => "text-squiggly",
            Icon::TextFreeText => "text-freetext",
            Icon::TextSticky => "text-sticky",
            Icon::Stamp => "stamp",
            Icon::Combine => "combine",
            Icon::Split => "split",
            Icon::InsertPages => "insert-pages",
            Icon::ImportFormData => "import-form-data",
            Icon::FontFolders => "font-folders",
            Icon::Redact => "redact",
            Icon::Print => "print",
            Icon::Export => "export",
            Icon::Settings => "settings",
            Icon::InsertImage => "insert-image",
            Icon::SetScale => "set-scale",
            Icon::PageSingle => "page-single",
            Icon::PageContinuous => "page-continuous",
            Icon::PageFacing => "page-facing",
            Icon::PageFacingContinuous => "page-facing-continuous",
            Icon::ZoomRegion => "zoom-region",
            Icon::ZoomSelection => "zoom-selection",
            Icon::Cut => "cut",
            Icon::Paste => "paste",
            Icon::Cursor => "cursor",
            Icon::CursorNode => "cursor-node",
            Icon::Hand => "hand",
            Icon::Rulers => "rulers",
            Icon::Grid => "grid",
            Icon::Guides => "guides",
            Icon::Pages => "pages",
            Icon::Forms => "forms",
            Icon::ReadMode => "read-mode",
            Icon::Fullscreen => "fullscreen",
            Icon::FloatingPanels => "floating-panels",
            Icon::ResetLayout => "reset-layout",
            Icon::Delete => "delete",
            Icon::PageExtract => "page-extract",
            Icon::FormFlatten => "form-flatten",
            Icon::ManageList => "list",
            // ui-text-exempt: diagnostic/lookup keys, matched by ui-verify and
            // by `from_key`; never rendered.
            Icon::PickText => "pick-text",
            // ui-text-exempt: diagnostic/lookup key, never rendered.
            Icon::PickPath => "pick-path",
            // ui-text-exempt: diagnostic/lookup key, never rendered.
            Icon::PickPart => "pick-part",
            // ui-text-exempt: diagnostic/lookup key, never rendered.
            Icon::PickFormXObject => "pick-form-xobject",
            // ui-text-exempt: diagnostic/lookup key, never rendered.
            Icon::PickLink => "pick-link",
        }
    }

    /// Resolve an application icon key back to an [`Icon`].
    ///
    /// This is the lookup [`super::paint_ribbon_icon`] performs on every
    /// icon-bearing ribbon control, every frame.
    ///
    /// # Why a linear scan and not a `match` or a `HashMap`
    ///
    /// A reverse `match` would be a second copy of the key vocabulary, and
    /// two copies of a mapping is exactly how a rename lands in one of them.
    /// [`Icon::name`] stays the single source of truth and this walks it.
    ///
    /// The cost is one pointer-length comparison per catalogue entry
    /// ([`Icon::ALL`]`.len()`) with an early exit, for the
    /// handful of icons a ribbon draws per frame — comfortably under a
    /// microsecond, against a frame budget of 16 ms. A `HashMap` would need
    /// a lazily-initialised static, would hash the key anyway, and would buy
    /// nothing measurable. If the set ever reaches the hundreds, revisit;
    /// `every_name_round_trips_through_from_key` makes the swap safe.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|icon| icon.name() == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// ★ [`Icon::ALL`] must really be all of them.
    ///
    /// Everything catalogue-wide — "every asset parses", "every asset
    /// rasterizes to something visible", "redaction is the only filled one"
    /// — iterates `ALL`. A variant left out of it is therefore not merely
    /// untested: it is *silently* untested, and a broken asset behind it
    /// ships green.
    ///
    /// There is no reflection in Rust to count enum variants, so this checks
    /// the two things that would actually go wrong: a duplicate entry (a
    /// copy-paste that hid the variant it was meant to add) and a count that
    /// no longer matches the number of distinct keys.
    #[test]
    fn all_is_exhaustive_and_free_of_duplicates() {
        let unique: HashSet<Icon> = Icon::ALL.iter().copied().collect();
        assert_eq!(
            unique.len(),
            Icon::ALL.len(),
            "Icon::ALL contains a duplicate variant"
        );
        // 47 until 2026-08-14, when the pass that filled the ribbon's
        // remaining text buttons added 25 — and 76 until later the same day,
        // when the three unblocked Phase 6 markup kinds added `shape-polyline`,
        // `shape-polygon` and `shape-ink`. If this fails, the fix is not to
        // edit the number: it is to check that the variant you added really is
        // in `ALL`, and only then to update this count.
        //
        // ★ This comment used to also say "and update the two prose figures
        // that quote it". That instruction was followed exactly once. On
        // 2026-08-21 the count here was 86 while both of those paragraphs
        // still said 82 — the drift the instruction existed to prevent,
        // committed by the instruction's own readers, twice.
        //
        // So the paragraphs no longer carry a number. `from_key` now says
        // "one comparison per catalogue entry" and `super::cache` says "one
        // entry per icon per weight", both of which are true at every size
        // the set will ever be. THIS assertion is the only figure left, and it
        // is in a test, where drift fails the build instead of misinforming a
        // reader. Prefer that shape for any future count.
        assert_eq!(
            Icon::ALL.len(),
            93,
            "the catalogue changed size: add the new variant to Icon::ALL and update this count"
        );
    }

    /// Every key is unique. Two icons answering to one key would make
    /// [`Icon::from_key`] return whichever came first in `ALL`, which is a
    /// silently-wrong glyph rather than a missing one — the worse failure.
    #[test]
    fn every_name_is_distinct() {
        let mut seen: HashSet<&str> = HashSet::new();
        for &icon in Icon::ALL {
            assert!(
                seen.insert(icon.name()),
                "duplicate icon key '{}'",
                icon.name()
            );
        }
    }

    /// ★ The key vocabulary has exactly one definition.
    ///
    /// [`Icon::from_key`] is documented as the inverse of [`Icon::name`].
    /// This is what keeps that true if `from_key` is ever rewritten as a
    /// `match` or a map for speed.
    #[test]
    fn every_name_round_trips_through_from_key() {
        for &icon in Icon::ALL {
            assert_eq!(
                Icon::from_key(icon.name()),
                Some(icon),
                "'{}' did not round-trip",
                icon.name()
            );
        }
    }

    /// An unknown key resolves to nothing rather than to something plausible.
    ///
    /// The whole missing-icon story downstream ([`super::super::paint`])
    /// depends on this returning `None` instead of guessing at a nearest
    /// match: a fuzzy resolver would draw the *wrong* glyph for a typo,
    /// which is undetectable, where `None` is drawn as a visible mark and
    /// traced.
    #[test]
    fn an_unknown_key_resolves_to_nothing() {
        assert_eq!(Icon::from_key("no-such-icon"), None);
        assert_eq!(Icon::from_key(""), None);
        // Case and separator variants are NOT accepted: the vocabulary is
        // kebab-case, exactly, and a near-miss should be reported rather
        // than silently repaired.
        assert_eq!(Icon::from_key("Open"), None);
        assert_eq!(Icon::from_key("fit_page"), None);
    }

    /// Keys are kebab-case with no surprises, because they appear verbatim
    /// in command definitions that a human types by hand.
    #[test]
    fn keys_are_lowercase_kebab_case() {
        for &icon in Icon::ALL {
            let name = icon.name();
            assert!(!name.is_empty(), "{icon:?} has an empty key");
            assert!(
                name.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
                "icon key '{name}' is not lowercase kebab-case"
            );
        }
    }

    /// The roles that deliberately share one asset still share it, and nothing
    /// else accidentally does.
    ///
    /// Asset sharing is a real decision (one glyph, two places it appears,
    /// never simultaneously) but an *accidental* share means two controls that
    /// should be distinguishable are not — which reads to an operator as a
    /// wiring bug in whichever control they clicked second.
    ///
    /// # ★★ The second pair, added 2026-08-27, and why the test's shape is what
    /// made it a decision
    ///
    /// This asserted an **exact list** of one pair, so adding a second could
    /// not be done quietly — which is the whole point of writing it this way
    /// rather than as "sharing is allowed". The argument had to be made:
    ///
    /// `import-form-data` shares `insert-pages`' upload arrow. Both are
    /// *"something comes in from a file"*, they are on **different tabs**
    /// (File ▸ Export and Pages) so they are never drawn together, and the only
    /// alternative was no icon at all — which would leave one control in a
    /// two-control group bare, and `super`'s own header records what that looks
    /// like: *"47 named and 41 bare with no rule behind which was which, so a
    /// band drew pictures and words side by side and the ribbon read as
    /// half-finished because it was."*
    ///
    /// ★ Drawing new art was never the option. `icons/assets/PROVENANCE.md`
    /// declares that directory the **operator's own work**, which is what
    /// exempts it from `check-shipped-assets`, and a machine-drawn SVG would
    /// make that note false.
    ///
    /// ★★ What was refused: keying it to `insert-pages` itself. A shared *key*
    /// says *two controls about one thing*, and inserting pages and importing
    /// form data have nothing in common but a direction — a pages-named key on
    /// a form command is the near-miss reuse this catalog's refusal table
    /// exists to prevent.
    /// Every pair of icons permitted to share one asset, with the argument for
    /// each in [`only_the_documented_assets_are_shared`]'s doc comment.
    const SHARED_PAIRS: &[&[&str]] = &[
        &["font-folders", "open"],
        &["import-form-data", "insert-pages"],
    ];

    #[test]
    fn only_the_documented_assets_are_shared() {
        let mut by_source: std::collections::HashMap<&str, Vec<Icon>> =
            std::collections::HashMap::new();
        for &icon in Icon::ALL {
            by_source.entry(icon.source()).or_default().push(icon);
        }
        for (_, icons) in by_source {
            if icons.len() > 1 {
                let mut names: Vec<&str> = icons.iter().map(|i| i.name()).collect();
                names.sort_unstable();
                assert!(
                    SHARED_PAIRS.contains(&names.as_slice()),
                    "an unexpected pair of icons shares one asset: {names:?}. Sharing is \
                     permitted and is a DECISION — add the pair to `SHARED_PAIRS` with \
                     the argument for it, in this test's own doc comment"
                );
            }
        }
    }

    /// Every variant has non-empty art. A `source()` arm wired to the wrong
    /// (or an empty) constant would otherwise only show up as a blank
    /// button.
    #[test]
    fn every_icon_has_source_text() {
        for &icon in Icon::ALL {
            let src = icon.source();
            assert!(
                src.contains("<svg"),
                "icon '{}' has no <svg> root in its source",
                icon.name()
            );
        }
    }
}
