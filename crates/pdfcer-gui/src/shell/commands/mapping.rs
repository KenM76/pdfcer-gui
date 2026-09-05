//! # `shell::commands::mapping` — the single binding between a command id and
//! the value it names
//!
//! Four pairs of functions, each pair one *forward* mapping from a canvas or
//! view value to the `&'static str` id that names it, and one *inverse* derived
//! from the forward one rather than written out a second time:
//!
//! | value | forward | inverse | who reads which |
//! |---|---|---|---|
//! | [`crate::viewer::PageDisplay`] | [`page_display_command`] | [`page_display_for_command`] | `conditions` publishes the pressed radio position; `dispatch` turns a click into a mode |
//! | [`crate::app::actions::ViewChrome`] | [`chrome_command`] | [`chrome_for_command`] | the same pair, for three independent toggles rather than a radio |
//! | [`crate::canvas::markup::MarkupKind`] | [`markup_command`] | [`markup_for_command`] | `dispatch` arms the pen; `conditions` lights exactly one shape button |
//! | [`crate::canvas::measure::MeasureKind`] | [`measure_command`] | [`measure_for_command`] | the same pair, for the dimension tools |
//!
//! ## ★ Why this is a file of its own
//!
//! **R2** (no `.rs` file over 1,500 lines) forced a split when
//! `measure.finish` was registered and [`super`] reached 1,521 lines. The seam
//! is the one the file had already drawn for itself with a blank line and a
//! `★` banner: everything before it answers *"what commands exist, with what
//! label, icon and predicate"*, and everything here answers *"which id names
//! this value, and which value does this id name"*.
//!
//! They change for genuinely different reasons. A registration changes when the
//! ribbon gains a control or a tooltip is rewritten; a mapping changes only
//! when an **enum** the canvas owns gains a variant — and when that happens the
//! test that fails is in this file, iterating that enum's `ALL`, which is
//! exactly where a reader looking for "why does my new kind not arm anything"
//! should land.
//!
//! ## ★ The forward direction is a `match`; the inverse is a search over it
//!
//! Every inverse here is `ALL.iter().find(|k| forward(k) == id)`. That is not
//! an optimisation question — the lists are four to seven entries and the call
//! happens on an operator click, not per frame — it is a **correctness**
//! property: two hand-written tables can disagree, and one table plus a derived
//! search cannot. The failure the derivation removes is the worst kind
//! available here, because it is silent: a button that arms one tool while the
//! ribbon lights another.
//!
//! ## ★ The guard arms in `app::dispatch` are tried in order, so these must not
//! overlap
//!
//! `PdfcerApp::dispatch_command` has several arms of the shape
//! `id if …_for_command(id).is_some()`, and `match` takes the first that
//! matches. If a `measure.*` id ever answered to [`markup_for_command`] the
//! measure arm would still win by being written first and the defect would be
//! invisible; if a `markup.*` id answered to [`measure_for_command`] the markup
//! arm would be **swallowed** and four ribbon buttons would silently stop
//! arming anything. [`tests::every_measure_kind_has_a_registered_command`]
//! asserts the disjointness in both directions.

/// ★ **The command id that names a page-display mode**, and its inverse.
///
/// One binding between [`crate::viewer::PageDisplay`] and the ribbon, written
/// down once. The two directions are used by different surfaces and would drift
/// apart if each spelled the mapping for itself:
///
/// * `crate::app::dispatch` turns an invoked command into a mode;
/// * `PdfcerApp::conditions` turns the active mode into the `selected:`
///   condition that makes its ribbon button render pressed.
///
/// It lives here rather than on the enum for the reason the enum's own
/// `id`/`from_id` pair lives *there*: `viewer` must not know what a ribbon is.
/// `viewer::PageDisplay::id` is the **on-disk** spelling and this is the
/// **command** spelling, and keeping them separate is what lets either change
/// without silently rewriting the other's files.
///
/// [`tests::every_page_display_mode_has_a_registered_command`] asserts both
/// directions against the live registry, so a fifth mode that is added and not
/// registered fails the suite rather than becoming a mode with no control.
#[must_use]
pub fn page_display_command(display: crate::viewer::PageDisplay) -> &'static str {
    use crate::viewer::PageDisplay as D;
    match display {
        // ui-text-exempt: command ids, never displayed
        D::Single => "view.page_single",
        // ui-text-exempt: command ids, never displayed
        D::Continuous => "view.page_continuous",
        // ui-text-exempt: command ids, never displayed
        D::Facing => "view.page_facing",
        // ui-text-exempt: command ids, never displayed
        D::FacingContinuous => "view.page_facing_continuous",
    }
}

