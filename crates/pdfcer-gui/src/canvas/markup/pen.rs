//! # `canvas::markup::pen` — the colour and width the next markup is authored with
//!
//! ## What this closes
//!
//! `RIBBON_IA.md` §5.5 specifies a **Style** group on the Markup tab —
//! *"Colour · Line width · Fill · Opacity"* — and marks it `partial G`,
//! *"colour only"*, describing the **old** shell. This shell had none of it:
//! `MarkupKind::rgb()` returned a hard-coded red, `PEN_WIDTH_PTS` was a
//! hard-coded `2.0`, and the manifest's `colour_swatch` item was declared and
//! never built, so the Style group rendered an empty caption.
//!
//! §5.5's own note on it is the operator's complaint restated in advance:
//!
//! > The `Style` group sets defaults for the next markup. … Both must exist;
//! > today only the first does, **which is why a placed markup feels final**.
//!
//! ## ★ The seam was already named, and this took it
//!
//! `MarkupKind::rgb`'s doc comment predicted this module almost exactly:
//!
//! > The old shell carried `markup_color`/`markup_width` on the application
//! > and a swatch in its ribbon. This shell has neither … So the pen is a
//! > **default**, stated once, in the one place that builds a spec, and the
//! > seam for a real pen control is exactly this function: **give it a colour
//! > and a width from the document's markup state and nothing else in the
//! > module changes.**
//!
//! That is what happened: `spec` and `action` gained a `Pen` parameter and
//! nothing else in `canvas::markup` moved. The prediction was right down to
//! the shape of the change, which is worth recording — a doc comment that
//! names its own seam is the cheapest refactoring aid this project has.
//!
//! ## Why the pen lives on the application and not on the document
//!
//! A pen is a **tool setting**, not document content. An operator who picks
//! green expects the next rectangle to be green in *whatever* file they draw
//! it in, exactly as a pencil does not change colour when you turn the page.
//! Putting it on `OpenDoc` would reset it on every open, which is the
//! behaviour of a program that has forgotten what you told it.
//!
//! It is deliberately **not persisted** to the settings file, and that is a
//! narrower statement than it sounds. `pdfcer_core::settings` is for choices the
//! *standard* leaves open — its window says so in its own first paragraph —
//! and a pen colour is not an ambiguity, it is a preference. Persisting it
//! belongs with the ribbon layout and the keymap, under the same `userdata/`
//! roof and in their own file, which is `SHELL_FRAMEWORK.md`'s subject rather
//! than this one's.
//!
//! ## ★★★ EIGHT PENS, NOT TWO — and the argument that said otherwise is kept
//! ## here, superseded rather than deleted
//!
//! ### What this section used to say, verbatim
//!
//! > **Two pens, not one, and it is not a per-kind palette.** … What this is
//! > **not** is a colour per markup kind. `MarkupKind::rgb`'s own note argued
//! > that down when three kinds were added: they are comment linework, they have
//! > to be seen against a drawing that is already black on white, and *"a
//! > per-kind palette would be a style decision made in code where the Style
//! > group is the surface that owns it."* One pen for every geometric kind is
//! > the same answer, now made choosable.
//!
//! ### Why that was a good argument
//!
//! It is worth saying plainly, because the correction is only useful if the
//! thing being corrected was reasonable. The argument had two halves and both
//! were sound at the time:
//!
//! 1. **A colour chosen in code is a style decision made in the wrong place.**
//!    True, and still true. A shell that hard-codes a green underline because
//!    somebody liked green has put a preference in a source file where no
//!    operator can reach it.
//! 2. **Comment linework has one job — to be seen over black-on-white CAD.**
//!    Also true. Nothing about a per-kind palette makes an underline more
//!    legible over a drawing than the pen colour would.
//!
//! ### ★★ Why it is superseded anyway
//!
//! Because it answered the wrong question. It asked *"can this shell justify
//! inventing eight colours?"* — and the answer to that is still no. The
//! operator's ask of 2026-09-06 asks something else:
//!
//! > *"Also make sure you've used the same default colours and style look for
//! > these things as Adobe."*
//!
//! ⇒ The values are no longer this shell's to choose. [`super::palette`] reads
//! them out of **Acrobat's own tool-defaults store**, and Acrobat does not use
//! one colour for everything: its highlighter is orange, its underline is blue,
//! its strikeout is a light red that is not the shape red, and its sticky note
//! is violet. A single pen cannot express that table, so the table is the shape
//! the pen has to have. The style decision is not being made in code — it is
//! being **transcribed from the program the operator compares against**, which
//! is this project's standing tie-breaker for anything of this kind.
//!
//! ★ Half of the old argument survives intact and is worth keeping: the values
//! must still be the operator's to override, and every slot below is. What
//! changed is only where the *shipped* value comes from.
//!
//! ### The slots, and why they are named rather than an array
//!
//! [`PenSlot`] has one variant per **key Acrobat keeps a separate default
//! under** — not per [`MarkupKind`] variant, and the difference matters in both
//! directions:
//!
//! * The seven geometric kinds share `cSquare`/`cCircle`/`cLine`/… , all holding
//!   the identical red, so they share [`Pen::ink`]. Giving each its own field
//!   would be seven copies of one number and seven places for it to drift.
//! * Squiggly and Stamp hold that same red **under their own registry keys**, so
//!   they get their own slots even though the shipped values agree today. An
//!   operator who recolours the shape pen has not thereby asked for a recoloured
//!   squiggly — Acrobat's do not move together, and collapsing two slots that
//!   happen to agree is how a per-kind palette quietly becomes a single pen
//!   again.
//!
//! [`Pen::colour_for`] is the total function from a kind to its slot's colour;
//! [`Pen::colour_of`] and [`Pen::set_colour`] are the slot-addressed pair the
//! swatch uses. Both matches are exhaustive, so a ninth slot fails to compile
//! rather than silently landing on the shape pen.

use egui::Color32;

use super::MarkupKind;
use super::palette;

