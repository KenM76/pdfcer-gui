//! # `canvas::markup::linestyle` — solid or dashed, on all three surfaces that
//! ask
//!
//! `RIBBON_IA.md` §5.8's Markup row lists eight controls. Seven shipped on the
//! morning of 2026-09-06; **Line style** was the eighth, and it was the only
//! entry in the whole row with *"no engine verb at all"*. That stopped being
//! true the same afternoon, when `pdfcer-core` answered this shell's own
//! request with three halves rather than the one it asked for:
//!
//! | half | engine | what it means here |
//! |---|---|---|
//! | **preserve** | a restyle that does not mention `dash` keeps one, *including a dash pdfcer never authored* (`edit.rs:4396-4425`) | this module has **nothing to build** for it, and the fact that leaving the control alone is safe is what makes [`DashReading::Foreign`] legitimate |
//! | **author** | `MarkupOptions::dash` (`edit.rs:4782`) | [`super::pen::Pen::dash`] and the Markup ▸ Style chooser |
//! | **restyle** | `MarkupStyle::dash: Option<StyleEdit<BorderDash>>` (`edit.rs:4422`) | the Format ▸ Markup chooser and the Properties panel row |
//!
//! (All engine line numbers in this file were read from
//! `D:\Dev\pdfcer\crates\pdfcer-core\` at pin `95a936e` on 2026-09-06.)
//!
//! ## ★★★ Why one module and not three controls
//!
//! Because *what a dash is* has to be spelled once. Three surfaces offer this —
//! the pen that authors, the ribbon band that restyles, and the Properties panel
//! that restyles — and if each wrote its own list of patterns then an operator
//! could draw a mark at a pattern no restyle control could put back, or restyle
//! one to a pattern the pen could never have produced. The two width controls in
//! this shell already assert that they share a range for exactly that reason
//! (`app::markupband::tests::the_width_range_matches_the_pen_that_authors`);
//! this module makes the same property structural rather than asserted, because
//! there is only one list.
//!
//! It also holds the **chooser widget** itself, so the three surfaces cannot
//! come to disagree about what the entries are *called* either.
//!
//! ## ★★ It is a new file rather than more of `pen.rs`, and that is R2
//!
//! `pen.rs` stood at 1,010 lines. This subject is a type, a reading of a
//! dictionary key, a widget and their tests, and every one of those choices
//! needs its argument written beside it — comfortably past the headroom.
//! `tools/gates/check-file-size.sh` says in its own header that shaving prose to
//! fit a threshold is the behaviour it exists to refuse, so the subject moved
//! instead. The seam is the same one `text::commands::markupstyle` took: `pen`
//! is *what the pen is*, this is *what a line style is*.
//!
//! ## ⚠ THERE IS NO PHASE, AND NO CONTROL FOR ONE IS OFFERED
//!
//! The content-stream `d` operator takes an array **and** a phase, but Table
//! 166's `/D` carries the array alone and the standard says the phase *"shall be
//! assumed 0"* — the engine states it at
//! `D:\Dev\pdfcer\crates\pdfcer-core\src\annot_author.rs:141-149`, and emits `0`
//! when it bakes the appearance. A phase control here would be a value the file
//! cannot hold: the operator would set it, the writer would ignore it, and the
//! control would look like completeness while being a lie. It is written down
//! here so that nobody adds one later on the reasoning that a dash "obviously"
//! has an offset.

use egui::Ui;

use crate::text::markup as t;

