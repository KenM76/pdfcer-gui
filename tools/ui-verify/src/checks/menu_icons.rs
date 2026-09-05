//! `menu_rows_draw_their_icons` — the regression check for the menu icon
//! painter that existed, was tested, and was never handed to a context menu.
//!
//! # The defect
//!
//! `egui_shell::menu::ContextMenu::with_icon_painter` had existed since the
//! menu engine landed. Nothing called it. `shell::menus::MenuHost::attach_with`
//! built its `ContextMenu` with `reporting_rects_to` and no painter, so **every
//! context-menu row in every build of this application drew a label and nothing
//! else** — including 25 rows whose commands already named a resolved,
//! catalogued, rasterizable icon key.
//!
//! This is [`crate::checks::qat_icons`]'s defect on a second surface, and the
//! two are the same shape to the letter: an icon set that draws, a shell seam
//! that accepts it, and a call site where nobody wrote the argument. That
//! header's central sentence applies here word for word — *"a green suite is
//! evidence about the code that was written and not about the code that was
//! not"* — and it is why this file exists rather than another unit test.
//!
//! It went further here than it did on the QAT, and the extra distance is the
//! reason this check is worth its launch. The missing wire was **written into
//! the project's record as a design decision**: an icon-coverage audit refused
//! a glyph for `view.panel_close` on the ground that *"the icon column exists
//! on the ribbon, not in a context menu"*, and that sentence was then quoted
//! until it read as the operator's own ruling. A missing line and a considered
//! refusal are indistinguishable from inside the source; they are
//! distinguishable from a running window.
//!
//! # ★★★ Why this check needs a name the QAT's check did not
//!
//! `qat_controls_are_icon_only` asserts a **shape**: an icon-only button is
//! roughly square, a text button is a word wide, and the reserved rectangle
//! separates them without a screenshot.
//!
//! **There is no menu equivalent, and this is the whole difficulty.** A menu
//! row is justified to the body width — that is what makes the chord column a
//! column — so every row measures exactly the same whether its icon slot holds
//! a glyph, holds a blank, or does not exist at all. `menu.item.<context>.<id>`
//! proves a row was drawn and says nothing about what is in it. A harness
//! reading only those rectangles could not have caught this defect, and did
//! not: `right_clicking_a_form_field_opens_its_menu` had been asserting on
//! menu row rects since 2026-08-28, through the whole period every one of those
//! rows was bare.
//!
//! ⇒ So `egui_shell::menu::report::icon` was added with the fix, and it is
//! published **only from the branch that calls the application's painter**:
//!
//! ```text
//! ui-rect name=menu.item.dock.tab.view.panel_float rect=…   <- the row was drawn
//! ui-rect name=menu.icon.dock.tab.view.panel_float rect=…   <- ...and a painter was handed its slot
//! ```
//!
//! The second line is absent when the slot is blank, when there is no slot, and
//! when no painter was supplied — which are exactly the three states this check
//! exists to tell apart from a working one. That module's header carries the
//! argument in full.
//!
//! It is deliberately **not** proof that pixels changed. Whether a key resolves
//! to art is the icon set's business and `crate::icons`' own tests assert it
//! offline; the fact that was missing was never *"does a glyph render"* but
//! *"did anything ask for one"*.
//!
//! # What this drives, and why it asserts on whichever menu opens
//!
//! One right-click on the page, and then an assertion about **the menu that
//! actually resolved**, named by the `canvas-menu context=…` line rather than
//! chosen in advance.
//!
//! That indirection is not vagueness, it is what makes the check robust against
//! the document it is pointed at. A right-click on a sheet may land on an
//! object (`canvas.object`), inside a form field (`canvas.field`), in a text
//! draft (`canvas.text`) or on blank paper (`canvas.empty`), and which one
//! depends entirely on the fixture. **Every one of pdfcer's nine context menus
//! reserves an icon column** — `shell::menus_wiring::tests` asserts that from
//! the shipped documents and the shipped registry, 25 glyph rows against 2
//! blanks — so whichever menu opens must publish at least one painted slot.
//! Pinning a context here would make the check fail on a fixture change for a
//! reason that has nothing to do with icons.
//!
//! ★ The blank rows are the reason this is "at least one" rather than "one per
//! row". `view.zoom_actual` and `view.panel_close` are argued refusals, not
//! gaps, and a check that demanded a glyph per row would be demanding art the
//! project has decided against — the wrong-picture failure, arriving through a
//! harness.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// `canvas-menu context=… sel=… level=…` — which menu a right-click resolved.
const MENU_EVENT: &str = "canvas-menu";
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page";

/// Where the right-click lands, as page fractions.
///
/// Well inside the sheet, so the popup is nowhere near an edge — `egui` flips a
/// popup to keep it on screen and a flipped menu is harder to reason about in a
/// failure message, though this check reads names rather than positions and
/// would survive it.
const CLICK_AT: (f64, f64) = (0.45, 0.45);

/// See the module documentation.
pub struct MenuRowsDrawTheirIcons;

