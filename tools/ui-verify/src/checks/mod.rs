//! The named checks — the suite this harness exists to run.
//!
//! ## The three, and what each is for
//!
//! | Check | Defect | Oracle |
//! |---|---|---|
//! | [`delete_key`] | **D1** — Delete stops working after the first canvas click | the trace |
//! | [`ribbon_captions`] | group captions rendering illegibly, or not at all | the pixels |
//! | [`ribbon_mockup`] | the band drawn to different proportions from the mockup, and a resting control drawn in a box | the pixels |
//! | [`settings_headings`] | **D2** — section headings near-white on light grey | the pixels |
//!
//! These are the three `GUI_ROADMAP.md` names as "the smallest useful set",
//! and the three `PROJECT_PLAN.md` stage S1 makes the gate on the harness
//! itself. They are not a sample of what could be checked; they are the
//! specific defects that shipped past a green suite, turned into the tests
//! that would have caught them.
//!
//! ## What has been added since, and on what principle
//!
//! The suite is no longer three. [`all`] is the list; the additions are not a
//! drift away from the founding three but the same rule applied to each new
//! surface as it landed — [`qat_icons`] for the icon painter that was never
//! passed to the ribbon, [`find_bar`] for the chord that was in the keymap and
//! bound to nothing, [`markup_rectangle`] for a ribbon click whose whole
//! four-link chain had unit tests and had never been performed,
//! [`measure_linear`] for `SALVAGE.md`'s step 5 — *"assert it in `ui-verify`
//! before calling it done; a green unit test is the floor"* — and
//! [`read_mode`] for a mode gate whose one untested link is the one a refactor
//! breaks silently.
//!
//! ★ [`text_markup`] is the newest and adds a direction the suite did not have:
//! **it asserts that a control is correctly DISABLED** before asserting that it
//! works. Every check above it drives a control that should act; this one first
//! clicks a control that should not, and reads the *absence* of
//! `ribbon-command-invoked` as the evidence — which is admissible under rule 4
//! below precisely because the same control is then shown to invoke, in the same
//! run, once its operand exists.
//!
//! [`driving`] is not a check. It holds the moves the three ribbon-driving
//! checks share; its header carries the argument for why it exists and why
//! `markup_rectangle` deliberately keeps its own copies.
//!
//! The principle each satisfies, and the one to hold a proposed check to:
//! **it must fail against a build where the wiring is absent, and the wiring
//! must be something no unit test in the workspace can observe.** Every check
//! here has been run against such a build and seen to fail; that is what
//! `PROJECT_PLAN.md` §4 stage S1's acceptance criterion below asks for, and it
//! is not optional.
//!
//! ## The acceptance criterion for the harness
//!
//! > The three assertions **fail** against the old GUI (proving they detect
//! > the real defects) and **pass** against the new one.
//! > — `PROJECT_PLAN.md` §4, stage S1
//!
//! That is the reason [`crate::profile::PDFCER_LEGACY`] exists. A check suite
//! that has only ever been seen to pass is not evidence of anything: it is
//! indistinguishable from a suite that cannot fail. This is the same argument
//! that put a `--self-test` in `tools/gates/check-ui-strings.sh`, and it comes
//! from the same recorded incident — a deliberately planted violation that a
//! gate failed to detect, briefly making it look as though the fix had
//! produced a gate that could only pass.
//!
//! ## Writing a new check
//!
//! Four rules, each of which exists because breaking it produced a real
//! problem in this codebase:
//!
//! 1. **Say what you are about to do, in a note, before you do it.** The notes
//!    are what make a SKIP diagnosable and a PASS believable.
//! 2. **Only ever write [`crate::coords::DocPoint`] or
//!    [`crate::geom::FracRect`] literals.** Never a screen coordinate, never a
//!    window coordinate. See [`crate::coords`].
//! 3. **Establish the precondition explicitly, and SKIP on it.** "The click
//!    selected something" must be *asserted* before "Delete removed it" can be
//!    a failure rather than a mystery.
//! 4. **Never treat an absence as evidence unless you have shown the thing
//!    that would have produced it was working.** The distinction is drawn in
//!    [`crate::report`] and applied in [`delete_key`].
//! 5. **A SKIP reason names the component that is actually blocked, and gets
//!    re-audited whenever the application gains a capability.** This one was
//!    earned at S2. `ribbon_group_captions_legible` spent a stage reporting
//!    *"the trace declared no `ui-rect` regions"* about a binary that declares
//!    three of them on every frame — because nothing in this crate parsed the
//!    event, so the reason described the harness's own blindness as though it
//!    were the application's silence. A reader following it would have gone to
//!    `diag.rs`, which was finished, and found no defect to fix.
//!
//!    The rule that prevents a repeat is mechanical: **a reason may only
//!    assert what the check actually looked at.** [`legibility::resolve_set`]
//!    takes the trace evidence as an argument and builds its reason from it,
//!    with a distinct sentence for "nothing was consulted", "the application
//!    said nothing" and "the application said these things and none of them is
//!    what I need" — because those three send a reader to three different
//!    files.
//!
//! ## Where a check's evidence comes from, in preference order
//!
//! Both apply to every new check, and both say the same thing in two domains:
//! **prefer the evidence the application produced this run.**
//!
//! | Domain | Preferred | Fallback |
//! |---|---|---|
//! | *Where* to measure | a `ui-rect` the application declared this frame | a calibrated fraction in [`crate::profile`] |
//! | *Whether* state changed | a count of the thing itself (`objects n=`) | the event for the verb that should have changed it |
//!
//! The first pair is argued in [`legibility`]; the second in [`delete_key`].
//! In both cases the fallback is kept, because a dated screenshot cannot
//! declare its regions and the old binary cannot count its objects — and in
//! both cases the check says in its own output which one it used.