/// **The border line style — `/BS` `/S` and `/D` (§12.5.4, Table 166)** — as
/// the four choices this shell offers, on every surface that offers them.
///
/// # ★★★ Why this type exists rather than a `BorderDash` on the pen
///
/// `annot_author::BorderDash` owns a `Vec<f64>`
/// (`D:\Dev\pdfcer\crates\pdfcer-core\src\annot_author.rs:157`), so it is
/// `Clone` and **not** `Copy`. [`super::pen::Pen`] is `Copy` and is passed by
/// value through every gesture commit in `canvas::markup`; both restyle
/// surfaces pass a `Copy` `Current` by value through half a dozen control
/// functions. Putting the engine's type into any of them would have turned a
/// field addition into a borrow-checker sweep of files three other tracks were
/// writing in on the day this landed — for no gain, because **this shell offers
/// four patterns and not an arbitrary array**.
///
/// So the choice is modelled as the choice, and the engine's value is built at
/// the boundary by [`Self::dash`]. One `Copy` enum, three surfaces, one list.
///
/// # ★★ `BorderDash::new` returns `Option`, and this shell REFUSES IN THE UI
///
/// `annot_author.rs:185` refuses a pattern §8.4.3.6 does not admit: empty
/// (which *is* the standard's solid line, so it is not a dash), negative,
/// non-finite, or every element zero. The reply that shipped the field said to
/// *"refuse in your own UI or let the `None` refuse for you"*. This shell does
/// the **first**, and here is the decision written down:
///
/// * **Every offered pattern is a compile-time constant** satisfying §8.4.3.6,
///   so the refusal is not reachable by any operator action. There is no numeric
///   entry anywhere in this control, and therefore no operator-supplied array to
///   validate — which is the whole reason refusing in the UI is available at
///   all. A control that let an operator type `0 0` would have to do the other
///   thing.
/// * [`tests::every_offered_pattern_is_one_the_engine_accepts`] asserts it for
///   every variant, so a fifth pattern added carelessly — a typo'd negative, a
///   `[0.0, 0.0]` — fails the build rather than silently becoming *solid* in the
///   operator's file.
/// * Where the `Option` still has to be handled at run time, [`Self::dash`] and
///   [`Self::style_edit`] are the only places, and their callers **park
///   nothing** on a `None`: no substitution, no `unwrap`, no undo entry for a
///   refusal. Substituting [`pdfcer_core::annot_author::BorderDash::table_166_default`]
///   would be this shell writing a pattern the operator did not pick, which is
///   the sneaky half of R8b.
///
/// # ★ Where the four patterns come from, and where they DELIBERATELY do not
///
/// [`Self::Dashed`] is **sourced**: `[3]` is Table 166's own default for `/D` —
/// the pattern the standard gives an annotation that declares `/S /D` and no
/// array (`annot_author.rs:196-203`). It is the one entry here that is not this
/// shell's choice, and it is deliberately the first dash in the list.
///
/// The other two are this shell's, argued rather than measured, and the
/// distinction is stated because [`super::pen::Pen::default`]'s colours *are*
/// measured out of Acrobat's own store and these are not. **Acrobat's `cAnnots`
/// tree holds no dash key**: the search that established there is no Acrobat
/// line-width to match (see `Pen::default`'s own account of it) found no dash,
/// thickness or border entry either. So there is no Adobe pattern to transcribe,
/// and this project's claim-bearing-copy rule forbids inventing one and calling
/// it Adobe's. What decided them instead:
///
/// * they must be **unmistakable from one another** at 100 % over dense CAD
///   linework, which rules out two patterns differing by a fraction of a point;
/// * they should span the two broken-line conventions a draughtsman already
///   reads — a long even dash for hidden geometry, a dash-dot for a centre or
///   reference line. That is a claim about what an operator will **recognise**,
///   not a citation of a line-type standard: ISO 128's run lengths are
///   scale-dependent and this shell has not measured them, so it does not cite
///   them.
///
/// ⇒ If a measurement of Acrobat's own patterns ever turns up, this list should
/// be revisited **with that measurement**, exactly as `Pen::default` says of the
/// 2 pt width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    /// No dash — Table 166's `/S /S`, and the border every mark this shell
    /// authored before 2026-09-06 carried.
    ///
    /// ★ A **member of this enum** rather than the enum wrapped in an `Option`,
    /// because *solid* is a thing an operator picks from the same list with the
    /// same click as the three dashes. The engine models it the same way:
    /// `StyleEdit::Clear` is an arm of the edit, not the absence of one.
    Solid,
    /// Table 166's own default dash, `[3]` — three points on, three points off.
    Dashed,
    /// A long even dash, `[8 4]`.
    LongDash,
    /// A long dash with a dot between, `[8 3 1 3]`.
    DashDot,
}

