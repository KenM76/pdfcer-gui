//! # `shell::commands::mapping` — the single binding between a command id and
//! the value it names
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/commands/mapping.md`.

/// **The command id that names a page-display mode**, and its inverse.
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

/// **The command id that names a piece of View ▸ Display chrome**, and its
/// inverse.
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
        // ui-text-exempt: command ids, never displayed
        C::OffPage => "view.off_page",
        // ui-text-exempt: command ids, never displayed
        C::OcrLayer => "view.ocr_layer",
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
        // `markup.cloud`, registered 2026-08-19. It was in
        // `crate::shell::manifest::PLANNED` with the reason *"the ONLY markup
        // kind still absent for an ENGINE reason rather than a gesture one"* —
        // and that had stopped being true: `MarkupSpec::Cloud` shipped in
        // `pdfcer-core` and nothing in this shell had noticed.
        // ui-text-exempt: command ids, never displayed
        K::Cloud => "markup.cloud",
        // The id is `markup.ink` and the LABEL is "Freehand". The
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

/// The command id that **marks the selection** with `kind`, and its inverse.
#[must_use]
pub fn text_mark_command(kind: crate::canvas::markup::text::TextMarkKind) -> &'static str {
    use crate::canvas::markup::text::TextMarkKind as K;
    match kind {
        // Highlight has no text-markup COMMAND of its own, and that is not
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
        // Armed from the Set-scale DIALOG, not from the ribbon, so it maps
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
    fn registry() -> CommandRegistry {
        let mut reg = CommandRegistry::new();
        super::super::register(&mut reg);
        reg
    }
    /// **Every chrome toggle has a registered command, and every one of
    /// those commands names a toggle.**
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

    /// **Every markup kind the canvas can draw has a registered command,
    /// and no other mapping claims a `markup.*` id.**
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
        // `markup.finish` in particular, which is `measure.finish`'s twin and
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

    /// **Every text-markup kind has a registered command, and no shape id
    /// answers to it — nor it to a shape id.**
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
            // The enable predicate is part of the mapping's contract here, in
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
        // The two `markup.*` families do not overlap, in either direction.
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

    /// **Every measure kind has a registered command, and every one of those
    /// commands names a kind.**
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

    /// **`measure.finish` ends a gesture; it must never arm one.**
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
        // The second is still true and is what this assertion now pins. The
        // worry was never "Finish should have no picture", it was **"Finish
        // must not draw the measure ruler"** — a fourth control wearing the
        // glyph of the three tools around it, for a command that places
        // nothing. So the shape of the check is inverted rather than deleted:
        // it names the glyph Finish must wear and re-states the one it must
        // not.
        //
        // `check` and not `finish-shape`, and the split is deliberate.
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

    /// **Every page-display mode has a registered command, and every one of
    /// those commands names a mode.**
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