/// The page-display mode `id` names, or `None` if it names none.
///
/// The inverse of [`page_display_command`], derived from it rather than
/// written out a second time — so the two cannot disagree even in principle.
#[must_use]
pub fn page_display_for_command(id: &str) -> Option<crate::viewer::PageDisplay> {
    crate::viewer::PageDisplay::ALL
        .iter()
        .copied()
        .find(|&m| page_display_command(m) == id)
}

/// ★ **The command id that names a piece of View ▸ Display chrome**, and its
/// inverse.
///
/// Exactly the shape [`page_display_command`] has, and here for exactly the
/// same reasons: two surfaces need the mapping in opposite directions —
/// `crate::app::dispatch` turns an invoked command into a
/// [`crate::app::actions::ViewChrome`], and `PdfcerApp::conditions` turns each
/// toggle's state into the `selected:` condition that renders its button
/// pressed — and a mapping spelled twice is a mapping that drifts.
///
/// The difference from the page-display pair is that these three are
/// **independent toggles rather than a radio**: all, none or any two may be
/// on at once, so `conditions` publishes between zero and three of these
/// conditions where it publishes exactly one page-display condition. That is
/// the whole of what makes them read as three switches instead of one
/// three-position control.
#[must_use]
pub fn chrome_command(chrome: crate::app::actions::ViewChrome) -> &'static str {
    use crate::app::actions::ViewChrome as C;
    match chrome {
        // ui-text-exempt: command ids, never displayed
        C::Rulers => "view.rulers",
        // ui-text-exempt: command ids, never displayed
        C::Grid => "view.grid",
        // ui-text-exempt: command ids, never displayed
        C::Guides => "view.guides",
        // ui-text-exempt: command ids, never displayed
        C::ShowPoints => "view.show_points",
        // ui-text-exempt: command ids, never displayed
        C::LineWeights => "view.line_weights",
    }
}

/// The chrome toggle `id` names, or `None` if it names none.
///
/// Derived from [`chrome_command`] rather than written out a second time, so
/// the two cannot disagree even in principle.
#[must_use]
pub fn chrome_for_command(id: &str) -> Option<crate::app::actions::ViewChrome> {
    crate::app::actions::ViewChrome::ALL
        .iter()
        .copied()
        .find(|&c| chrome_command(c) == id)
}