/// **Which pen** — one variant per default Acrobat keeps a separate key for.
///
/// # ★ Why this exists rather than the swatch naming a field
///
/// Because there are now eight colours and one control. A swatch that wrote to
/// `pen.underline` by naming the field would need a second swatch for every
/// slot, and the slot a control edits is a *runtime* choice — which is the
/// definition of a value rather than a field name. [`Pen::colour_of`] and
/// [`Pen::set_colour`] take one of these and are total over it, so the control
/// is written once and a ninth slot is a compile error rather than a silent
/// fallback to the shape pen.
///
/// # The mapping to Acrobat's store
///
/// | slot | Acrobat key | shipped colour |
/// |---|---|---|
/// | [`Self::Shape`] | `cSquare`, `cCircle`, `cLine`, `cLine:LineArrow`, `cPolyLine`, `cPolygon`, `cPolygon:PolygonCloud`, `cInk` | [`palette::MARKUP_RED`] |
/// | [`Self::Highlighter`] | `cHighlight`, `cInk:InkHighlight` | [`palette::HIGHLIGHTER_ORANGE`] |
/// | [`Self::Underline`] | `cUnderline` | [`palette::UNDERLINE_BLUE`] |
/// | [`Self::StrikeOut`] | `cStrikeOut` | [`palette::STRIKEOUT_PINK`] |
/// | [`Self::Squiggly`] | `cSquiggly` | [`palette::MARKUP_RED`] |
/// | [`Self::Note`] | `cText` | [`palette::NOTE_PURPLE`] |
/// | [`Self::TextBox`] | `cFreeText` | [`palette::MARKUP_RED`] |
/// | [`Self::Stamp`] | `cStamp` | [`palette::MARKUP_RED`] |
///
/// See [`super::palette`]'s header for where those readings come from and for
/// the evidence that they are Adobe's factory values rather than this machine's
/// history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PenSlot {
    /// Every kind drawn as **linework by a pointer gesture**: rectangle,
    /// ellipse, arrow, polyline, polygon, revision cloud, freehand.
    ///
    /// One slot for seven kinds because Acrobat holds one value across their
    /// seven keys. Called `Shape` rather than `Ink` because [`Pen::ink`] is the
    /// field and `MarkupKind::Ink` is the *freehand* kind — three meanings of
    /// one word, and the enum is where they are told apart.
    Shape,
    /// The highlight band, whether drawn as an area or over found text.
    Highlighter,
    /// `/Underline`.
    Underline,
    /// `/StrikeOut`.
    StrikeOut,
    /// `/Squiggly`.
    ///
    /// Ships at the same red as [`Self::Shape`] and is a separate slot anyway —
    /// see the module header on why two slots that agree today are not one slot.
    Squiggly,
    /// `/Text` — the sticky note.
    Note,
    /// `/FreeText` — the text box.
    ///
    /// ★ Acrobat splits this in two and this shell cannot. `cFreeText` holds a
    /// *border* colour (`#F86464`) and a *text* colour (`#DB3425`), and
    /// `canvas::textannot` authors one `ink` used for both. This slot ships at
    /// Acrobat's **text** colour, because the words are what the operator reads
    /// and a frame is a frame. The divergence is real and is written down here
    /// rather than smoothed over: a pdfcer text box has an Acrobat-red frame
    /// where Acrobat's would be pink.
    TextBox,
    /// `/Stamp` — a framed label.
    Stamp,
}

impl PenSlot {
    /// Every slot, in the order the module header's table lists them.
    ///
    /// Exists for the reason `MarkupKind::ALL` does: it is what lets a test
    /// sweep every slot rather than a hand-written subset, so a ninth slot is
    /// covered without anybody remembering to add it.
    pub const ALL: &'static [PenSlot] = &[
        PenSlot::Shape,
        PenSlot::Highlighter,
        PenSlot::Underline,
        PenSlot::StrikeOut,
        PenSlot::Squiggly,
        PenSlot::Note,
        PenSlot::TextBox,
        PenSlot::Stamp,
    ];

    /// **Which pen draws this kind.**
    ///
    /// The successor to the two-arm `match` [`Pen::colour_for`] used to be, and
    /// the one place the geometric family's shared slot is asserted. Exhaustive
    /// over [`MarkupKind`], so a ninth kind fails to compile here rather than
    /// arriving in whatever colour the catch-all happened to name.
    #[must_use]
    pub const fn of(kind: MarkupKind) -> Self {
        match kind {
            MarkupKind::Highlight => Self::Highlighter,
            MarkupKind::Rectangle
            | MarkupKind::Ellipse
            | MarkupKind::Arrow
            | MarkupKind::PolyLine
            | MarkupKind::Polygon
            | MarkupKind::Cloud
            | MarkupKind::Ink => Self::Shape,
        }
    }

    /// **Which pen writes this text annotation.**
    ///
    /// The sticky/text-box/stamp counterpart of [`Self::of`]. Its one caller is
    /// `app::actions::apply`'s `CommitTextAnnot` arm, which until 2026-09-06
    /// passed `pen.ink` for all three — so a sticky note came out shape-red
    /// where Acrobat's is violet.
    #[must_use]
    pub const fn of_text_annot(kind: crate::canvas::textannot::TextAnnotKind) -> Self {
        match kind {
            crate::canvas::textannot::TextAnnotKind::TextBox => Self::TextBox,
            crate::canvas::textannot::TextAnnotKind::Sticky => Self::Note,
            crate::canvas::textannot::TextAnnotKind::Stamp => Self::Stamp,
        }
    }
}