/// ★★★ **The ninth handle on an ANNOTATION** — draw a shape, grab the rotate
/// handle, and it turns.
///
/// The sibling of [`rotate`], which drives the same handle over page content.
/// It is a separate check rather than a parameter on that one because the two
/// exercise different verbs behind an identical gesture — `rotate_annotation`
/// here, `transform_objects` there — and the whole risk this check exists for
/// is a build that reaches the *other* one. It asserts which trace line
/// appeared **and** that the other's did not.
///
/// ★ It also asserts the affordance BEFORE pressing, through the
/// `canvas.rotate-handle` region, which [`rotate`] could not: a build with no
/// ninth handle and a build with a mis-routed one produce the same silence
/// otherwise.
pub mod annot_delete_gate;
pub mod annot_rotate;
pub mod blend_space;
/// A check box dragged larger must be REDRAWN, not stretched —
/// `OPERATOR_REQUESTS.md` O76. Reads `regenerated=` because the two outcomes
/// are the same pixels.
pub mod checkbox_resize;
/// Export to DXF: the file reaches disk, and its contents agree with the
/// counts the shell reported.
/// ★★ **Press Export form data and a file appears on disk with the form's
/// values in it.** The oracle is the FILE, not a trace line: a build that
/// computed the bytes and wrote them nowhere would satisfy a trace-only check
/// completely and ship an Export button that exports nothing.
/// ★★★ **Reading a comment where the comment is** — the canvas note
/// pop-up, driven in READ mode, which is the mode the operator reported he could
/// not read a sticky note in. ⬜ NOT RUN; see the module's own header.
pub mod comment_popup;
/// Not a check — the Comments panel's census, read the one way that is honest.
/// Shared by `save_copy` and `undo_redo`, which each carried a copy of it and
/// therefore each carried the same defect. See its header.
pub mod comments_census;
pub mod compact_save;
pub mod delete_key;
/// The Manage-dimension-groups window: it opens, it creates a group, and the
/// group comes back joinable.
pub mod dimension_groups;
/// ★ **The second half of the descent** — something inside a wrapped drawing
/// can be DRAGGED, not merely selected (`OPERATOR_REQUESTS.md` O70). Reads the
/// funnel's own applied line, which the engine's `Ok` is what produces.
/// ★ **The ladder goes as deep inside a container as outside one** — the
/// second double-click, the Part rung and a drag that commits
/// (`OPERATOR_REQUESTS.md` O70).
/// ★ **Double-clicking a text box edits the text** — the last rung of the
/// Smart-Selector chain, and the one where "deeper" means the words rather
/// than a smaller shape (`OPERATOR_REQUESTS.md` O70).
pub mod double_click_text;
pub mod driving;
/// ★ **A drawing dragged in from Explorer and dropped on the page thumbnails**
/// — `OPERATOR_REQUESTS.md` O67. The only check in this suite whose subject is
/// a coordinate the toolkit throws away: `winit` discards the OLE drop point,
/// so the application asks the operating system, and this drives the real
/// cursor onto a real tile to prove the answer is used.
pub mod drop_onto_thumbnails;
pub mod embed_bundled;
pub mod embed_fonts;
pub mod export_dxf;
pub mod export_form_data;
/// ★★★ **The CAD "line weights off" display mode** — `OPERATOR_REQUESTS.md`
/// O137, asked for by name. The pixel assertion is SIGNED (strictly LESS ink),
/// because the convention he asked for and the one it is confused with are
/// opposites. ⬜ NOT RUN; see the module's own header.
pub mod line_weights;
/// Putting a control where the pointer can hit it — scrolling a pane, raising a
/// dock tab, bringing a control inside its panel's body. Split from
/// [`driving`] on 2026-09-05 under R2; its header carries the seam.
pub mod reaching;
// O120's fourth export format, and the one a driven check is worth most for:
// EMF is the ONLY vector route LibreOffice 24.x and Word's Paste Special have,
// so a radio that draws and does not bind hands the operator a file those
// programs open as an empty frame. Written 2026-09-04 and NOT RUN — see the
// module header, which says so in its own words rather than leaving an absent
// result to imply it.
/// ★★★ O120's copy-OUT — the only observation of `native-clipboard`'s `unsafe` and of the placement ORDER against a real clipboard, which it REPLACES. Written 2026-09-04, **NOT RUN**; its module header says why.
pub mod copy_as_vector;
pub mod export_image_emf;
/// The fourth export on File ▸ Export, and the one whose interesting assertion
/// is an exact identity rather than a bound: the characters in the file equal
/// the characters the shell reported plus one separator per page boundary.
///
/// Written 2026-09-04 and **NOT RUN** — another session owned the desktop. The
/// module header says so in its own words rather than leaving an absent result
/// to imply it.
pub mod export_text;
/// The same defect one `/Subtype` along: a certified document's FORM FIELDS.
///
/// ★★★ [`annot_delete_gate`]'s fix closed one surface of three and left the
/// form-field door a no-op by construction — the condition's guard was false
/// whenever a field was selected, the `canvas.field` menu carried no
/// `visible_when` at all, and the Delete key's field rung asked nothing. This
/// aims at the merged signature widget in the SAME fixture pair that check
/// steers away from, and asserts the branch it avoids.
pub mod field_delete_gate;
/// **The first driven context menu in this project.** Its header records the
/// gap it closed: 92 checks and no `Driver::right_click_at`, so a whole gesture
/// class was outside R1's reach and left no failing test behind to say so.
pub mod field_menu;
pub mod find_bar;
pub mod form_field;
/// ★★★ The two forms surfaces `EDITABLE_SURFACES.md` found the engine had
/// shipped and this shell had never grown: **deleting a field group**, which no
/// control in the program could name, and **the structural refusal queries**,
/// which nothing consulted — so Rename and both Delete buttons were drawn live
/// on documents that refuse them.
///
/// The second is the shape a unit test cannot see: the correct behaviour is an
/// **absence**, and a test asserting "the query returns Some" passes on a build
/// where nothing calls the query. That is the state the audit found, under
/// 2,538 passing tests. Its header carries both fixtures' arguments.
pub mod form_groups;
pub mod form_leaf_descend;
pub mod form_leaf_move;
/// ★★★ **Where to click so that a form-field selection CHANGES**, shared by the
/// three checks that author a field and then try to select it.
///
/// Its header carries the 2026-08-29 finding in full: authoring a field leaves
/// it SELECTED (`OPERATOR_REQUESTS.md` O53) and `canvas::forms::select_click`
/// traces only on a change, so a check that clicked the field it had just
/// placed was asking the program to announce a selection that had not moved.
/// Neither the mapping nor the hit test was at fault — the clicks landed dead
/// centre — and the repair is to clear the selection on blank paper first,
/// which makes the checks assert both halves of `select_click`'s own table.
pub mod formaim;
/// Insert an image: the picture reaches the page, and the resolution the
/// window promised is the one the document reports.
pub mod insert_image;
/// ★ The OTHER half of the insert window: pressing *Place it on the page…*
/// makes the window step aside, a click on the page fills its numbers in, and
/// the window comes back. Beside `insert_image` because the two are one
/// surface — that one asserts the picture lands, this one asserts the operator
/// never has to type where.
pub mod insert_image_place;
/// The left rail — O123 part 7. ⚠ WRITTEN, NOT RUN: see the module header.
pub mod left_rail;
pub mod legibility;
pub mod markup_move;
pub mod markup_rectangle;
/// ★ The three Phase 6 markup kinds that are **not drag-shaped** — Freehand,
/// Polyline and Polygon — and the one control in this application whose
/// availability is decided by a gesture in progress rather than by the document.
/// It carries the only measurement of the ink simplification taken against a real
/// pointer, and the only falsifier in the suite that needs a control to be
/// **greyed at a specific moment mid-gesture**. Its header carries the argument.
pub mod markup_shapes;
/// ★★★ `OPERATOR_REQUESTS.md` O123's layout claims, driven — Objects over
/// Properties in one column with a draggable split, at a width whose rows fit.
pub mod master_detail;
pub mod progressive;
/// ★ **Smart-Selector** — a click selects the wrapped drawing, a double-click
/// goes inside it (`OPERATOR_REQUESTS.md` O70). Reads `canvas-selection …
/// first=`, the one field that tells the two index spaces apart.
/// ★ **Read mode copies a picture** — `OPERATOR_REQUESTS.md` O71. Two halves
/// in one sequence: an image is selectable while reading, and `Ctrl+C` puts a
/// bitmap on the Windows clipboard rather than a sentence.
pub mod read_image_copy;
/// ★★ **A signed document is warned about before it is saved.** The guard both
/// HOLDS the write while the question is on screen and RELEASES it when the
/// operator authorises one — asserted as a pair, because a build that showed
/// the window and wrote the file anyway satisfies either half alone.
pub mod signature_save;
pub mod smart_select;
pub mod trust_store;
pub mod unembed_fonts;
pub mod widget_move;
pub mod widget_rotate;

