//! ★ **This module's documentation lives in `OVERVIEW.md`** beside this file,
//! included below — moved there 2026-09-04 for the reason `app::actions`
//! records for the same move: *"the bulk is prose"*, and prose in `//!`
//! comments spends R2's line budget without giving a compiler anything to
//! check. **That file also carries the argument for the next split**, which
//! this one is only a reprieve from: treat this file as full.
#![doc = include_str!("OVERVIEW.md")]

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
    /// **File ▸ Security ▸ Sign…** — authoring a digital signature.
    ///
    /// A page with a cursive stroke on a signing rule. It inherits
    /// [`Icon::Signatures`]' constraint word for word — **not** a seal, badge,
    /// shield or checkmark — and the reason is stronger here: this control is
    /// the one that *makes* a signature, so a glyph promising a verdict would
    /// promise it about a signature that does not exist yet. The page
    /// silhouette is [`Icon::Encrypt`]'s. Full argument in `sign.svg`.
    Sign,
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
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/text-select.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
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
    /// of the fill path (see `fill_is_semantic_and_the_set_that_uses_it_is_closed`,
    /// renamed from `redaction_is_the_only_filled_icon` on 2026-08-19). The fill
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

    /// Edit ▸ Apply redactions (`edit.redact_apply`) — the one irreversible
    /// command in the redaction family.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/apply-redactions.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ApplyRedactions,

    /// Attachments (`edit.attachments`) — the files this document carries
    /// inside itself, which appear on no page.
    ///
    /// ★ The ruling — what the glyph depicts, and which neighbours it
    /// must not be mistaken for — lives in `assets/attachment.svg`, beside the art
    /// it constrains. Migrated there 2026-09-11 under the seam `OVERVIEW.md`
    /// names: the enum is a list of roles, the asset carries its own rules.
    Attachment,

    /// Complete the gesture in progress — `markup.finish` and `measure.finish`.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/check.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
    ///
    /// ★ This variant **retires two recorded refusals**, which were written in the
    /// same sentence at both registrations: *"There is no check-mark, tick or
    /// accept glyph in the set, and no existing key means 'complete this
    /// gesture'."* That was true of the catalogue and is now spent — the same
    /// shape of correction [`Icon::Pages`] records. Both notes must be rewritten
    /// at their registrations rather than left standing as though their premise
    /// still held.
    ///
    /// A bare tick with two unequal limbs — a short down-stroke and a long
    /// up-stroke. Distinct from [`Icon::ChevronRight`], whose limbs are equal and
    /// symmetric about its vertex, and from [`Icon::ShapeArrow`], which has a
    /// head. **The asymmetry is the cue**; evened out, this reads as a bent arrow
    /// or a `>` at 16 px.
    Accept,

    /// Place a **check box** — one independent on/off box.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/check-box.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    CheckBox,

    /// Close every open document except one — `view.close_other_documents`.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/close-others.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
    ///
    /// ★ Adopting this retires half of a recorded refusal rather than ignoring it.
    /// The registration declines an icon on the ground that a context-menu row's
    /// glyph is decoration — sound for the menu, and the menu may still draw none.
    /// But `manifest::view` also places this command on the View ▸ Window ribbon,
    /// between two iconed neighbours, because
    /// `every_menu_command_is_also_reachable_from_the_ribbon` holds that a
    /// right-click-only command is undiscoverable. On a ribbon the un-iconed row is
    /// the odd one out, which is the same defect the refusal was avoiding. Rewrite
    /// the note; do not leave it standing as though still load-bearing.
    CloseOthers,

    /// A tree row whose children are **showing** — press to hide them.
    ///
    /// The other half of [`Icon::Expand`], and the half that forced the
    /// workaround it retires: `▼` U+25BC is the glyph the shipped font stack
    /// cannot draw, which is why the Bookmarks panel takes both its triangles
    /// from `emoji-icon-font`'s `⏴⏵⏶⏷` block instead of the Geometric Shapes pair
    /// every style guide names.
    ///
    /// Distinct from [`Icon::ChevronDown`], which is a menu-disclosure marker on
    /// a dropdown *button*: that is an open chevron, this is a closed triangle,
    /// and the closing edge is the cue. Two different controls, two different
    /// promises — a chevron says "this button opens something below it", a
    /// triangle says "this row contains these rows".
    ///
    /// Adopt it with [`Icon::Expand`] or not at all. Half a pair reintroduces
    /// exactly the failure `text::panels::bookmarks` names: a state and a
    /// rendering fault that look the same.
    Collapse,

    /// Copy the whole document's text to the clipboard — `file.copy_document_text`.
    ///
    /// Three square-on sheets cascading down-left, only the topmost complete and
    /// carrying two short text rules. Three sheets is *the whole file*, and the
    /// rules on the top one alone say text is being taken from all of them rather
    /// than typed onto one.
    ///
    /// Distinct from [`Icon::CopyPageText`], which it currently shares a key with,
    /// by COUNT and by RULE LENGTH: two sheets and three full-width rules is one
    /// page's text; three sheets and two short rules is the file's.
    ///
    /// Distinct from [`Icon::Pages`] — also three square-on sheets — by the text
    /// rules, which that glyph deliberately omits because the Pages panel is about
    /// sheets as objects, not about what is printed on them. Distinct from
    /// [`Icon::Layers`] by being square-on rather than ISOMETRIC, which is
    /// `pages.svg`'s own stated rule: a layer is a plane you look through, a page
    /// is a thing you look at.
    CopyDocumentText,

    /// Copy this page's text to the clipboard — `file.copy_page_text`.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/copy-page-text.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
    ///
    /// Distinct from [`Icon::CopyDocumentText`] by COUNT, deliberately, because
    /// the two sit adjacent in the same group and differ only in scope: two sheets
    /// with the front one fully ruled = one page's text; three cascading sheets
    /// with only the top ruled = the whole file's.
    CopyPageText,

    /// Dimension groups — `measure.manage_groups`, and the caption's dock tab
    /// (`crate::panels::Panel::DimensionGroups`) with it.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/dimension-groups.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    DimensionGroups,

    /// Switch to the next open document — `view.next_document` (Ctrl+Tab).
    ///
    /// [`Icon::PreviousDocument`] mirrored about x=24: a page with a
    /// right-pointing shafted arrow beside it. Mirroring rather than redrawing is
    /// the convention every navigation pair in this set follows —
    /// `chevron-right.svg`'s entire comment is "mirror of chevron-left.svg", and
    /// `upload.svg`/`download.svg` are described as exact mirrors about y=24.
    ///
    /// Replaces a borrow of [`Icon::ChevronRight`] (documented "Next page"), for
    /// the reason recorded on its twin: a bare chevron is reserved for a STEP
    /// through pages, and switching files is a jump, which `back.svg` rules earns
    /// a shaft.
    NextDocument,

    /// Switch to the previous open document — `view.previous_document`
    /// (Ctrl+Shift+Tab).
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/document-previous.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
    ///
    /// Distinct from [`Icon::Back`], the set's other shafted left arrow, by the
    /// page: Back leaves a surface with nothing else in frame, this carries the
    /// thing being switched to. Distinct from [`Icon::NextDocument`] only by
    /// mirroring, which is the convention every navigation pair in this set
    /// follows.
    PreviousDocument,

    /// Place a **drop-down** (the `/Ch` choice field).
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/drop-down.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
    ///
    /// Authored 2026-09-04 out of the same five-way share as [`Icon::CheckBox`].
    DropDown,

    /// Embed the font programs a document references but does not carry
    /// (`tools.embed_fonts`).
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/embed-fonts.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
    ///
    /// Deliberately not from the I-beam family ([`Icon::AddText`],
    /// [`Icon::TextSelect`]) and not [`Icon::EditText`]'s pencil: those act on
    /// the *words*, and this acts on the *faces the words are drawn in*.
    EmbedFonts,

    /// A tree row whose children are **hidden** — press to reveal them.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/expand.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
    ///
    /// ★ This variant **retires a font workaround**, the way [`Icon::Pages`]
    /// retired a recorded refusal. The Bookmarks panel draws `⏵` U+23F5 today
    /// only because the pair it actually wanted is half missing: `▶` U+25B6 draws
    /// in the shipped stack and `▼` U+25BC does not, so a collapsed row would
    /// show a triangle and an expanded one a substitution box — and the missing
    /// glyph would read as a *state* rather than as a defect. The workaround was
    /// to take both halves from one emoji face. Authoring the pair is the
    /// operator's standing ruling applied (a missing glyph is AUTHORED, not
    /// worked around) and removes the font dependency from a control's two
    /// states entirely.
    ///
    /// Distinct from [`Icon::ChevronRight`], which is Next Page: that is an OPEN
    /// chevron, two strokes meeting at a point with no closing edge. This is a
    /// CLOSED triangle, which is the tree-disclosure convention in every file
    /// manager, IDE and sidebar the operator uses. **The closing edge is the only
    /// cue**, so it may not be dropped in a later tidy-up.
    Expand,

    /// Markup ▸ Finish shape (`markup.finish`) — commit the vertex run a
    /// Polyline or Polygon gesture has laid down. The same glyph answers
    /// `measure.finish`, which is its twin in every respect that matters.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/finish-shape.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    FinishShape,

    /// A row the **document** forbids changing — an optional-content group
    /// carrying `/Locked` (ISO 32000-1 Table 101), or a `/Ff` read-only form
    /// field.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/lock.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    Locked,

    /// Two-line measurement — `measure.two_line`.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/measure-angle.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    MeasureAngle,

    /// Path-length measurement — `measure.length`.
    ///
    /// A meandering **open** run with a short upright tick standing off each end:
    /// "this thing, from here to here, is how long", which is what the tooltip
    /// promises for a pipe, a cable or a kerb line.
    ///
    /// ★ Its dangerous neighbour is [`Icon::ShapeInk`] — one irregular flowing
    /// stroke spanning the tile, no baseline, no periodicity, which describes
    /// both glyphs. The entire difference is the **two terminator ticks**.
    /// Freehand ink has no ends worth marking; a measured run is bounded, and
    /// the ticks sit outside the curve's own endpoints the way real extension
    /// lines do, so they read as measurement furniture rather than stray marks.
    ///
    /// Open where [`Icon::MeasurePerimeter`] closes, and the two sit side by side
    /// on the Measure tab, so that contrast is the first thing an operator sees.
    /// Curved where [`Icon::ShapePolyline`] is angular, because the vertices are
    /// what that glyph is about and a cable run has none.
    MeasureLength,

    /// Perimeter measurement — `measure.perimeter`.
    ///
    /// An irregular **closed** quadrilateral drawn **dashed**, and both halves
    /// carry meaning. Closed is the word on the label: `MeasureKind::PathLength`
    /// is a separate control precisely because "Perimeter" promises a ring, so
    /// the closure is the distinction from [`Icon::MeasureLength`]'s open run.
    ///
    /// ★ Dashed is the distinction from [`Icon::ShapePolygon`], which is an
    /// irregular closed outline in solid stroke. That one means "an annotation
    /// you author by clicking corners"; this one means "a route traced round
    /// something already on the page". A measurement path is not ink the
    /// document keeps, and at 16 px the broken line is the only cue that says
    /// so — the corner count (four against five) is not legible at that size.
    ///
    /// The only asset in the set that uses `stroke-dasharray`; see
    /// `icons::svg`'s note on why that attribute is parsed rather than ignored.
    MeasurePerimeter,

    /// Radius / diameter measurement — `measure.radius_diameter`.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/measure-radius.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
    ///
    /// ★ Its dangerous neighbour is [`Icon::ShapeEllipse`], whose bare circle is
    /// this one's outer ring to within a unit. The ring cannot carry the
    /// difference, so both cues are interior: the **centre dot**, which a markup
    /// ellipse has no reason to draw, and the **spoke**, which is the radius
    /// itself and also stops dot-inside-ring reading as a radio button — the
    /// failure [`Icon::ManageList`] records against the `icon-ring.svg` the
    /// ui-spec proposed here. Distinct from [`Icon::SetScale`]'s round glyph by
    /// being a closed ring with an interior, where that one is two arrowed arcs
    /// with an empty middle.
    ///
    /// Replaces this command's share of [`Icon::Measure`]'s ruler, which stays
    /// with `measure.linear`.
    MeasureRadius,

    /// Merge another file's pages INTO the open document (`pages.merge_into`).
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/merge.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    MergeInto,

    /// New (blank) document — `file.new`.
    ///
    /// A page with a folded top-right corner and a plus at its optical centre.
    /// This is the glyph `shell::commands::catalog::file`'s `file.new` note calls
    /// "the operator's to draw", and the three reuses that note refuses by name
    /// are exactly the three neighbours it must stay apart from:
    /// [`Icon::Properties`] (`document.svg`) is the same square-on page but with
    /// three text rules and no fold — it means *the file already open*;
    /// [`Icon::InsertPages`] is a tray with an arrow going IN; and [`Icon::Save`]
    /// shares the cut-corner body on purpose and separates on the interior mark —
    /// one label slot there, a crossed plus here.
    ///
    /// Distinct from [`Icon::NewFromTemplate`], its ribbon neighbour, by the SOLID
    /// plus against that one's dashed placeholder frame: a plus is "empty and
    /// yours to fill", a dashed box is "something is already laid out here".
    New,

    /// New document from a template — `file.new_from_template`.
    ///
    /// The same folded-corner page as [`Icon::New`], carrying a DASHED rectangle
    /// where that one carries a solid plus. The shared body is deliberate: the
    /// registration argues the two New controls belong together, and a shared
    /// silhouette is how a ribbon says so. The dash is the whole distinction, and
    /// it is load-bearing — solid plus means "empty, yours to fill", a dashed
    /// frame means "a layout is already here and you will fill it in".
    ///
    /// Deliberately not a second sheet behind the page: that is [`Icon::Copy`]'s
    /// and [`Icon::CopyPageText`]'s vocabulary and would say this command
    /// duplicates something already open, which is precisely what it does not do —
    /// a template is on disk, not in the window.
    NewFromTemplate,

    /// Place a **push button** — the `/Btn` field with no on/off state.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/push-button.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    PushButton,

    /// Put the armed tool down — the Tool panel's row 4.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/put-down.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    PutDown,

    /// Place a **radio button** — one of a mutually exclusive set.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/radio-button.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
    ///
    /// Authored 2026-09-04, breaking the same five-way share as
    /// [`Icon::CheckBox`]: this command drew [`Icon::FormField`], two items from
    /// [`Icon::ManageList`] in the same ribbon group.
    ///
    /// ★ **`list.svg` reserved this shape for it years before it existed.** That
    /// asset's recorded deviation from ui-spec §8.2 refuses `icon-ring.svg`
    /// because "at 16 px two concentric circles read as a target or a radio
    /// button — neither of which is a list of named things". The deviation is now
    /// load-bearing in both directions, and the two glyphs are genuinely adjacent.
    RadioButton,

    /// Recently-opened documents — `file.recent`, the menu button in File ▸ File.
    ///
    /// A clock face: a closed circle with an hour and a minute hand. Time is the
    /// one thing every entry in that list has in common, and a clock is the shape
    /// that says it without a label.
    ///
    /// Deliberately **not** [`Icon::Open`]'s folder — the registration's own
    /// refusal is that reusing it "would make two adjacent controls in one band
    /// look like one control drawn twice", and Open sits immediately beside this.
    ///
    /// Distinct from [`Icon::Undo`], the other time-flavoured glyph, by being a
    /// CLOSED ring with hands: undo is an open ~270° arc with an arrowhead and no
    /// interior. That difference is a claim, not decoration — an arrow says
    /// "go back and change what happened", and this command changes nothing; it
    /// only reopens. Distinct from [`Icon::Info`] by having hands rather than a
    /// dot-and-stem inside the ring.
    Recent,

    /// Recognise text (OCR) — `file.ocr`.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/recognise-text.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
    ///
    /// The near neighbour is [`Icon::Fonts`], which is also a capital A on a rule.
    /// Two cues separate them and both matter: this A is INSIDE a page outline,
    /// and its rule is DASHED where the Fonts rule is solid. `fonts.svg`'s own
    /// note explains why its baseline is solid — a solid rule turns "a letter"
    /// into "a typeface", and that panel only reports. This command writes a text
    /// layer into the file, so it may not borrow the glyph of a surface that
    /// changes nothing.
    RecogniseText,

    /// Edit ▸ Check for content off the sheet (`edit.offpage`) — the census of
    /// everything drawn outside a page boundary, and the fourth member of the
    /// redaction family.
    ///
    /// ★★★ **It does not borrow the family's solid bar, and that is the point.**
    /// [`Icon::Redact`], [`Icon::RedactSelection`] and [`Icon::ApplyRedactions`]
    /// share a filled rectangle because all three are about content being
    /// removed permanently; [`tests::fill_is_semantic_and_the_set_that_uses_it_is_closed`]
    /// enumerates them and fails on any other filled glyph. This command
    /// **reports** — it walks the document and says what it found — so a filled
    /// version would promise destruction in the cue that resolves first at
    /// 16 px. Stroke-only, and it must stay that way.
    ///
    /// What it draws: a sheet, with one rule crossing its left edge and a second
    /// sitting entirely clear of it — the engine's `OffPage::Partial` and
    /// `OffPage::Fully` in one picture. The **crossing** is what makes it
    /// unreadable as a two-pane layout glyph.
    OffPage,

    /// Edit ▸ Redact selection (`edit.redact_selection`) — mark whatever is
    /// selected, in one action.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/redact-selection.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
    ///
    /// It borrowed [`Icon::Redact`] until now, which drew the button that ARMS
    /// a marking tool and the button that marks A SELECTION as the same
    /// picture, three buttons apart on the same tab. The redaction family is
    /// not one verb over three operands — it is arm, mark, obliterate — so it
    /// is the wrong place for the shared-glyph convention.
    RedactSelection,

    /// Reflow paragraph (`edit.reflow_block`) — re-wrap the paragraph the caret
    /// is in so its lines fill their box again.
    ///
    /// ★ The ruling — what the glyph depicts, and which neighbours it
    /// must not be mistaken for — lives in `assets/reflow.svg`, beside the art
    /// it constrains. Migrated there 2026-09-11 under the seam `OVERVIEW.md`
    /// names: the enum is a list of roles, the asset carries its own rules.
    Reflow,

    /// Report how the page was actually drawn (`tools.render_diagnostics`).
    ///
    /// A folded page carrying a measurement trace across it. It replaces a
    /// borrowed [`Icon::Tools`] wrench, and the reason is the set's standing one:
    /// **an icon is a claim.** A wrench says *adjust this*; this command adjusts
    /// nothing — it reports what the renderer had to substitute or leave out.
    /// The same argument that keeps [`Icon::Fonts`] from being a pencil.
    ///
    /// Distinct from [`Icon::Text`]'s folded page by the trace, which is the only
    /// thing on it that is not typography; distinct from [`Icon::Properties`]
    /// because Properties answers *what the document records* and this answers
    /// *what the drawing cost*.
    RenderDiagnostics,

    /// Save As — `file.save_as` (`OPERATOR_REQUESTS.md` O95).
    ///
    /// The save body with a pencil laid across its lower-left field. Save As is
    /// Save plus *you name it*, so the glyph is Save plus the set's existing mark
    /// for authoring.
    ///
    /// Three separations, all deliberate. From [`Icon::Save`]: that one is a bare
    /// body with a single label slot and no instrument over it — the plain,
    /// fifty-times-a-day press. From [`Icon::EditText`] (`edit.svg`): that is a
    /// full-size standalone pencil meaning *edit the page's text*; here the pencil
    /// is a small modifier over a body that dominates the frame, and a modifier on
    /// a save body cannot be read as a page-editing tool. From
    /// [`Icon::SaveCopy`]: that one repeats the body, this one marks it — "another
    /// file" versus "this file, renamed".
    SaveAs,

    /// Save compacted — `file.save_compacted`, the copy with unused objects
    /// reclaimed.
    ///
    /// The save body with a downward arrow filling the field beneath the shutter.
    /// Down is "smaller", and putting the arrow INSIDE the body is what keeps the
    /// glyph about the file rather than about a transfer.
    ///
    /// That containment is the load-bearing cue against [`Icon::Export`]
    /// (`download.svg`) and [`Icon::PageExtract`], which are the set's other
    /// downward arrows. Both of those point OUT of something into a tray or away
    /// from a page — `download.svg`'s own note calls the arrow direction "the
    /// family's grammar" for in/out of this document. Nothing leaves here. The
    /// save body encloses the arrow, and enclosure is the difference between
    /// "smaller" and "outbound".
    ///
    /// Distinct from [`Icon::SaveAs`] (pencil) and [`Icon::SaveCopy`] (a second
    /// body) by carrying neither: all three are the same body with one different
    /// thing said about it.
    SaveCompacted,

    /// Save a copy — `file.save_copy`.
    ///
    /// Two save bodies, offset: a full one in front with its shutter and label
    /// field, and the outline of a second behind it. The doubling is the whole
    /// message — a copy is a second file, and the front body keeps the shutter so
    /// the pair still reads as the save family rather than as a generic duplicate.
    ///
    /// Distinct from [`Icon::Copy`], which is two blank rounded rects offset
    /// diagonally: those carry no shutter and no label field, because that glyph
    /// means the CLIPBOARD — take this and hold it. This one writes a file. The
    /// shutter is the cue and it is doing real work; without it the two collapse.
    ///
    /// Distinct from [`Icon::Save`] by the second outline, and from
    /// [`Icon::SaveAs`] by repeating the body rather than marking it: a copy is
    /// another file, Save As is this file under another name.
    SaveCopy,

    /// Remove embedded font programs, leaving the references behind
    /// (`tools.unembed_fonts`).
    ///
    /// [`Icon::EmbedFonts`]' drawing with the frame **dashed** instead of solid:
    /// the container is still named, but what was inside it is gone. The dash is
    /// the entire distinction between the two, and it renders — `icons::svg`
    /// parses `stroke-dasharray` (see `Shape::dash`, added 2026-09-04 naming this
    /// pair as the reason).
    ///
    /// This is the destructive half of the font pair, and until now it drew the
    /// same picture as [`Icon::Fonts`] — the panel that only *lists* faces. A
    /// glyph is a claim, and "identical to the read-only panel" was the wrong one
    /// for the command whose own action module records that unembedding
    /// "genuinely breaks that guarantee".
    UnembedFonts,

    /// The wheel-paging toggle on the status bar — `OPERATOR_REQUESTS.md` O30.
    ///
    /// ★ The ruling for the ART — what the glyph depicts, and which
    /// neighbours it may not be mistaken for — is in `assets/wheel-flip.svg`.
    /// The paragraphs that repeated it here were deleted 2026-09-11 under the
    /// seam `OVERVIEW.md` § "The real seam" names: the enum lists roles, the
    /// asset states its own rules, and a rule stated twice drifts.
    ///
    /// What survives below is the part the asset does not say — the
    /// registration history, and the refusals this variant retired.
    ///
    /// A mouse, and a sheet carrying a forward chevron. Read left to right it is
    /// the sentence the control makes: *the wheel turns the page*. The control
    /// draws its two words today in a bar whose own module documents a fixed 30
    /// point height and a right-hand cluster that must not move
    /// ([`crate::app::status`]), so this is the one place on the surface where a
    /// glyph buys width rather than spending it.
    WheelFlip,
    // ── assets orphaned by breaking their aliases, 2026-09-04 ────────────
    // ★ `Document` rejoined the ribbon on 2026-09-05; `Convert` below is the
    // one still unbound.
    /// A document, as a subject — `document.svg`. **`file.document_properties`
    /// since 2026-09-05.**
    ///
    /// ★ Ruling in `document.svg`: the one-day orphan and what found it, why an
    /// unbound variant's comment must read as an invitation, why the asset was
    /// kept while nothing named it, and the separation from [`Icon::Properties`].
    /// **First variant migrated under `OVERVIEW.md`'s seam** (2026-09-05), to
    /// make room for [`Icon::LineWeights`].
    Document,

    /// Change one form into another — `convert.svg`.
    ///
    /// Orphaned on 2026-09-04 with [`Icon::Document`] and kept for the same
    /// reason; see that variant for the argument.
    ///
    /// It was reached only through [`Icon::SetScale`], and the alias was a
    /// false claim rather than merely a borrowing: setting a scale converts
    /// nothing. It declares what the drawing's units mean. The set's standing
    /// rule is that **an icon is a claim**, and this one was making one its
    /// command could not support.
    Convert,
    // ── five glyphs no control reaches yet, 2026-09-04. One replaces a live
    // borrow; four are art before button ([`Icon::EditObjects`] is the precedent).
    /// Export the page as a raster image — `file.export_image`. A picture tile
    /// (rect, mountain horizon) with an arrow leaving it to the right.
    /// ★ Replaces a LIVE BORROW made hours earlier: the command registered wearing
    /// [`Icon::Export`], defended as one act in three formats — true of DXF and
    /// form data, false of a picture, because [`Icon::InsertImage`] has already
    /// taught the operator what a framed tile with a horizon means here. Distinct
    /// from IT by DIRECTION alone, from [`Icon::Export`] (an empty tray) by naming
    /// its cargo, from [`Icon::PickLink`] by leaving a closed frame horizontally
    /// where that escapes a corner GAP diagonally.
    ExportImage,

    /// Hand this file to the system's PDF viewer. An application WINDOW — a frame
    /// with a title bar — with a document sheet overlapping its lower-left corner
    /// and breaking its outline. No command names it yet.
    /// ⚠ **The label names a vendor; the art carries nothing of that mark** — no
    /// letterform, no badge, no traced shape. ★★ The title bar is the whole
    /// distinction: [`Icon::PickLink`] already claims the box-with-escaping-arrow
    /// by name, so an arrow on a plain frame would draw one sentence for two
    /// meanings. A frame DIVIDED by a rule is an application, not a box something
    /// leaves, and it is the set's only one; the overlap is the "handed to" cue,
    /// the window being an open path so the sheet passes in FRONT of it.
    OpenInAcrobat,

    /// Copy the selection to the clipboard as vector geometry rather than as a
    /// picture of it. **Not built**; the art exists so the proposal can be drawn.
    /// [`Icon::Copy`]'s two offset sheets, a Bezier between two square nodes on
    /// the FRONT one.
    /// The arrangement IS the clipboard family and may not be given up — what
    /// [`Icon::SaveCopy`] refuses to borrow — so every distinguishing mark is
    /// inside the front sheet, answering what lands on the clipboard: the
    /// construction. Distinct from [`Icon::Copy`] by that interior alone (the
    /// curve runs corner to corner so it cannot read as a smudge at 16 px), from
    /// [`Icon::EditObjects`] by having sheets, [`Icon::ShowPoints`] by curving.
    CopyAsVector,

    /// Put a password on this document — the engine's `set_encryption`, **awaiting
    /// the operator's ruling** as O119. A folded page, padlock over its lower half.
    /// ★★★ The page is the entire point. [`Icon::Locked`] is a BARE padlock marking
    /// a ROW the document forbids operating — its asset says "not 'secure', not
    /// 'verified', not 'encrypted'" — and this makes the sentence that one refuses:
    /// the DOCUMENT is what is locked. ⚠ Its constraint travels too: never on a
    /// signature row, because pdfcer performs no cryptographic verification and a
    /// padlock reads as VALIDATED. Distinct from the other folded pages, whose
    /// interiors are all LINEAR, by a chunky closed block; from [`Icon::Redact`]
    /// by being unfilled — nothing here is removed.
    Encrypt,

    /// What the document permits — the engine's `set_permissions`, the other half
    /// of O119 and awaiting the same ruling. A folded-corner page carrying two
    /// ticked rows: a tick, then a rule, twice. `/P` is a bit field, and on screen
    /// a bit field is a list of things with ticks beside them.
    /// ⚠ The ticks mean THIS BOX IS ON, never *verified* — [`Icon::Accept`] carries
    /// the same warning, and a permission setting is a request any program may
    /// ignore. Distinct from [`Icon::ManageList`] on both axes at once: ticks where
    /// that has square markers, a page where that has, by its own note, "no frame
    /// at all". From [`Icon::Accept`] by scale and enclosure, `check-box.svg` by
    /// count, [`Icon::Encrypt`] by the interior alone — one page, two questions.
    Permissions,
    /// Select everything on the page — `edit.select_all`.
    ///
    /// A dashed marquee enclosing the pointer. ★★★ **This variant exists
    /// because a refusal was mistaken for a ruling.** Its absence was argued in
    /// prose by a build session on 2026-09-01, quoted in four places, and had
    /// begun to be reported to the operator as settled. He corrected it on
    /// 2026-09-04: *"I didn't refuse that."*
    ///
    /// ⇒ The lesson is kept beside the art rather than filed away: **a
    /// well-argued refusal written by whoever happened to be building that day
    /// is not an operator decision, and quoting it does not make it one.** The
    /// two are told apart by asking who said it, and the answer belongs in the
    /// sentence.
    ///
    /// The surviving half of the old argument is drawn into the glyph — see
    /// `select-all.svg`'s own comment for why the marquee encloses a pointer
    /// rather than standing alone.
    SelectAll,

    /// Set the swept run bold — `format.bold`. A capital B, stroked HEAVY.
    /// ★ The first pair authored under `OVERVIEW.md`'s seam: the ruling lives in
    /// `bold.svg`, and this is the pointer. Weight, correction, neighbours there.
    Bold,
    /// Set the swept run italic — `format.italic`. A slanted capital I.
    /// ★ Ruling in `italic.svg`: the slant, the offset serifs that clear
    /// [`Icon::TextSelect`], and the refusal both of this pair correct.
    Italic,

    /// Draw every stroke at one device pixel — `view.line_weights`, O137.
    /// Three horizontal bars, thick to thin, the CAD lineweight glyph.
    ///
    /// ★ Ruling in `line-weights.svg`: why this is the ONE asset in the set
    /// that breaks the 2.5-stroke style contract (the varying weight IS the
    /// subject), what keeps it clear of `list.svg`, and the 16 px floor the
    /// thinnest bar was measured against.
    ///
    /// ★★ Authored 2026-09-05 for a control that had been **deleted** — the
    /// operator's standing ruling (2026-08-06, quoted at [`Icon::Back`]) is
    /// that a missing glyph is AUTHORED, never worked around.
    LineWeights,
}

// ★ The mapping lives next door. `Icon::ALL`, `Icon::source` and `Icon::name`
// are three total functions over this enum; a new variant must join all three
// lists, and they are kept adjacent to each other rather than adjacent to the
// enum so that the "did I add it everywhere" check is one file.
mod mapping;
mod tests;