/// The colour and width the next markup gesture will be authored with.
///
/// `Copy`, and small enough that it is passed by value everywhere. That is
/// deliberate: a pen borrowed from the application while a gesture is being
/// committed would conflict with the `&mut self` the commit needs, and the
/// alternative — cloning at each call — would be the same bytes with a
/// question about whether the copy is stale.
///
/// ⚠ **Keeping it `Copy` is what decided [`super::linestyle::LineStyle`]'s
/// shape.** The engine's `BorderDash` owns a `Vec<f64>` and would have taken
/// `Copy` away from every caller in `canvas::markup` and `app::actions`; see
/// that type's header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pen {
    /// The comment-linework colour, as PDF `/DeviceRGB` components in
    /// `0.0..=1.0`.
    ///
    /// **DOCUMENT COLOUR.** This is written into the annotation's `/C` and
    /// therefore into the saved file — restyling the application must never
    /// move it, which is the case the theme gate's escape hatch exists for.
    pub ink: (f64, f64, f64),
    /// The highlight band's colour, same units and the same warning.
    pub highlighter: (f64, f64, f64),
    /// `/Underline`'s colour — [`PenSlot::Underline`]. Same units, same warning.
    pub underline: (f64, f64, f64),
    /// `/StrikeOut`'s colour — [`PenSlot::StrikeOut`].
    pub strike_out: (f64, f64, f64),
    /// `/Squiggly`'s colour — [`PenSlot::Squiggly`].
    pub squiggly: (f64, f64, f64),
    /// The sticky note's colour — [`PenSlot::Note`].
    pub note: (f64, f64, f64),
    /// The text box's border **and** painted text — [`PenSlot::TextBox`]. See
    /// that variant on why one value serves two of Acrobat's.
    pub text_box: (f64, f64, f64),
    /// The stamp's colour — [`PenSlot::Stamp`].
    pub stamp: (f64, f64, f64),
    /// Border and stroke width, in PDF points.
    ///
    /// Clamped to [`MIN_WIDTH_PTS`]`..=`[`MAX_WIDTH_PTS`] by the control that
    /// sets it — see those constants for why the range is what it is.
    pub width_pts: f64,
    /// ★★★ **The annotation's constant opacity, `/CA`** — `0.0`–`1.0`, where
    /// `1.0` is fully opaque and is the default.
    ///
    /// # This field is why the mark can be seen THROUGH
    ///
    /// A comment on an engineering drawing sits on top of the thing it is about.
    /// An opaque cloud round a dimension hides the dimension; a 40% one does
    /// not. That is the whole use, and it is the reason the operator's own
    /// argument against a **fill** does not apply here — a translucent outline
    /// obscures nothing.
    ///
    /// # ★★ `1.0` writes no key at all, and that is deliberate
    ///
    /// [`Self::opacity_option`] answers `None` at `1.0`. §12.5.2 Table 164 makes
    /// 1.0 the default, so writing it explicitly would add a key that changes
    /// nothing and make a pdfcer-authored opaque annotation textually different
    /// from every other producer's — which is the engine's own reasoning on
    /// `MarkupOptions::opacity`, adopted rather than re-derived.
    ///
    /// ⇒ It also keeps the standing rule for a capability becoming choosable:
    /// **a build which omits nothing must behave as it did before the choice
    /// existed**, byte for byte.
    ///
    /// # ★ Clamped by the control, refused by the engine
    ///
    /// [`MIN_OPACITY`]`..=1.0` at the widget. The engine **refuses** an
    /// out-of-range author-time alpha by name rather than clamping it, because
    /// *"quietly authoring 1.0 would put an opaque annotation on the page while
    /// reporting success"* — so a value that escaped this range would produce a
    /// refusal, not a silent surprise.
    pub opacity: f64,
    /// ★★★ **The border line style the next mark is drawn in** — `/BS` `/S` and
    /// `/D` (§12.5.4, Table 166).
    ///
    /// # It is a [`super::linestyle::LineStyle`] and not a `BorderDash`
    ///
    /// Because that type is `Copy` and the engine's is not — see its header,
    /// which carries the whole argument and the reason this struct's `Copy` is
    /// load-bearing. [`Self::dash_option`] is the boundary that builds the
    /// engine's value.
    ///
    /// # ★★ Solid ships, and that keeps the standing rule
    ///
    /// [`Self::opacity`]'s doc states the rule this project applies when a
    /// capability becomes choosable: *"a build which omits nothing must behave
    /// as it did before the choice existed, byte for byte."* Unlike the colour
    /// change of 2026-09-06, this one **does** keep it: the default is
    /// [`super::linestyle::LineStyle::Solid`], `dash_option` answers `None`, and
    /// `MarkupOptions::dash: None` authors *"the solid border pdfcer authored
    /// exclusively before `Pass 258.0`"*
    /// (`D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:4772-4774`). An operator
    /// who never opens the chooser gets the file they got yesterday.
    ///
    /// # ⚠ Ignored by the text-markup family, and that is the format
    ///
    /// A highlight is a colour wash and an underline is its own line; neither
    /// draws a `/BS` border, so `MarkupOptions::dash` is ignored for all four
    /// (`edit.rs:4776-4781`). The pen carries one value and the highlighter
    /// shares it, so the chooser's tooltip says so —
    /// [`crate::text::markup::pen_dash_tooltip`] — rather than leaving an
    /// operator to conclude the setting did not take.
    pub dash: super::linestyle::LineStyle,
}

/// The thinnest pen offered.
///
/// **A quarter point, not zero.** Zero is a legal PDF border width and means
/// *"the thinnest line the device can draw"*, which on a 2400 dpi plotter is
/// invisible and on screen is one pixel — so it is a width whose appearance
/// depends on the output device, which is precisely the property a comment
/// annotation must not have. A quarter point is the thinnest value that means
/// the same thing everywhere.
pub const MIN_WIDTH_PTS: f64 = 0.25;

/// **The most transparent mark offered**, as a fraction.
///
/// A tenth, not zero. An annotation at `/CA 0` is **invisible** — it is in the
/// file, it is selectable, it prints as nothing, and nothing on screen says it
/// is there. A control whose bottom end authors an invisible mark is a control
/// whose bottom end is a defect report waiting to be filed, and the operator
/// would have no way to tell it from a markup that failed to author at all.
///
/// A tenth is faint enough to be a wash over dense linework and still visible
/// as a mark.
pub const MIN_OPACITY: f64 = 0.1;

/// The thickest pen offered.
///
/// Twelve points is about a sixth of an inch — a marker rather than a pen, and
/// already heavy enough to obscure the drawing underneath, which is the failure
/// a comment on an engineering sheet has to avoid. Beyond it the annotation
/// stops being linework and starts being a fill, and `MarkupSpec` has a
/// separate concept for that which this shell deliberately does not offer.
pub const MAX_WIDTH_PTS: f64 = 12.0;