/// `ocr_recognises_a_page_and_the_document_keeps_it` — the whole Recognise-text
/// chain against a genuinely image-only document, ending in the one assertion
/// no unit test can make: **the file that was opened is byte-identical
/// afterwards.**
/// `a_click_inside_a_form_selects_what_is_drawn_there` — the driven proof of the
/// operator's headline complaint: a click inside a form XObject must select
/// what is drawn there, and a click on blank paper inside one must select
/// nothing.
pub mod bookmark_clipboard;
pub mod cut_gate;
/// ★★★ **A shape the operator drew can gain and lose a corner** — his own
/// report of 2026-09-05. ⬜ WRITTEN AND NOT DRIVEN; see the module header.
pub mod dimension_corner_count;
pub mod field_clipboard;
pub mod form_selection;
/// ★★ **Clicking a `/Link`** — the two checks for a capability that did not
/// exist in this shell at all until 2026-09-01, because a link's destination
/// could not be READ. The second of the two is the one that matters: a viewer
/// that treats all five `Destination` variants as navigable resolves the four
/// it cannot perform to a defaulted page 0 and navigates anyway, which has no
/// symptom an operator would report. Its header carries the argument.
pub mod link_follow;
/// ★ Markup ▸ Style — a ribbon group whose one item the manifest declared at S2
/// and no renderer ever drew, so it shipped as a caption over an empty band.
///
/// A **third** shape of invisible wiring, and the quietest: the manifest test
/// asserted the item was *declared* and passed correctly, and the reachability
/// check could not see it at all because a `Custom` item carries no command id.
pub mod markup_style;
pub mod measure_calibrate;
/// ★ The operator's own report (O105), driven on a fixture built to reproduce
/// it: one path object holding a small circle and forty unrelated segments.
pub mod measure_circular_points;
pub mod measure_hover;
pub mod measure_linear;
pub mod measure_perimeter;
/// Hovering with a measure tool armed says which line and which node.
/// The regression check for the icon painter that was never handed to a
/// context menu — the twin of [`qat_icons`] on the second surface it
/// happened on. See its header for why a menu row's own rectangle cannot
/// express the defect and `menu.icon.*` had to be published for it.
pub mod menu_icons;
/// ★ File ▸ New — the first command that makes a document out of **compiled-in
/// bytes** rather than out of a file the operator named, and the only check in
/// the suite whose subject is a page that is *supposed* to be blank. That is
/// what makes it worth having: a blank page and a page that failed to
/// rasterize are the same screenshot, so it reads the canvas's own `drawn=`
/// count instead of a pixel. Its header carries the argument and the
/// falsifying phase.
pub mod new_document;
pub mod new_document_size;
pub mod ocr;
/// ★ The three checks about a recognition run **while it is still running** —
/// the tally advancing, Stop keeping the work, Cancel discarding it. Separate
/// from [`ocr`] because all three need a multi-page run and that module's
/// one-page fixture has no observable middle. Its header carries the argument.
pub mod ocr_progress;
/// ★★ **A band dragged into the grey margin reaches an object off the page** —
/// `OPERATOR_REQUESTS.md` O92, asked by driving it rather than reasoned about.
/// Its fixture has exactly two squares and the band is aimed to miss the
/// on-page one, so `hits == 1` alone proves the marquee left the sheet — no
/// index, no ordering assumption. Its header carries why the band must touch
/// the off-page square without being able to enclose it.
pub mod off_page_marquee;
/// ★ The **Pages tab**, all of which did nothing: six verbs registered, drawn,
/// offered by a context menu and four of them bound to chords, with no dispatch
/// arm between them. The only check in the suite whose subject is a
/// **structural** change to a document rather than a mark drawn on it, and
/// therefore the only one that can assert the thing a page delete uniquely
/// breaks — that the shell's page vector, its rasters and its two selections
/// stop describing a document that no longer exists. Its header carries the
/// argument and the three falsifying phases.
/// ★★ A page pdfcer has already drawn is not drawn again — the operator's
/// *"they constantly redraw with larger files"*, measured by scrolling a real
/// drawing set away and back.
///
/// Its header carries why the oracle is the REQUEST STREAM and not the cache's
/// size: a build that held a gigabyte and still re-requested would pass a size
/// assertion and fail the operator, and a screenshot cannot help at all,
/// because a re-rendered page and a remembered one are the same picture.
pub mod os_fonts_setting;
pub mod page_cache;
pub mod page_clipboard;
/// ★★ **A page-display choice survives a close and reaches a new document** —
/// `OPERATOR_REQUESTS.md` O80. A two-process check, and the close must be
/// GRACEFUL: dropping a session kills the process, and a killed process runs
/// no exit hook, so the debounced write that carries the preference never
/// happens. Its header carries why the second document must be one the
/// program has never seen.
pub mod page_display_pref;
pub mod page_ops;
/// ★★★ **A save that would produce blank pages in Acrobat is refused** —
/// the operator's report of 2026-09-05. `pdfcer-core`'s `delete_pages` leaves
/// every ancestor's `/Count` above the immediate parent stale, so pdfcer's own
/// `/Kids`-walking reader sees a healthy document and Acrobat, which reads the
/// root `/Count`, shows the removed pages as blanks. The shell refuses the
/// write.
///
/// Its header carries the two things a reader has to know before touching it:
/// the fixture is **pinned** and nested, because on a flat page tree the defect
/// cannot occur and this check would pass against a build carrying it in full;
/// and it has **NOT BEEN RUN** — written 2026-09-05 while another track owned
/// the pointer.
pub mod pagetree_guard;
/// The capability the security audit found missing: an encrypted PDF could not
/// be opened at all. Its phase D reads the harness's own captured trace and
/// asserts the password is not in it.
pub mod password_prompt;
/// ★★ **Closing the program asks before losing unsaved work** —
/// `OPERATOR_REQUESTS.md` O102. It drives the one state no other check ever
/// constructed: a document with an unsaved edit at the moment of close.
pub mod quit_unsaved;
/// ★★ **Save As rebinds the document** — O95. Its oracle is the ORIGINAL
/// file's digest after a LATER save, because every cheaper oracle passes
/// against the defect it exists to catch.
pub mod save_as;
/// The **two** driven checks of *"give this page its own copy"*:
/// `the_context_menu_gives_this_page_its_own_copy_of_a_shared_form` and
/// `the_unshare_declines_when_nothing_else_draws_the_form`.
///
/// ★★ Two things no other check in this file can claim. It is the first to
/// **press a context-menu row** — until 2026-08-28 pdfcer's menus published no
/// `ui_rect` for any row, so no coordinate existed to aim at and the whole
/// "does the row do the thing" question was unaskable. And its subject is a
/// command whose SUCCESS is invisible: the copy `unshare_form` makes is
/// byte-identical to the original, so a page that was unshared renders
/// pixel-for-pixel as one that was not, and *"nothing appeared to happen"* is
/// what a pass and every possible failure look like alike.
///
/// ★★★ **A pair rather than one check, since 2026-08-29, and the pairing is
/// the point.** The command's two outcomes are decided by a property of the
/// *file* — is this form drawn on any other page — so each case is a fixture,
/// and a single check could only ever exercise one of them. The original
/// exercised the wrong one: it was named `…_of_a_shared_form`, it was pinned to
/// a document with exactly one invocation, and its pass note asserted that
/// "every other invocation site" was byte-identical about a set that was empty.
/// It measured nothing and passed. Whenever a behaviour is selected by the
/// input document rather than by the gesture, the fixture IS the test, and one
/// fixture is half of it.
pub mod unshare_form;