/// The command id that arms `kind`.
///
/// The **single** binding between a `markup.*` id and a
/// [`crate::canvas::markup::MarkupKind`], in the shape [`chrome_command`]
/// established and for the same reason: a match here plus a derived inverse
/// cannot disagree, where two hand-written tables can.
///
/// ## Why these seven and not the ten `RIBBON_IA.md` §5.5 names
///
/// [`crate::canvas::markup::MarkupKind`] enumerates the kinds a `markup.*`
/// command **arms the canvas tool with** — nothing else. What is still outside
/// it, and each for its own reason:
///
/// * **Underline, strikeout and squiggly** mark a *text selection* and arm no
///   tool at all, so they have their own enum and their own pair of functions
///   ([`text_mark_command`]) — see `canvas::markup::text` §3;
/// * **Cloud** is blocked on `MarkupSpec::Cloud`, which pdfcer accepted on
///   2026-08-14 and has not started;
/// * **a plain line** is the existing Arrow with a different `/LE`, which is a
///   Style question rather than a kind.
///
/// ★ **This said "these four" until 2026-08-14**, and the sentence it rested on
/// was *"polygon, polyline and ink are not drag-shaped"*. That was true and it
/// stopped being a reason the day those gestures were built: two of the three
/// are now clicked (`canvas::markup::vertex`) and one is dragged freehand
/// (`canvas::markup::ink`). The wording is kept in this note rather than
/// deleted, because the boundary it was proxying for — *a variant nothing can
/// arm is a dead state* — is the real rule and is unchanged. See
/// `canvas::markup`'s header, where the boundary is restated as the property the
/// tests below actually assert.
///
/// Declaring the remaining kinds here early would put dead arms in a type whose
/// job is to say what the tool is doing — the same argument the old shell made
/// about its own tool enum, applied at the gesture boundary. They arrive with
/// the gestures that can draw them.
#[must_use]
pub fn markup_command(kind: crate::canvas::markup::MarkupKind) -> &'static str {
    use crate::canvas::markup::MarkupKind as K;
    match kind {
        // ui-text-exempt: command ids, never displayed
        K::Rectangle => "markup.rectangle",
        // ui-text-exempt: command ids, never displayed
        K::Ellipse => "markup.ellipse",
        // ui-text-exempt: command ids, never displayed
        K::Arrow => "markup.arrow",
        // ui-text-exempt: command ids, never displayed
        K::PolyLine => "markup.polyline",
        // ui-text-exempt: command ids, never displayed
        K::Polygon => "markup.polygon",
        // ★ `markup.cloud`, registered 2026-08-19. It was in
        // `crate::shell::manifest::PLANNED` with the reason *"the ONLY markup
        // kind still absent for an ENGINE reason rather than a gesture one"* —
        // and that had stopped being true: `MarkupSpec::Cloud` shipped in
        // `pdfcer-core` and nothing in this shell had noticed.
        // ui-text-exempt: command ids, never displayed
        K::Cloud => "markup.cloud",
        // ★ The id is `markup.ink` and the LABEL is "Freehand". The
        // specification's word and the operator's word differ here, and the two
        // vocabularies are kept apart deliberately — ids are the shell's, labels
        // are `text::commands`'. See `canvas::markup`'s header.
        // ui-text-exempt: command ids, never displayed
        K::Ink => "markup.ink",
        // ui-text-exempt: command ids, never displayed
        K::Highlight => "markup.highlight",
    }
}

/// The markup kind `id` arms, or `None` if it names none.
///
/// Derived from [`markup_command`] rather than written out a second time, so
/// the two cannot disagree even in principle.
#[must_use]
pub fn markup_for_command(id: &str) -> Option<crate::canvas::markup::MarkupKind> {
    crate::canvas::markup::MarkupKind::ALL
        .iter()
        .copied()
        .find(|&k| markup_command(k) == id)
}

/// ★ The command id that **marks the selection** with `kind`, and its inverse.
///
/// [`markup_command`]'s sibling and deliberately a separate pair, because the
/// two families do different things to a press: a `markup.*` shape id *arms a
/// tool*, and one of these *authors an annotation immediately* from the text
/// selection already on the document. See [`crate::canvas::markup::text`]'s
/// header §1 for the interaction decision that makes them different, and §3 for
/// why the three kinds are not [`crate::canvas::markup::MarkupKind`] variants.
///
/// ## ★ The disjointness is load-bearing, not tidy
///
/// All six ids begin `markup.`, and `app::dispatch` matches both families with
/// guard arms of the shape `id if …_for_command(id).is_some()`, tried in order.
/// If a shape id ever answered here, pressing Rectangle would author an
/// annotation over whatever text was selected and never arm the pen; if one of
/// these answered to [`markup_for_command`], pressing Underline would arm a
/// **shape tool** — `arm_markup` would take a `MarkupKind` from an id that does
/// not name one, which it cannot, so the arm would swallow the command and
/// Underline would do nothing at all. Both directions are asserted by
/// [`tests::every_text_mark_kind_has_a_registered_command`].
///
/// ## Why Highlight is not here
///
/// `markup.highlight` authors the same `MarkupSpec::TextMarkup` these three do,
/// with `TextMarkupKind::Highlight` — and it is a **drag** across an area, not a
/// mark on a selection, so it stays a `MarkupKind`. That is a genuine seam
/// rather than an inconsistency: a highlight over an image or a title-block cell
/// is a thing operators want and a text markup cannot express. See
/// `canvas::markup::text` §3, which also records what it would take to offer
/// *both* and why that is the operator's taxonomy decision rather than this
/// file's.
#[must_use]
pub fn text_mark_command(kind: crate::canvas::markup::text::TextMarkKind) -> &'static str {
    use crate::canvas::markup::text::TextMarkKind as K;
    match kind {
        // ★★ Highlight has no text-markup COMMAND of its own, and that is not
        // an omission. It is the one kind reachable by two gestures — an armed
        // tool that follows text where there is text and draws an area box
        // where there is not (`OPERATOR_REQUESTS.md` O54) — so its control is
        // `markup.highlight`, which arms that tool, and there is no separate
        // "highlight the selection" verb to name here.
        //
        // ⇒ Returning the tool's own id would be wrong in a way that compiles:
        // this function answers *"which command authors this from a selection"*,
        // and for Highlight the answer is that none does.
        // ui-text-exempt: command ids, never displayed
        K::Highlight => "markup.highlight",
        // ui-text-exempt: command ids, never displayed
        K::Underline => "markup.underline",
        // ui-text-exempt: command ids, never displayed
        K::StrikeOut => "markup.strikeout",
        // ui-text-exempt: command ids, never displayed
        K::Squiggly => "markup.squiggly",
    }
}