impl Default for Pen {
    /// ★★★ **The shipped pen: Acrobat's own eight defaults, at 2 pt, opaque.**
    ///
    /// # Every colour here is measured, and none of it is chosen
    ///
    /// [`super::palette`]'s header carries the reading — Acrobat DC's
    /// `HKCU\…\Annots\cAnnots\<subtype>\cstrokeColor`, on 2026-09-06 — the
    /// registry key each value came from, and the evidence that those are
    /// Adobe's factory values rather than this machine's history. Nothing below
    /// is a colour this shell picked, which is the entire difference between
    /// this version and the one before it.
    ///
    /// # ★★★ THIS DELIBERATELY BREAKS THE STANDING "OMITS NOTHING" RULE, and
    /// # the rule is not being ignored — it is being answered
    ///
    /// [`Self::opacity`]'s doc comment states the rule this project applies when
    /// a capability becomes choosable:
    ///
    /// > **a build which omits nothing must behave as it did before the choice
    /// > existed**, byte for byte.
    ///
    /// That rule is about a *capability arriving*: an operator who never touches
    /// a new control must not discover that the new control changed their output
    /// anyway. It is a rule against **silent** change, and it is a good one.
    ///
    /// ⇒ It does not bind here, and the reason is that this change is not
    /// silent — **it is the thing the operator asked for by name**:
    ///
    /// > *"Also make sure you've used the same default colours and style look
    /// > for these things as Adobe."*
    ///
    /// A markup authored by this build is therefore a different colour from one
    /// authored by the build before it. That is a deliberate, requested,
    /// operator-visible change, and pretending otherwise by keeping the old
    /// values *"for compatibility"* would be answering a request with a refusal
    /// dressed as a principle. Saying so here, rather than quietly departing
    /// from a rule written four lines above, is the whole reason this paragraph
    /// exists.
    ///
    /// ⚠ The concrete consequence, so nobody has to discover it: a drawing
    /// marked up before 2026-09-06 and marked up again after it will carry two
    /// slightly different reds and — much more visibly — an orange highlight
    /// beside a yellow one. Both are in [`super::palette::ACROBAT`], one click
    /// apart, which is why the grid keeps [`super::palette::CLASSIC_YELLOW`].
    ///
    /// # ★★ 2 pt is KEPT, and Adobe's number is not being ignored — there
    /// # isn't one
    ///
    /// The brief for this change said to weigh Acrobat's default line width
    /// against this shell's 2 pt and decide, recording both sides. The weighing
    /// found only one side, and the finding is the decision:
    ///
    /// **There is no measurable Acrobat width to match.** `cAnnots` holds a
    /// colour key, a fill key, a text key, an opacity key and an icon name for
    /// every subtype, and **no width, thickness or border key at all** — the
    /// whole `…\Adobe Acrobat\DC` tree was searched for `width`, `thick` and
    /// `border` and the only hits were print-N-up and multimedia settings. So an
    /// "Acrobat default of 1 pt" could only have come from memory or from a
    /// web page, and this project's **claim-bearing copy** rule is explicit that
    /// a plausible number from a marketplace convention is not a source. A line
    /// width is written into `/BS /W` and reaches the operator's file; it is a
    /// claim.
    ///
    /// The two sides, since both were asked for:
    ///
    /// | for adopting Adobe's | for keeping 2 pt |
    /// |---|---|
    /// | the operator asked for parity with Adobe, and asked for it about style | **the number is not sourced** — the ask was for Adobe's value, not for a guess at it |
    /// | a thinner default is easier to thicken than a thick one is to find | the operator's own drawings are dense CAD exports whose linework is 0.25 pt, and *"a hairline vanishes among the drawing's own linework"* is his use case, argued here since the constant existed |
    /// | | 2 pt is what every markup this shell has authored is drawn at, so an old and a new comment on one sheet match |
    ///
    /// ⇒ **2 pt wins**, on the first row of the right-hand column alone. If a
    /// measurement of Acrobat's width later turns up — its Properties dialog
    /// shows a `Thickness` field, so the number exists somewhere this search did
    /// not reach — this decision should be revisited **with that measurement**,
    /// not with a recollection of it.
    fn default() -> Self {
        Self {
            // DOCUMENT COLOUR: Acrobat's shape-tool red, written into `/C`.
            ink: palette::components(palette::MARKUP_RED),
            // DOCUMENT COLOUR: Acrobat's highlighter — ORANGE, measured, see
            // `palette`'s header for why that is not the mistake it looks like.
            highlighter: palette::components(palette::HIGHLIGHTER_ORANGE),
            // DOCUMENT COLOUR: Acrobat's `cUnderline`.
            underline: palette::components(palette::UNDERLINE_BLUE),
            // DOCUMENT COLOUR: Acrobat's `cStrikeOut`.
            strike_out: palette::components(palette::STRIKEOUT_PINK),
            // DOCUMENT COLOUR: Acrobat's `cSquiggly` — the same red as the shape
            // pen, under its own key, so it is its own slot.
            squiggly: palette::components(palette::MARKUP_RED),
            // DOCUMENT COLOUR: Acrobat's `cText` — the sticky note's violet.
            note: palette::components(palette::NOTE_PURPLE),
            // DOCUMENT COLOUR: Acrobat's `cFreeText` TEXT colour. See
            // `PenSlot::TextBox` on why the text colour and not the border's.
            text_box: palette::components(palette::MARKUP_RED),
            // DOCUMENT COLOUR: Acrobat's `cStamp`.
            stamp: palette::components(palette::MARKUP_RED),
            width_pts: 2.0,
            // Fully opaque, which writes no `/CA` at all — see the field's own
            // doc comment. This is what every markup this shell authored before
            // 2026-08-28 did, so the default is the old behaviour exactly.
            opacity: 1.0,
            // Solid, which writes no dash at all — see the field's own doc
            // comment. Unlike the eight colours above, this default DOES keep
            // the "omits nothing" rule: a build whose operator never opens the
            // chooser authors the same bytes it authored before the chooser
            // existed.
            dash: super::linestyle::LineStyle::Solid,
        }
    }
}