pub mod display_two_rows;
pub mod field_shading;
pub mod forms_spotlight;
/// **Selecting a page object names the layer it is on** — O126's third
/// feature, driven at last. Its header carries the vacuous-pass argument for
/// why the fixture is pinned, the two oracles, and the ⚠ notice that it has
/// **not been run**.
pub mod layers_membership;
/// ★ `file.print` — the dialog that told every operator this build could not
/// print, on a machine with twelve printers, in a build that had the printing
/// crate linked into it.
///
/// A new shape of the founding failure and the reason this module exists: the
/// adapter's own unit test asserted that all four of its calls **refused**,
/// which was correct while `pdfcer-print` was unlinked and became a lock
/// holding the defect in place the moment the manifest line landed. A green
/// suite defended the absence of the feature. See the module header.
/// Drag a page thumbnail to a new position, and see where it will land
/// before letting go.
/// **The Layers panel's search field is on screen and reachable** — O126.
/// Its header carries why the check refuses to pass on an absence.
pub mod layers_search;
pub mod pages_drag;
/// **A panel tears out into a real OS window, comes back, and closes** —
/// O126. Its header carries the two-line oracle and why one line is not
/// enough.
pub mod panel_float;
pub mod preset_group_reachable;
/// The commit button's clip count is corrected by what the preview has
/// already examined — operator request O113. See the module header for why
/// no unit test can observe the recording half of it.
pub mod preview_popout;
pub mod print_clip_claim;
pub mod print_dialog;
pub mod print_layout;
pub mod print_paper;
/// The Properties panel's document-metadata half: a title typed into it
/// reaches the file, and an undo takes it back out of the box too.
pub mod properties_metadata;
/// ★★★ `OPERATOR_REQUESTS.md` O123 part 2, driven — every control the Tool
/// panel held is on screen in Properties, its new home.
pub mod properties_tool;
/// O119, driven — File ▸ Security reports the document's own state and refuses
/// a signed document instead of drawing a form. ⚠ Written and NOT RUN.
pub mod protect;
pub mod qat_icons;
pub mod read_mode;
pub mod redact_image_warning;
pub mod tab_order_drag;
pub mod title_build_stamp;

/// ★ `tools.render_diagnostics` — the inert control whose data was already
/// being computed. `shell::commands::reach` called it *"the least defensible
/// kind — the work behind it is done"*: the renderer has produced the report
/// since S0, and what was missing was a `match` arm and a window.
pub mod render_diagnostics;

