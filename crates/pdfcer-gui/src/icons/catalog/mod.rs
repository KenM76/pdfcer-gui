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

// ★ The mapping lives next door. `Icon::ALL`, `Icon::source` and `Icon::name`
// are three total functions over this enum; a new variant must join all three
// lists, and they are kept adjacent to each other rather than adjacent to the
// enum so that the "did I add it everywhere" check is one file.
mod mapping;
mod tests;
