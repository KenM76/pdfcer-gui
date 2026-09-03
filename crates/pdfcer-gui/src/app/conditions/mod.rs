//! # `app::conditions` — what the shell may ask about the application
//!
//! One function. It publishes the set of **named conditions** the ribbon
//! evaluates every frame to decide whether a control is enabled and whether
//! it renders pressed.
//!
//! ## Why the vocabulary is names rather than closures
//!
//! `egui_shell` stores `Enable::When("doc.pages")` — a string — and asks this
//! set whether it is present. Data rather than a closure, because a name is
//! serializable (so an operator's customized manifest can reference it),
//! testable headlessly, and cannot capture state that would make a command's
//! availability depend on *when* it was registered.
//!
//! The cost of that choice lands here: every name is a **promise the
//! application has to keep**, and the only thing keeping the two halves in
//! step is `shell::commands`' `KNOWN` list and the test that walks it.
//!
//! ## Why this is its own file
//!
//! Split from `app/mod.rs` when that file crossed the 1,500-line gate for the
//! second time — the first split produced `app/dispatch.rs`. The seam is the
//! same shape as that one and just as real: `mod.rs` composes a frame,
//! `dispatch.rs` answers *what does this verb do*, and this file answers
//! *what is true right now*. They change for different reasons.
//!
//! ## ★ Two sources, one convention
//!
//! Conditions come from two different places and it matters that they arrive
//! the same way:
//!
//! * **application state** — `doc.open`, `doc.pages`, `selection.any`,
//!   `selection.bounds`, and the page-display and view-chrome pressed states,
//!   all read from `PdfcerApp` and the open document; and
//! * **`egui::Memory`** — the armed canvas tool and the armed region zoom,
//!   which is why this function takes an `egui::Context`.
//!
//! The second source is the reason this function grew a parameter. Three
//! separate pieces of work recorded that the hand tool and the region zoom had
//! no pressed state, and each declined to invent a mechanism for it — rightly.
//! The alternative was a shadow copy of the armed tool on `PdfcerApp` that the
//! canvas would have to remember to update, which puts the truth in two places
//! and fails as a ribbon that says Hand while the canvas selects: a
//! disagreement no test catches, because each half is self-consistent.

use crate::app::PdfcerApp;
use crate::app::state::Status;

/// **Which control renders pressed.** Split out 2026-08-31 under R2; see its
/// header for why the pressed conditions are a different subject from the
/// enable conditions above them.
mod armed;

