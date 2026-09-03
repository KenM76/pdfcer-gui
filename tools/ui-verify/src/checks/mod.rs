//! The named checks — the suite this harness exists to run.
//!
//! ## The three, and what each is for
//!
//! | Check | Defect | Oracle |
//! |---|---|---|
//! | [`delete_key`] | **D1** — Delete stops working after the first canvas click | the trace |
//! | [`ribbon_captions`] | group captions rendering illegibly, or not at all | the pixels |
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
/// Hovering with a measure tool armed says which line and which node.
pub mod measure_hover;
pub mod measure_linear;
pub mod measure_perimeter;
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
pub mod pages_drag;
pub mod preset_group_reachable;
pub mod print_dialog;
pub mod print_layout;
pub mod print_paper;
/// The Properties panel's document-metadata half: a title typed into it
/// reaches the file, and an undo takes it back out of the box too.
pub mod properties_metadata;
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
pub mod clipboard_text;
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
    /// A path under the run's output directory.
    #[must_use]
    pub fn out(&self, name: &str) -> PathBuf {
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

/// Every check, in the order the suite runs them.
#[must_use]
pub fn all() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(delete_key::DeleteKeyAfterCanvasClick),
        // The Delete key's OTHER subject. `delete_key` drives it over page
        // content on an ordinary document; this drives it over an annotation on
        // a certified one, where the honest outcome is that the control is not
        // offered at all. Adjacent because a reader comparing two Delete
        // verdicts wants them together, and because a run in which the key
        // channel is broken should say so in the cheaper check first.
        Box::new(annot_delete_gate::ACertifiedDocumentWithholdsAnnotationDelete),
        // The FORM-FIELD half of the same question, on the same fixture pair,
        // immediately after the annotation half — a reader comparing two Delete
        // verdicts on one certified document wants them together, and the two
        // checks aim at the two objects that file carries for exactly that
        // reason.
        Box::new(field_delete_gate::ACertifiedDocumentWithholdsFieldDelete),
        Box::new(ribbon_captions::RibbonGroupCaptionsLegible),
        // Reads the trace only — no window is raised and no capture is taken,
        // so it costs nothing and cannot take the operator's focus. Placed
        // after the captions check because both launch, and a reader
        // comparing two ribbon verdicts wants them adjacent.
        Box::new(qat_icons::QatControlsAreIconOnly),
        // ★ The first *driving* check, and it goes first among them on
        // purpose: it is the cheapest — two clicks on one always-enabled
        // control, no canvas gesture, no keystroke, no capture — and it is the
        // one whose failure most changes what a later failure means. Every
        // check below assumes a document is on screen; this one is the only
        // one that makes a document rather than being handed one, so if the
        // ribbon-click channel is broken it says so here, in seconds, instead
        // of at the end of a canvas drag.
        Box::new(new_document::NewDocumentMakesAPage),
        // ★ Immediately after its sibling, and it drives the OTHER New.
        //
        // `new_document` asserts that `file.new` makes a page at all;
        // this one asserts that `file.new_from_template` makes a page of the
        // size that was asked for. Adjacent because a failure in the first
        // explains a failure in the second, and reading them the other way
        // round wastes the reader's first hypothesis.
        //
        // It launches with NO fixture, like its sibling and for the command's
        // own reason: `file.new_from_template` is registered with no
        // `enabled_when` because an operator with nothing open is the one it
        // exists for.
        Box::new(new_document_size::NewDocumentSizesThePage),
        // Clicks and captures, so it takes the desktop — but only with the
        // mouse, and only for a few seconds. Placed after the three ribbon
        // chrome checks because it depends on the same rects they read and a
        // reader comparing ribbon verdicts wants them together; placed before
        // the two typing checks because a run that fails here should fail
        // before paying for a keystroke that may never arrive.
        Box::new(markup_move::DraggingAMarkupMovesIt),
        // ★★★ Immediately after the move, and deliberately: the two share
        // steps 1-3 verbatim in shape — draw a rectangle, put the pen down,
        // click it — so a failure in EITHER of those here should be read
        // against `dragging_a_markup_moves_it`'s result first. If both fail at
        // the same step, the defect is in authoring or selection rather than in
        // either gesture.
        Box::new(annot_rotate::RotatingAMarkupTurnsIt),
        Box::new(widget_move::DraggingAFormFieldMovesIt),
        // `OPERATOR_REQUESTS.md` O76, 2026-08-31. Beside the move check
        // because they are the same gesture family on the same operand and
        // differ in one field of one trace line — which is exactly the
        // distinction a reader comparing them needs to see.
        Box::new(checkbox_resize::AResizedCheckBoxIsRedrawn),
        // ★ Beside the other form checks. It reuses `widget_move`'s first two
        // steps verbatim in shape, so a failure in EITHER of them here should
        // be read against that check's result first.
        Box::new(field_menu::RightClickingAFormFieldOpensItsMenu),
        Box::new(markup_rectangle::MarkupRectangleArmsFromTheRibbon),
        // ★ Form-field placement and selection. After the markup checks and
        // not before, because it borrows their gesture machinery — a band drag
        // and a canvas click — so a failure in either would be reported there
        // first, where the cause is, rather than here where the symptom is.
        Box::new(form_field::FormFieldPlaceAndSelect),
        // ★★ Directly after it, and the order is a **dependency** rather than a
        // preference. The refusals half ends by clicking a widget the canvas
        // census names and reading the Properties pane — which is
        // `form_field`'s phases C and D exactly. A run in which canvas
        // selection is broken should report that as `form_field`'s failure,
        // where the cause is, and a reader who has already seen it fail knows
        // to ignore this one's phase G.
        //
        // The group half comes first of the two because it needs no canvas
        // gesture at all: it is three panel clicks, so a failure in it is
        // unambiguously about the Forms panel's own wiring.
        Box::new(form_groups::FieldGroupDeleteRemovesTheSubtree),
        Box::new(form_groups::StructuralRefusalsAreSentencesNotControls),
        // ★ The blend-space disclosure. After the zoom checks, because it
        // climbs with Ctrl+wheel and a wheel that does not reach the canvas is
        // `zoom_gallery`'s failure to report, not this one's.
        Box::new(blend_space::BlendSpaceFallbackIsDisclosed),
        // ★ Immediately after `pan_refresh`, because they are the two halves of
        // one gesture: that one asserts the new area RENDERS, this one asserts
        // the page is not blank WHILE it renders. A failure of the first should
        // be read first — if nothing renders at all, what is on screen during
        // the wait is a secondary question.
        Box::new(progressive::ProgressiveRenderNeverGoesBlank),
        // Beside it, because it is the same shape of check on the same
        // surface — a ribbon control that arms a canvas tool — and a reader
        // comparing the two verdicts wants them adjacent. It goes second
        // because it is the longer of the two: it clicks the page as well as
        // the ribbon, and a snap candidate that needs confirming costs it an
        // extra click.
        // Directly after `markup_rectangle`, because it is the same surface and
        // the longer of the two: it arms three tools, clicks out two runs and
        // drives one drag. A run where the four-link chain itself is broken
        // should report that as `markup_rectangle`'s failure first — this one
        // would fail for the same reason with three more candidate causes in
        // front of it.
        Box::new(markup_shapes::MarkupFreehandAndVertexKinds),
        // ★ Directly after the markup checks, and for the same dependency
        // reason: this one is about the pen those gestures author WITH, so a
        // run in which the ribbon-click channel is broken should report that as
        // `markup_rectangle`'s failure first.
        Box::new(markup_style::MarkupStyleGroupIsDrawn),
        Box::new(measure_perimeter::MeasurePerimeterTracesAndCloses),
        // ★ Early, and deliberately: it is the cheapest possible statement of
        // "can this program open the file at all", and a failure here changes
        // what every later failure on an encrypted document would mean.
        Box::new(password_prompt::AnEncryptedDocumentCanBeOpenedWithItsPassword),
        Box::new(measure_linear::MeasureLinearPlacesADimension),
        // Beside the linear check: same tab, same arming path, and a reader
        // comparing two measure verdicts wants them together. It pins its own
        // fixture rather than taking --pdf — see its header for why a sweep's
        // document cannot exhibit the defect it exists for.
        Box::new(measure_circular_points::ThreeClicksRoundAHoleMeasureTheHole),
        // Straight after the linear tool, because it is the same gesture with a
        // different ending and a reader scanning the suite should meet them
        // together — and because a calibration is worthless if the linear pick
        // it reuses is broken, so failing in that order reads as a diagnosis.
        Box::new(measure_calibrate::MeasureCalibratesByPickingTwoPoints),
        Box::new(measure_hover::MeasureHoverShowsWhatItWillTake),
        // ★ The Manage-groups window, wired 2026-08-18 after being registered,
        // drawn and inert for the whole life of this build. Beside the two
        // measure checks because it is the third link in the same chain: a
        // tool places a dimension, a window calibrates its group, and this one
        // is where the group comes from.
        Box::new(dimension_groups::DimensionGroupsPanelMakesAGroup),
        // ★ Directly after the markup checks, and the order is a **dependency**
        // rather than a preference: this one begins by arming Rectangle and
        // dragging, which is `markup_rectangle`'s and `markup_shapes`' whole
        // subject. A run in which the four-link arm chain is broken should
        // report that as their failure and this one's second — here the same
        // symptom has the entire save path stacked behind it, and a reader who
        // has already seen Rectangle fail knows to ignore the rest.
        //
        // It is the most expensive check in the suite by some way: it launches
        // the binary **twice**, because the round trip it exists to prove is
        // that a second process can read what the first one wrote. Placed
        // before the two typing-dependent checks all the same, because it is
        // the only one whose subject is a file on disk and a run that cannot
        // save should say so before it spends anything on a keystroke that may
        // never arrive.
        Box::new(save_in_place::SaveWritesOverTheFileYouOpened),
        Box::new(save_copy::SaveCopyRoundTrip),
        // ★ Directly after it, and the order is a **dependency** rather than a
        // preference: this check ends by saving a copy and re-opening it, so a
        // run in which `file.save_copy` itself is broken should report that as
        // `save_copy_round_trip`'s failure and this one's second. A reader who
        // has already seen the save fail knows to ignore everything below
        // phase E here.
        //
        // It is the second most expensive check in the suite, and for the same
        // reason: it launches the binary **twice**, because a page count read
        // in the process that deleted the page is a count the code under test
        // wrote about itself.
        Box::new(page_cache::PagesStayDrawnWhenYouScrollBack),
        Box::new(page_ops::PageOpsRoundTrip),
        // ★ The FIRST check anywhere that drives the Pages PANEL rather than
        // the Pages tab. `page_ops` above drives the ribbon and records why it
        // does not touch the panel — the tile's context menu is an egui popup
        // that declares no regions. The tiles themselves declare regions as of
        // 2026-08-18, which is what made this possible.
        Box::new(pages_drag::PagesDragShowsWhereItLands),
        Box::new(tab_order_drag::TabOrderDragMovesAFieldAndShowsWhere),
        Box::new(forms_spotlight::ClickingAFormRowLightsTheFieldOnThePage),
        Box::new(display_two_rows::TheDisplayButtonsStackInTwoRows),
        Box::new(title_build_stamp::TheTitleBarCarriesTheBuildTime),
        Box::new(field_shading::FillableFieldsAreShadedOnThePage),
        Box::new(preset_group_reachable::TheStandardsPresetsGroupIsReachable),
        Box::new(redact_image_warning::MarkingOverAnImageSaysSoBeforeApply),
        // The DXF export, wired 2026-08-19 after being the FIRST entry in
        // `reach`'s scaffold list. Beside the page checks because it is the
        // other verb that writes a file the operator hands to somebody else.
        Box::new(embed_fonts::EmbeddingFontsPutsAProgramInTheDocument),
        Box::new(embed_bundled::EmbeddingWorksWithNoFontFolderAtAll),
        Box::new(compact_save::ACompactedCopyIsActuallySmaller),
        // ★★★ The signature warning, wired 2026-08-28. Beside the save checks
        // because it is the fourth thing this shell can do to a file somebody
        // else will open — and the only one whose subject is what the file
        // CLAIMS about itself rather than what it contains.
        //
        // After `compact_save` deliberately: that check drives the one save
        // path that already disclosed its effect on signatures (a full rewrite
        // destroys them all, §12.8.1, and its window says so before the picker
        // opens), so a reader comparing the two verdicts is reading the two
        // halves of one subject in the order they were built.
        Box::new(signature_save::AnInvalidatingSaveIsWarnedAbout),
        Box::new(os_fonts_setting::FontFoldersLandsOnTheFontsSetting),
        Box::new(unembed_fonts::RemovingEmbeddedFontsReachesTheDocument),
        Box::new(export_form_data::ExportingFormDataWritesAFile),
        Box::new(export_dxf::ExportDxfWritesThePagesGeometry),
        // Insert an image, wired 2026-08-19. Its last assertion is the one
        // that matters: the promised resolution and the reported one are the
        // same number, which is the shell's half of a single-derivation
        // guarantee `pdfcer-core` holds up on its side with a test.
        Box::new(attachment_clip::AnAttachmentMovesBetweenTwoOpenDocuments),
        Box::new(marquee_table::AMarqueeOverATableTakesItsTextAsWellAsItsLines),
        Box::new(bookmark_dest::ABookmarkLandsOnTheDetailItNames),
        Box::new(ocr_text_select::TextOnAScanCanStillBeSweptOverTheImage),
        Box::new(save_after_edit::CtrlSAfterAnEditSavesAndTheProgramIsStillRunning),
        Box::new(button_action::APlacedButtonCanBeGivenSomethingToDo),
        Box::new(insert_image::InsertImagePlacesAPicture),
        // `OPERATOR_REQUESTS.md` O66, 2026-08-31. Immediately after its
        // sibling, because it depends on everything that one establishes — the
        // picker seam, the window opening, the fixture PNG — and adds exactly
        // one thing: that the window gets out of the way when asked and comes
        // back afterwards. Its real subject is the JOIN; every part of the
        // placement arm is unit-tested and each part passes alone.
        Box::new(insert_image_place::TheInsertWindowStepsAside),
        // `OPERATOR_REQUESTS.md` O67, 2026-08-31. Beside the insert checks
        // because it is the third route to the same verb — the picker, the
        // dialog, and now a file dropped on the grid — and the one that has to
        // prove a POSITION was used rather than a default.
        Box::new(drop_onto_thumbnails::ADrawingDroppedOnTheThumbnails),
        // `OPERATOR_REQUESTS.md` O70, 2026-08-31. It reads the same
        // `canvas-selection` line the font-group check does and asks the one
        // question that line was extended for: WHICH index space did the click
        // land in? Both answers are `sel=1 level=Object`.
        Box::new(smart_select::AClickSelectsTheWholeDrawing),
        // Immediately after the descent, because it depends on everything that
        // check establishes and adds exactly one thing: that what you reached
        // can be edited. Reaching something and being unable to move it is a
        // worse state than not reaching it — the outline is a promise the
        // gesture then breaks.
        Box::new(form_leaf_move::AThingInsideAWrappedDrawingCanBeDragged),
        // …and one rung deeper again. Third of the three, in the order an
        // operator meets them: reach it, edit it, go inside it.
        Box::new(form_leaf_descend::TheLadderGoesAsDeepInsideAContainer),
        // The chain's last rung, and the one that leaves the geometry: text.
        Box::new(double_click_text::DoubleClickingATextBoxEditsTheText),
        // `OPERATOR_REQUESTS.md` O71, 2026-08-31. Beside the Smart-Selector
        // check because both are about what a plain click MEANS — that one in
        // Edit, this one in Read, which is the stance where the answer had
        // always been "text, or nothing".
        Box::new(read_image_copy::ReadModeCopiesAPicture),
        // The Properties panel's document half, wired 2026-08-19 after a
        // recorded blocker — "`pdfcer-core` exposes no /Info accessor" — turned
        // out to have cleared without the prose moving. Beside the page checks
        // because it is the other surface that edits the DOCUMENT rather than
        // a page's content.
        Box::new(properties_metadata::PropertiesMetadataRoundTrips),
        // ★ Two ribbon clicks and one trace line — cheap, no capture, no
        // canvas gesture, no keystroke — so its position is chosen for what a
        // reader wants adjacent rather than for cost.
        //
        // It sits here, among the checks that drive a real document, because
        // its precondition is one: `file.print` is gated on `doc.open`. It
        // must NOT move up among the chrome checks, which run without a
        // fixture.
        //
        // ★ It never presses the commit button, and no future edit may make it
        // do so. That button is the one control in the application that
        // consumes paper and cannot be undone; a harness that can start a
        // print job will eventually start one by accident. The module header
        // states what that costs and why the cost is worth paying.
        Box::new(print_dialog::PrintDialogReachesTheSpooler),
        // ★ Immediately after the spooler check, because the two are the two
        // halves of "does Print work": one asks whether the job reaches the
        // device, the other whether the window the operator drives it from is
        // usable. The gap between those halves is where four defects shipped —
        // see `print_layout`'s header.
        Box::new(print_layout::PrintDialogBodyDoesNotDeadlockItsScrollbars),
        // ★ Immediately after it, and the ORDER is load-bearing rather than
        // tidy. This check's every skip message defers to `print_dialog` for
        // the diagnosis — "the dialog never opened", "the spooler refused",
        // "the ribbon control is missing" are all its subject, not this one's.
        // Running it first means the reader of a failing run meets the
        // specific cause before the vaguer one, instead of reading a paper
        // check skip and having to go looking for why.
        //
        // ★ It never presses Properties…, and no future edit may make it do
        // so. That button opens a VENDOR DRIVER's own modal dialog: a nested
        // Win32 message loop whose layout pdfcer does not know, cannot publish
        // rects for, and cannot reliably dismiss — and one left standing
        // blocks the application's event loop, so a failed dismissal does not
        // fail this check, it hangs every check after it.
        //
        // ★ And it never presses commit, for the reason stated above.
        Box::new(print_paper::PrintPaperChangesThePlan),
        // ★ Beside it because it is the same shape — two ribbon clicks into a
        // dialog — and because both are checks whose subject is a control that
        // was drawn and did nothing.
        //
        // It launches with NO fixture, deliberately: `file.settings` is
        // application-scoped and must work with nothing open. That also makes
        // it the cheapest driving check in the suite, so a run whose ribbon
        // channel is broken says so here without paying for a render.
        Box::new(settings_theme::SettingsThemeTakesEffect),
        // ★ Directly after it, and for the same dependency reason it sits
        // after the markup checks: this one also begins by arming Rectangle and
        // dragging, so a run in which the four-link arm chain is broken should
        // report that as `markup_rectangle`'s failure and this one's last.
        //
        // Placed AFTER `save_copy` rather than before it, although it is the
        // cheaper of the two (one process, no file on disk): a shell that
        // cannot write what an operator authored is a worse finding than one
        // that cannot take it back, and a run is likelier to be read from the
        // top than from the bottom.
        Box::new(undo_redo::UndoRedoRoundTrip),
        // ★ Third of the two-process checks, and placed here for the same
        // dependency reason the two above it are: it ends by writing a file and
        // re-opening it, so a run in which writing itself is broken should
        // report that as `save_copy_round_trip`'s failure and this one's
        // second.
        //
        // It is the most expensive check in the suite by a small margin —
        // two launches, eight clicks, and a full rewrite of a document
        // performed synchronously inside one of them — and the most valuable
        // per second spent, because its subject is the only irreversible
        // operation the program has. Its fixture is **generated**, so unlike
        // every other driving check it does not consult `--pdf` and cannot be
        // aimed at a document that lacks the strings it scans for.
        Box::new(object_clipboard::CopyAndPastePageContent),
        Box::new(clipboard_text::CtrlCCopiesTextToTheOsClipboard),
        Box::new(select_filter::SelectFilterChangesWhatAClickHits),
        Box::new(scroll_input::ScrollingFarKeepsTheCanvasItsPointerInput),
        Box::new(max_zoom::TheZoomReadoutOpensTheMaximumZoomPopup),
        Box::new(deep_zoom::ZoomingPastThePixmapCeilingStillRenders),
        Box::new(deep_pan::PanningAtDeepZoomStaysWhereItWasPut),
        Box::new(zoom_keeps_place::ZoomingDoesNotThrowAwayWhereTheOperatorPanned),
        // ★ Its inverse. The climb above never rolls the wheel the other way, so
        // the DOWNWARD hand-over between the f32 scroll offset and the f64
        // anchor had never been driven. O26e.
        Box::new(zoom_out_keeps_place::ZoomingBackOutKeepsTheView),
        // ★ O28 and O29: a fit sets the scale AND places the view, and there
        // is a third mode. Pans into the pasteboard first, because the state
        // the request is about did not exist before O23.
        Box::new(fit_places_the_view::AFitCommandPutsThePageOnScreen),
        // ★ Immediately after it: same subject, opposite outcome, and a failure
        // in the sibling should be read first because this one builds on it.
        Box::new(fit_left_by_a_pan::APanLeavesTheFit),
        // ★ O30: the wheel as a page turn, from the status-bar toggle. Asserts
        // the DEFAULT is silent first, so a build that flipped unconditionally
        // could not pass — and that the control is absent where the choice
        // does not exist.
        Box::new(wheel_flips_pages::TheWheelTurnsPagesWhenTheOperatorAsksItTo),
        Box::new(zoom_gallery::ThePageStillRendersAtEveryDecadeOfZoom),
        Box::new(pan_refresh::PanningPastTheOverscanRendersTheNewArea),
        Box::new(resize::ResizeScalesAShape),
        // ★ Directly after `resize`, because it is that check plus one switch:
        // a failure in `resize_scales_a_shape` should be read first, since
        // every link it covers is in front of this one.
        Box::new(scale_switch::TheLineWeightSwitchReachesTheResize),
        Box::new(rotate::RotateHandleTurnsASelection),
        Box::new(shift_constrains::ShiftConstrainsAResize),
        Box::new(geometry_fields::GeometryFieldsResizeAShape),
        Box::new(restyle_text::RestylingSelectedTextReachesTheDocument),
        // ★ Directly after `restyle_text`, because it is that check plus a
        // popup: every link it covers — the sweep, the section, the read-back
        // stamp — is in front of this one, so a failure there should be read
        // first.
        Box::new(std14_face::TheFaceChooserOffersAFaceTheDocumentDoesNotContain),
        Box::new(font_group::TheFormatTabOffersFontControlsForSweptText),
        Box::new(multi_node::MultiNodeMoveMovesEveryPickedAnchor),
        Box::new(shape_preview::DraggingANodeBendsTheLine),
        Box::new(bezier_handle::BezierHandleDragChangesACurve),
        Box::new(tool_row::TheTextToolTypesOnOneClick),
        Box::new(tool_row::ThePointsToolShowsPointsOnOneClick),
        Box::new(tool_row::ShowPointsDrawsAnObjectsPointsWithoutDescending),
        Box::new(dropped_file::ADroppedImageReachesThePlacementWindow),
        Box::new(first_frame::TheFirstFrameNamesTheTools),
        Box::new(redaction::RedactionRemovesAndProvesIt),
        // ★ Beside the text-editing checks and owning its own fixture, like
        // `text_edit` and `redaction` above: its verdict is a LINE COUNT that
        // only `fixtures/paragraph.pdf` produces, so it takes no `--pdf`.
        Box::new(reflow::ReflowingAParagraphRewrapsIt),
        // ★ Directly after `redaction`, and before the two selection checks,
        // because it is the second most expensive check in the suite — it
        // launches the binary twice, for the same reason `save_copy` does — and
        // because its subject is the one this project exists for. A run in which
        // `save_copy` failed should be read first: every link from Ctrl+S
        // onwards is that check's, and this one has the whole text-edit path
        // stacked in front of them.
        // ★ The multi-document pair, 2026-08-20. Both SKIP without
        // `--second-pdf`, and the SKIP reason says why the same file cannot be
        // passed twice.
        Box::new(document_tabs::TwoDocumentsGetTwoTabs),
        Box::new(tab_reorder::DocumentTabsCanBeRearranged),
        Box::new(page_drag_between_documents::PageDraggedBetweenDocuments::COPY),
        Box::new(page_drag_between_documents::PageDraggedBetweenDocuments::MOVE),
        Box::new(about::AboutReportsTheBuild),
        // Beside About because it is its neighbour in every way that matters to
        // a run: it launches with NO document, it opens one window, and it
        // asserts on a trace rather than on pixels. It is also cheap.
        Box::new(shortcuts::ShortcutsReferenceIsLive),
        // ★ Immediately after the two checks whose SKIPs found the defect it
        // exists for. Both of those now maximise the window and are green;
        // this one deliberately does NOT, so the narrow band they used to
        // trip over is still driven by something every run.
        Box::new(band_scroll::ACommandTwoScrollStopsAwayIsStillReachable),
        Box::new(block_nav::ArrowKeysWalkBetweenBlocks),
        Box::new(dialog_windows::DialogsOpenInTheirOwnWindow),
        Box::new(draft_selection::ShiftArrowsSelectText),
        Box::new(bookmark_add::BookmarkCanBeWritten),
        Box::new(bookmark_edit::ABookmarkCanBeRenamedAndRemoved),
        Box::new(bookmark_move::ABookmarkCanBeDraggedAndABranchCollapsed),
        Box::new(attachments::AFileCanBeAttachedAndTakenBackOut),
        Box::new(comment_note::ANoteCanBeWrittenOntoAShape),
        // Last of the three new ones and the most expensive: it drives Insert
        // pages, the Forms panel and the Tab-order section in one session,
        // because the shape it registers does not exist in any fixture — pdfcer
        // makes it. See that module's header.
        Box::new(adopt_widget::AdoptWidgetPutsAFormControlBack),
        Box::new(add_text::AddTextTakesRealKeystrokes),
        Box::new(chords::EveryDeclaredChordDispatches),
        Box::new(text_annot::TextAnnotPlacesAndAuthors),
        Box::new(text_annot_focus::TextAnnotTakesTheKeyboardUnclicked),
        Box::new(text_box::TextBoxTakesAParagraph),
        Box::new(text_edit::TextEditPinsAnAlignedTail),
        Box::new(text_edit_real::TextEditOnARealDrawing),
        // After both, because it is the only driving check that does not touch
        // the ribbon band at all — it clicks mode segments and the page — and
        // because it is the slowest: it searches for a point with content
        // under it, and every candidate costs four clicks.
        Box::new(read_mode::ReadModeRefusesCanvasEdits),
        Box::new(text_selection::TextSelectionSweepsAndCopies),
        // ★ Directly after the sweep it depends on. This one asserts EXACT
        // character and box counts on its own committed fixture, where the
        // check above asserts liveness on whatever `--pdf` names — so a run in
        // which the sweep gesture is broken at all should report THAT here
        // first, and this one's more specific failure second.
        Box::new(rotated_text::RotatedTextSelectsAndCopiesAsOneLine),
        // Directly after it, and the order is a dependency rather than a
        // preference: this one *begins* by making a text selection, so a run
        // where the sweep itself is broken should report that as the sweep's
        // failure first and this one's SKIP second. It is also the longer of
        // the two — three ribbon clicks and a drag, one of which authors an
        // annotation into the open document (never onto disk: nothing here
        // saves).
        Box::new(text_markup::TextMarkupMarksASelection),
        // Directly after it, and again the order is a dependency rather than a
        // preference: this one does everything `text_markup` does and then some,
        // in a different mode and behind a tool that has to arm first. A run
        // where the marking path itself is broken should report that as
        // `text_markup`'s failure, and this one's second — because here the same
        // symptom has one more candidate cause (the tool), and a reader who has
        // already seen Review fail knows to ignore it.
        //
        // It is the longest driving check in the suite: five ribbon clicks
        // across three tabs, two drags, and one annotation authored into the
        // open document (never onto disk; nothing here saves).
        // ★ Last of the driving checks, and the slowest: it runs a real
        // recognition, which is a second in a release build. Placed after the
        // cheap ones so a run that is going to fail on something structural
        // fails before spending it.
        // ★★ O58's discharge, 2026-08-29. It runs AFTER the selection checks
        // deliberately: its first three phases are `field_menu`'s (place,
        // clear, select), so if those are broken this check should not be the
        // first thing to say so — it would name the clipboard for a selection
        // defect. Its own SKIP messages distinguish the two.
        // ★ O59's first item, before the clipboard pair: it is the one whose
        // failure is DESTRUCTIVE. The other two prove a capability works; this
        // proves one cannot happen.
        Box::new(cut_gate::CuttingARedactionMarkIsRefusedBeforeAnythingIsRemoved),
        // ★ O60 — redacting what is selected, the third marking route.
        Box::new(redact_selection::ASelectedObjectCanBeMarkedForRedaction),
        // ★ O61 — the document-safety disclosure.
        Box::new(reach_out::ADocumentThatPhonesHomeSaysSo),
        // ★ O62 — the rotation direction, which is one sign and invisible.
        Box::new(widget_rotate::TurningAFieldRightTurnsItRight),
        // ★ O59 item 2 — the page clipboard.
        Box::new(page_clipboard::PagesCanBeCopiedAndPasted),
        // ★ O59 item 3 — the bookmark clipboard, and the one operation in
        // this program Acrobat cannot do between two files at all.
        Box::new(bookmark_clipboard::ABookmarkSubtreeCanBeCopiedAndPasted),
        Box::new(field_clipboard::AFormFieldCanBeCopiedAndPastedBothWays),
        // ★★ Immediately after its sibling, and it is the MIRROR of it: same
        // gestures, opposite expectations, one environment variable apart. A
        // build that ignored the paste-order setting passes the first and fails
        // this one on its first assertion.
        Box::new(field_clipboard::TheAcrobatPasteOrderSwapsWhichChordDoesWhich),
        Box::new(form_selection::AClickInsideAFormSelectsWhatIsDrawnThere),
        // ★ Immediately after it, and the order is a dependency rather than a
        // preference: this check's second step is `form_selection`'s first
        // assertion — a click inside a form must select the leaf — and it
        // SKIPs rather than fails when that does not hold, so that a broken
        // deep hit test is reported once, by the check that owns it, instead of
        // twice with the second reading blaming the wrong file.
        Box::new(unshare_form::TheContextMenuGivesThisPageItsOwnCopyOfASharedForm),
        // ★★★ Its pair, and it must stay adjacent to it. The two press the same
        // row through the same five steps and differ only in the document they
        // open — `shared-across-two-pages.pdf` against `page-sized-form.pdf` —
        // so they are one behaviour's two halves, not two features. Reading the
        // suite output, an author who sees only one of them run has been told
        // that the other half of a branch is unmeasured, which is exactly the
        // state that let a check named `…_of_a_shared_form` sit on an unshared
        // fixture for a day.
        //
        // ★ It costs a second launch and a second window, deliberately: the
        // condition under test is a property of the open file, so it cannot be
        // reached by any further gesture within the first check's session.
        Box::new(unshare_form::TheUnshareDeclinesWhenNothingElseDrawsTheForm),
        // The two link checks, before OCR: they are two launches and two clicks
        // apiece against kilobyte fixtures, where the OCR checks below are a
        // minute each. A run that fails on something cheap should fail before
        // paying for something expensive.
        Box::new(off_page_marquee::ABandDraggedIntoTheMarginReachesAnObjectOffThePage),
        // Two launches and an Alt+F4, so it is placed with the other
        // multi-process checks rather than among the single-window ones.
        Box::new(page_display_pref::APageDisplayChoiceSurvivesACloseAndReachesANewDocument),
        Box::new(save_as::SaveAsRebindsTheDocument),
        // Last of the new group: it is TWO launches and it presses Alt+F4,
        // so a run that fails on something cheaper should fail first.
        Box::new(quit_unsaved::ClosingTheProgramAsksBeforeLosingUnsavedWork),
        Box::new(link_follow::ALinkGoesToThePageItNames),
        Box::new(link_follow::ALinkItCannotFollowSaysSo),
        Box::new(ocr::OcrRecognisesAPageAndTheDocumentKeepsIt),
        // The three about a run in progress. After the one-page check, because
        // a build in which recognition does not work at all should say so
        // before three checks spend a minute apiece observing it not working.
        Box::new(ocr_progress::OcrSaysHowFarItHasGot),
        Box::new(ocr_progress::StoppingOcrKeepsWhatItHasDone),
        Box::new(ocr_progress::CancellingOcrThrowsAwayWhatItHadDone),
        Box::new(text_tool::TextToolSelectsAndMarksInEdit),
        // Last, because it is the only check that TYPES. Everything above
        // either reads a trace or captures a window; this one presses a
        // chord, types a needle and presses Enter into a real foreground
        // window, so it costs the operator their focus for a few seconds.
        // A run that fails earlier should fail before paying that.
        // Cheap and non-destructive: two ribbon clicks, no canvas gesture, no
        // keystroke, and a window that changes nothing. Placed here rather than
        // among the first driving checks only because it depends on a raster
        // having landed, and everything above has already waited for one.
        Box::new(render_diagnostics::RenderDiagnosticsOpensItsReport),
        Box::new(find_bar::FindOpensAndFinds),
        // ★ Second to last among the driving checks, and the placement is a
        // property of what it does rather than of what it costs: it is the only
        // check that **cannot put the application back**. Read mode's exit is
        // `Ctrl+H` and this machine cannot inject keystrokes, so the session
        // ends with the chrome hidden. That harms nothing — every check launches
        // its own process and read mode is per-session by design — but a reader
        // scanning a run for the first failure should not meet a check whose
        // window looks broken in its artefacts before the ones whose windows
        // look ordinary.
        //
        // Cheap otherwise: two ribbon clicks, two captures, no canvas gesture
        // and no keystroke.
        Box::new(read_mode_chrome::ReadModeHidesTheChrome),
        Box::new(settings_headings::SettingsHeadingsLegible),
        // ★ LAST, and for a reason that is the mirror of the one above.
        //
        // This check WRITES `userdata/preferences.txt` beside the binary and
        // deliberately does not restore it — see `write_preference` on why
        // tidying up would hide the state the next check inherits. Every check
        // that runs after it would therefore start at whatever scale it left
        // behind, and a window at 1.8x is a window whose every coordinate
        // differs from what the others were written against.
        //
        // Putting it last makes that a property of the *file on disk* between
        // runs rather than a property of the *suite*, which is the same
        // distinction `delete_key`'s persisted-mode defect turned on. The file
        // is left holding the large scale on purpose: the next run's base
        // launch then has something to move away from, which is what makes its
        // `ui-scale` trace line meaningful rather than vacuous.
        Box::new(ui_scale::UiScaleResizesTheChrome),
    ]
}
