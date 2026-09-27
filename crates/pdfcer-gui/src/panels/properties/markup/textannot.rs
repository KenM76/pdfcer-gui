//! # `panels::properties::markup::textannot` — restyling the marks that carry
//! WORDS
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/markup/textannot.md`.

use egui::Ui;
use pdfcer_core::annot::{StampLabelParameters, StampSizeSource};
use pdfcer_core::annot_author::{Color, StickyIcon, TextAnnotSpec};
use pdfcer_core::edit::TextAnnotStyle;
use pdfcer_gui_base::entry;

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::text::panels::properties as t;
use crate::text::panels::textannotstyle as ts;
use crate::text::textannot as tt;

/// The region this subsection publishes, so a driven check can find the icon
/// chooser on a placed note rather than only on the placing dialog.
pub(super) const REGION: &str = "properties.markup.textannot"; // ui-text-exempt: trace region name, never displayed

/// The **label-size spinner**'s own region, published only when the row is
/// actually on screen.
pub(super) const SIZE_REGION: &str = "properties.markup.textannot.size"; // ui-text-exempt: trace region name, never displayed

/// The **fit chooser**'s region — see [`SIZE_REGION`] for the argument.
pub(super) const FIT_REGION: &str = "properties.markup.textannot.fit"; // ui-text-exempt: trace region name, never displayed

/// The trace slot the label row reports **its own reading** through.
///
/// ```text
/// pdfcer-diag stamp-label-row size=24 source=declared-in-da fit=grow
/// ```
///
/// # Why a line, when the number is on the screen
///
/// Because a driven check cannot read a number off a screenshot, and the
/// alternative is a check that *describes* the absence it never measured — an
/// unevidenced excuse, which reads as an answered question and stops anybody
/// looking again. `stamp-size-chooser` is this line's twin on the placing
/// dialog and exists for the identical reason.
///
/// It carries `source=` as well as `size=`, because the two answer different
/// questions. `size=` is what the operator sees. `source=` is where it came
/// from, and it is the only way a check can tell *"the file declared 24"* from
/// *"pdfcer read 24 off the picture because the file declares nothing"* — a
/// distinction no screenshot contains and the whole reason
/// `StampSizeSource` has three variants rather than being an `Option`.
const ROW_SLOT: &str = "stamp-label-row"; // ui-text-exempt: diagnostic trace slot, never displayed

/// **Which of `pdfcer-core`'s TWO annotation-style verbs reaches the
/// selected mark** — the guard between them, as a `match` the compiler
/// checks.
#[derive(Debug, Clone)]
pub(super) enum Reach {
    /// `EditSession::set_markup_style` — the geometric family and the four text
    /// markups. `annot_author::spec_from_dict` read a spec.
    Markup,
    /// `EditSession::set_text_annot_style` — a `/Text` or a `/Stamp`.
    /// `annot_author::text_spec_from_dict` read a spec and
    /// [`Reading::of`] accepted the face.
    TextAnnot(Reading),
    /// A `/FreeText`. Reachable by the second verb and **declined here**.
    TextBoxWithheld,
    /// Neither reader could produce a spec: a `/Subtype` outside both families,
    /// or geometry pdfcer does not model.
    Neither,
}

/// **Which text-bearing face is selected** — the closed set
/// `set_text_annot_style` is offered for by this shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Face {
    /// `/Text` — a sticky note. Icon and colour.
    Sticky,
    /// `/Stamp` — a rubber stamp. Colour only: its face comes from its own
    /// `/Name` vocabulary (Table 181), which is not Table 172's, and the engine
    /// refuses a `StickyIcon` on one **by name** rather than swallowing it
    /// (`EditError::StylePropertyNotApplicable`).
    Stamp,
}

impl Face {
    /// Whether this face takes an icon.
    pub(super) const fn takes_icon(self) -> bool {
        match self {
            Self::Sticky => true,
            Self::Stamp => false,
        }
    }
}