impl PdfcerApp {
    /// The conditions the ribbon evaluates its predicates against.
    ///
    /// Rebuilt every frame because that is what it describes — the state
    /// *this* frame is drawn from. The set is **closed**, and the vocabulary
    /// is written down once in `crate::shell::commands`' `KNOWN` list rather
    /// than counted here — a count in prose drifts the moment a condition is
    /// added, and this sentence has already been wrong once for saying
    /// "five". That module has a test asserting no predicate names anything
    /// outside the list, so a typo in a manifest cannot silently produce a
    /// control that is disabled forever.
    ///
    /// # ★ `selection.any` is published from here, and only now
    ///
    /// It was deliberately absent while the selection lived in
    /// `egui::Memory`: this function has no `egui::Context`, so it could not
    /// have read the selection even if it wanted to, and publishing a
    /// condition it could not evaluate would have armed a **destructive**
    /// control that could not work — the inverse of the no-placeholders rule
    /// and the exact shape of defect D1.
    ///
    /// The selection now lives on [`state::OpenDoc`], so the answer is one
    /// field read. Two surfaces come alive with it, both of which the manifest
    /// has been carrying unpowered: the contextual **Format** tab
    /// (`visible_when: "selection.any"`, which is the appear-on-selection
    /// affordance `RIBBON_IA.md` §5.8 calls the single largest usability
    /// change) and the **Delete** inside it (`enabled_when` the same). One
    /// spelling, one source — see `shell::manifest::format::VISIBLE_WHEN`.
    ///
    /// **The Objects panel's focus is not a selection and must never satisfy
    /// this**, which is what `panels::PanelsState::focus`'s own test asserts
    /// through the enable machinery: a panel row being focused must not arm a
    /// destructive command, because the operator would have no way to tell
    /// which of two "selections" it was about to act on. This reads
    /// `doc.selection` and nothing else.
    /// `pub(super)` rather than private: this moved out of `app/mod.rs` and
    /// its three callers stayed. Deliberately NOT `pub` — nothing outside
    /// `app` may publish or read the condition set, because a second producer
    /// is how a control comes to be enabled by one rule and drawn by another.
    pub(super) fn conditions(&self, ctx: &egui::Context) -> egui_shell::commands::ConditionSet {
        let mut set = egui_shell::commands::ConditionSet::new();
        // ★ **More than one document is open**, and therefore something to
        // switch to. Set OUTSIDE the `Status::Open` arm below, deliberately: a
        // tab whose file failed to open is still a tab
        // (`crate::app::documents` §2), and an operator sitting on a damaged
        // file with three good ones behind it needs Ctrl+Tab to work more than
        // anybody.
        if self.document_count() > 1 {
            set.set("docs.multiple");
        }
        if let Status::Open(doc) = &self.status {
            set.set("doc.open");
            if !doc.pages.is_empty() {
                set.set("doc.pages");
                // ★★★ **THE ONE LINE**, and it was written as one line on
                // purpose a fortnight before it could be uncommented.
                //
                // `edit.form_push_button` is `enabled_when("forms.push_button_runnable")`
                // and nothing set that name for the life of the project, because a
                // button pdfcer placed ran nothing: `/A` was unauthorable by decision
                // 009 posture A. The catalog comment beside the command said the
                // greying was *"one line"* away from lifting; this is the line.
                //
                // `pdfcer-core` shipped `set_button_action` on 2026-08-30 (`Pass
                // 182.0`/`183.0`/`183.1`). The condition rides `doc.pages` because a
                // button needs a sheet to sit on and nothing more — the seven things
                // it can be set to are all writable into any document a field can be
                // placed in, so there is no narrower precondition to state.
                //
                // ★ Welded to `FormFieldKind::is_useful_once_placed` by a test, and
                // that weld is the mechanism: the ribbon asks by condition string and
                // `app::dispatch::forms` asks by predicate, and a build where the two
                // disagree is one where a greyed control still works by chord.
                set.set("forms.push_button_runnable");
            }
            if !doc.selection.is_empty() {
                set.set("selection.any");
            }
            // ★★★ **Something Delete and Properties can act on**, which is a
            // WIDER question than `selection.any` and has to be its own name.
            //
            // A form field is not in `SelectionState` at all — it lives in
            // `doc.selected_field`, because a `/Widget` is deliberately not an
            // annotation selection — so `selection.any` is false while a field
            // is selected and every control gated on it resolves disabled.
            //
            // ⇒ That was invisible while the only route to `format.delete` was
            // the Format tab, which is not drawn for a form selection. Giving
            // the canvas a `canvas.field` right-click menu on 2026-08-28 gave
            // the command a second door and the state became reachable: the
            // menu's Delete would have been greyed on a field the Delete KEY
            // removes.
            //
            // ★★ It is deliberately NOT a widening of `selection.any`. That
            // condition also decides whether the contextual **Format tab**
            // appears, and a form field has no font, no stroke and no fill for
            // that tab to offer — so widening it would draw a tab of controls
            // that cannot act on what is selected. Two questions, two names.
            if !doc.selection.is_empty() || doc.selected_field.is_some() {
                set.set("selection.actionable");
            }
            // ★★★ **Deleting what is selected would not be refused** — the
            // condition `format.delete` carries as its `visible_when`, so the
            // control is **absent** rather than greyed where the engine would
            // refuse it (R9).
            //
            // # ★★★ Why this exists, and it is a defect rather than a polish
            //
            // `EditSession::annotation_deletion_refusal` is a pure query whose
            // own doc comment names this call site by rule number, and until
            // 2026-08-29 **nothing in this shell called it**. On a certified or
            // encrypted drawing the Format tab's Delete, both canvas menus'
            // Delete and the Delete key were all live, and every press ended in
            // `actions::apply::vector_edit`'s `Err` arm — one line to the trace,
            // nothing to the operator. That is the day-before forms defect
            // (`deletion_refusal`, consulted by nothing) wearing a different
            // `/Subtype`.
            //
            // # ★★ It is a POSITIVE name for a negative fact, deliberately
            //
            // `Enable::When` and `Item::visible_when` both accept a leading `!`,
            // so `!selection.delete_refused` would compile and read backwards at
            // the one place it matters — a manifest an operator may edit. The
            // set is also *closed* (`shell::commands`' `KNOWN`), so a name that
            // has to be negated at every use is a name that will one day be used
            // un-negated by accident, and the symptom is a Delete that appears
            // only on the documents that refuse it.
            //
            // # ★★★ The default is TRUE, and that is the whole safety argument
            //
            // Set for every state except the two narrow cases below. In
            // particular it is set when **nothing** is selected and when a
            // *content* object is selected — because this condition answers
            // only the *refusal* question and must not silently become a
            // second, weaker spelling of `selection.actionable`.
            // `format.delete` keeps that condition as its `enabled_when`; this
            // one decides only whether it is drawn at all.
            //
            // ★ It is NOT set for a selected form field on a document whose
            // form structure is frozen, and that arm was missing until
            // 2026-08-29 — see the ladder below.
            //
            // ⇒ A control that is *drawn and refuses* is the defect being fixed.
            // A control that is *withheld where it would have worked* is a worse
            // one, because the operator has no gesture that reports it. So the
            // false answer this predicate must never give is `false`, and every
            // path that cannot prove a refusal leaves it `true`.
            //
            // # ★★ The form-field arm mirrors the dispatcher's ladder exactly
            //
            // `app::dispatch::format`'s `format.delete` arm checks
            // `doc.selected_field` FIRST and returns, so on the (reachable)
            // frame where a field and an annotation are both selected the
            // command deletes the *widget* and never consults the annotation at
            // all. Withholding the control there on the strength of the
            // annotation's gate would hide a Delete that works. `canvas::keys`'
            // Delete ladder has the same precedence, which is why it can be
            // stated once here.
            //
            // What that argument did NOT license, and what it was read as
            // licensing for a day, is asking the annotation question and then
            // giving up: the field arm has a gate of its own and it is
            // `EditSession::deletion_refusal`. Mirroring the ladder means
            // mirroring both rungs.
            //
            // # Cost
            //
            // Rebuilt every frame like the rest of this set, and that is sound
            // rather than tolerated: the query reads the signature census and
            // the trailer, mutates nothing, and core documents it as *"safe to
            // call every frame from a UI"*. The expensive half of this feature —
            // `annotation_deletion_preview`, which walks `/Annots` — is **not**
            // asked here; it is memoised on `(id, epoch)` in the panel.
            //
            // ★ One derivation per rung, four consumers each.
            // `panels::properties::annotdelete::gate` and
            // `panels::properties::formfield::refuses_delete` are each called
            // from here, from the panel that draws the sentence, from the
            // Delete key's ladder and from the dispatcher's arm — so a control
            // can never be withheld for one reason while the panel explains
            // another.
            //
            // ★★★ **A LADDER, and the `is_none()` guard it replaces was a gate
            // that was a no-op by construction.**
            //
            // It read `doc.selected_field.is_none() && annotdelete::…`. With a
            // field selected the first conjunct is FALSE, so `delete_refused`
            // was false and this condition was set **unconditionally for every
            // selected field on every document** — including the certified
            // fillable form that is the ordinary case. The comment above
            // defended that as *"this condition answers only the annotation
            // question"*, which was true and was the wrong shape: the condition
            // is `format.delete`'s `visible_when`, and `format.delete` deletes a
            // WIDGET when a field is selected, so a condition that answers only
            // the annotation question is answering about a verb that will not
            // run.
            //
            // ⇒ The two arms are the dispatcher's ladder, in its order. The
            // field arm asks the **forms** query
            // (`EditSession::deletion_refusal`, through
            // `panels::properties::formfield::refuses_delete`) and the
            // annotation arm asks the **annotation** one. They are not
            // interchangeable — see `refuses_delete`'s own doc, which argues it
            // at length — and the reason the ladder is stated here rather than
            // collapsed into one predicate is that `app::dispatch::format`'s
            // `format.delete` arm and `canvas::keys`' Delete ladder both check
            // `doc.selected_field` FIRST and return. This mirrors them exactly,
            // so the drawn control and the arm behind it can never be answering
            // about different objects.
            let delete_refused = if doc.selected_field.is_some() {
                crate::panels::properties::formfield::refuses_delete(doc)
            } else {
                crate::panels::properties::annotdelete::refuses_selected(doc)
            };
            if !delete_refused {
                set.set("selection.delete_permitted");
            }
            // ★★★ **`selection.cut_permitted` — default TRUE, like its
            // neighbour above, and cleared only for the handful of things the
            // clipboard cannot carry.**
            //
            // The engine asked for this by name: *"do not offer Cut as enabled
            // and let it fail. A copy of something pdfcer cannot carry costs
            // nothing — the original stays. A cut of the same thing is a
            // deletion wearing a clipboard's clothes."*
            //
            // ★ It is NOT a refinement of `selection.delete_permitted`, and the
            // two disagree in both directions. A redaction mark can be
            // **deleted** and cannot be **cut**: deleting it removes a pending
            // operation, which is a thing an operator may want, while cutting it
            // would put it on a clipboard that could arm it somewhere else. And
            // a locked annotation can be neither. Two questions, two names.
            //
            // ★★ Cheap by construction — one dictionary read — because this is
            // rebuilt every frame and the honest oracle (`copy_selection`)
            // decomposes the page. `canvas::cutgate`'s header carries the
            // measurement and the reason the mirror is permissive.
            if crate::canvas::cutgate::blocker(doc).is_none() {
                set.set("selection.cut_permitted");
            }
            // ★★ **Something selected on this page lives inside a form
            // XObject**, so `format.select_form` has a container to offer.
            //
            // # Why this is a refinement of `selection.any` and still its own name
            //
            // Every selection that satisfies this satisfies `selection.any`
            // too, which is unusual here — `selection.bounds` deliberately is
            // *not* a refinement, and its docs say so. The distinction still
            // has to be its own condition, because it answers a question the
            // other cannot: *is there a container to select?* A page with no
            // forms at all can never satisfy it, and on such a page the
            // control is greyed for its whole life, correctly.
            //
            // # The current page only
            //
            // A selection can span pages (`Selection` carries one), but the
            // container act resolves through the **decomposition**, and this
            // shell decomposes exactly one page — `ObjectModelProvider` returns
            // nothing for any other. Publishing the condition for a leaf on
            // another page would light a control whose arm must then decline,
            // which is the disagreement `selection.bounds` exists to prevent
            // for zoom-to-selection. Asked here in the same words the arm asks
            // it in.
            if !doc
                .selection
                .leaf_indices_on(doc.view.page_index)
                .is_empty()
            {
                set.set("selection.in_form");
            }
            // ★ **The two conditions this function used to say were deliberately
            // absent**, and the comment they replace is worth quoting rather
            // than deleting, because it was right when it was written:
            //
            // > `undo.available` and `redo.available` are still deliberately
            // > absent: there is no undo stack to report on yet. Setting them
            // > would arm controls that cannot work. They arrive with their
            // > subsystem.
            //
            // The subsystem was always there — `EditSession` has carried one
            // command log, 44 `CommandKind` variants and a depth bounded at 256
            // since long before this shell existed. What was absent was the
            // shell's half: `edit.undo` and `edit.redo` were registered, on the
            // quick-access toolbar, bound to three chords, and had **no dispatch
            // arm**. Publishing the conditions then would have lit two controls
            // that traced `command-unimplemented` — which is the "no
            // placeholders" rule broken in the most expensive direction, since
            // an operator who believes undo works edits differently from one who
            // knows it does not.
            //
            // Both halves landed together, and this is the order that mattered:
            // the arm first (`crate::app::actions::apply`'s `history_step`), the
            // conditions second. A control is armed only after the verb behind
            // it exists.
            //
            // # Why these ask the ENGINE rather than counting anything here
            //
            // `can_undo`/`can_redo` are one field read each on the session that
            // owns the log. A shell-side count — "how many mutating actions have
            // I applied?" — would be a second copy of the truth, and it would be
            // wrong in three ways the engine's answer is not: the log is
            // **bounded at 256**, so the count and the log diverge on a long
            // session; a no-op edit records no command at all (setting a field to
            // the value it already has); and `EditSession::commit` **clears the
            // redo stack** whenever a new command is recorded, which nothing on
            // this side would know to mirror.
            //
            // The same two predicates answer the worded declines
            // (`crate::app::status::decline::History`), so the greyed control and
            // the sentence in the bar cannot come from different questions.
            if doc.session.can_undo() {
                set.set("undo.available");
            }
            if doc.session.can_redo() {
                set.set("redo.available");
            }
            // ★ **A live text selection** — the operand the three Text markup
            // commands act on, and the condition that keeps them from being
            // controls that do nothing on almost every press.
            //
            // # It is NOT a refinement of `selection.any`, and confusing them
            // would grey the controls exactly where they work
            //
            // `selection.any` is the **object** selection — page content, the
            // thing Edit's marquee builds. This is the **text** selection, and
            // the two are mutually exclusive by construction: a press means text
            // only when the mode cannot select content
            // (`canvas::textsel::takes_the_press`), so in every mode at most one
            // of these two conditions can ever be set. A predicate written as
            // `selection.any` on a text-markup command would therefore be false
            // in Review — the one mode where marking text works — and true in
            // Edit, where it cannot.
            //
            // # Why `live`, and why the same question the command asks
            //
            // A selection records the revision it was resolved against
            // (`canvas::textsel` §7), and after an edit its stored boxes may sit
            // over different glyphs. `markup::text::mark` refuses a stale one
            // rather than authoring a `/QuadPoints` annotation over
            // possibly-wrong words, so the condition asks the *same* question —
            // otherwise the control would be live at exactly the moment pressing
            // it declines, which is the disagreement `selection.bounds` was
            // added to prevent for zoom-to-selection.
            //
            // Note the visible consequence, which is deliberate: authoring a
            // text markup is itself an edit, so the selection that authored it
            // is stale on the next frame and these three controls **grey
            // themselves** immediately afterwards. That reads as the operator's
            // work being finished, and it is honest — marking the same words a
            // second way needs a second sweep.
            if doc
                .text_selection
                .as_ref()
                .is_some_and(|s| s.live(doc.edit_epoch))
            {
                set.set("selection.text");
            }
            // ★★★ **The Format tab has a subject** — either kind of
            // selection, and it is deliberately NOT a synonym for either.
            //
            // # Why a third name, when two already exist
            //
            // The contextual Format tab appears *"only while something is
            // selected"* (`RIBBON_IA.md` §5.8), and since 2026-08-27 it holds
            // two kinds of control: the Selection group, which acts on a page
            // **object**, and the Font group, which acts on a swept **text
            // range**. Those are the two conditions immediately above, and
            // they are different index spaces — `selection.any` is a
            // paint-order index, `selection.text` is a run range — so neither
            // one of them is the tab's condition. Spelling the tab as
            // `selection.any` means sweeping text in Edit restyles nothing
            // because the tab carrying the controls never appeared; spelling
            // it as `selection.text` loses the Delete the tab has carried
            // since it shipped.
            //
            // ★ The expression language is deliberately one condition name
            // with an optional leading `!` (`egui_shell::commands::Enable`'s
            // own docs: *"a grammar in a string is a parser and a parser is a
            // thing that has its own bugs"*), so an `A || B` predicate is
            // published as a **named fact** rather than assembled in a
            // manifest. That is the right shape anyway: what the tab is asking
            // is not *"is A or B true"* but *"is there anything for me to be
            // about"*, and this is that question's name.
            //
            // # It is a union, not a refinement, and the two can BOTH hold
            //
            // `selection.text`'s own note above says the object and text
            // selections are *"mutually exclusive by construction"*, and that
            // sentence was written before the text tool could be armed in Edit
            // — `canvas::textsel::takes_the_press` now answers true for an
            // armed text tool in **any** mode, so an operator who clicks an
            // object and then presses T and sweeps has both. This condition is
            // correct either way, which is the point of publishing the
            // question rather than the operands.
            if !doc.selection.is_empty() || set.is_set("selection.text") {
                set.set("selection.formattable");
            }
            // ★ `selection.bounds` is NOT `selection.any`, and the gap
            // between them is a real state rather than a defensive check.
            //
            // A selection is an identity — page, object, subpath, node —
            // and identities can outlive the box they described: the
            // selection may name an object on a page that is no longer
            // shown, or one whose index an edit renumbered. `selection.any`
            // is then true and there is nothing to frame.
            //
            // Zoom-to-selection is the one command where that difference is
            // visible, because framing "nothing" is not a no-op — it is a
            // jump to the origin at some arbitrary scale, which looks
            // exactly like a bug and loses the operator's place. So the
            // control greys instead, and it asks the same function the
            // grips are laid out on, so what greys and what is drawn can
            // never disagree.
            if crate::canvas::zoom::can_zoom_to_selection(doc) {
                set.set("selection.bounds");
            }
            // ★ **The page-display radio's pressed position.**
            //
            // `egui_shell::ribbon::selected_condition` is the framework's
            // convention for "this command is currently ON", and
            // `render_command` reads it to draw the button pressed. Without
            // this line View ▸ Page display is four buttons with no indication
            // of which one you are in — which for a radio is not a cosmetic
            // gap, it is the control's entire state.
            //
            // Exactly one is ever set, because `view.display` is one enum
            // value and `page_display_command` is a total function over it.
            // That is what makes it a radio rather than four toggles, and it
            // is asserted from the registry side by
            // `shell::commands::tests::every_page_display_mode_has_a_registered_command`.
            set.set(egui_shell::ribbon::selected_condition(
                crate::shell::commands::page_display_command(doc.view.display),
            ));
            // ★ **The three View ▸ Display toggles' pressed state.**
            //
            // Between zero and three of these are set, where exactly one
            // page-display condition above always is — which is the whole
            // difference between three switches and one three-position
            // control, expressed in the conditions rather than in the drawing.
            //
            // Rulers, grid and guides live on `doc.view`; the hand tool and
            // the armed region zoom live in `egui::Memory` and are published
            // below. Both routes end in the same `selected_condition`, which
            // is what kept a second mechanism from being invented for either.
            for &chrome in crate::app::actions::ViewChrome::ALL {
                if chrome.read(&doc.view) {
                    set.set(egui_shell::ribbon::selected_condition(
                        crate::shell::commands::chrome_command(chrome),
                    ));
                }
            }
            // ★ **Is there a circle fit waiting to be committed?** — the one
            // condition on this list that is about a *gesture in progress*
            // rather than about the document or the view.
            //
            // `measure.finish` is the ribbon half of the radius/diameter
            // tool's ending. That gesture has no natural end (see
            // `canvas::measure::MeasureKind::Circular`), so the operator
            // supplies one — and a Finish that were always enabled would be a
            // control that does nothing on almost every press, which P3
            // forbids and which is the placeholder shape this project refuses.
            //
            // # ★ Why it is INSIDE the `Status::Open` arm when the armed-tool
            // conditions below are deliberately outside it
            //
            // Those publish *"which tool you are in"*, which is true of the
            // application and survives closing a document — a ribbon that
            // forgot your tool the moment you closed a file would be reporting
            // something untrue about itself. This one publishes *"there is a
            // pick set on a page that is ready to become a dimension"*, and
            // the action it leads to names that page. With no document open
            // there is no page for it to name: the pick set would still be
            // sitting in `egui::Memory`, the control would be live, and
            // pressing it would raise a `CommitDimension` against a document
            // that is not there. Two different kinds of fact, so two different
            // scopes — and this is the one place in this function where that
            // distinction has had to be drawn.
            //
            // Costs one memory lookup per frame with nothing armed, which is
            // what `canvas::measure::finishable` reduces to when the tool is
            // not the circular one.
            if crate::canvas::measure::finishable(ctx) {
                set.set("measure.finishable");
            }
            // ★ …and the same fact for the two **vertex markup** tools, which
            // have the same problem and were given the same answer.
            //
            // PolyLine and Polygon are runs of clicks with no natural end, so
            // `markup.finish` is their ribbon ending — and a Finish that were
            // always enabled would be the same control-that-does-nothing P3
            // forbids. Everything the paragraph above says about scope applies
            // here unchanged and for the identical reason: this publishes *"there
            // is a run on a page that is ready to become an annotation"*, and the
            // action it leads to names that page, so with no document open there
            // is no page for it to name. Inside the `Status::Open` arm,
            // therefore, beside its twin.
            //
            // ★ It is also where the polygon/polyline difference reaches the
            // operator: `finishable` asks `markup::action`, which needs **three**
            // vertices for a polygon and two for a polyline, so after two clicks
            // this control is live for one tool and greyed for the other — the
            // rule stated where they can see it before pressing anything, rather
            // than as a refusal after.
            //
            // Costs one memory lookup per frame with nothing armed, which is what
            // `markup::vertex::finishable` reduces to when the armed tool is not
            // one of the two.
            if crate::canvas::markup::vertex::finishable(ctx) {
                set.set("markup.finishable");
            }
        }

        // ★★★ **This mode may change page content**, and it is the only
        // condition here that describes the MODE rather than the document, the
        // view, a gesture or an armed tool.
        //
        // # What it is for, and why it is visibility rather than enablement
        //
        // The Format tab's **Font** group restyles a swept text range, which
        // is an edit to page content. Read and Review cannot make one — the
        // mode taxonomy says so, `Capabilities::edit_content` is where it says
        // it, and `canvas::gesture::press_kind` already enforces it on the
        // canvas. R9 then decides how the ribbon must show that: *an
        // unavailable **capability** renders nothing; greying is reserved for
        // temporarily unavailable and is always explained on hover.* A mode
        // that cannot edit content has not temporarily mislaid the ability —
        // it does not have it — so the group is **absent** there, which is a
        // `visible_when` on each of its items and not an `enabled_when`.
        //
        // The greying is the other half and it is a different question:
        // `selection.text` greys those same controls inside Edit when there is
        // no swept range, with a tooltip that says how to get one. Two
        // conditions, two rules, one group — and it is the second of them that
        // makes the text tool discoverable at all, because a greyed control an
        // operator can hover is a control that can explain itself.
        //
        // # ★ Outside the `Status::Open` arm, with the armed-tool conditions
        //
        // For the reason stated below them: a mode is a property of the
        // application and survives closing a document. It is safe here because
        // the tab carrying the group is itself gated on
        // `selection.formattable`, which cannot hold with nothing open — so
        // publishing this with no document cannot draw anything.
        //
        // # Why it asks the ribbon rather than `self.modes`
        //
        // Because [`Self::capabilities`] does, and it says why: the ribbon is
        // where the operator's click lands and `self.modes` catches up later
        // in the same frame. A second derivation here would put the Font
        // group one frame behind the mode selector on exactly the frame a
        // stray click is most likely.
        if self.capabilities().edit_content {
            set.set("mode.edit_content");
        }
        // ★★ **Every pressed state, in one place** — `app::conditions::armed`,
        // split out 2026-08-31 under R2.
        //
        // It answers a different question from everything above it: not *"may
        // this control be pressed?"* but *"is it already in the state it
        // names?"* — a different source (`egui::Context` rather than `&self`),
        // a different scope (outside `Status::Open`, because an armed tool
        // survives closing a file) and a different shape. Its header carries
        // the argument, and the defect it is the home of: adding a tool is five
        // changes and the fifth has no unit test to remind you.
        self.armed_conditions(ctx, &mut set);

        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **`undo.available` and `redo.available` follow the session, in both
    /// directions, through the real verbs.**
    ///
    /// # The defect this exists for
    ///
    /// This function published neither for the whole life of the project, with a
    /// comment saying they were "deliberately absent". The obvious way to land
    /// them — set them beside `doc.open` and move on — arms both controls
    /// permanently, and a build that did that is **indistinguishable from a
    /// correct one** in every scenario that presses undo *after* an edit. The
    /// first two assertions here are the only ones that tell the two apart.
    ///
    /// # Why it drives the real actions rather than setting fields
    ///
    /// Because the thing under test is a **join**, and this crate has been bitten
    /// by exactly that before: `every_armable_tool_kind_reports_a_pressed_state`
    /// exists because Phase 7 shipped a measure tool with four passing unit tests
    /// and no `conditions` call site, so the button never lit up. A test that set
    /// a boolean and read a condition would prove that `set.set` works.
    ///
    /// So the log is filled by [`crate::app::actions::Action::CommitMarkup`] going
    /// through `vector_edit` and emptied by [`crate::app::actions::Action::Undo`]
    /// going through the same funnel — the two paths an operator's gesture takes
    /// — and the conditions are read through the **registered commands'** own
    /// predicates, so a rename of either condition string fails here rather than
    /// silently greying a control forever.
    #[test]
    fn the_history_conditions_follow_the_session() {
        use crate::app::actions::Action;
        use crate::canvas::markup::{Geometry, MarkupKind};

        let ctx = egui::Context::default();
        let mut app = crate::app::tests::opened();
        let mut reg = egui_shell::CommandRegistry::new();
        crate::shell::commands::register(&mut reg);
        let live = |app: &PdfcerApp, ctx: &egui::Context, id: &str| {
            reg.get(id)
                .expect("registered")
                .is_enabled(&app.conditions(ctx))
        };

        // ★ A freshly opened document has an empty command log, so BOTH controls
        // are greyed. This is the assertion an unconditional publication fails,
        // and it is the reason the pair below is not enough on its own.
        assert!(
            !live(&app, &ctx, "edit.undo"),
            "nothing has been changed, so Undo must be greyed — a control armed here is one that \
             can only decline"
        );
        assert!(!live(&app, &ctx, "edit.redo"), "and so must Redo");

        // One real edit through the real funnel.
        app.apply_actions(
            vec![Action::CommitMarkup {
                pen: crate::canvas::markup::pen::Pen::default(),
                page: 0,
                kind: MarkupKind::Rectangle,
                geometry: Geometry::Band {
                    start: (100.0, 100.0),
                    end: (200.0, 160.0),
                },
            }],
            1.0,
        );
        assert!(
            live(&app, &ctx, "edit.undo"),
            "an annotation was authored, so there is something to take back"
        );
        assert!(
            !live(&app, &ctx, "edit.redo"),
            "nothing has been undone yet — an edit does not fill the redo stack"
        );

        // …and the undo swaps them, which is the half that proves neither is
        // latched. `EditSession::commit` clears the redo stack on a new command,
        // so a build that never re-read `can_redo` would pass the line above and
        // fail this one.
        app.apply_actions(vec![Action::Undo], 1.0);
        assert!(
            !live(&app, &ctx, "edit.undo"),
            "the only command on the log has been taken back"
        );
        assert!(
            live(&app, &ctx, "edit.redo"),
            "…and is now on the redo stack, which is what makes Redo pressable"
        );

        // Round trip: redoing empties the redo stack and refills the undo log.
        app.apply_actions(vec![Action::Redo], 1.0);
        assert!(live(&app, &ctx, "edit.undo"));
        assert!(!live(&app, &ctx, "edit.redo"));
    }

    /// ★ **Every armable tool kind reports a pressed state** — asserted over
    /// `ALL` for both families rather than over a list written here.
    ///
    /// # The defect this exists for, which shipped
    ///
    /// `app::conditions` published the armed **markup** kind and did not
    /// publish the armed **measure** kind. Phase 7 had `CanvasTool::Measure`,
    /// `arm_measure`, `measure_command` and a dispatch arm using its inverse —
    /// all four with passing unit tests — so Linear armed the tool, placed a
    /// dimension the engine accepted, and the button never lit up. It was found
    /// by `ui-verify` driving the real window, because the missing link was a
    /// **call site**, and no unit test observed two adjacent links being
    /// connected.
    ///
    /// Iterating `MarkupKind::ALL` and `MeasureKind::ALL` is what stops the
    /// same omission recurring: a fifth kind added to either enum with no
    /// `selected_condition` fails here rather than shipping as a control that
    /// arms without looking armed. A list of ids spelled out in this test would
    /// have to be remembered, which is the thing that was not.
    #[test]
    fn every_armable_tool_kind_reports_a_pressed_state() {
        use crate::canvas::markup::MarkupKind;
        use crate::canvas::measure::MeasureKind;
        use crate::canvas::tool::{self, CanvasTool};

        let app = PdfcerApp::new();
        let ctx = egui::Context::default();

        for &kind in MarkupKind::ALL {
            let id = crate::shell::commands::markup_command(kind);
            let cond = egui_shell::ribbon::selected_condition(id);
            tool::select(&ctx, CanvasTool::Select);
            assert!(
                !app.conditions(&ctx).is_set(&cond),
                "`{id}` must not read pressed while the select tool is armed"
            );
            tool::select(&ctx, CanvasTool::Markup(kind));
            assert!(
                app.conditions(&ctx).is_set(&cond),
                "`{id}` names {kind:?}, which is armed, and the ribbon does not say so"
            );
        }

        for &kind in MeasureKind::ALL {
            let id = crate::shell::commands::measure_command(kind);
            let cond = egui_shell::ribbon::selected_condition(id);
            tool::select(&ctx, CanvasTool::Select);
            assert!(
                !app.conditions(&ctx).is_set(&cond),
                "`{id}` must not read pressed while the select tool is armed"
            );
            tool::select(&ctx, CanvasTool::Measure(kind));
            assert!(
                app.conditions(&ctx).is_set(&cond),
                "`{id}` names {kind:?}, which is armed, and the ribbon does not say so"
            );
        }

        // ★ …and the two tools that carry **no** kind, which is why they cannot
        // be reached by walking an `ALL`. They are the ones this test's own
        // mechanism would silently miss, so they are named — and naming them is
        // exactly the "list of ids that has to be remembered" this test's header
        // warns about, which is why the warning is narrowed rather than ignored:
        // walk the kinds where kinds exist, and enumerate the kindless ones,
        // because there is nothing else to enumerate them from.
        //
        // The text tool is the more exposed of the pair. Arming it changes the
        // cursor and nothing else on the canvas, so a missing pressed state
        // would leave an operator with no evidence at all of the mode they are
        // in — where a hand at least shows a grab cursor and moves the page.
        for (tool, id) in [
            (CanvasTool::Hand, "view.tool_hand"),
            (CanvasTool::Text, "view.tool_text"),
        ] {
            let cond = egui_shell::ribbon::selected_condition(id);
            tool::select(&ctx, CanvasTool::Select);
            assert!(
                !app.conditions(&ctx).is_set(&cond),
                "`{id}` must not read pressed while the select tool is armed"
            );
            tool::select(&ctx, tool);
            assert!(
                app.conditions(&ctx).is_set(&cond),
                "`{id}` names {tool:?}, which is armed, and the ribbon does not say so"
            );
            // …and arming it must not leave the OTHER kindless tool pressed,
            // which is the property that makes them behave as a radio without
            // anything enforcing it — the same payoff the kind-carrying enum
            // gives the two families above.
            let other = if tool == CanvasTool::Hand {
                "view.tool_text"
            } else {
                "view.tool_hand"
            };
            assert!(
                !app.conditions(&ctx)
                    .is_set(&egui_shell::ribbon::selected_condition(other)),
                "arming {tool:?} must not leave `{other}` pressed"
            );
        }

        // …and exactly one is pressed at a time, which is the payoff of the
        // kind-carrying enum shape: a tool that could be two kinds at once is
        // unrepresentable, so two pressed buttons are too.
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Linear));
        let set = app.conditions(&ctx);
        let pressed = MeasureKind::ALL
            .iter()
            .chain(std::iter::empty())
            .filter(|&&k| {
                set.is_set(&egui_shell::ribbon::selected_condition(
                    crate::shell::commands::measure_command(k),
                ))
            })
            .count();
        assert_eq!(pressed, 1, "exactly one measure control renders pressed");
        for &kind in MarkupKind::ALL {
            assert!(
                !set.is_set(&egui_shell::ribbon::selected_condition(
                    crate::shell::commands::markup_command(kind)
                )),
                "arming a measure tool must not leave a markup control pressed"
            );
        }
    }

    /// ★ **The three Text markup controls are live exactly when there is a
    /// live text selection**, and are asserted through the **registry's own
    /// enable machinery** rather than by reading the condition name.
    ///
    /// Reading `set.is_set("selection.text")` would assert that this function
    /// agrees with itself. What matters is whether the *control* comes alive,
    /// which is the registration's predicate and this function's publication
    /// joined — the same join `every_armable_tool_kind_reports_a_pressed_state`
    /// exists for, and the same join `ui-verify` had to find in a running window
    /// when the measure tools armed without lighting up.
    ///
    /// Three states, and the third is the one a build would plausibly get wrong:
    /// a selection made *before* an edit is not an operand, because its recorded
    /// boxes may now sit over other glyphs — and a control that is live while
    /// the press would decline is the disagreement `selection.bounds` was
    /// invented to prevent one command over.
    #[test]
    fn the_text_markup_controls_need_a_live_text_selection() {
        use crate::app::tests::opened;
        use crate::canvas::markup::text::TextMarkKind;
        use crate::canvas::textsel::TextSelection;
        use pdfcer_core::annot_author::Quad;
        use pdfcer_core::page_tree::Rect as PageRect;

        let ctx = egui::Context::default();
        let mut app = opened();
        let ids: Vec<&str> = TextMarkKind::ALL
            .iter()
            .map(|&k| crate::shell::commands::text_mark_command(k))
            .collect();
        let mut reg = egui_shell::CommandRegistry::new();
        crate::shell::commands::register(&mut reg);

        let live = |app: &PdfcerApp, ctx: &egui::Context, id: &str| {
            reg.get(id)
                .expect("registered")
                .is_enabled(&app.conditions(ctx))
        };

        for id in &ids {
            assert!(
                !live(&app, &ctx, id),
                "`{id}` must be greyed with nothing selected — it would do nothing"
            );
        }

        let Status::Open(doc) = &mut app.status else {
            unreachable!("`opened` opens a document")
        };
        let epoch = doc.edit_epoch;
        doc.text_selection = Some(TextSelection::for_test(
            0,
            epoch,
            vec![Quad::from_rect(PageRect::from_corners(
                72.0, 700.0, 300.0, 710.0,
            ))],
        ));
        for id in &ids {
            assert!(
                live(&app, &ctx, id),
                "`{id}` acts on the text selection there now is"
            );
        }

        // One edit later the same selection is not an operand.
        let Status::Open(doc) = &mut app.status else {
            unreachable!()
        };
        doc.edit_epoch = epoch.wrapping_add(1);
        for id in &ids {
            assert!(
                !live(&app, &ctx, id),
                "`{id}` must not offer to mark boxes recorded against an older revision"
            );
        }
    }

    /// ★★★ **`selection.formattable` is the UNION, and the two halves are
    /// asserted separately because each on its own is a shipped defect.**
    ///
    /// It is the contextual Format tab's `visible_when`, and the tab now
    /// carries controls for two unrelated kinds of selection:
    ///
    /// * spelled `selection.any` — the object selection — a text sweep would
    ///   raise **no tab**, so the Font group could not be reached at all and
    ///   `format_text` would be a capability with no surface. That is what
    ///   shipped between the panel landing and this condition existing;
    /// * spelled `selection.text`, the tab would vanish on an ordinary object
    ///   selection, taking the Delete it has carried since it shipped with it.
    ///
    /// So the assertion is a truth table rather than a pair of positives, and
    /// the **fourth row** — both at once — is the one that could not happen
    /// when `selection.text`'s own note was written (*"the two are mutually
    /// exclusive by construction"*) and can now: `takes_the_press` answers true
    /// for an armed text tool in **any** mode, so an operator who clicks an
    /// object and then presses `T` and sweeps has both.
    #[test]
    fn the_formattable_condition_is_the_union_of_the_two_selections() {
        use crate::app::tests::{opened, select_object};
        use crate::canvas::textsel::TextSelection;
        use pdfcer_core::annot_author::Quad;
        use pdfcer_core::page_tree::Rect as PageRect;

        let ctx = egui::Context::default();
        let mut app = opened();

        assert!(
            !app.conditions(&ctx).is_set("selection.formattable"),
            "nothing is selected, so the Format tab has no subject and must not appear"
        );

        // 1. An object, and no sweep.
        select_object(&mut app, 0, false);
        assert!(app.conditions(&ctx).is_set("selection.any"));
        assert!(!app.conditions(&ctx).is_set("selection.text"));
        assert!(
            app.conditions(&ctx).is_set("selection.formattable"),
            "an object selection is a subject — this is the Delete the tab has always had"
        );

        // 2. …and a sweep on top of it: BOTH, which used to be unrepresentable.
        let epoch = {
            let Status::Open(doc) = &mut app.status else {
                unreachable!("`opened` opens a document")
            };
            let epoch = doc.edit_epoch;
            doc.text_selection = Some(TextSelection::for_test(
                0,
                epoch,
                vec![Quad::from_rect(PageRect::from_corners(
                    72.0, 700.0, 300.0, 710.0,
                ))],
            ));
            epoch
        };
        assert!(app.conditions(&ctx).is_set("selection.any"));
        assert!(app.conditions(&ctx).is_set("selection.text"));
        assert!(app.conditions(&ctx).is_set("selection.formattable"));

        // 3. A sweep alone — the state the Font group exists for, and the one
        //    a `selection.any` spelling would have shown no tab for.
        {
            let Status::Open(doc) = &mut app.status else {
                unreachable!()
            };
            doc.selection.clear();
        }
        assert!(!app.conditions(&ctx).is_set("selection.any"));
        assert!(app.conditions(&ctx).is_set("selection.text"));
        assert!(
            app.conditions(&ctx).is_set("selection.formattable"),
            "a swept range is a subject: it is what the Font group acts on"
        );

        // 4. …and a STALE sweep is not. `selection.text` refuses a selection
        //    recorded against an older revision, and the union must not
        //    resurrect it — a tab appearing over boxes that may now sit on
        //    different glyphs is worse than no tab.
        {
            let Status::Open(doc) = &mut app.status else {
                unreachable!()
            };
            doc.edit_epoch = epoch.wrapping_add(1);
        }
        assert!(!app.conditions(&ctx).is_set("selection.text"));
        assert!(
            !app.conditions(&ctx).is_set("selection.formattable"),
            "a stale sweep is not an operand, so it is not a subject either"
        );
    }

    /// ★★★ **The Font group's five commands are drawn in Edit and ABSENT in
    /// Read and Review**, which is R9 split across two conditions.
    ///
    /// `mode.edit_content` is **visibility** — a mode that cannot change page
    /// content does not have a mislaid ability to restyle text, it does not
    /// have the ability, so the controls render nothing. `selection.text` is
    /// **enablement** — inside Edit the capability is present and only the
    /// operand is missing, which greys and explains itself on hover.
    ///
    /// # ★★ Why both are needed, stated as the two one-condition builds
    ///
    /// With only `selection.text`, sweeping text in **Read** — which Read does
    /// with the plain select tool, because copying is not authoring — would
    /// draw an enabled Bold that the mode gate must then refuse. With only
    /// `mode.edit_content`, Edit would draw an enabled Bold with nothing
    /// swept: a control that does nothing on almost every press, which is the
    /// placeholder shape P3 forbids.
    ///
    /// ★ Asserted through `Capabilities::for_mode` and the shipped manifest,
    /// not through a hand-made `Capabilities` value, so a mode taxonomy edit
    /// that gave Read `edit_content` fails here as well as wherever else it is
    /// wrong.
    #[test]
    fn the_font_groups_visibility_follows_the_mode_and_its_enablement_the_sweep() {
        use crate::app::tests::opened;
        use crate::canvas::textsel::TextSelection;
        use pdfcer_core::annot_author::Quad;
        use pdfcer_core::page_tree::Rect as PageRect;

        let ctx = egui::Context::default();
        let mut app = opened();
        let mut reg = egui_shell::CommandRegistry::new();
        crate::shell::commands::register(&mut reg);
        // ui-text-exempt: registered command ids, never displayed.
        let ids = [
            "format.font",
            "format.font_size",
            "format.bold",
            "format.italic",
            "format.font_colour",
        ];

        // Read: the whole group is invisible, whatever is selected.
        app.ribbon.set_mode("read");
        assert!(
            !app.conditions(&ctx).is_set("mode.edit_content"),
            "Read cannot change page content, so the Font group is not drawn there"
        );

        app.ribbon.set_mode("edit");
        assert!(app.conditions(&ctx).is_set("mode.edit_content"));

        // …and inside Edit, with nothing swept, every one of the five is
        // GREYED rather than absent. That state is the discoverability
        // surface: the operator has clicked a piece of text, the Format tab is
        // up, and hovering a greyed Bold is what tells them to sweep.
        for id in ids {
            let command = reg.get(id).expect("registered");
            assert!(
                !command.is_enabled(&app.conditions(&ctx)),
                "`{id}` must be greyed with nothing swept — it has no operand"
            );
            assert!(
                command.tooltip.is_some(),
                "`{id}` is greyed for most of its life, and R9 permits that only when it \
                 explains itself on hover"
            );
        }

        let Status::Open(doc) = &mut app.status else {
            unreachable!("`opened` opens a document")
        };
        let epoch = doc.edit_epoch;
        doc.text_selection = Some(TextSelection::for_test(
            0,
            epoch,
            vec![Quad::from_rect(PageRect::from_corners(
                72.0, 700.0, 300.0, 710.0,
            ))],
        ));
        for id in ids {
            assert!(
                reg.get(id)
                    .expect("registered")
                    .is_enabled(&app.conditions(&ctx)),
                "`{id}` acts on the swept range there now is"
            );
        }
    }

    /// ★ **THE P3 TENSION, CLOSED** — in Edit, with the text tool armed and a
    /// live text selection, the three text-markup controls come alive and their
    /// press authors an annotation.
    ///
    /// # What was wrong, and why it was a rule violation rather than a gap
    ///
    /// Edit shows the Markup tab, so `markup.underline`, `markup.strikeout` and
    /// `markup.squiggly` were **drawn** there — and `selection.text` could never
    /// be true in Edit, because `canvas::textsel::takes_the_press` gave the press
    /// its text meaning only where the mode could *not* select content. So three
    /// controls rendered, greyed, in every Edit session for the life of the
    /// build, with no state that could ever enable them.
    ///
    /// `RIBBON_IA.md` **P3** reserves greying for *temporarily* unavailable and
    /// says an absent capability renders nothing. Permanently greyed is neither,
    /// and it could not be fixed by hiding: a command lives on exactly one tab,
    /// and the Markup tab is in **both** Review and Edit, so hiding them in Edit
    /// would have required a per-command per-mode visibility rule that this
    /// manifest does not have and that would have been a mechanism invented to
    /// conceal a gap rather than to close one.
    ///
    /// # Why this test is worth its length
    ///
    /// It is the only assertion in the workspace that joins **four** things that
    /// each have their own passing tests: the mode's capabilities, the armed
    /// tool, the condition, and the dispatch. `the_text_markup_controls_need_a_
    /// live_text_selection` above proves the condition-to-enable join and says
    /// nothing about the mode; `canvas::textsel`'s tests prove the press rule and
    /// know nothing about the ribbon. A build that armed the tool and left
    /// `press_kind` reading the mode first would pass both of those and fail
    /// here.
    ///
    /// The **negative** half is asserted first and is what makes the positive
    /// half mean something: with the tool down, the same mode with the same
    /// selection must still refuse, because that is the state the operator was
    /// in before this feature and it must not have been quietly widened. Note it
    /// is the *press rule* that is asserted there, not the condition — the
    /// condition reads only whether a selection exists and is live, and in Edit
    /// without the tool no gesture could have made one.
    #[test]
    fn in_edit_the_text_tool_makes_the_text_markup_controls_reachable() {
        use crate::app::tests::opened;
        use crate::canvas::markup::text::TextMarkKind;
        use crate::canvas::textsel::{self, TextSelection};
        use crate::canvas::tool::{self, CanvasTool};
        use pdfcer_core::annot_author::Quad;
        use pdfcer_core::page_tree::Rect as PageRect;

        let ctx = egui::Context::default();
        let mut app = opened();
        app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
        let caps = app.capabilities();
        assert!(
            caps.edit_content && caps.author_markup,
            "the premise: Edit both selects content and authors markup — which is exactly why \
             the two halves collided"
        );

        // The old world: no tool, no text gesture, so no operand could exist.
        tool::select(&ctx, CanvasTool::Select);
        assert!(
            !textsel::takes_the_press(tool::selected(&ctx), caps),
            "with the select tool, Edit's primary button is still the content marquee"
        );

        // Arm the tool, and the gesture that makes the operand exists.
        tool::select(&ctx, CanvasTool::Text);
        app.on_mode_capabilities_changed(&ctx);
        assert!(
            textsel::takes_the_press(tool::selected(&ctx), caps),
            "the armed text tool takes the press in Edit — that is the whole feature"
        );

        // A sweep would produce this. Planted rather than driven, because
        // `interact` needs a window; the sweep itself is asserted against a real
        // extraction in `canvas::textsel` and against the real binary in
        // `ui-verify`'s `text_tool_selects_and_marks_in_edit`.
        let Status::Open(doc) = &mut app.status else {
            unreachable!("`opened` opens a document")
        };
        let epoch = doc.edit_epoch;
        doc.text_selection = Some(TextSelection::for_test(
            0,
            epoch,
            vec![Quad::from_rect(PageRect::from_corners(
                72.0, 700.0, 300.0, 710.0,
            ))],
        ));

        let mut reg = egui_shell::CommandRegistry::new();
        crate::shell::commands::register(&mut reg);
        for &kind in TextMarkKind::ALL {
            let id = crate::shell::commands::text_mark_command(kind);
            assert!(
                reg.get(id)
                    .expect("registered")
                    .is_enabled(&app.conditions(&ctx)),
                "`{id}` is drawn on Edit's Markup tab and must now be able to enable there — \
                 this is the P3 tension the text tool exists to close"
            );
        }

        // …and pressing one really authors, rather than merely lighting up. The
        // action is the proof that Edit's `author_markup` and the text selection
        // meet: a control that enabled and then declined would be the
        // `selection.bounds` failure one command over.
        let mut actions = Vec::new();
        app.dispatch_command(
            &ctx,
            crate::shell::commands::text_mark_command(TextMarkKind::Underline),
            &mut actions,
        );
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, crate::app::actions::Action::CommitTextMarkup { .. })),
            "the press must raise CommitTextMarkup in Edit, not decline: {actions:?}"
        );
    }

    /// ★ **`measure.finishable` needs a document, not merely a pick set.**
    ///
    /// The one condition published from inside the `Status::Open` arm that is
    /// about a gesture rather than about the document, and this is the reason
    /// it is inside it. A circular pick set lives in `egui::Memory`, which
    /// **outlives documents** — that is the property the armed-tool conditions
    /// below it are published outside the arm to preserve. Here it is exactly
    /// the hazard: the action Finish raises names a page, and with the document
    /// closed there is no page for it to name. A live control that raises a
    /// `CommitDimension` against nothing is the placeholder shape this project
    /// refuses.
    ///
    /// Both directions are asserted, because the first alone would pass on a
    /// build where the condition was never published at all.
    #[test]
    fn finish_is_not_offered_with_no_document_open() {
        use crate::app::state::{FOUR_PAGES, open_fixture};
        use crate::canvas::measure::{self, MeasureKind};
        use crate::canvas::tool::{self, CanvasTool};

        let mut app = PdfcerApp::new();
        let ctx = egui::Context::default();
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Circular));
        measure::circular::plant_pick_for_test(&ctx, 0);
        assert!(
            measure::finishable(&ctx),
            "the canvas really does have a finishable fit"
        );
        assert!(
            !app.conditions(&ctx).is_set("measure.finishable"),
            "…and the ribbon must still not offer to place it into no document"
        );

        // Open one, and the same fit is offered.
        app.status = Status::Open(Box::new(open_fixture(FOUR_PAGES)));
        assert!(
            app.conditions(&ctx).is_set("measure.finishable"),
            "with a document open the control is live"
        );
    }

    /// ★ **`markup.finishable` is the same fact for the vertex tools, and it is
    /// scoped the same way** — plus the one thing that is genuinely different
    /// about it.
    ///
    /// The document half is a near-copy of the test above, deliberately: the two
    /// conditions have the same shape, the same hazard and the same argument for
    /// living inside `Status::Open`, so a build that got the scope right for one
    /// and wrong for the other is what a near-copy catches.
    ///
    /// What is **not** a copy is the last section, and it is the interesting
    /// half: this condition is where the polygon/polyline difference reaches the
    /// operator. `markup::action` needs **three** vertices for a `/Polygon` and
    /// two for a `/PolyLine`, so the same two-click run leaves the ribbon's
    /// Finish live for one tool and greyed for the other. Asserting it here
    /// rather than only in `markup::vertex` is the point — the rule is worth
    /// nothing until it reaches the control.
    #[test]
    fn markup_finish_needs_a_document_and_enough_corners_for_its_kind() {
        use crate::app::state::{FOUR_PAGES, open_fixture};
        use crate::canvas::markup::{MarkupKind, vertex};
        use crate::canvas::tool::{self, CanvasTool};

        let mut app = PdfcerApp::new();
        let ctx = egui::Context::default();
        tool::select(&ctx, CanvasTool::Markup(MarkupKind::Polygon));
        vertex::plant_run_for_test(&ctx, 0, MarkupKind::Polygon);
        assert!(
            vertex::finishable(&ctx),
            "the canvas really does have a finishable run"
        );
        assert!(
            !app.conditions(&ctx).is_set("markup.finishable"),
            "…and the ribbon must still not offer to place it into no document"
        );

        app.status = Status::Open(Box::new(open_fixture(FOUR_PAGES)));
        assert!(
            app.conditions(&ctx).is_set("markup.finishable"),
            "with a document open the control is live"
        );
        // …and it is not the measure tab's condition wearing another name: a
        // measure tool and a markup tool cannot both be armed, so exactly one of
        // the two may ever be set, and a build that collapsed them would light
        // one tab's Finish from the other tab's gesture.
        assert!(!app.conditions(&ctx).is_set("measure.finishable"));

        // ★ The polygon/polyline difference, at the control. Two vertices is a
        // polyline and is not a polygon.
        vertex::plant_short_run_for_test(&ctx, 0, MarkupKind::Polygon);
        assert!(
            !app.conditions(&ctx).is_set("markup.finishable"),
            "two corners are a line drawn there and back, not a polygon"
        );
        tool::select(&ctx, CanvasTool::Markup(MarkupKind::PolyLine));
        vertex::plant_short_run_for_test(&ctx, 0, MarkupKind::PolyLine);
        assert!(
            app.conditions(&ctx).is_set("markup.finishable"),
            "…and the same two corners ARE a polyline"
        );
    }

    /// ★★★ **`selection.delete_permitted` follows the FORMS gate when a form
    /// field is selected**, which is the arm this condition did not have.
    ///
    /// # The defect, and why it is invisible without the second document
    ///
    /// The publication read
    /// `doc.selected_field.is_none() && annotdelete::refuses_selected(doc)`.
    /// With a field selected the first conjunct is **false**, so the whole
    /// expression is false and the condition was set **unconditionally for
    /// every selected field on every document** — a gate that is a no-op by
    /// construction. `format.delete`'s `visible_when` on the `canvas.field`
    /// menu therefore resolved *shown* on a certified fillable form, and the
    /// press it invited deleted nothing and said nothing.
    ///
    /// ⇒ Both halves are asserted, against a fixture **pair** that differs in
    /// exactly one dictionary (`tools/gen-certified-fixture.py`), because the
    /// negative half alone is satisfied by a build that withholds Delete
    /// always — which is a worse defect than the one being fixed: a control
    /// absent where it would have worked leaves the operator no gesture that
    /// reports it.
    ///
    /// ★ Driven through `app.conditions()` and `open_path` rather than by
    /// calling the derivation, for this module's standing reason one test up:
    /// what is under test is the **join** between the query and the published
    /// name, and a test that read the derivation would prove `set.set` works.
    #[test]
    fn a_certified_document_withholds_delete_for_a_selected_form_field() {
        use crate::app::state::SelectedField;

        let ctx = egui::Context::default();
        let certifier = SelectedField {
            field: "Certifier".to_owned(),
            widget: 0,
            page: 0,
        };
        let local = |rel: &str| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures")
                .join(rel)
        };

        let mut app = PdfcerApp::new();
        app.open_path(local("certified-comments.pdf"));
        let Status::Open(doc) = &mut app.status else {
            panic!("the certified fixture opens") // ui-text-exempt: test panic, never displayed
        };
        doc.selected_field = Some(certifier.clone());
        assert!(
            !app.conditions(&ctx).is_set("selection.delete_permitted"),
            "Delete is offered over a form field on a document whose /Perms \
             /DocMDP freezes the form's structure. §12.8.2.2 Table 257 permits \
             filling such a form and forbids restructuring it, so \
             `EditSession::deletion_refusal` answers Some and the control must \
             not be drawn (R9)"
        );
        assert!(
            app.conditions(&ctx).is_set("selection.actionable"),
            "the field is still SELECTED — `selection.actionable` and \
             `selection.delete_permitted` answer different questions, and \
             collapsing them would take the Properties command away too"
        );

        let mut app = PdfcerApp::new();
        app.open_path(local("threaded-comments.pdf"));
        let Status::Open(doc) = &mut app.status else {
            panic!("the uncertified twin opens") // ui-text-exempt: test panic, never displayed
        };
        doc.selected_field = Some(certifier);
        assert!(
            app.conditions(&ctx).is_set("selection.delete_permitted"),
            "the condition refused on the uncertified twin, which differs from \
             the certified fixture only in the catalog's /Perms entry — so this \
             build hides Delete on every signed document"
        );
    }
}