impl LineStyle {
    /// Every style, in the order all three choosers offer them.
    ///
    /// Solid first because it is the state every mark starts in and the one an
    /// operator reaches for to undo an experiment, and because Table 166 lists
    /// `/S` first. The three dashes then run **shortest to longest**, so the
    /// list reads as increasing distance from a solid line rather than in an
    /// order somebody happened to type.
    ///
    /// A `const` rather than a literal at each call site, for
    /// `panels::properties::markup::ALL_ENDINGS`' reason: three choosers writing
    /// their own lists would come to offer different sets, and the one that went
    /// stale would be the one nobody opened.
    pub const ALL: &'static [LineStyle] = &[
        LineStyle::Solid,
        LineStyle::Dashed,
        LineStyle::LongDash,
        LineStyle::DashDot,
    ];

    /// The `/D` run lengths this style writes, in points, or `None` for solid.
    ///
    /// ★ Exhaustive with no `_` arm, deliberately: a fifth style must be taught
    /// its pattern here or fail to compile, rather than falling into a catch-all
    /// and being authored as somebody else's dash.
    #[must_use]
    pub const fn pattern(self) -> Option<&'static [f64]> {
        match self {
            Self::Solid => None,
            // Table 166's own default — see this type's header on why it is the
            // one sourced entry in the list.
            Self::Dashed => Some(&[3.0]),
            Self::LongDash => Some(&[8.0, 4.0]),
            Self::DashDot => Some(&[8.0, 3.0, 1.0, 3.0]),
        }
    }

    /// **The engine value this style authors with**, or `None` for solid.
    ///
    /// What goes into `MarkupOptions::dash` (`edit.rs:4782`). `None` writes no
    /// dash at all, which is byte-for-byte what this shell authored before the
    /// control existed — so a build whose operator never touches the chooser
    /// produces the same file it did before, which is this project's standing
    /// rule for a capability becoming choosable.
    ///
    /// ⚠ A `None` from `BorderDash::new` would collapse into the same answer as
    /// *solid*. That is why the refusal lives in the choice of constants rather
    /// than here — see this type's header, and
    /// [`tests::every_offered_pattern_is_one_the_engine_accepts`], which is what
    /// makes the collapse unreachable rather than merely unlikely.
    #[must_use]
    pub fn dash(self) -> Option<pdfcer_core::annot_author::BorderDash> {
        self.pattern()
            .and_then(|p| pdfcer_core::annot_author::BorderDash::new(p.to_vec()))
    }

    /// **The restyle edit this style raises**, or `None` when the pattern could
    /// not be built.
    ///
    /// `StyleEdit::Clear` for [`Self::Solid`] — the engine's own spelling of
    /// *make it solid*, rather than the removal of a control's value — and
    /// `StyleEdit::Set` for a dash (`edit.rs:4396-4400`).
    ///
    /// ★ The `None` is what the two restyle surfaces decline on: they park
    /// nothing and raise nothing, so an unbuildable pattern produces no write
    /// and no undo entry. It is unreachable for the four constants above, and it
    /// is *expressed* rather than `expect`ed because a paint-loop panic on a
    /// state that is merely unexpected is a worse failure than a control that
    /// declines.
    #[must_use]
    pub fn style_edit(
        self,
    ) -> Option<pdfcer_core::edit::StyleEdit<pdfcer_core::annot_author::BorderDash>> {
        match self {
            Self::Solid => Some(pdfcer_core::edit::StyleEdit::Clear),
            _ => self.dash().map(pdfcer_core::edit::StyleEdit::Set),
        }
    }

    /// Which offered style a `/D` array **is**, if it is one of them.
    ///
    /// ★ Compared element by element with an exact `==` on `f64`, and that is
    /// correct rather than sloppy: both sides came from the same four literals —
    /// this shell wrote the array, the file stored it, and a small decimal
    /// round-trips through `Object::Real` exactly. A tolerance would make
    /// `[3.0001]` read as *Dashed*, and the next press would silently rewrite it
    /// to `[3]` — the file changing under an operator who touched a different
    /// control.
    #[must_use]
    pub fn of_pattern(pattern: &[f64]) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|s| s.pattern() == Some(pattern))
    }

    /// What the operator reads for this entry.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Solid => t::line_style_solid(),
            Self::Dashed => t::line_style_dashed(),
            Self::LongDash => t::line_style_long_dash(),
            Self::DashDot => t::line_style_dash_dot(),
        }
    }
}

