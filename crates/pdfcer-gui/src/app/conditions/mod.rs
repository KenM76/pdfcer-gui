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
        // ★★★ **At least one panel is in a window of its own**, which is the
        // only thing `view.dock_all_panels` needs to know.
        //
        // Read from the dock's live layout rather than from a counter this
        // type keeps, and that is the same rule
        // `egui_shell::dock::DockFrameReport::panels_drawn`'s own docs argue
        // for: *"an application deriving a toolbar toggle's selected state
        // should read this rather than keeping a boolean of its own"*. A
        // separate flag could disagree with the layout, and the operator would
        // see a greyed recovery command with a floating window on screen —
        // which is the one moment the command has to work.
        //
        // Set OUTSIDE the `Status::Open` arm: a floated panel does not stop
        // floating because the document was closed, and the way to get it back
        // must not depend on having a file open.
        if !self.dock.layout().floating.is_empty() {
            set.set("panels.floating");
        }
        // ★★★ **An Acrobat exists on this machine** — `OPERATOR_REQUESTS.md`
        // O122, and the ONE thing that decides whether the control beside the
        // mode selector is drawn.
        //
        // Set OUTSIDE the `Status::Open` arm, deliberately and for a reason
        // that is easy to get backwards: this condition answers *"does this
        // machine have an Acrobat?"* and NOT *"can I press the button now?"*.
        // The second question is the command's own
        // `enabled_when("doc.open")`, and R9 is what splits them — a machine
        // with no Acrobat renders nothing, a machine with one and no document
        // open renders a greyed control that explains itself on hover.
        //
        // Nesting it inside `Status::Open` would collapse the two into one and
        // lose the distinction the operator can actually see: a button that
        // flickers in and out as documents open and close, rather than one
        // that is simply always there on a machine that has Acrobat.
        if self.acrobat.is_some() {
            set.set(crate::shell::manifest::ACROBAT_AVAILABLE);
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
            // ★★★ **THE MODE IS A REFUSAL TOO, AND ITS ABSENCE HERE DREW AN
            // ENABLED DELETE IN READ** — 2026-09-03.
            //
            // The ladder above asks the ENGINE whether a delete would be
            // refused. That was the whole question, on the strength of the note
            // `canvas::keys` wrote beside its own guard:
            //
            //   > in practice this is unreachable, because entering such a mode
            //   > clears the selection and no gesture can build a new one
            //
            // **`OPERATOR_REQUESTS.md` O71 falsified that nine days later.**
            // `canvas::clicking`'s image arm runs precisely when
            // `!caps.edit_content` — it exists so a reader can click a picture
            // and copy it — so a content selection has been reachable in Read
            // since 2026-08-31, `selection.any` and everything built on it has
            // been set there, and the Format tab's Delete was **drawn and
            // enabled** in the mode that authors nothing.
            //
            // ★ Not a widening into `selection.actionable`'s territory, which
            // this condition's own header forbids. That one asks *is there
            // anything to delete*; this asks *would the delete be refused* —
            // and a mode that traces `canvas-delete-declined` is refusing, by
            // the same reading that makes `/Encrypt` a refusal.
            //
            // ★★ ONE PREDICATE PER CAPABILITY, and the rungs are the
            // dispatcher's own precedence — field, then annotation, then
            // content — so the drawn control and the arm behind it cannot
            // answer about different capabilities. **`author_markup` guards the
            // annotation rung**, which is what keeps Review's Format tab and
            // its markup Delete alive: deleting a markup is exactly what Review
            // is for, and collapsing this to `edit_content` would take the
            // working verb away from the mode that owns it while leaving the
            // broken one in Read.
            let caps = self.capabilities();
            let mode_may_delete = if doc.selection.annot().is_some() {
                caps.author_markup
            } else {
                caps.edit_content
            };
            if mode_may_delete && !delete_refused {
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
            // ★★★ **A markup annotation is selected, and this mode may author
            // markup** — the Format ▸ Markup group's whole existence condition,
            // published from 2026-09-06 for `app::markupband`'s five controls.
            //
            // # Why ONE condition carries two facts
            //
            // `egui_shell::commands::Enable`'s grammar is one condition name
            // with an optional leading `!` — *"a grammar in a string is a
            // parser and a parser is a thing that has its own bugs"* — so an
            // `A && B` predicate is published as a **named fact** rather than
            // assembled in a manifest. `selection.formattable` above is the
            // same shape for the same reason.
            //
            // It is the right shape here anyway, and the argument is R9's. The
            // Font group takes two conditions because its two failure states
            // want two ANSWERS: absent in a mode that cannot edit content,
            // greyed with an explanation when nothing is swept — and the
            // greying is the feature, because reaching the operand means
            // pressing `T` and nothing else on screen says so (O37). Nothing
            // here is like that. The operand is *the mark you clicked*, so a
            // greyed Markup group could only say *select a mark*, which the
            // operator has already done or the contextual tab would not be
            // drawn. R9 then requires absence in both states, and one name is
            // what absence in both states is called.
            //
            // # ★★ `AnnotKind::Markup`, matched — Rule 15, and it is a
            // DIFFERENT VERB, not a stricter filter
            //
            // A **ce dimension** is also an annotation and is also selectable,
            // and `panels::properties::dimension` owns it through
            // `set_dimension_style`. Handing one to `set_markup_style`
            // regenerates it as a bare line with its label and witness lines
            // gone — the engine refuses it by name, and this condition is what
            // stops the controls ever being drawn for one. The test is a
            // `match` on `AnnotKind` that the compiler checks rather than a
            // comparison of `/Subtype` strings, because a ce dimension's
            // `/Subtype` is `/Line` exactly like an arrow's: a string test
            // would restyle the operator's dimensions into bare lines and
            // would look correct while doing it.
            //
            // # ★ `author_markup`, NOT `edit_content`
            //
            // One predicate per capability — the rule `canvas::keys` states
            // beside its own pair, and `dispatch::format`'s Delete arm repeats.
            // **Review must keep this**: restyling a mark is exactly what
            // Review is for, and a guard reaching for `edit_content` would take
            // the working verb away from the mode that owns it. Read has
            // neither capability and gets no group.
            //
            // # ★★ The LOCK is deliberately not folded in
            //
            // §12.5.3 Table 165 bit 8 is a fact about one annotation, not about
            // the build or the mode — click a different mark and the controls
            // work — which is exactly the case R9 reserves greying for. Folding
            // it in here would make the group flicker out of the ribbon on
            // every click that landed on a locked mark, and would leave nothing
            // on screen to say why. `app::markupband` greys instead, with
            // `text::panels::properties::markup_locked`: the same sentence the
            // Properties panel shows, so the two surfaces cannot refuse for
            // different reasons.
            //
            // # ★ Why it asks the ribbon for the mode rather than `self.modes`
            //
            // Because [`Self::capabilities`] does, and its own note says why:
            // the ribbon is where the operator's click lands and `self.modes`
            // catches up later in the same frame. A second derivation would put
            // this group one frame behind the mode selector on exactly the
            // frame a stray click is most likely.
            if self.capabilities().author_markup
                && doc.selection.annot().is_some_and(|annot| {
                    matches!(
                        annot.target.kind,
                        crate::canvas::selection::annot::AnnotKind::Markup
                    )
                })
            {
                set.set("selection.markup_restylable");
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

mod tests;