impl Pen {
    /// **The freehand simplification tolerance this pen implies**, in PDF
    /// points — a quarter of the stroke width.
    ///
    /// # ★ This function exists because the constant it replaced went stale
    /// the day the pen became a control
    ///
    /// `ink::SIMPLIFY_TOLERANCE_PTS` was `PEN_WIDTH_PTS / 4.0`, a `const`
    /// derived from the pen's **shipped** width of 2 pt. That was exactly
    /// right while the width was a constant, and `canvas::markup::ink`'s §3.2
    /// wrote down what to do when it stopped being one:
    ///
    /// > it is a *rule* — **if the pen ever becomes an operator control, the
    /// > tolerance follows it** rather than being re-tuned by eye.
    ///
    /// The pen became an operator control on 2026-08-17 and the tolerance did
    /// not follow. This is that rule, honoured — and the module wrote the rule
    /// down, was read by the session that broke it, and was broken **the same
    /// day**. Recording the rule in prose was not enough; only a test that
    /// varies the input could have caught it, and that test now exists.
    ///
    /// # Why the stale value was wrong in a way an operator would see
    ///
    /// The derivation is not arbitrary. Ramer–Douglas–Peucker guarantees that
    /// no removed point lay further than ε from the line replacing it, so ε
    /// bounds how far the **drawn centreline** can move. Setting ε to half of
    /// the stroke's *half*-width means the simplified centreline stays strictly
    /// inside the body of the stroke the raw trail would have drawn: **no pixel
    /// of the mark can move outside the mark.** That is the strongest statement
    /// available about a lossy simplification, and it is the whole reason the
    /// number is defensible rather than tuned.
    ///
    /// A fixed 0.5 pt breaks it in one direction and merely wastes work in the
    /// other:
    ///
    /// | pen width | half-width | fixed ε = 0.5 | verdict |
    /// |---:|---:|---:|---|
    /// | 0.25 pt | 0.125 pt | **4× the half-width** | ⛔ the centreline can move well outside the stroke. An operator drawing a fine detail line gets a visibly different curve from the one they drew |
    /// | 2 pt | 1 pt | 0.5 = half the half-width | ✅ the shipped case, and the only one that was ever right |
    /// | 12 pt | 6 pt | 0.17× the half-width | ⚠ correct but pointlessly tight — keeps far more points than the guarantee needs |
    ///
    /// The thin-pen row is the one that matters: it is the direction that
    /// **changes what is authored**, it is silent, and it is worst on exactly
    /// the drawings this shell is for, where a 0.25 pt pen exists to match a
    /// CAD sheet's own linework.
    ///
    /// # It lives here rather than in `ink`
    ///
    /// Because the rule is *"the tolerance follows the pen"*, and a derivation
    /// kept beside the value it derives from cannot be forgotten when that
    /// value changes — which is precisely what happened when it lived
    /// elsewhere as a `const`.
    ///
    /// # ★★★ THE RULE WAS RE-READ ON 2026-09-06 AND STILL HOLDS, because the
    /// # width did NOT go per-kind
    ///
    /// The per-kind change of 2026-09-06 made the **colour** per-slot and left
    /// [`Self::width_pts`] a single value. That was checked against this rule
    /// before it was decided, not after, because this exact rule was written
    /// down once and broken **the same day** by the session that read it, and
    /// the file records that. It is not being broken a third time.
    ///
    /// Two things follow, and both are load-bearing:
    ///
    /// 1. **Today**, `self.width_pts` is the only width, so this signature is
    ///    correct and needs no kind. A `Pen` is one thickness and eight colours.
    /// 2. **The day a width goes per-slot**, this function must gain the slot
    ///    and every caller must pass it. `ink::simplify` is the only consumer
    ///    and it already knows the kind it is simplifying, so the change is
    ///    mechanical — but it is *not optional*: an ink stroke simplified at the
    ///    shape pen's tolerance while drawn at a per-slot width would move the
    ///    centreline outside the stroke on exactly the thin-pen row of the table
    ///    above, silently, on the dense drawings this shell is for.
    ///
    /// [`tests::the_tolerance_follows_the_width`] is what enforces this rather
    /// than the prose — it varies the width and asserts the tolerance moves,
    /// which is the only form of the rule a future edit cannot read past.
    #[must_use]
    pub fn simplify_tolerance_pts(self) -> f32 {
        (self.width_pts as f32) / 4.0
    }

    /// The colour this kind is authored in.
    ///
    /// Two total functions composed: [`PenSlot::of`] says which pen draws the
    /// kind, [`Self::colour_of`] says what colour that pen is. Neither has a
    /// catch-all arm, so a ninth [`MarkupKind`] or a ninth [`PenSlot`] is a
    /// compile error rather than a silent landing on the shape pen — which is
    /// what the old two-arm `match` with its `_ => self.ink` did, and was right
    /// to do while there were exactly two pens.
    #[must_use]
    pub fn colour_for(self, kind: MarkupKind) -> (f64, f64, f64) {
        self.colour_of(PenSlot::of(kind))
    }

    /// **The colour in one slot**, as PDF `/DeviceRGB` components.
    ///
    /// Exhaustive over [`PenSlot`], deliberately and without a `_` arm: this is
    /// the function a new slot must be taught about, and a catch-all here would
    /// let a new slot compile while authoring the shape pen's red — a defect
    /// with no symptom except a colour nobody chose.
    #[must_use]
    pub fn colour_of(self, slot: PenSlot) -> (f64, f64, f64) {
        match slot {
            PenSlot::Shape => self.ink,
            PenSlot::Highlighter => self.highlighter,
            PenSlot::Underline => self.underline,
            PenSlot::StrikeOut => self.strike_out,
            PenSlot::Squiggly => self.squiggly,
            PenSlot::Note => self.note,
            PenSlot::TextBox => self.text_box,
            PenSlot::Stamp => self.stamp,
        }
    }

    /// **Set one slot's colour** from a screen colour, discarding alpha.
    ///
    /// The write half of [`Self::colour_of`] and the one place the operator's
    /// override lands. *"Once they set a colour for a kind, it sticks for that
    /// kind"* is this function plus the fact that [`Pen`] lives on the
    /// application rather than on the document — see the module header's own
    /// section on why a pencil does not change colour when you turn the page.
    ///
    /// Alpha is dropped; see [`Self::set_ink`] for the argument, which is about
    /// `/C` having three components and no fourth.
    pub fn set_colour(&mut self, slot: PenSlot, colour: Color32) {
        let rgb = rgb_of(colour);
        match slot {
            PenSlot::Shape => self.ink = rgb,
            PenSlot::Highlighter => self.highlighter = rgb,
            PenSlot::Underline => self.underline = rgb,
            PenSlot::StrikeOut => self.strike_out = rgb,
            PenSlot::Squiggly => self.squiggly = rgb,
            PenSlot::Note => self.note = rgb,
            PenSlot::TextBox => self.text_box = rgb,
            PenSlot::Stamp => self.stamp = rgb,
        }
    }

    /// One slot's colour as a screen colour, for the swatch that shows it.
    #[must_use]
    pub fn color32_of(self, slot: PenSlot) -> Color32 {
        color32_of(self.colour_of(slot))
    }