/// What the selected text-bearing mark's dictionary currently says, in the
/// terms this subsection can change.
#[derive(Debug, Clone)]
pub(super) struct Reading {
    /// Which face, and therefore which properties mean anything.
    pub(super) face: Face,
    /// `/C`, as a swatch can show it, and whether showing it cost a conversion.
    ///
    /// Through [`super::swatch_of`], the parent's function, rather than a
    /// second conversion written here. The CMYK narrowing disclosure is a
    /// property of *showing a `/C` in an sRGB button* and has nothing to do
    /// with which verb writes it back — two copies of that arithmetic would be
    /// two chances to disagree about a colour on the operator's sheet.
    pub(super) colour: super::Swatch,
    /// `/Name`, for a `/Text` — **including a name pdfcer does not model**,
    /// which arrives as [`StickyIcon::Other`].
    ///
    /// **`None` means exactly one thing: this face has no icon** — i.e. a
    /// `/Stamp`, whose `/Name` is a stamp face from a different vocabulary
    /// altogether. It never means *"the `/Name` is one pdfcer does not model"*:
    /// the engine's reader is lossless, so such a name arrives as
    /// [`StickyIcon::Other`] and is a value like any other.
    pub(super) icon: Option<StickyIcon>,
    /// **`true` when the file's `/Name` is a name pdfcer does not model.**
    ///
    /// §12.5.6.4's seven are *"a standard set, not a closed one"*, so a
    /// producer's own icon name is conforming and this is a legitimate state,
    /// not a defect.
    ///
    /// # What it is FOR
    ///
    /// The name round-trips, so this is not a warning about destruction. What
    /// it says is that **pdfcer draws its own picture for it**: the engine
    /// paints the same glyph for all seven standard variants, so the icon
    /// chooses the `/Name` written, not the picture drawn. An operator
    /// comparing this window with Acrobat's is entitled to know why the two
    /// differ.
    ///
    /// ⚠ The disclosure this drives must **not** promise that the name will be
    /// replaced — a colour-only restyle carries it through untouched, so such a
    /// sentence would be false. See
    /// `text::panels::textannotstyle::markup_icon_foreign_note`.
    ///
    /// Derived from the **spec**, never from a second read of the dictionary:
    /// a second reader of a structure `pdfcer-core` owns is a second chance to
    /// disagree with the engine about an operator's file.
    pub(super) foreign_icon: bool,
    /// **What the stamp's own appearance says its label is** — the words, the
    /// size in points, and where that size came from. `None` for a sticky note,
    /// and `None` for a stamp whose picture shows no text pdfcer can read a
    /// size off.
    ///
    /// # Why this is NOT filled by [`Reading::of`]
    ///
    /// Because `of` is **pure** and takes the spec and nothing else, which is
    /// what lets six tests build a `Reading` in one expression. The label
    /// parameters are not in the spec at all — they are recovered by parsing
    /// the annotation's `/AP` `/N` stream, so reading them needs the session,
    /// the object graph and the R45 staging buffer behind it.
    ///
    /// ⇒ The read stays in [`super::Reach::read`], where the session already
    /// is, and arrives here through [`Reading::with_stamp_label`]. That keeps
    /// the impure half in the one function that was always impure, rather than
    /// making every test of the pure half construct a document.
    ///
    /// ⚠ A `Reading` built by `of` alone therefore has `label: None`, which is
    /// indistinguishable from *"this stamp has no describable label"*. That is
    /// deliberate and it is safe in the only direction that matters: the size
    /// row is **absent** rather than wrong. A test that means to assert the row
    /// appears must call `with_stamp_label`.
    pub(super) label: Option<StampLabelParameters>,
}