impl Check for MenuRowsDrawTheirIcons {
    fn name(&self) -> &'static str {
        "menu_rows_draw_their_icons"
    }

    fn defect(&self) -> &'static str {
        "context-menu rows fall back to bare labels because no icon painter was supplied"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check's whole subject is a popup that exists \
             only while a secondary click is held open, and there is no offline mode: a menu is \
             an `egui::Area` laid out at paint time, so nothing about it can be recovered from a \
             captured PNG without re-implementing the layout.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to right-click on."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. This check aims its click in page fractions \
                 and needs the box to turn them into points. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("menu-icons.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is nothing to right-click. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- A: right-click the page -------------------------------------------
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let at = DocPoint::new(0, CLICK_AT.0 * page.width_pt, CLICK_AT.1 * page.height_pt);
    let screen = session.frame()?.to_screen(mapping.doc_to_window(at)?);
    driver.right_click_at(screen)?;
    session.settle(35);

    let trace = session.trace()?;
    let Some(menu) = trace.events(MENU_EVENT).last() else {
        return Ok(Some(format!(
            "THE RIGHT-CLICK RESOLVED NO MENU AT ALL: no `{MENU_EVENT}` line after a secondary \
             click on the page. `canvas::menus::attach` writes that line on every frame carrying \
             a secondary click, so its absence means the click never reached the canvas response \
             — suspect the harness before the menu. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let context = menu.get("context").unwrap_or_default().to_owned();
    report.note(format!("the right-click resolved `{}`", menu.raw));

    // --- B: the menu drew rows ---------------------------------------------
    //
    // ★ Asserted before the icons, and separately, because the two failures
    // have nothing to do with each other: no rows means the menu declined to
    // open (the empty-menu rule, `plan::offers_anything`), which is a
    // conditions problem; rows with no icon slots is the painter problem this
    // check is named for. Folding them together would report the second cause
    // for the first symptom. `crate::checks` rule 4.
    let rows = declared_names(&trace, ui_rect, &format!("menu.item.{context}."));
    if rows.is_empty() {
        return Ok(Some(format!(
            "`{context}` RESOLVED AND DREW NO ROWS: `{}`, and no `menu.item.{context}.*` region \
             was ever declared, so every item was disabled or withheld and `Menu::attach` \
             declined to open a popup. That is the empty-menu rule working, not the painter — \
             this check has learned nothing about icons. Regions beginning `menu.`: {}. \
             Trace: {}.",
            menu.raw,
            list(&declared_names(&trace, ui_rect, "menu.")),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "`{context}` opened with {} row(s): {}",
        rows.len(),
        list(&rows)
    ));

    // --- C: ...and at least one of them was handed to a painter ------------
    //
    // ★★★ THE ASSERTION. `menu.icon.<context>.<id>` is published only from
    // inside the branch that calls the application's icon painter — never for a
    // blank slot, never when no painter was supplied — so its presence is the
    // one fact a driven check can establish about this surface and its absence
    // is the defect.
    //
    // ★ Admissible as evidence because phase B proved rows were laid out on
    // this very frame: the absence below can only mean "rows drew and none was
    // given a glyph", never "no menu opened".
    let slots = declared_names(&trace, ui_rect, &format!("menu.icon.{context}."));
    if slots.is_empty() {
        return Ok(Some(format!(
            "★★★ EVERY ROW OF `{context}` DREW A BARE LABEL: {} row region(s) were declared and \
             not one `menu.icon.{context}.*` region was.\n\
             That name is published only from the branch that hands a rectangle to the \
             application's icon painter, so this says one of three things, in descending order \
             of likelihood:\n\
             1. `shell::menus_wiring::attach` no longer calls \
             `ContextMenu::with_icon_painter(...)` — the exact state this application shipped in \
             for the whole life of the menu engine, and what this check exists for;\n\
             2. no command in this menu names an icon key, so the menu reserves no column at all \
             (`egui_shell::menu::plan::reserves_icon_column`) — check the registrations before \
             the wiring;\n\
             3. the rows were laid out on a sizing pass whose slots had no rect.\n\
             Rows seen: {}. Trace: {}.",
            rows.len(),
            list(&rows),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ {} of {} row(s) were handed a glyph: {}",
        slots.len(),
        rows.len(),
        list(&slots)
    ));

    // --- D: every painted slot is inside the row it belongs to --------------
    //
    // Cheap, and it is the difference between "a glyph was requested" and "a
    // glyph was requested somewhere the operator will see it". A slot outside
    // its row would be drawn over the menu's neighbour or off the popup
    // entirely, and nothing else in the suite would notice.
    let mut stray = Vec::new();
    for name in &slots {
        let Some(slot) = declared(&trace, ui_rect, name) else {
            continue;
        };
        let row_name = name.replacen("menu.icon.", "menu.item.", 1);
        let Some(row) = declared(&trace, ui_rect, &row_name) else {
            stray.push(format!(
                "{name} was painted but `{row_name}` was never declared"
            ));
            continue;
        };
        if !row.contains_rect(slot) {
            stray.push(format!("{name} at {slot:?} is outside its row at {row:?}"));
        }
    }
    if !stray.is_empty() {
        return Ok(Some(format!(
            "{} painted icon slot(s) do not lie inside the row they belong to, so a glyph was \
             requested somewhere the operator will not read it: {}. Trace: {}.",
            stray.len(),
            stray.join("; "),
            session.trace_path().display()
        )));
    }

    // Escape, so the popup is not left over the page for whatever runs next in
    // the sweep. `right_click_at` deliberately does not do this itself.
    driver.press(crate::sys::vk::ESCAPE)?;
    session.settle(10);
    Ok(None)
}