/// ★ The **Bézier handle** drag — the last Phase 1 row, and the one whose
/// failure mode is a perfectly plausible gesture: a handle sits inside the
/// selection box, so without a priority rule every attempt to shape a curve
/// moves the whole object instead.
pub mod bezier_handle;
/// ★★ **The one assertion no unit test in this workspace can make** — what is
/// actually on the operating system's clipboard after Ctrl+C.
///
/// Defect O18 shipped under 1,628 passing tests because the failure is not in
/// any function's return value: it is in WHICH OF TWO HANDLERS reached the OS
/// last. A trace cannot see that either — `text-copy source=selection` can be
/// emitted truthfully by a frame whose clipboard is then overwritten.
/// ★★★ **The annotation clipboard** — a sticky note, a stamp, a text box, a
/// link or a file attachment carried to another drawing with its baked
/// appearance intact. None of them could be copied at all before 2026-09-05.
/// Written that day and **NOT RUN**; its module header says so in its own
/// words rather than leaving an absent result to imply it.
pub mod clipboard_annotation;
pub mod clipboard_mode;
pub mod clipboard_text;
/// ★★★ **O89's object route** — the colour control on the text you CLICKED,
/// where `font_group` asserts only the sentence telling you to sweep. ⚠ **NOT
/// RUN**; its header carries the falsification table and the reason.
pub mod colour_clicked_text;
/// ★★★ **The operator's own MAX_PIXMAP_EDGE failure, driven** — zoom past the
/// ceiling and assert the page still renders.
///
/// Every piece of the region tier had unit tests before he hit this, and all of
/// them passed while the feature did not exist, because nothing called the
/// strategy. A complete unreachable mechanism is indistinguishable from a
/// working one from inside a test suite.
pub mod deep_pan;
pub mod deep_zoom;
/// ★★ **Drag-and-drop**, driven through the one seam that can carry it — a drop
/// originates in Explorer and cannot be synthesised by moving a mouse, so
/// without `PDFCER_DIAG_DROP_PATH` this would be the single feature in the shell
/// that R1 cannot reach.
pub mod dropped_file;
/// ★★★ **Enter makes a second line, and Ctrl+Enter finishes it** —
/// `OPERATOR_REQUESTS.md` O127, defect 2.
///
/// ⚠ Written 2026-09-04 and **not executed** — the operator was at his keyboard
/// and a second run would have fought his pointer. See its header.
pub mod enter_newline;
/// ★★★ **Zero clicks.** The only check in this suite that drives no gesture: it
/// opens a document, enters Edit, and asks what an operator SEES. Every other
/// test of the tool list asks whether a named command is present — a question
/// that stays green while the answer goes stale, because a list can be complete
/// about yesterday's tools and silent about today's.
pub mod first_frame;
/// **The exception O55 names**: a fit that a pan has left must not be re-placed
/// by the next resize. Its sibling asserts the opposite for the wheel, so the
/// pair pins both directions.
pub mod fit_left_by_a_pan;
pub mod fit_places_the_view;
/// ★★★ **The ribbon route to a restyle, and the sentence that tells an operator
/// how to reach it.** `restyle_text` below drives the PANEL; this drives the
/// Format tab's Font group, and it drives the half of O37 that is not a
/// capability — the two surfaces that answer *"nothing on screen tells you to
/// press T"*, observed in the state before anything is swept, because that is
/// the state an operator is in when they need them.
pub mod font_group;
/// ★★ **Redaction** — the one operation in this program that cannot be undone,
/// and the only check in the suite whose verdict is a **byte scan of a file on
/// disk** rather than a trace field or a pixel. The application's own absence
/// proof reports `verified=true` from inside the process that performed the
/// removal; this asks the same question from outside it, three times, over two
/// strings, in two processes — and it says which of the three answers is the
/// verdict and which two exist to stop the verdict passing vacuously. Its
/// header carries the falsification table.
/// ★ The eight resize grips, driven — they were cursored, hit-tested and
/// drag-consuming from S4 and committed nothing until 2026-08-19.
///
/// Its header carries the six links and names the one that would fail silently
/// and plausibly: a resize about the wrong anchor still resizes.
/// ★ The **typed** route to a resize — the Properties panel's X/Y/W/H fields.
/// Shares only its last link with `resize`: the four in between are a panel
/// drawing, a draft surviving frames, and a button un-greying, and the third of
/// those is invisible to every unit test because it is a property of the
/// SEQUENCE of frames rather than of a function.
pub mod geometry_fields;
/// ★★ **The zoom readout is a button now, and buttons must be proved to do
/// something** — O24.
///
/// The Select popup shipped with a double toggle that made its button inert,
/// green on 1,628 unit tests and a smoke launch confirming its rect. Every one
/// of those observed the button, which was never the broken part.
pub mod max_zoom;
/// ★ Shift-picked anchors move TOGETHER — the row `pdfcer`'s own `gui` column
/// ticked `[x]` and their 2026-08-19 sweep corrected to "objects move together;
/// nodes one at a time". The capability was in the selection model from the day
/// the Node rung landed and no consumer read it that way.
pub mod multi_node;
/// ★★★ **The operator's oldest open request** — cut, copy and paste of page
/// content. Its header carries why the interesting failure is silent: a clip
/// that carried the operators and dropped the resources pastes the right glyphs
/// in the wrong typeface and errors nowhere, so the assertion is a COUNT the
/// engine reports rather than a picture.
pub mod object_clipboard;
pub mod pan_refresh;
pub mod reach_out;
/// ★ `view.read_mode` — the command with a control, a glyph, a group, `Ctrl+H`
/// and a line in the shortcuts reference, and **no dispatch arm** for the whole
/// life of the project. Its whole behaviour is one `if` in the frame
/// composition, which every unit test in the workspace is blind to.
///
/// Named `read_mode_chrome` rather than `read_mode` because that name is
/// already taken by the check one line up, and the two are about genuinely
/// different things: that one is `mode.read`'s **capability** gate (a click in
/// Read must not select), this one is `view.read_mode`'s **chrome** toggle (the
/// ribbon and the docks stop being drawn). `app::window` §1 carries the
/// argument for why those are two commands rather than a duplicate.
pub mod read_mode_chrome;
/// ★★★ **The way back OUT of read mode**, which the check above deliberately
/// does not cover: its own header says *"the return trip is not driven here"*,
/// because the exit is a chord and it drives the mouse only.
///
/// The operator fell through exactly that gap on 2026-09-05 — *"I didn't see a
/// way to get back out of read mode"* — and the answer is a statement on the
/// window title and on the status bar naming the chord the **keymap** holds.
/// This reads that statement from a trace: no pointer, no keystroke, so unlike
/// its neighbour it can run beside somebody working.
pub mod read_mode_exit;
pub mod redact_selection;
pub mod redaction;
/// **Paragraph reflow, driven.** The one check whose operand is a caret in
/// egui's temporary memory — put there by a click and read by a command, with
/// no other instrument that can see the handover.
pub mod reflow;
pub mod resize;
/// ★★ **Sweep text, press Bold, and the file changes** — O37's font tools,
/// driven. `app::actions::textstyle`'s eight unit tests would all still pass on
/// a build where the panel never draws, because they call the verb directly;
/// three links of the chain in front of it have no other instrument.
pub mod restyle_text;
pub mod ribbon_captions;
/// The zoom ladder and the closed-loop aim `scale_sweep` drives its battery
/// with. Split out under R2 on 2026-09-05; its header carries the seam.
pub mod scale_aim;
pub mod scale_sweep;
// The band's PROPORTIONS against `mockups/pdfcer-shell.html`, and the two
// claims about it that only a rendered screenshot can settle: a resting
// control drawn with no frame, and a control drawn with no glyph. Written
// 2026-09-04 with the fix it verifies, and deliberately left UNRUN -- see
// its header for why, and for the command that runs it.
pub mod ribbon_mockup;
/// ★★ **The ninth grip** — the rotate handle above the selection box, and the
/// third word of the operator's *"reposition, resize, or rotate"*. Its header
/// carries the three links that would each produce a working gesture aimed at
/// the wrong verb, and why the sign of the committed angle is the assertion
/// that matters.
pub mod rotate;
/// **The Tool-row scale switches, driven.** `OPERATOR_REQUESTS.md` O51's
/// `Scale line weight`, from the checkbox to the engine's `/BS /W`. Three of
/// the five links in front of the verb are wiring no unit test can see.
pub mod scale_switch;
/// ★★★ **The shape follows your hand** — `OPERATOR_REQUESTS.md` O63. Asserts
/// the live geometry preview is BUILT, that it reaches the PAINTER, and that it
/// OUTLIVES the release; its header carries why "built and never painted" needs
/// a trace line of its own.
pub mod shape_preview;
/// ★★★ **The chooser offers a face the document does not contain** —
/// `pdfcer-core` v0.15.0's standard-14 authoring, reached from a font list.
///
/// The engine shipped the capability and the shell could not reach it: the
/// chooser built its list from `preview_font_resources`, which enumerates the
/// *page's own* `/Font` resources, so the one thing the release note is about
/// was absent from every surface in the program. Its header carries the six
/// links and names the fourth as the one worth writing the check for on its own
/// — pdfcer embeds nothing, so the text is drawn with the READER'S copy of the
/// face, and a disclosure that is catalogued, unit-tested and never painted has
/// discharged nothing.
pub mod std14_face;