impl Reading {
    /// **Read one, or answer `None` for a mark this subsection does not serve.**
    pub(super) fn of(spec: &TextAnnotSpec) -> Option<Self> {
        match spec {
            TextAnnotSpec::Sticky { color, icon, .. } => Some(Self {
                face: Face::Sticky,
                colour: super::swatch_of(Some(color)),
                // An ABSENT `/Name` is not foreign, and the engine's reader
                // already draws that line for us: Table 172's default is
                // `Note`, so a note carrying no `/Name` arrives as `Note` and
                // showing `Note` for it is reporting the standard rather than
                // inventing anything.
                foreign_icon: matches!(icon, StickyIcon::Other(_)),
                icon: Some(icon.clone()),
                // A sticky note draws an ICON, not text. `set_text_annot_style`
                // refuses a label size on one BY NAME
                // (`StylePropertyNotApplicable`, property "a label font
                // size"), so this is not "we did not read it" — there is
                // nothing to read.
                label: None,
            }),
            TextAnnotSpec::Stamp { color, .. } => Some(Self {
                face: Face::Stamp,
                colour: super::swatch_of(Some(color)),
                icon: None,
                foreign_icon: false,
                // Filled by [`Self::with_stamp_label`] from the session; see the
                // field's own note on why `of` cannot.
                label: None,
            }),
            // (2) above. The verb would take it; this shell will not send it.
            TextAnnotSpec::FreeText { .. } => None,
            // `TextAnnotSpec` is `#[non_exhaustive]`. A fourth text-bearing
            // face this build does not know the shape of gets no rows and the
            // withheld sentence — the same answer a `/FreeText` gets, and for a
            // compatible reason: this shell cannot say what a control over it
            // would do. R9 in the conservative direction, which is also
            // `MarkupStyleSupport::for_subtype`'s own stated posture.
            _ => None,
        }
    }

    /// **Carry the stamp's label parameters in**, read from the session by
    /// [`super::Reach::read`].
    #[must_use]
    pub(super) fn with_stamp_label(mut self, label: Option<StampLabelParameters>) -> Self {
        self.label = label;
        self
    }
}

/// **Draw the rows `set_text_annot_style` can commit.**
pub(super) fn rows(
    ui: &mut Ui,
    current: &Reading,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    crate::diag::ui_rect(REGION, ui.max_rect());
    // **What this subsection is showing, on the trace channel.**
    //
    // The published region says *the rows drew*; it cannot say **what they
    // say**. That distinction is the whole reason this line exists: a build
    // that flattened a producer's `/Sparkle` to `Note` and one that carried it
    // draw the same rectangle, in the same place, with the same number of
    // controls — and differ only in the words inside the combo, which no rect
    // carries.
    //
    // `icon=` is the NAME as the file spells it, lossily decoded, not a
    // variant label. A check reading `icon=Note` cannot tell the flattened case
    // from a note that genuinely says `Note`; reading `icon=Sparkle` on a
    // fixture planted with `/Sparkle` can. `foreign=` is the panel's own
    // verdict beside it, so a build that carried the name and forgot to
    // disclose is a different line from one that did neither.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "textannot-rows face={:?} icon={} foreign={}",
            current.face,
            current.icon.as_ref().map_or_else(
                || "none".to_owned(),
                |i| String::from_utf8_lossy(i.name()).into_owned()
            ),
            u8::from(current.foreign_icon),
        )
    });
    colour_row(ui, current, target, actions);
    // The narrowing disclosure sits directly under the swatch it qualifies,
    // which is also the parent's placement. A caveat placed after the NEXT
    // control arrives once the operator has already drawn their conclusion, so
    // it must sit against its own subject and before anything else.
    if current.colour.narrowed {
        ui.label(
            egui::RichText::new(t::markup_colour_narrowed())
                .small()
                .weak(),
        );
    }
    // The size sits between the colour and the icon, and the two never
    // appear together: a stamp takes a size and no icon, a sticky note takes an
    // icon and no size. It is placed here rather than last so that the ORDER a
    // reader sees is stable across the two faces — colour, then whatever the
    // face's own property is — rather than the icon jumping above the size on
    // one selection and below it on the next.
    size_row(ui, current, target, actions);
    icon_row(ui, current, target, actions);
    ui.label(
        egui::RichText::new(ts::markup_text_annot_note())
            .small()
            .weak(),
    );
}

/// The smallest and largest label size the spinner offers, in points.
const MIN_LABEL_PT: f64 = 4.0;
/// See [`MIN_LABEL_PT`].
const MAX_LABEL_PT: f64 = 144.0;