/// The text-markup kind `id` authors, or `None` if it names none.
///
/// Derived from [`text_mark_command`] rather than written out a second time, so
/// the two cannot disagree even in principle.
#[must_use]
pub fn text_mark_for_command(id: &str) -> Option<crate::canvas::markup::text::TextMarkKind> {
    crate::canvas::markup::text::TextMarkKind::ALL
        .iter()
        .copied()
        .find(|&k| text_mark_command(k) == id)
}

/// The command id that arms `kind`.
///
/// The **single** binding between a `measure.*` id and a
/// [`crate::canvas::measure::MeasureKind`] — [`markup_command`]'s twin, in the
/// same shape and for the same reason: a match here plus a derived inverse
/// cannot disagree, where two hand-written tables can.
///
/// ## Which are absent, and why each
///
/// `measure.aligned` is a *constraint* on a linear pick rather than a tool
/// (the old shell's `LinearPick` carries an `AxisConstraint`), so it belongs on
/// a property control, not here. `measure.angular`, `measure.area`,
/// `measure.distance`, `measure.perimeter` and `measure.count` need engine
/// verbs that do not exist — `RIBBON_IA.md` §5.6 marks them **N** — and
/// `measure.calibrate` is a second entry path into the scale dialog rather than
/// a fifth tool. All remain in [`super::manifest::PLANNED`], which is where an
/// absent command is supposed to be.
///
/// `measure.set_scale` is registered and is deliberately **not** a kind: it
/// changes what measurements are read against rather than placing one. (This
/// sentence used to end *"and its dialog does not exist yet"*. It does —
/// `crate::dialogs::scale`, since 2026-08-17 — and the reason for not being a
/// kind never depended on that clause.)
///
/// `measure.finish` is registered and is not a kind either, for a sharper
/// reason: it does not *arm* anything. It **ends** the radius/diameter
/// gesture, and it is dispatched through its own arm to
/// [`crate::canvas::measure::finish`] rather than through `arm_measure`. If it
/// answered here, pressing Finish would toggle the tool off — see
/// `crate::canvas::tool::arm_measure`'s same-kind-retires rule — which is the
/// opposite of what it is for.
///
/// ## ★ What this doc comment used to say
///
/// Until 2026-08-14 it carried a paragraph explaining that `Circular` was
/// absent because *"the gesture has no natural end, and the only place to say
/// so was an accept box decision 024 retired"*. That was accurate while it
/// stood and it is now false: the operator decided the tool should have **two**
/// endings, neither of them a floating box, and both are built. The paragraph
/// is not merely deleted — a reader who finds `measure.radius_diameter`
/// reaching a real tool and remembers the old note should be able to see what
/// replaced it. What shipped is a **double-click** on the canvas and this
/// command, both routed through one commit path
/// (`canvas::measure::circular::commit`), with the ribbon control
/// enabled only while there is a non-degenerate fit to commit.
#[must_use]
pub fn measure_command(kind: crate::canvas::measure::MeasureKind) -> &'static str {
    use crate::canvas::measure::MeasureKind as K;
    match kind {
        // ui-text-exempt: command ids, never displayed
        K::Linear => "measure.linear",
        // ui-text-exempt: command ids, never displayed
        K::Circular => "measure.radius_diameter",
        // ui-text-exempt: command ids, never displayed
        K::Perimeter => "measure.perimeter",
        // ui-text-exempt: command ids, never displayed
        K::PathLength => "measure.length",
        // ui-text-exempt: command ids, never displayed
        K::TwoLine => "measure.two_line",
        // ★ Armed from the Set-scale DIALOG, not from the ribbon, so it maps
        // to no command id at all.
        //
        // The empty string is deliberate and is safe by construction:
        // `measure_for_command` finds the kind whose command equals the id it
        // was given, and no registered command has an empty id — `egui-shell`'s
        // registry rejects one. So this cannot be armed by any ribbon press,
        // which is exactly the property wanted. See `MeasureKind::ALL` for why
        // this kind lives off the ribbon.
        K::Scale => "",
    }
}