/// **Text that does not run along the page's x axis** — the operator's
/// 2026-08-26 report about a vertical stamp in a title block, driven end to
/// end: it must select as one line, band as one box, turn the I-beam, and reach
/// the OS clipboard without a newline in it.
pub mod rotated_text;
/// ★ `file.save_copy` — the command that was registered, drawn, on the
/// quick-access toolbar and bound to `Ctrl+S` with **no dispatch arm**, so
/// nothing this shell could author could reach a disk. The only check in the
/// suite that spans **two processes**: it authors an annotation with a real
/// drag, saves a copy, and then re-opens the saved file in a fresh binary to ask
/// whether the annotation is in it. Its header carries the three falsifying
/// phases and the different wrong build each one catches.
pub mod save_copy;
pub mod save_in_place;
/// ★★★ **Does scrolling a long way cost the canvas its pointer input?**
///
/// The experiment that decides whether O23's pasteboard failure is a feature
/// problem or a defect the operator already meets. It reproduces from an
/// ordinary wheel scroll with no pasteboard in the build, or it does not.
pub mod scroll_input;
/// ★★ **The selection filter is load-bearing, not decorative** — switching a
/// class off changes what the next click on the same pixel selects.
///
/// Deliberately NOT "the popup opens", which is a unit test and is also the one
/// claim that stays true of an inert control — this popup shipped on 2026-08-21
/// with a double toggle that made its button do nothing, under 1,628 passing
/// tests and a smoke launch confirming the button's rect.
pub mod select_filter;
pub mod settings_headings;
/// ★ **Shift preserves aspect on a resize** — `ui-conventions/drag-moves.md`
/// D5, found absent from every drag in this shell by the 2026-08-20 sweep.
/// Proves the constraint by the DIFFERENCE between two drags in one process,
/// because a single locked drag reporting equal factors proves nothing.
pub mod shift_constrains;
/// ★★ **The operator's own two gestures** — press T and type, press A and see
/// the points. Both features existed before 2026-08-19; reaching them took four
/// steps and three gestures respectively, neither discoverable and neither
/// resembling any other program. These assert the COUNT: one key, one click.
pub mod tool_row;
pub mod wheel_flips_pages;
pub mod zoom_gallery;
pub mod zoom_keeps_place;
pub mod zoom_out_keeps_place;

/// Marking a text selection — underline, strikeout, squiggly. The first
/// commands in this shell whose operand is **not the pointer**, and therefore
/// the first check that asserts a control is *correctly disabled* as well as
/// that it works. Its header carries the argument.
/// **The text-editing round trip** — `DEFECTS.md` D4b, driven: a chord arms the
/// caret tool, a click resolves a run, a commit plans the follower disposition,
/// a save writes it, and a second process reads it back. Its verdict is a byte
/// scan for an operator the operator did NOT touch.
/// ★ The three markup kinds that carry WORDS, and the one property that
/// separates them from the seven geometric ones: the RELEASE MUST NOT AUTHOR.
/// Its header carries why no unit test can see that.
/// **Opening a second PDF adds a tab, and the tab switches to it** — the
/// registration half of the multi-document work of 2026-08-20, which no unit
/// test can observe.
pub mod document_tabs;