/// **The stamp's label size, and what to do when it stops fitting.**
fn size_row(
    ui: &mut Ui,
    current: &Reading,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    // Two guards, and they are not the same guard twice. The first is about
    // the FACE — a sticky note draws an icon and has no label to size, which is
    // the refusal `set_text_annot_style` makes by name
    // (`StylePropertyNotApplicable`, property "a label font size"). The second
    // is about this PARTICULAR stamp — the face is right and its appearance
    // still shows no text pdfcer can read a size off.
    if current.face != Face::Stamp {
        return;
    }
    let Some(label) = current.label.as_ref() else {
        return;
    };

    let read_size = label.size;
    let mut size = read_size;
    let ctx = ui.ctx().clone();
    let mut fit = crate::canvas::stampfit::read(&ctx);

    // The row's own reading, before anything is pressed. `trace_changed`
    // rather than `trace`, so a panel drawn at sixty frames a second writes one
    // line per actual change; see [`ROW_SLOT`] for why the line exists at all
    // and why it carries `source=`.
    crate::diag::trace_changed(ROW_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed. The exemption sits
            // HERE and not above `format!` because check-ui-strings reads the comment
            // block immediately above the LITERAL; one intervening line of code and
            // the reason is invisible to it.
            "{ROW_SLOT} size={read_size} source={} fit={}",
            source_token(label.size_source),
            crate::canvas::stampfit::trace_token(fit)
        )
    });

    ui.horizontal(|ui| {
        ui.label(ts::stamp_text_size_label());
        let (widget, refusal) =
            entry::drag_value(ui, &mut size, entry::Kind::Length(entry::LengthUnit::Point));
        let response = refusal.show(
            ui.add(
                widget
                    // The range ADMITS whatever the file said. See
                    // [`MIN_LABEL_PT`] — a spinner that clamps a value it did not
                    // author is a spinner that edits documents nobody asked it to.
                    .range(MIN_LABEL_PT.min(read_size)..=MAX_LABEL_PT.max(read_size))
                    .speed(0.5)
                    .suffix(ts::stamp_text_size_suffix()),
            ),
        );
        // `drag_stopped` and `lost_focus`, never `changed` — the parent's
        // `width_row` carries the full argument. A `DragValue` reports a change
        // on every pixel of a drag and each one here is an appearance re-bake
        // plus an undo entry, so one drag across the control would leave forty
        // entries on the stack.
        //
        // **And `size != read_size` beside it**, which `width_row` does not
        // need and this one does. A `lost_focus` fires when the operator clicks
        // away having changed nothing, and on a stamp whose declared size lies
        // outside this shell's range the displayed number is not the file's
        // number — so an unconditional commit here would rewrite a `/DA` the
        // operator never touched, on a mark they only looked at.
        // `ui_rect_visible`, and the clip rect is the panel's — not the
        // window's. A row scrolled below the properties panel's viewport is
        // still laid out and still has a rectangle; publishing it would hand a
        // driven check a coordinate that lands on whatever is drawn over it,
        // and the resulting report would name this feature for a defect in the
        // panel above it.
        crate::diag::ui_rect_visible(SIZE_REGION, response.rect, ui.clip_rect());
        if (response.drag_stopped() || response.lost_focus()) && size != read_size {
            push_size(target, size, fit, actions);
        }
    });

    // The fit chooser sits UNDER the number it qualifies, because it is
    // read at the moment the operator has just typed a larger one and is
    // wondering what will happen. The placement rule for a *caveat* cuts the
    // other way — a warning below the thing it warns about arrives after the
    // conclusion has been drawn — but this is not a caveat, it is the second
    // half of one instruction, and an operator reads the two in the order they
    // are committed.
    ui.horizontal(|ui| {
        ui.label(ts::stamp_fit_label());
        let combo = egui::ComboBox::from_id_salt("properties-stamp-fit") // ui-text-exempt: widget id salt, never displayed.
            .selected_text(ts::stamp_fit_option(fit))
            .show_ui(ui, |ui| {
                // `stampfit::FITS`, never a hand-written list here. A
                // completeness test keys on that constant, and a second list
                // written out at a call site is invisible to it — the exact
                // shape of defect this project has a standing rule about.
                for policy in crate::canvas::stampfit::FITS.iter().copied() {
                    ui.selectable_value(&mut fit, policy, ts::stamp_fit_option(policy));
                }
            });
        crate::diag::ui_rect_visible(FIT_REGION, combo.response.rect, ui.clip_rect());
    });
    if fit != crate::canvas::stampfit::read(&ctx) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "stamp-fit-chosen {}",
                crate::canvas::stampfit::trace_token(fit)
            )
        });
        crate::canvas::stampfit::store(&ctx, fit);
    }

    // ⚠ **One source of the three owes a sentence, and it is not the one a
    // shell reaches for first.** `RecoveredFromAppearance` — no `/DA` at all,
    // the size read off the baked `Tf` — covers *every stamp whose producer
    // wrote no `/DA`, which is most of them*, and the engine is explicit that
    // it is "not an anomaly and owes no warning … the number is exactly what is
    // on the page". A caution there would fire on the majority
    // of stamps in the world and teach the operator to ignore the one that
    // matters. `DaUnreadable` is the one that matters: the file states a size,
    // pdfcer cannot parse it, and setting one here overwrites it.
    if label.size_source == StampSizeSource::DaUnreadable {
        ui.label(
            egui::RichText::new(ts::stamp_size_da_unreadable())
                .small()
                .weak(),
        );
    }
}