    /// **The colour a sticky note, text box or stamp is authored in.**
    ///
    /// ★ Its one caller is `app::actions::apply`'s `CommitTextAnnot` arm, and
    /// the line it replaced read `self.pen.ink` — one colour for all three,
    /// which made a pdfcer sticky note shape-red where Acrobat's is violet.
    /// Routing through [`PenSlot::of_text_annot`] rather than exposing the three
    /// fields keeps the mapping total and keeps it here, beside the table it is
    /// derived from.
    #[must_use]
    pub fn text_annot_colour(
        self,
        kind: crate::canvas::textannot::TextAnnotKind,
    ) -> (f64, f64, f64) {
        self.colour_of(PenSlot::of_text_annot(kind))
    }

    /// Set the ink colour from a screen colour, discarding alpha.
    ///
    /// # ★ Alpha is dropped, deliberately, and this is not a limitation
    ///
    /// `egui`'s colour picker offers an alpha channel; a PDF annotation's `/C`
    /// entry is **three components and no more** (§12.5.2 Table 164: an array
    /// of 0, 1, 3 or 4 numbers in the annotation's colour space, with
    /// transparency carried by `/CA` instead). Feeding the picker's alpha into
    /// `/C` would be a value with nowhere to go.
    ///
    /// Opacity is therefore a **separate control**, and it is one that **now
    /// exists** — [`Self::opacity`], drawn beside the swatches in
    /// [`super::swatch::show`] since 2026-08-28. This paragraph used to end
    /// *"it is not built yet: `/CA` support was filed against `pdfcer-core` and
    /// is accepted-and-scheduled rather than shipped"*, which was true when
    /// written and stopped being true when `Pass 81.1` landed
    /// `MarkupOptions::opacity`. Corrected rather than deleted, because a stale
    /// blocker is this project's most-repeated defect and the shape of it is the
    /// useful part.
    ///
    /// ⇒ The alpha channel is *still* not offered **on the colour picker**, and
    /// that is unchanged and correct: `/C` has three components, and a picker
    /// alpha would be a fourth with nowhere to go. Transparency is `/CA` and it
    /// has its own control.
    pub fn set_ink(&mut self, colour: Color32) {
        self.set_colour(PenSlot::Shape, colour);
    }

    /// **The `/CA` value to author with, or `None` for "write no key".**
    ///
    /// `None` at fully opaque. See [`Self::opacity`] for why that is not the
    /// same as `Some(1.0)` even though it is the same on screen, and why the
    /// difference is worth a method rather than a comparison at each call site
    /// — there are three, and the one that forgot would be the one that made a
    /// pdfcer annotation textually unlike everybody else's.
    #[must_use]
    pub fn opacity_option(&self) -> Option<f64> {
        (self.opacity < 1.0).then_some(self.opacity)
    }

    /// **The `/BS` dash to author with, or `None` for "write a solid border".**
    ///
    /// The exact twin of [`Self::opacity_option`], and for the same reason it is
    /// a method rather than a comparison at each call site: `None` is what
    /// preserves this shell's pre-2026-09-06 bytes, and the call site that
    /// forgot would be the one that made a pdfcer annotation textually unlike
    /// every one it had authored before.
    ///
    /// ⚠ **Not passed on the text-markup route.** `app::actions::apply`'s
    /// `CommitMarkup` arm sends it; `CommitTextMarkup` does not, because a
    /// highlight, underline, strikeout and squiggly draw no `/BS` border and the
    /// engine ignores the field for all four (`edit.rs:4776-4781`). Sending a
    /// value that is documented as ignored would be this shell asking for
    /// something and calling the silence success — which is the exact failure
    /// shape `MarkupStyleSupport` was shipped to end.
    #[must_use]
    pub fn dash_option(&self) -> Option<pdfcer_core::annot_author::BorderDash> {
        self.dash.dash()
    }

    /// Set the highlighter colour from a screen colour. As [`Self::set_ink`].
    pub fn set_highlighter(&mut self, colour: Color32) {
        self.set_colour(PenSlot::Highlighter, colour);
    }

    /// The ink colour as a screen colour, for the swatch that sets it.
    #[must_use]
    pub fn ink_color32(self) -> Color32 {
        self.color32_of(PenSlot::Shape)
    }

    /// The highlighter colour as a screen colour.
    #[must_use]
    pub fn highlighter_color32(self) -> Color32 {
        self.color32_of(PenSlot::Highlighter)
    }
}

/// PDF components from a screen colour.
///
/// Alpha is dropped — see [`Pen::set_ink`]. The division is by `255.0` rather
/// than by `256.0`: the component range is *inclusive* of both ends, so `255`
/// must map to exactly `1.0` or a "pure red" chosen in the picker would be
/// written as `0.996` and round-trip to a slightly different swatch.
fn rgb_of(c: Color32) -> (f64, f64, f64) {
    (
        f64::from(c.r()) / 255.0,
        f64::from(c.g()) / 255.0,
        f64::from(c.b()) / 255.0,
    )
}