/// **A page dragged out of one open document and into another**, through a
/// spring-loaded tab — the operator's request of 2026-08-19, end to end.
///
/// Two registrations, one implementation: the unmodified drag must **copy** and
/// the Shift-held drag must **move**. Running only one would pass against a
/// build that always did the same thing, which is exactly what a modifier read
/// at the wrong moment produces.
pub mod page_drag_between_documents;

/// **Dragging a document tab along the strip moves it**, and does not change
/// which document is on screen — the second half being the failure that would
/// otherwise ship silently.
pub mod tab_reorder;

pub mod about;
pub mod add_text;
/// Insert a form's pages to make orphaned widgets, then register one back.
pub mod adopt_widget;
pub mod attachment_clip;
/// ★★★ **The attachments round trip** — attach a file, read it back out, and
/// compare the BYTES. Its header carries why a trace line is not enough here:
/// an embedded file changes no pixel, so a truncated stream has no visible
/// symptom at all.
pub mod attachments;
/// ★ The harness's own ribbon search, driven: a command two scroll stops past
/// the fold is still reachable. Its header records a HARNESS defect that
/// reported the application as broken for eight days.
pub mod band_scroll;
/// A bookmark can be written into a document that has no outline.
/// ★★ **The cursor walks between blocks of text** — salvage from the shell this
/// project replaces, on the operator's report of 2026-08-21. Its header carries
/// why the assertion is a CHANGE OF RUN rather than a caret movement: a build
/// that moved within the same run looks identical from outside and does nothing
/// that was asked for.
pub mod block_nav;
pub mod bookmark_add;
/// ★★ **The Bookmarks panel could only ever create** — rename and delete
/// shipped in `Pass 156.0` and this drives both through the one block that
/// carries them. Its header carries why the delete oracle asserts the count
/// EXACTLY rather than "fewer than before".
pub mod bookmark_dest;
pub mod bookmark_edit;
/// ★★★ **The Bookmarks panel could not REORGANISE** — `Pass 161.0` shipped
/// `move_outline_item` and `set_outline_open`, and this drives both through the
/// row list: a bookmark is dragged onto the middle of another and nests, then
/// the branch is folded away with its triangle.
///
/// Its header carries the two oracles no unit test can reach: the moved row's
/// **level**, which a reorder cannot produce however wrong it is, and the
/// **disagreement** between the panel's item count and the number of rows it
/// draws after a collapse — which is the whole of *"the sign is honoured"*.
pub mod bookmark_move;
pub mod button_action;
pub mod chords;
/// ★★★ **The Comments panel stopped being a viewer** — a note can be written
/// onto a shape that already exists, which needed a verb `pdfcer-core` did not
/// have until `Pass 154.0`. Its header carries why link 3 of the chain — a
/// widget raising the action — is unreachable by any unit test.
pub mod comment_note;
pub mod dialog_windows;
/// A selection INSIDE a text draft — Shift+arrows and the rule that drops it.
/// Not to be confused with `text_selection`, which is about sweeping the
/// page's own text with the pointer.
pub mod draft_selection;
/// ★ `DEFECTS.md` **D10**'s second half — three themes shipped and nothing an
/// operator could press chose one. Proved the only way a theme can be proved:
/// two captures of one window, before and after the click.
pub mod marquee_table;
pub mod ocr_text_select;
pub mod save_after_edit;
pub mod settings_theme;
/// The keyboard reference lists every chord, and every chord names a command.
pub mod shortcuts;
pub mod text_annot;
pub mod text_annot_focus;
/// ★ The operator's own report, driven: Edit text on a REAL CAD sheet, aimed at a
/// point the ENGINE says carries text. Its header carries why two passing text
/// checks were not enough — both drive fixtures this repository generated to
/// verify itself.
/// ★★★ **Multi-line text** — the operator's ask of 2026-08-21. Its header
/// carries why multi-line needs a rectangle (a PDF has no paragraph) and which
/// of the four links would ship silently: a plain Enter that still commits ends
/// the draft at the first line break and discards everything after it.
pub mod text_box;
pub mod text_edit;
pub mod text_edit_real;
pub mod text_markup;
/// The canvas text-selection sweep: the one feature whose entire behaviour is a
/// drag and whose entire feedback is a translucent wash, so a screenshot cannot
/// tell it from a page with nothing selected. Its header carries the argument.
pub mod text_selection;
/// ★ The **text tool** in Edit, and the `RIBBON_IA.md` P3 tension it closes. The
/// only check in the suite that observes one control **dead and then live in the
/// same mode in the same run**, and the only one whose subject changes nothing an
/// operator can see except the mouse pointer — which a window capture does not
/// carry at all. Its header carries the argument.
pub mod text_tool;
/// ★★★ The two theme blind spots `REVIEW_TRIAGE.md` PartC named — the Airy
/// preset, which nothing in this repository had ever clicked, and the PAGE,
/// which nothing had ever sampled under a theme. Its header carries why the
/// page is the one invariant a dark theme in this product must hold.
pub mod theme_page;
/// ★★★ **A refusal an operator can read** — `OPERATOR_REQUESTS.md` O140, driven
/// on the file he reported it on.
///
/// The only check in the suite that takes its **negative control through the
/// same instrument in the same process**: it commits an edit the engine refuses
/// and asserts the `⊗` slot drew, then commits one that succeeds and asserts it
/// did **not**. Its header carries why a one-sided reading of that region is not
/// a verdict, and why the tempting `Identity-H` forecast is falsified by
/// `pdfcer-core`'s own fixture.
pub mod typo_refusal;
/// ★ `edit.undo` and `edit.redo` — the pair that was registered, drawn on the
/// quick-access toolbar in **every** mode and bound to three chords with **no
/// dispatch arm**, so an operator could author dimensions, seven markup kinds,
/// text marks and form fills and take none of it back. The only check in the
/// suite that asserts a document change was **un-made**, and the only one whose
/// oracles include two *invalidation* signals — a fresh `objects` line and a
/// fresh `render-spawn` — because the build it exists to catch is one whose
/// every count is already correct. Its header carries the argument.
pub mod ui_scale;
pub mod undo_redo;