/// **Where the displayed label size came from**, as one word for the trace.
pub(super) fn source_token(source: StampSizeSource) -> &'static str {
    // ui-text-exempt: diagnostic trace tokens, never displayed.
    match source {
        StampSizeSource::DeclaredInDa => "declared-in-da",
        StampSizeSource::RecoveredFromAppearance => "recovered-from-appearance",
        StampSizeSource::DaUnreadable => "da-unreadable",
        _ => "unknown",
    }
}

/// Raise the restyle that carries a new label size — and **nothing else**.
fn push_size(
    target: &crate::canvas::selection::annot::AnnotTarget,
    size: f64,
    fit: pdfcer_core::annot_author::StampFit,
    actions: &mut Vec<Action>,
) {
    actions.push(Action::Annot(AnnotAction::SetTextAnnotStyle {
        id: target.id,
        style: TextAnnotStyle {
            font_size: Some(size),
            // Named even though it equals the engine's default, because the
            // operator has an opinion about it and a `None` here would hide
            // that the chooser above had been read at all. `stamp_fit` is
            // "ignored unless `font_size` is set" — which is exactly the call
            // this is.
            stamp_fit: Some(fit),
            color: None,
            icon: None,
        },
    }));
}

/// The annotation's colour, `/C`.
fn colour_row(
    ui: &mut Ui,
    current: &Reading,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    // The fallback is the mark's own default rather than black, and it
    // matters here in a way it does not in the parent: `text_spec_from_dict`
    // supplies a default `/C` when the key is absent (yellow for a note, black
    // for a stamp), so `current.colour.rgb` is `None` only when the value is
    // present and names no device space §8.6.3 defines. Black is then as good
    // an answer as any and the operator's first pick replaces it.
    let mut rgb = current.colour.rgb.unwrap_or([0, 0, 0]);
    ui.horizontal(|ui| {
        ui.label(t::markup_colour_label());
        if ui.color_edit_button_srgb(&mut rgb).changed() {
            actions.push(Action::Annot(AnnotAction::SetTextAnnotStyle {
                id: target.id,
                style: TextAnnotStyle {
                    color: Some(Color::Rgb(
                        f64::from(rgb[0]) / 255.0,
                        f64::from(rgb[1]) / 255.0,
                        f64::from(rgb[2]) / 255.0,
                    )),
                    // Explicit `None` on EVERY other field, and it is
                    // the contract rather than a formality: "a field left
                    // `None` is left alone", so a call that names only the
                    // colour does not touch the icon, the label size or the
                    // fit policy.
                    //
                    // **Spelt out rather than reached through
                    // `..Default::default()`, and that is load-bearing rather
                    // than tidy.** When the engine grows a field on this
                    // struct, an exhaustive literal breaks the build here —
                    // which is the outcome we want. `..Default::default()`
                    // would compile silently and take the new field's `None`,
                    // i.e. it would DECLINE a new capability on the operator's
                    // behalf without a single word appearing anywhere. A
                    // compile error is an invitation to read the engine's
                    // reply; a default is a way of not receiving it.
                    icon: None,
                    font_size: None,
                    stamp_fit: None,
                },
            }));
        }
    });
}