/// **What a mark's `/BS` currently says**, in the terms the choosers can show.
///
/// # ★★★ Why a third variant exists, and why it is not selectable
///
/// A producer's dash is not required to be one of the four this shell offers,
/// and the engine now **preserves** it: a restyle that does not mention `dash`
/// keeps it, *including a dash pdfcer never authored* (`edit.rs:4396-4425`). So
/// a chooser that could only show four states would have to show one of them for
/// a mark that is none of them — and whichever it picked would be a claim about
/// the operator's file that the file does not make.
///
/// [`Self::Foreign`] is that state, named as what it is. It is **displayed and
/// never offered**: picking an entry replaces it, and there is no entry meaning
/// *"put it back"*, because putting it back would need this shell to hold the
/// old array across an undo boundary it does not own. Leaving the chooser alone
/// is what keeps it — which is exactly what the engine's preservation
/// guarantees, and is why *doing nothing* is a real answer here rather than an
/// omission.
///
/// # ⚠ It carries no pattern, and that is a trade rather than an oversight
///
/// Showing the file's own run lengths — *"Dashed 6, 2"* — would be a little more
/// informative and would cost this type its `Copy`, which both restyle surfaces'
/// `Current` structs depend on and pass by value through every control function
/// they have. The state is what the chooser needs: **this mark is dashed, in a
/// pattern that is the file's and not one of ours.** The array itself is `/BS`
/// `/D`'s bytes, and a ribbon combo is not where a file's bytes are read out.
///
/// ⇒ If a surface ever genuinely needs the numbers — a Properties panel line
/// that reports rather than offers — this variant gains a payload and both
/// `Current`s stop being `Copy`. That is a real cost and it should be paid for a
/// real need, not for a nicety.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashReading {
    /// No dash. `/S /S`, a `/BS` carrying neither key, or no `/BS` at all.
    Solid,
    /// A dash whose pattern is one of [`LineStyle::ALL`].
    Offered(LineStyle),
    /// A dash the file states in a pattern this shell does not offer.
    Foreign,
}

impl DashReading {
    /// Which entry of the chooser is selected, if any is.
    ///
    /// `None` for [`Self::Foreign`] — nothing in the list is what the file says,
    /// which is the whole reason that variant exists.
    #[must_use]
    pub const fn selected(self) -> Option<LineStyle> {
        match self {
            Self::Solid => Some(LineStyle::Solid),
            Self::Offered(style) => Some(style),
            Self::Foreign => None,
        }
    }

    /// The text the closed chooser shows.
    ///
    /// ★ For [`Self::Foreign`] it is a sentence about the **file**, not the name
    /// of an entry — see [`t::line_style_foreign`]. A combo whose closed state
    /// showed *Dashed* for a pattern that is not this shell's *Dashed* would be
    /// the quiet lie the swatch's CMYK arm was rewritten to stop telling.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self.selected() {
            Some(style) => style.label(),
            None => t::line_style_foreign(),
        }
    }
}