use std::path::{Path, PathBuf};

use crate::coords::DocPoint;
use crate::profile::Profile;
use crate::report::CheckReport;

/// Everything a check needs to know about this run.
#[derive(Clone, Debug)]
pub struct CheckContext {
    /// The target binary's vocabulary and regions.
    pub profile: &'static Profile,
    /// The binary to drive. `None` means the checks that drive one SKIP.
    pub exe: Option<PathBuf>,
    /// The document to open.
    pub pdf: Option<PathBuf>,
    /// **A second, DIFFERENT document**, for the checks that need two open at
    /// once.
    ///
    /// It must not be the same file as [`Self::pdf`]. `crate::app::documents`
    /// §3 makes pdfcer activate the tab a path is already open in rather than
    /// open a duplicate — deliberately, because two `EditSession`s over one
    /// file would be two undo stacks and a save from either would discard the
    /// other's work. So passing the same path twice would make a multi-document
    /// check assert the opposite of what it is for, and the checks that need
    /// this SKIP rather than fall back to [`Self::pdf`].
    pub second_pdf: Option<PathBuf>,
    /// An already-captured image to assert against instead of driving the
    /// application — the offline mode for pixel checks. Its purpose is
    /// falsification against a dated artefact; see
    /// [`crate::profile::Calibration`].
    pub image: Option<PathBuf>,
    /// Where screenshots and trace copies are written.
    pub out_dir: PathBuf,
    /// WCAG contrast floor. Defaults to [`crate::pixels::AA_LARGE`].
    pub contrast_threshold: f64,
    /// Whether the harness may take the operator's pointer and keyboard.
    /// `false` makes every driving check SKIP — never pass.
    pub allow_input: bool,
    /// Drive a binary older than its sources.
    pub allow_stale: bool,
    /// Source tree the staleness gate compares against.
    pub source_root: Option<PathBuf>,
    /// Explicit page size, when the fixture's `/MediaBox` cannot be read.
    pub page_size: Option<(f64, f64)>,
    /// The document point a driving check aims at.
    ///
    /// **There is deliberately no default.** A default would be a guess about
    /// where the fixture keeps an object, and a click on empty page is
    /// symptom-identical to a broken hit test — the confusion that produced a
    /// filed-then-retracted defect in this codebase. Absent, the driving
    /// checks SKIP and say what to pass.
    pub target: Option<DocPoint>,
}

impl CheckContext {
    /// A path under the run's output directory — **and the directory is made
    /// to exist before the path is handed back.**
    ///
    /// # ★★★ Why the `create_dir_all` is here and not at every call site —
    /// 2026-09-05
    ///
    /// It was at *some* call sites. [`crate::launch`] creates the parent of the
    /// trace file it is about to open, and [`crate::image`] creates the parent
    /// of a PNG it is about to save, so every check whose first use of this
    /// directory was a launch or a screenshot worked. Five checks write a
    /// **fixture** into it *before* they launch anything —
    /// `save_writes_over_the_file_you_opened` copies the document it is going to
    /// overwrite, `redaction_removes_and_proves_it` writes the PDF it will
    /// redact, `insert_image_places_a_picture`,
    /// `the_insert_window_steps_aside_so_you_can_point` and
    /// `a_dropped_image_reaches_the_placement_window` each write a PNG to drop —
    /// and every one of them failed with
    ///
    /// ```text
    /// cannot write …\insert_image_fixture.png: The system cannot find the
    /// path specified. (os error 3)
    /// ```
    ///
    /// on the first sweep that ever ran them (2026-09-05, `--out` pointed at a
    /// fresh per-check directory). They **SKIPPED**, which is not red, so the
    /// suite reported its usual cheerful INCOMPLETE and five checks that have
    /// never once driven the application looked like ordinary
    /// wrong-fixture skips.
    ///
    /// ★★ That is the same shape as the `repo_fixture` defect fixed earlier the
    /// same day: *a path that cannot resolve produces a SKIP, and a SKIP is not
    /// a failure, so the check can be dead for ever while the suite looks
    /// healthy.* The durable fix for that shape is a funnel, not a fifth
    /// `create_dir_all` — every path into this directory now comes from here.
    ///
    /// # Why the error is swallowed
    ///
    /// The return type is a path, not a result, and forty call sites read it in
    /// expression position. A directory that genuinely cannot be created — a
    /// read-only volume, a name that is not a directory — still produces an
    /// error at the moment of the write, from the code that knows what it was
    /// writing and can say so. Making this fallible would trade a precise
    /// message at the write for a vague one here.
    #[must_use]
    pub fn out(&self, name: &str) -> PathBuf {
        // Best effort, deliberately: see the doc comment.
        let _ = std::fs::create_dir_all(&self.out_dir);
        self.out_dir.join(name)
    }

    /// The exe to use: the explicit one, or the profile's default if it is
    /// actually there.
    ///
    /// A default that does not exist is `None` rather than a path, so the SKIP
    /// reason says "no binary" once rather than describing a path the caller
    /// never chose.
    #[must_use]
    pub fn resolve_exe(&self) -> Option<PathBuf> {
        if let Some(e) = &self.exe {
            return Some(e.clone());
        }
        let default = Path::new(self.profile.default_exe);
        default.is_file().then(|| default.to_path_buf())
    }
}

/// One check.
pub trait Check {
    /// The name `--check` accepts.
    fn name(&self) -> &'static str;
    /// Which defect it detects, in one line, for the report.
    fn defect(&self) -> &'static str;
    /// Run it. A check never panics and never returns an error: every outcome,
    /// including "I could not start", is a [`CheckReport`].
    fn run(&self, ctx: &CheckContext) -> CheckReport;
}

/// **The roster** — every check, in the order the suite runs them.
///
/// Its own file under **R2**, and the seam is argued in its header: this
/// module is the harness's *vocabulary* (the [`Check`] trait, the
/// [`CheckContext`]) and that one is the *list*, which is the only thing here
/// that grows with every landing.
mod roster;

pub use roster::all;