/// **The sticky note's icon, `/Name`** (§12.5.6.4, Table 172).
fn icon_row(
    ui: &mut Ui,
    current: &Reading,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    if !current.face.takes_icon() {
        return;
    }
    ui.horizontal(|ui| {
        ui.label(tt::sticky_icon_heading());
        let mut chosen = current.icon.clone();
        egui::ComboBox::from_id_salt("properties-textannot-icon") // ui-text-exempt: internal widget id, never displayed
            .selected_text(match &current.icon {
                // **The file's own name, shown as the file spells it.**
                // The engine's reader carries an unmodelled `/Name` through as
                // [`StickyIcon::Other`], so this shell has the bytes and the
                // honest thing is to print them rather than to show a
                // placeholder for a value the document states plainly.
                Some(StickyIcon::Other(name)) => {
                    ts::markup_icon_foreign_named(&String::from_utf8_lossy(name))
                }
                Some(icon) => tt::sticky_icon_label(icon).to_owned(),
                // A face with no icon at all reaches this only through a build
                // error — `takes_icon` returned above — so it says nothing
                // rather than inventing an entry.
                None => String::new(),
            })
            .show_ui(ui, |ui| {
                // **The file's own name is the FIRST entry when it is not
                // one of the seven**, and this is not decoration. A combo whose
                // current value is absent from its own list is a one-way door:
                // the operator opens it to look, picks something to see what it
                // does, and cannot get back to what the file said. Offering it
                // costs one row and it is the only row that can restore the
                // document's own state.
                if let Some(other @ StickyIcon::Other(name)) = &current.icon {
                    ui.selectable_value(
                        &mut chosen,
                        Some(other.clone()),
                        ts::markup_icon_foreign_named(&String::from_utf8_lossy(name)),
                    );
                    ui.separator();
                }
                for icon in crate::canvas::textannot::STICKY_ICONS {
                    ui.selectable_value(
                        &mut chosen,
                        Some(icon.clone()),
                        tt::sticky_icon_label(icon),
                    );
                }
            });
        if let Some(icon) = chosen
            && Some(&icon) != current.icon.as_ref()
        {
            actions.push(Action::Annot(AnnotAction::SetTextAnnotStyle {
                id: target.id,
                style: TextAnnotStyle {
                    icon: Some(icon),
                    // Left alone — see [`colour_row`]'s note on naming every
                    // other field explicitly, and on why a `..Default::default()`
                    // here would be a silent decline rather than a shorthand.
                    color: None,
                    font_size: None,
                    stamp_fit: None,
                },
            }));
        }
    });
    // The two sentences under the chooser, and they are about different
    // things.
    //
    // The first is always true and says the icon changes the FILE and not
    // pdfcer's own picture.
    //
    // The second fires only for a name outside the seven, and it must NOT
    // promise that the name will be replaced: the reader is lossless and a
    // colour-only restyle carries an unmodelled `/Name` through untouched, so
    // such a sentence would describe a destruction that does not happen. What
    // it says instead is the part that was never about loss — pdfcer draws its
    // own glyph whatever the name says.
    //
    // Neither is a hover: an operator who has to hover to find out what a
    // control does has already been given the chance not to.
    ui.label(egui::RichText::new(tt::sticky_icon_bound()).small().weak());
    if current.foreign_icon {
        ui.label(
            egui::RichText::new(ts::markup_icon_foreign_note())
                .small()
                .weak(),
        );
    }
}