/// **Read `/BS` back as a [`DashReading`]** — Table 166, §12.5.4.
///
/// # ★★★ Why this is a SECOND reader of a key the engine already reads
///
/// Because the engine's reader is not public. `annot_author::read_border_dash`
/// is `pub(crate)` (`D:\Dev\pdfcer\crates\pdfcer-core\src\annot_author.rs:840`,
/// read 2026-09-06), and `spec_from_dict` does **not** carry the dash: a dash
/// cuts across `MarkupSpec`'s variants rather than belonging to any one of them,
/// so it travels in `AppearanceOptions` beside the spec instead of inside it
/// (`annot_author.rs:1633-1673`). There is therefore no public route from an
/// annotation dictionary to *"is this mark dashed, and how"* — and a control
/// that cannot show the current value is a control that shows an invented one,
/// which is the `fontband::size` defect this project has already paid for.
///
/// ⇒ So this is the copy, and it is **written down as a copy** rather than
/// presented as a reading. The table below is transcribed from
/// `read_border_dash`'s own doc comment (`annot_author.rs:810-830`); it was not
/// derived independently:
///
/// | `/S` | `/D` | read as |
/// |---|---|---|
/// | `/D` | present and usable | that pattern |
/// | `/D` | absent, or unusable | [`LineStyle::Dashed`] — Table 166's `[3]` default |
/// | absent | present and usable | that pattern |
/// | `/S`, `/B`, `/I`, `/U` | either | solid |
///
/// The third row is the one a literal reading gets wrong. `/S` defaults to `/S`,
/// so `/BS << /D [4 2] >>` is *technically* a solid border with a meaningless
/// array — and producers write exactly that, and Acrobat draws it dashed. The
/// engine honours it; so does this, because a chooser that read it as solid
/// would offer *Solid* as the current state of a mark the operator can see is
/// dashed.
///
/// # ⚠ What a divergence between the two readers can and cannot cost
///
/// It can only ever cost the **display**. Nothing is written unless the operator
/// picks an entry, and picking an entry sends an absolute value — `Set(pattern)`
/// or `Clear` — that does not depend on what was read here. So the worst outcome
/// of this copy drifting from the engine's is a chooser showing the wrong
/// current style until it is touched; it cannot silently rewrite a pattern.
///
/// That bound is the reason the copy is acceptable at all, and it is the thing
/// to re-check if anybody ever makes this function's result decide what gets
/// **written**. **If the engine publishes `read_border_dash`, delete this and
/// call it** — recorded on `NO_SURFACE.md`'s boundary table.
#[must_use]
pub fn read<G: pdfcer_core::graph::ObjectGraph + ?Sized>(
    graph: &G,
    annot: &pdfcer_core::object::Dict,
) -> DashReading {
    use pdfcer_core::object::Object;

    let Some(Object::Dict(bs)) = annot.get(b"BS").map(|o| graph.resolve(o)) else {
        return DashReading::Solid;
    };
    // Table 166: `/S` defaults to `/S` (solid). `/B`, `/I` and `/U` are borders
    // pdfcer does not author and are not dashes either — the engine returns
    // early on all three, and so does this.
    let declared_dashed = match bs.get(b"S").map(|o| graph.resolve(o)) {
        Some(Object::Name(n)) => match n.as_bytes() {
            b"D" => true,
            _ => return DashReading::Solid,
        },
        // `/S` absent: not dashed on its own, but a `/D` alongside is honoured.
        // The doc comment's third row.
        _ => false,
    };
    let pattern: Option<Vec<f64>> = match bs.get(b"D").map(|o| graph.resolve(o)) {
        Some(Object::Array(items)) => {
            let read: Vec<f64> = items
                .iter()
                .filter_map(|o| graph.resolve(o).as_number())
                .collect();
            // §8.4.3.6, mirrored from `BorderDash::new`: empty IS the solid
            // line, and an array that is negative, non-finite or all-zero
            // describes no line at all. Either way there is no pattern to show.
            let usable = !read.is_empty()
                && read.iter().all(|v| v.is_finite() && *v >= 0.0)
                && read.iter().any(|v| *v != 0.0);
            usable.then_some(read)
        }
        _ => None,
    };
    match pattern {
        Some(p) => LineStyle::of_pattern(&p).map_or(DashReading::Foreign, DashReading::Offered),
        // A `/D` that is present but unusable is a dropped pattern, not a reason
        // to call a declared-dashed border solid: the engine falls back to the
        // table default there, so `/S /D` still reads as dashed, and so does
        // this.
        None if declared_dashed => DashReading::Offered(LineStyle::Dashed),
        None => DashReading::Solid,
    }
}