/// The measure kind `id` arms, or `None` if it names none.
///
/// Derived from [`measure_command`] rather than written out a second time, so
/// the two cannot disagree even in principle.
#[must_use]
pub fn measure_for_command(id: &str) -> Option<crate::canvas::measure::MeasureKind> {
    crate::canvas::measure::MeasureKind::ALL
        .iter()
        .copied()
        .find(|&k| measure_command(k) == id)
}

/// The form-field kind a command id arms, if it is one of the five.
///
/// ★ A NAMED function taking `id`, deliberately, rather than an inline closure
/// at the dispatch site. `shell::commands::reach` *reads* `app/dispatch.rs` to
/// prove every registered command is routed, and it can follow a call like
/// `form_for_command(id)` while a closure is opaque to it — the first version
/// of this arm used `.iter().any(|k| ...)` and the reader rejected the whole
/// dispatcher as unreadable rather than guessing.
///
/// So the convention `markup_for_command` established is not style: it is what
/// keeps the routing table machine-checkable.
#[must_use]
pub fn form_for_command(id: &str) -> Option<crate::canvas::formfield::FormFieldKind> {
    crate::canvas::formfield::FormFieldKind::ALL
        .iter()
        .copied()
        .find(|&k| k.command_id() == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::CommandRegistry;

    /// The live registry, built the way `PdfcerApp` builds it.
    ///
    /// Every test below asserts against **this** rather than against the
    /// mapping's own table, which is the difference between asserting that the
    /// code agrees with itself and asserting that the control exists.
    fn registry() -> CommandRegistry {
        let mut reg = CommandRegistry::new();
        super::super::register(&mut reg);
        reg
    }
    /// ★ **Every chrome toggle has a registered command, and every one of
    /// those commands names a toggle.**
    ///
    /// The twin of [`every_page_display_mode_has_a_registered_command`], and
    /// it catches the same failure: a fourth toggle added to
    /// [`crate::app::actions::ViewChrome`] with no registration would be a
    /// piece of chrome no operator could reach, and nothing else in the suite
    /// would notice. Asserted against the **live registry** rather than
    /// against the mapping's own table, which is the difference between the
    /// code agreeing with itself and the control existing.
    #[test]
    fn every_chrome_toggle_has_a_registered_command() {
        let reg = registry();
        for &chrome in crate::app::actions::ViewChrome::ALL {
            let id = chrome_command(chrome);
            assert!(
                reg.get(id).is_some(),
                "`{id}` names {chrome:?} and is not registered"
            );
            assert_eq!(chrome_for_command(id), Some(chrome), "round trip");
        }
        let mut ids: Vec<&str> = crate::app::actions::ViewChrome::ALL
            .iter()
            .map(|&c| chrome_command(c))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), crate::app::actions::ViewChrome::ALL.len());
        // …and the two mappings do not overlap, which is what keeps a
        // page-display click from toggling a ruler.
        assert_eq!(chrome_for_command("view.page_single"), None);
        assert_eq!(page_display_for_command("view.rulers"), None);
    }

    /// ★ **Every markup kind the canvas can draw has a registered command,
    /// and no other mapping claims a `markup.*` id.**
    ///
    /// The third of this family, and the one with the most room to go wrong,
    /// because the kinds and the commands were built by different hands: the
    /// canvas enumerates what the *gesture* can draw, the manifest enumerates
    /// what `RIBBON_IA.md` §5.5 *names*, and those two sets are deliberately
    /// different sizes today — ten names, four kinds. This asserts the four
    /// are a genuine subset and reach real controls, not that the sets match.
    ///
    /// The failure it exists to catch is a fifth kind added to `MarkupKind`
    /// with no registration: a tool an operator could not arm, which is
    /// precisely the class of half-built surface `panels`' header is about.
    /// Asserted against the **live registry** rather than the mapping's own
    /// table — the difference between the code agreeing with itself and the
    /// control existing.
    #[test]
    fn every_markup_kind_has_a_registered_command() {
        let reg = registry();
        for &kind in crate::canvas::markup::MarkupKind::ALL {
            let id = markup_command(kind);
            assert!(
                reg.get(id).is_some(),
                "`{id}` names {kind:?} and is not registered"
            );
            assert_eq!(markup_for_command(id), Some(kind), "round trip");
        }
        let mut ids: Vec<&str> = crate::canvas::markup::MarkupKind::ALL
            .iter()
            .map(|&k| markup_command(k))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(
            ids.len(),
            crate::canvas::markup::MarkupKind::ALL.len(),
            "two kinds sharing one id would arm the wrong tool from one button"
        );
        // …and no other mapping answers to a markup id, nor this one to
        // theirs. The dispatch arm for markup is a GUARD arm — `id if
        // markup_for_command(id).is_some()` — so an overlap here would not
        // merely confuse a lookup, it would swallow another command's arm
        // entirely, and the arm it swallowed would simply stop happening.
        assert_eq!(markup_for_command("view.rulers"), None);
        assert_eq!(markup_for_command("view.tool_hand"), None);
        assert_eq!(markup_for_command("markup.comments"), None);
        // ★ `markup.finish` in particular, which is `measure.finish`'s twin and
        // carries the identical hazard its own assertion below records: this id
        // names no kind, and if it ever answered here, pressing **Finish** would
        // reach `arm_markup` — whose same-kind-retires rule would put the pen
        // down instead of committing the run. The dispatch arm is written ahead
        // of the guard arm for the same reason; this is the cheaper half of that
        // pair of guarantees.
        assert_eq!(markup_for_command("markup.finish"), None);
        assert_eq!(chrome_for_command("markup.rectangle"), None);
        assert_eq!(page_display_for_command("markup.rectangle"), None);
    }

    /// ★ **Every text-markup kind has a registered command, and no shape id
    /// answers to it — nor it to a shape id.**
    ///
    /// The third `markup.*` mapping and the one with the most room to go wrong,
    /// because both families share the id prefix and both are matched in
    /// `PdfcerApp::dispatch_command` by guard arms tried in order. The two
    /// crossings have different symptoms and neither is diagnosable from a
    /// screenshot:
    ///
    /// * a **shape** id answering here would author a text markup instead of
    ///   arming the pen — a Rectangle button that marks the selected words;
    /// * one of **these** answering to [`markup_for_command`] would be swallowed
    ///   by the arm below it, and Underline would do nothing at all.
    ///
    /// Asserted against the **live registry** rather than the mapping's own
    /// table, which is the difference between the code agreeing with itself and
    /// the control existing.
    #[test]
    fn every_text_mark_kind_has_a_registered_command() {
        use crate::canvas::markup::text::TextMarkKind;
        let reg = registry();
        for &kind in TextMarkKind::ALL {
            let id = text_mark_command(kind);
            assert!(
                reg.get(id).is_some(),
                "`{id}` names {kind:?} and is not registered"
            );
            assert_eq!(text_mark_for_command(id), Some(kind), "round trip");
            // ★ The enable predicate is part of the mapping's contract here, in
            // the way `finish_is_registered_and_is_not_a_tool` makes it part of
            // Finish's: a text-markup command with no operand does nothing, and
            // P3 reserves greying for exactly that.
            assert!(
                matches!(
                    &reg.get(id).expect("registered").enable,
                    egui_shell::commands::Enable::When(name) if name == "selection.text"
                ),
                "`{id}` acts on a text selection and must be greyed without one"
            );
        }
        let mut ids: Vec<&str> = TextMarkKind::ALL
            .iter()
            .map(|&k| text_mark_command(k))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(
            ids.len(),
            TextMarkKind::ALL.len(),
            "two kinds sharing one id would author the wrong subtype from one button"
        );
        // ★ The two `markup.*` families do not overlap, in either direction.
        for &kind in TextMarkKind::ALL {
            assert_eq!(markup_for_command(text_mark_command(kind)), None);
            assert_eq!(measure_for_command(text_mark_command(kind)), None);
        }
        for &kind in crate::canvas::markup::MarkupKind::ALL {
            assert_eq!(
                text_mark_for_command(markup_command(kind)),
                None,
                "a shape id must not author a text markup"
            );
        }
        // …and `markup.highlight` in particular, which authors the same
        // `MarkupSpec::TextMarkup` and is deliberately a drag. See
        // `text_mark_command`'s docs.
        assert_eq!(text_mark_for_command("markup.highlight"), None);
        assert_eq!(text_mark_for_command("markup.comments"), None);
    }

    /// ★ **Every measure kind has a registered command, and every one of those
    /// commands names a kind.**
    ///
    /// [`tests::every_markup_kind_has_a_registered_command`]'s twin, catching
    /// the same failure: a fifth `MeasureKind` added with no registration is a
    /// tool an operator cannot arm. Asserted against the **live registry**,
    /// which is the difference between the code agreeing with itself and the
    /// control existing.
    ///
    /// The guard-arm overlap check at the end matters more here than it did
    /// for markup, because there are now **two** guard arms in
    /// `PdfcerApp::dispatch_command` matching on `*_for_command(id).is_some()`
    /// and they are tried in order. If a measure id ever answered to
    /// `markup_for_command`, the measure arm would still win by being first and
    /// the defect would be invisible; if a markup id answered to
    /// `measure_for_command`, the markup arm would be **swallowed** and four
    /// ribbon buttons would silently stop arming anything.
    #[test]
    fn every_measure_kind_has_a_registered_command() {
        use crate::canvas::measure::MeasureKind;
        let reg = registry();
        for &kind in MeasureKind::ALL {
            let id = measure_command(kind);
            assert!(
                reg.get(id).is_some(),
                "`{id}` names {kind:?} and is not registered"
            );
            assert_eq!(measure_for_command(id), Some(kind), "round trip");
        }
        let mut ids: Vec<&str> = MeasureKind::ALL
            .iter()
            .map(|&k| measure_command(k))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(
            ids.len(),
            MeasureKind::ALL.len(),
            "two kinds sharing one id would arm the wrong tool from one button"
        );
        // The two guard arms must not overlap, in either direction.
        for &kind in MeasureKind::ALL {
            assert_eq!(markup_for_command(measure_command(kind)), None);
        }
        for &kind in crate::canvas::markup::MarkupKind::ALL {
            assert_eq!(measure_for_command(markup_command(kind)), None);
        }
        // …and `measure.manage_groups` is deliberately NOT a tool: it opens a
        // dialog. If it ever answered here it would arm a picking state the
        // operator never asked for.
        assert_eq!(measure_for_command("measure.manage_groups"), None);
        assert_eq!(measure_for_command("view.rulers"), None);
        assert_eq!(chrome_for_command("measure.linear"), None);
        assert_eq!(page_display_for_command("measure.linear"), None);
    }

    /// ★ **`measure.finish` ends a gesture; it must never arm one.**
    ///
    /// If it ever answered to [`measure_for_command`] the guard arm in
    /// `PdfcerApp::dispatch_command` would claim it before its own arm, and
    /// `arm_measure`'s same-kind-retires rule would turn a press of Finish into
    /// *putting the tool down* — the exact opposite of what the control says it
    /// does, with the pick set left standing and nothing authored. The failure
    /// would look like a Finish button that does nothing, which is the hardest
    /// kind to diagnose from a screenshot.
    #[test]
    fn finish_is_registered_and_is_not_a_tool() {
        let reg = registry();
        let finish = reg.get("measure.finish").expect("Finish is registered");
        assert_eq!(measure_for_command("measure.finish"), None, "not a tool");
        assert_eq!(markup_for_command("measure.finish"), None);
        assert!(
            matches!(
                &finish.enable,
                egui_shell::commands::Enable::When(name) if name == "measure.finishable"
            ),
            "a Finish that is always enabled is a control that does nothing on \
             almost every press, and P3 reserves greying for exactly this"
        );
        // ★★★ **The refusal this used to assert was DISCHARGED on 2026-09-04,
        // and only half of it was ever about supply.**
        //
        // It read: *"no accept glyph exists, and the `measure` ruler would draw
        // a fourth identical one for a command that places nothing."* Two
        // claims, and they aged differently.
        //
        // The first — no accept glyph exists — was a fact about the asset
        // directory, and it stopped being one when `check.svg` was adopted from
        // the outside review's sheet. Nothing about it was a design position;
        // the registration said as much, and said the remedy was art.
        //
        // ★ The second is still true and is what this assertion now pins. The
        // worry was never "Finish should have no picture", it was **"Finish
        // must not draw the measure ruler"** — a fourth control wearing the
        // glyph of the three tools around it, for a command that places
        // nothing. So the shape of the check is inverted rather than deleted:
        // it names the glyph Finish must wear and re-states the one it must
        // not.
        //
        // ★★ `check` and not `finish-shape`, and the split is deliberate.
        // `markup.finish` carries the identical refusal in its own file and
        // took `finish-shape` — a polyline closed with a tick — because the
        // markup band's Finish completes a DRAWN SHAPE. A measurement's Finish
        // accepts a RESULT: the readout, not the figure. Two commands, two
        // glyphs, and the set's one-asset-per-role rule holds. Recorded here
        // because two near-identical commands taking different art is exactly
        // the thing a later "consistency pass" would undo without this
        // paragraph.
        assert_eq!(
            finish.icon.as_deref(),
            Some("check"),
            "Finish accepts a measurement; it should wear the accept glyph"
        );
        assert_ne!(
            finish.icon.as_deref(),
            Some("measure"),
            "the surviving half of the recorded refusal: Finish must not wear \
             the `measure` ruler, which would draw a fourth control identical \
             to the three tools beside it for a command that places nothing"
        );
    }

    /// ★ **Every page-display mode has a registered command, and every one of
    /// those commands names a mode.**
    ///
    /// Both directions, against the **live registry** rather than against the
    /// mapping's own table — which is the difference between asserting the
    /// code agrees with itself and asserting that the control exists. The
    /// failure this catches is a fifth mode added to the enum with no
    /// registration: the ribbon would draw three buttons, the fourth would be
    /// unreachable, and nothing else in the suite would notice.
    #[test]
    fn every_page_display_mode_has_a_registered_command() {
        let reg = registry();
        for &mode in crate::viewer::PageDisplay::ALL {
            let id = page_display_command(mode);
            assert!(
                reg.get(id).is_some(),
                "`{id}` names {mode:?} and is not registered"
            );
            assert_eq!(page_display_for_command(id), Some(mode), "round trip");
        }
        // …and the ids are distinct, which the round trip alone would not
        // prove if two modes shared one command.
        let mut ids: Vec<&str> = crate::viewer::PageDisplay::ALL
            .iter()
            .map(|&m| page_display_command(m))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), crate::viewer::PageDisplay::ALL.len());
        assert_eq!(page_display_for_command("view.zoom_actual"), None);
    }
}