/// A screen colour from PDF components.
///
/// The inverse of [`rgb_of`], and **opaque**: the swatch shows the colour the
/// annotation will be, and an annotation has no alpha in `/C`. Rounding rather
/// than truncating, so the round trip through the picker is stable — truncation
/// would walk a colour down by one unit per visit.
fn color32_of((r, g, b): (f64, f64, f64)) -> Color32 {
    let byte = |v: f64| {
        // `clamp` first: a settings file or a future loader could hand this a
        // component outside the range, and `as u8` on an out-of-range float is
        // a saturating cast in Rust but on a NaN produces 0 — a silent black.
        // Clamping states the intent instead of relying on that.
        let scaled = (v.clamp(0.0, 1.0) * 255.0).round();
        // The value is provably in `0..=255` and finite, so this cast is exact.
        scaled as u8
    };
    // DOCUMENT COLOUR: the operator's own pen, arriving from the file's
    // `/DeviceRGB` components rather than from the palette. No theme may move
    // it — a restyle that changed the swatch would be claiming the annotation
    // had changed colour, and the annotation is in the document.
    Color32::from_rgb(byte(r), byte(g), byte(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **The shipped pen is Acrobat's, slot for slot.**
    ///
    /// The successor to `the_default_pen_is_the_constants_it_replaced`, which
    /// pinned `(0.85, 0.16, 0.16)` and `(1.0, 1.0, 0.0)` — this shell's own
    /// invented red and yellow — under the *"a build that omits nothing behaves
    /// as it did before"* rule. That test was **deleted deliberately**, not
    /// renamed: it asserted the exact values the operator asked to have
    /// replaced, so leaving it would have made the requested change fail the
    /// suite. `Pen::default`'s own doc comment carries the argument for why the
    /// rule does not bind here.
    ///
    /// What replaces it is stronger, because it is checkable against something
    /// outside this file: every slot must equal the [`super::palette`] constant
    /// whose doc comment names the Acrobat registry key it was read from. A
    /// hand-typed drift in either place fails here.
    ///
    /// Falsified by changing `ink` to `palette::NOTE_PURPLE`: the assertion
    /// fired naming `Shape`. Restored.
    #[test]
    fn every_slot_ships_at_the_acrobat_value_it_was_measured_from() {
        let pen = Pen::default();
        let expected = [
            (PenSlot::Shape, palette::MARKUP_RED),
            (PenSlot::Highlighter, palette::HIGHLIGHTER_ORANGE),
            (PenSlot::Underline, palette::UNDERLINE_BLUE),
            (PenSlot::StrikeOut, palette::STRIKEOUT_PINK),
            (PenSlot::Squiggly, palette::MARKUP_RED),
            (PenSlot::Note, palette::NOTE_PURPLE),
            (PenSlot::TextBox, palette::MARKUP_RED),
            (PenSlot::Stamp, palette::MARKUP_RED),
        ];
        assert_eq!(
            expected.len(),
            PenSlot::ALL.len(),
            "a slot was added and this table was not told about it"
        );
        for (slot, bytes) in expected {
            assert_eq!(
                pen.colour_of(slot),
                palette::components(bytes),
                "{slot:?} does not ship at the Acrobat value it is documented as"
            );
        }
        assert!((pen.width_pts - 2.0).abs() < f64::EPSILON);
    }

    /// ★★ **The highlighter is ORANGE, and that is not a typo.**
    ///
    /// Stated as its own test because it is the single value most likely to be
    /// "corrected" back to yellow by somebody who knows that PDF highlighters
    /// are yellow. They are not, in the program the operator compares against:
    /// `cHighlight\cstrokeColor` reads `1.0, 0.384308, 0.0`.
    ///
    /// The assertion is written as *"not the yellow it used to be"* rather than
    /// only as *"is the orange"*, so the failure message says what happened.
    #[test]
    fn the_highlighter_is_acrobats_orange_and_not_the_old_yellow() {
        let pen = Pen::default();
        assert_eq!(pen.highlighter, palette::components([255, 98, 0]));
        assert_ne!(
            pen.highlighter,
            (1.0, 1.0, 0.0),
            "the highlighter has been put back to this shell's old invented \
             yellow — Acrobat's is #FF6200, measured from cHighlight, and the \
             operator asked for Adobe's value. Yellow is still one click away \
             in the palette as `CLASSIC_YELLOW`."
        );
    }

    /// ★ **Every markup kind reaches a slot, and the geometric family shares
    /// one.**
    ///
    /// The successor to `only_the_highlight_kind_uses_the_highlighter`, over
    /// the whole `MarkupKind::ALL` list rather than a hand-written subset — so a
    /// ninth kind is covered without anybody remembering to add it here.
    ///
    /// What it asserts is now the *routing*, not the colour: eight
    /// distinguishable values are planted, one per slot, so a kind that took the
    /// wrong pen names itself. Planting real colours would let a wrong answer
    /// pass whenever two slots happened to ship the same red — which three of
    /// them do.
    #[test]
    fn every_kind_takes_the_slot_it_is_documented_to_take() {
        let pen = planted();
        for kind in MarkupKind::ALL {
            let slot = PenSlot::of(*kind);
            let expected = if matches!(kind, MarkupKind::Highlight) {
                PenSlot::Highlighter
            } else {
                PenSlot::Shape
            };
            assert_eq!(slot, expected, "{kind:?} took the wrong pen");
            assert_eq!(
                pen.colour_for(*kind),
                pen.colour_of(slot),
                "{kind:?}'s colour did not come from its own slot"
            );
        }
    }

    /// ★★★ **Every slot is separately settable, and setting one moves nothing
    /// else.**
    ///
    /// This is the *"once they set a colour for a kind, it sticks for that
    /// kind"* half of the operator's ask, and it is the property a collapsed
    /// slot would silently lose: if `Squiggly` were folded into `Shape` because
    /// they ship the same red, recolouring the shape pen would recolour every
    /// squiggly on every future page and nothing would say so.
    ///
    /// Falsified by making `set_colour`'s `Squiggly` arm write `self.ink`: the
    /// assertion fired on the `Shape` slot while setting `Squiggly`. Restored.
    #[test]
    fn setting_one_slot_leaves_the_other_seven_alone() {
        for target in PenSlot::ALL {
            let mut pen = planted();
            let before = pen_slots(&pen);
            // NOT A THEME COLOUR: an arbitrary value distinct from every
            // planted one, so "did this slot move" is unambiguous.
            pen.set_colour(*target, Color32::from_rgb(1, 2, 3));
            assert_eq!(
                pen.colour_of(*target),
                (1.0 / 255.0, 2.0 / 255.0, 3.0 / 255.0),
                "{target:?} did not take the colour it was given"
            );
            for (i, other) in PenSlot::ALL.iter().enumerate() {
                if other == target {
                    continue;
                }
                assert_eq!(
                    pen.colour_of(*other),
                    before[i],
                    "setting {target:?} also moved {other:?}"
                );
            }
        }
    }

    /// The three text-annotation kinds land on three different slots.
    ///
    /// The mapping `app::actions::apply` depends on. Before 2026-09-06 all
    /// three took `pen.ink`, so a sticky note came out shape-red where
    /// Acrobat's `cText` is violet — this is the assertion that would have
    /// caught that, stated as *"three kinds, three slots"* rather than as three
    /// hard-coded colours, because the colours may legitimately be edited and
    /// the separation may not.
    #[test]
    fn the_three_text_annotation_kinds_do_not_share_a_pen() {
        use crate::canvas::textannot::TextAnnotKind;
        let slots: Vec<PenSlot> = TextAnnotKind::ALL
            .iter()
            .map(|k| PenSlot::of_text_annot(*k))
            .collect();
        for i in 0..slots.len() {
            for j in (i + 1)..slots.len() {
                assert_ne!(
                    slots[i],
                    slots[j],
                    "{:?} and {:?} share a pen slot",
                    TextAnnotKind::ALL[i],
                    TextAnnotKind::ALL[j]
                );
            }
        }
        // …and the sticky note is on the note slot specifically, which is the
        // one whose Acrobat value differs from the shape pen's.
        assert_eq!(PenSlot::of_text_annot(TextAnnotKind::Sticky), PenSlot::Note);
        assert_eq!(
            Pen::default().text_annot_colour(TextAnnotKind::Sticky),
            palette::components(palette::NOTE_PURPLE)
        );
    }

    /// ★★★ **THE TOLERANCE FOLLOWS THE WIDTH.**
    ///
    /// The rule [`Pen::simplify_tolerance_pts`] carries at length: ε must be a
    /// quarter of the stroke width, because that is half of the half-width and
    /// therefore the bound that keeps a simplified centreline strictly inside
    /// the stroke the operator drew.
    ///
    /// It is asserted by **varying the width**, which is the only form of the
    /// rule a stale constant cannot pass: `ink::SIMPLIFY_TOLERANCE_PTS` was
    /// `PEN_WIDTH_PTS / 4.0` frozen at 2 pt, and it satisfied every test that
    /// used the default pen while being wrong by 4× at the thin end.
    ///
    /// ⚠ **If a width ever goes per-slot, this test must gain the slot too.**
    /// See the function's own note; the rule has been broken once already by a
    /// session that had just read it.
    #[test]
    fn the_tolerance_follows_the_width() {
        for width in [MIN_WIDTH_PTS, 0.5, 2.0, 7.5, MAX_WIDTH_PTS] {
            let pen = Pen {
                width_pts: width,
                ..Pen::default()
            };
            let expected = (width as f32) / 4.0;
            assert!(
                (pen.simplify_tolerance_pts() - expected).abs() < 1e-6,
                "at {width} pt the tolerance is {} and must be {expected} — a \
                 quarter of the width, so the simplified centreline stays \
                 inside the stroke",
                pen.simplify_tolerance_pts()
            );
        }
    }

    /// A pen with eight distinguishable colours, one per slot.
    ///
    /// Named rather than spelled out at three call sites, and built from the
    /// slot's own *index* so it cannot fall out of step with [`PenSlot::ALL`]:
    /// a ninth slot gets a ninth distinct value with no edit here.
    fn planted() -> Pen {
        let mut pen = Pen::default();
        for (i, slot) in PenSlot::ALL.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let step = (i as u8) * 16 + 8;
            // NOT A THEME COLOUR: eight distinguishable test values, so an
            // assertion says which slot was taken rather than which default
            // happened to match.
            pen.set_colour(*slot, Color32::from_rgb(step, step, step));
        }
        pen
    }

    /// Every slot's colour, in [`PenSlot::ALL`] order.
    fn pen_slots(pen: &Pen) -> Vec<(f64, f64, f64)> {
        PenSlot::ALL.iter().map(|s| pen.colour_of(*s)).collect()
    }

    /// ★ A colour survives the round trip through the picker unchanged.
    ///
    /// The property that makes the swatch usable rather than merely present.
    /// A conversion that truncated would walk a colour down by one unit every
    /// time the operator opened the picker and closed it without choosing —
    /// a slow, silent drift with no event to attach a bug report to.
    ///
    /// The endpoints are the ones that catch an off-by-one scale factor:
    /// dividing by 256 rather than 255 makes pure white round-trip to 254.
    #[test]
    fn a_colour_round_trips_through_the_swatch() {
        // DOCUMENT COLOUR: four `/DeviceRGB` values round-tripping through the
        // picker. Nothing here is chrome and no theme is involved; the endpoints
        // are chosen to catch an off-by-one scale factor.
        for original in [
            Color32::from_rgb(0, 0, 0),
            Color32::from_rgb(255, 255, 255),
            Color32::from_rgb(217, 41, 41),
            Color32::from_rgb(1, 128, 254),
        ] {
            let mut pen = Pen::default();
            pen.set_ink(original);
            assert_eq!(
                pen.ink_color32(),
                original,
                "a colour changed on its way to the document and back"
            );
        }
    }

    /// An out-of-range component clamps rather than wrapping to a nonsense
    /// colour.
    ///
    /// Not reachable from the picker, which cannot produce one — reachable from
    /// a hand-edited file the day the pen is persisted, and from any future
    /// loader. `as u8` on a NaN is 0, a silent black; the clamp states the
    /// intent instead of inheriting that.
    #[test]
    fn an_impossible_component_clamps_instead_of_wrapping() {
        // DOCUMENT COLOUR: expected `/DeviceRGB` results, not palette entries.
        assert_eq!(color32_of((2.0, -1.0, 0.5)), Color32::from_rgb(255, 0, 128));
        assert_eq!(
            color32_of((f64::NAN, 1.0, 0.0)),
            Color32::from_rgb(0, 255, 0)
        );
    }

    /// The shipped width is reachable on the control that sets it.
    ///
    /// # Why the range's own bounds are asserted in a `const` block
    ///
    /// `MIN_WIDTH_PTS > 0.0` is a relationship between two literals, so an
    /// ordinary `assert!` is a statement the compiler folds away and clippy
    /// rightly refuses. It is still worth stating — **zero is legal PDF** and
    /// means *"the thinnest line the device can draw"*, a width whose
    /// appearance depends on the output device and therefore exactly what a
    /// comment annotation must not be — so it is stated where a compile-time
    /// claim belongs, and a future edit that lowered the floor to zero would
    /// fail to build rather than fail to run.
    ///
    /// The runtime half is the one that can actually change: the shipped
    /// default is a value, and a default outside its own control's bounds
    /// would be silently rewritten the first time anybody touched the swatch.
    #[test]
    fn the_shipped_width_is_reachable_on_its_own_control() {
        const {
            assert!(MIN_WIDTH_PTS > 0.0, "a zero width is device-dependent");
            assert!(MAX_WIDTH_PTS > MIN_WIDTH_PTS);
        }
        assert!(
            (MIN_WIDTH_PTS..=MAX_WIDTH_PTS).contains(&Pen::default().width_pts),
            "the shipped width is not reachable on its own control"
        );
    }
}