/// **The chooser**, drawn once and used by all three surfaces.
///
/// Returns the style the operator picked, or `None` for a frame in which they
/// merely looked at it — the same *nothing was invoked* contract
/// `app::markupband`'s controls have.
///
/// # ★ It never reports the style that is already showing
///
/// Selecting the entry a mark already has must raise nothing: on the restyle
/// surfaces that would be an undo entry the operator did not earn and a re-bake
/// of an appearance that is already right, and on the pen it would be a trace
/// line for a change that did not happen. The comparison is against
/// [`DashReading::selected`], so a [`DashReading::Foreign`] mark reports on
/// **every** pick — which is correct: none of the four is what it currently is.
///
/// # ★★ `width` is the caller's, because the three surfaces have different room
///
/// The ribbon band budgets `CUSTOM_ITEM_WIDTH` per custom item and a Properties
/// panel row does not; passing it in is what lets one widget serve both without
/// the band's constraint leaking into the panel or vice versa.
pub fn chooser(ui: &mut Ui, id_salt: &str, current: DashReading, width: f32) -> Option<LineStyle> {
    let mut picked = None;
    egui::ComboBox::from_id_salt(id_salt)
        .width(width)
        .selected_text(current.label())
        .show_ui(ui, |ui| {
            let selected = current.selected();
            for &style in LineStyle::ALL {
                let is_current = selected == Some(style);
                if ui.selectable_label(is_current, style.label()).clicked() && !is_current {
                    picked = Some(style);
                }
            }
        });
    picked
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A graph in which every value is already direct.
    ///
    /// See [`table_166_is_read_back_the_way_the_engine_reads_it`] for why that
    /// is the right instrument for this reader rather than a weakened one.
    struct DirectGraph;

    impl pdfcer_core::graph::ObjectGraph for DirectGraph {
        fn value(&self, _id: pdfcer_core::object::ObjId) -> Option<&pdfcer_core::object::Object> {
            None
        }

        fn trailer_entry(&self, _key: &[u8]) -> Option<&pdfcer_core::object::Object> {
            None
        }
    }

    /// ★★★ **Every pattern this shell offers is one `BorderDash::new` accepts.**
    ///
    /// This is the assertion that makes *"refuse in the UI"* a real decision
    /// rather than a hope. §8.4.3.6 refuses an empty, negative, non-finite or
    /// all-zero array, and `BorderDash::new` answers `None` for each
    /// (`annot_author.rs:185`). A `None` reaching [`LineStyle::dash`] would
    /// collapse into the same answer as *solid* — so a fifth pattern typed with
    /// a stray minus sign would produce a chooser entry that silently drew a
    /// solid line, with no error anywhere.
    ///
    /// ★ **It is paired with a positive control**, and that is deliberate: the
    /// engine's own reply of 2026-09-06 records that its first foreign-appearance
    /// test asserted `!appearance_rebaked` and **passed with the whole feature
    /// disabled**. An assertion that `dash()` is `Some` for every dashed variant
    /// would be vacuous if `dash()` returned `Some` for everything, so the
    /// solid arm asserting `None` sits beside it — the two together say the
    /// function discriminates rather than merely answering.
    ///
    /// Falsified by changing [`LineStyle::LongDash`]'s pattern to `[0.0, 0.0]`,
    /// which turned the `is_some` assertion red, and by giving
    /// [`LineStyle::Solid`] a pattern, which turned the control red.
    #[test]
    fn every_offered_pattern_is_one_the_engine_accepts() {
        for &style in LineStyle::ALL {
            match style.pattern() {
                Some(pattern) => {
                    assert!(
                        !pattern.is_empty(),
                        "{style:?}: an empty array IS the solid line, not a dash"
                    );
                    assert!(
                        pattern.iter().all(|v| v.is_finite() && *v >= 0.0),
                        "{style:?}: §8.4.3.6 admits no negative or non-finite element"
                    );
                    assert!(
                        pattern.iter().any(|v| *v != 0.0),
                        "{style:?}: an all-zero pattern is never on and never off"
                    );
                    assert!(
                        style.dash().is_some(),
                        "{style:?}: BorderDash::new refused it, so this entry would draw SOLID"
                    );
                }
                // The positive control's other half: solid must have no pattern
                // and must build no dash, or `dash()`'s `Some`/`None` split
                // carries no information and the assertions above prove nothing.
                None => assert_eq!(
                    style,
                    LineStyle::Solid,
                    "only Solid may have no pattern; {style:?} would author a solid border"
                ),
            }
        }
        assert!(
            LineStyle::Solid.dash().is_none(),
            "solid must not build a dash, or the two states are one"
        );
    }

    /// **Solid clears, a dash sets** — the engine's two arms, not one.
    ///
    /// `Clear` makes the border solid and `Set` makes it dashed
    /// (`edit.rs:4396-4400`). Getting this backwards, or answering `None` for
    /// solid, would make the chooser's first entry a control that does nothing:
    /// `MarkupStyle::dash: None` means *leave whatever the annotation already
    /// has*, so a Solid that raised `None` would silently keep the dash it was
    /// pressed to remove.
    ///
    /// Falsified by returning `None` from the `Solid` arm of `style_edit`, which
    /// turned the first assertion red.
    #[test]
    fn solid_clears_the_dash_and_a_dash_sets_one() {
        use pdfcer_core::edit::StyleEdit;
        assert!(
            matches!(LineStyle::Solid.style_edit(), Some(StyleEdit::Clear)),
            "solid must CLEAR; `None` would leave the mark's existing dash alone"
        );
        for &style in LineStyle::ALL {
            if style == LineStyle::Solid {
                continue;
            }
            let Some(StyleEdit::Set(dash)) = style.style_edit() else {
                panic!("{style:?} must Set a dash");
            };
            assert_eq!(
                dash.pattern(),
                style.pattern().expect("a dash has a pattern"),
                "{style:?} set a pattern that is not its own"
            );
        }
    }

    /// The four entries are distinct in **both** the things that distinguish
    /// them — their patterns and their names.
    ///
    /// ★ Two entries with one pattern would be a list that looks like a choice
    /// and is not; two with one label would be a combo an operator cannot read.
    /// The order is asserted too, because it is argued in [`LineStyle::ALL`]'s
    /// doc and a reordering is a change to the control's shape rather than to
    /// its wording.
    #[test]
    fn the_four_styles_are_distinct_and_ordered() {
        assert_eq!(
            LineStyle::ALL,
            [
                LineStyle::Solid,
                LineStyle::Dashed,
                LineStyle::LongDash,
                LineStyle::DashDot
            ]
            .as_slice()
        );
        let mut labels: Vec<&str> = LineStyle::ALL.iter().map(|s| s.label()).collect();
        for label in &labels {
            assert!(!label.trim().is_empty());
        }
        let total = labels.len();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), total, "two styles share a label");

        let mut patterns: Vec<Option<&[f64]>> =
            LineStyle::ALL.iter().map(|s| s.pattern()).collect();
        let total = patterns.len();
        patterns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        patterns.dedup();
        assert_eq!(patterns.len(), total, "two styles share a pattern");
    }

    /// ★★ **A pattern round-trips through [`LineStyle::of_pattern`], and a
    /// foreign one does not become one of ours.**
    ///
    /// The property the [`DashReading::Foreign`] variant rests on: a producer's
    /// `[6 2]` must not be reported as this shell's `Dashed`, because the
    /// chooser would then show *Dashed* over a mark that is not, and a press on
    /// any other entry would look like a change from a state the file never had.
    ///
    /// Falsified by making `of_pattern` compare only the first element, which
    /// turned the `[8.0, 9.0]` assertion red.
    #[test]
    fn a_foreign_pattern_is_not_mistaken_for_one_this_shell_offers() {
        for &style in LineStyle::ALL {
            if let Some(pattern) = style.pattern() {
                assert_eq!(LineStyle::of_pattern(pattern), Some(style));
            }
        }
        assert_eq!(LineStyle::of_pattern(&[6.0, 2.0]), None);
        assert_eq!(LineStyle::of_pattern(&[8.0, 9.0]), None);
        assert_eq!(
            LineStyle::of_pattern(&[]),
            None,
            "an empty array is the solid line and is not a dash entry"
        );
        assert_eq!(
            LineStyle::of_pattern(&[3.0, 3.0]),
            None,
            "Table 166's [3] is not [3 3]; a near-miss is still the file's own"
        );
    }

    /// A reading names an entry, or names the file — never nothing, and never
    /// one of ours for a pattern that is not.
    #[test]
    fn a_reading_shows_the_entry_it_is_or_says_the_file_owns_it() {
        assert_eq!(DashReading::Solid.selected(), Some(LineStyle::Solid));
        assert_eq!(
            DashReading::Offered(LineStyle::DashDot).selected(),
            Some(LineStyle::DashDot)
        );
        assert_eq!(
            DashReading::Foreign.selected(),
            None,
            "no entry may be shown as selected for a pattern this shell does not offer"
        );
        assert_eq!(DashReading::Foreign.label(), t::line_style_foreign());
        for &style in LineStyle::ALL {
            assert_eq!(DashReading::Offered(style).label(), style.label());
        }
    }

    /// ★★★ **Table 166 read back, all four rows, including the two a literal
    /// reading gets wrong.**
    ///
    /// # ★ The graph is a **direct** one, and that is the right instrument
    ///
    /// [`DirectGraph`] answers `None` for every id, so `resolve` returns each
    /// value unchanged. That is not a weakened test: reference-following is
    /// `ObjectGraph::resolve`'s **default method**, written and tested in the
    /// engine, and this module contributes nothing to it. What this module
    /// contributes is the decision about Table 166's key combinations, and those
    /// are what the rows below vary. A session-backed graph would exercise the
    /// engine's resolver a seventh time and this reader's table zero extra
    /// times.
    ///
    /// The rows, and what each one would cost if it were wrong:
    ///
    /// | dictionary | expected | the failure it prevents |
    /// |---|---|---|
    /// | no `/BS` | solid | a chooser showing *Dashed* on every unbordered mark |
    /// | `/S /S` | solid | the same |
    /// | `/S /D`, no `/D` array | `Dashed` — Table 166's `[3]` | *Solid* shown over a mark Acrobat draws dashed |
    /// | `/D [4 2]`, no `/S` | `Foreign` | the same, on the producer shape the engine's own doc calls out |
    /// | `/S /D` + `[8 4]` | `LongDash` | a pattern of ours reported as the file's |
    /// | `/S /B` | solid | a bevelled border offered as a dash |
    /// | `/S /D` + `[0 0]` | `Dashed` | a §8.4.3.6-invalid array read as a pattern |
    ///
    /// Falsified by deleting the `declared_dashed` fallback (the `/S /D` rows
    /// went red) and by removing the `/S` early return (the `/S /B` row did).
    #[test]
    fn table_166_is_read_back_the_way_the_engine_reads_it() {
        use pdfcer_core::object::{Dict, Name, Object};

        let graph = DirectGraph;

        let mut plain = Dict::new();
        plain.insert(Name::from(b"Subtype"), Object::Name(Name::from(b"Square")));
        assert_eq!(read(&graph, &plain), DashReading::Solid, "no /BS at all");

        let bs = |entries: Vec<(&[u8], Object)>| {
            let mut inner = Dict::new();
            for (k, v) in entries {
                inner.insert(Name::from(k), v);
            }
            let mut annot = Dict::new();
            annot.insert(Name::from(b"BS"), Object::Dict(inner));
            annot
        };
        let array =
            |values: &[f64]| Object::Array(values.iter().copied().map(Object::Real).collect());

        assert_eq!(
            read(&graph, &bs(vec![(b"S", Object::Name(Name::from(b"S")))])),
            DashReading::Solid,
            "/S /S is the solid border"
        );
        assert_eq!(
            read(&graph, &bs(vec![(b"S", Object::Name(Name::from(b"D")))])),
            DashReading::Offered(LineStyle::Dashed),
            "/S /D with no array is Table 166's [3], not solid"
        );
        assert_eq!(
            read(&graph, &bs(vec![(b"D", array(&[4.0, 2.0]))])),
            DashReading::Foreign,
            "a /D array with no /S is honoured, and [4 2] is not one this shell offers"
        );
        assert_eq!(
            read(&graph, &bs(vec![(b"D", array(&[8.0, 4.0]))])),
            DashReading::Offered(LineStyle::LongDash),
            "…and one that IS ours is named as ours"
        );
        assert_eq!(
            read(&graph, &bs(vec![(b"S", Object::Name(Name::from(b"B")))])),
            DashReading::Solid,
            "a bevelled border is not a dash"
        );
        assert_eq!(
            read(
                &graph,
                &bs(vec![
                    (b"S", Object::Name(Name::from(b"D"))),
                    (b"D", array(&[0.0, 0.0])),
                ])
            ),
            DashReading::Offered(LineStyle::Dashed),
            "an unusable array falls back to the table default rather than to solid"
        );
        assert_eq!(
            read(
                &graph,
                &bs(vec![
                    (b"S", Object::Name(Name::from(b"S"))),
                    (b"D", array(&[0.0, 0.0])),
                ])
            ),
            DashReading::Solid,
            "…but an unusable array on a SOLID border stays solid"
        );
    }
}
