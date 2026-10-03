//! # `declined` — the worded decline: what a command that did not run says
//!
//! The [`Declined`] enum, its sentence ([`Declined::line`]) and how long it
//! owes that sentence ([`Declined::still_true`]). The store, the recorders and
//! the status-bar slot are `pdfcer_gui::app::status::decline`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/decline.md`.

mod fresh;
mod line;
mod remedy;

pub use fresh::History;

/// A framing zoom that did not happen, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Declined {
    /// Nothing on the page resolved to a box to frame.
    ///
    /// `ZoomOutcome::NoBounds`, which `crate::canvas::zoom` raises for three
    /// situations it rules are one from the operator's side — nothing
    /// selected, the selection on another page, or a selection that no longer
    /// resolves after an edit.
    NothingToFrame,
    /// The canvas had not drawn a page, so there was no viewport to frame
    /// anything into. `ZoomOutcome::NoCanvas`.
    CanvasNotDrawn,
    /// **`file.save_copy` was given a destination and produced no file.**
    ///
    /// The first decline here that is not about zoom, which is why this enum's
    /// two constructors are now `Declined::of` (zoom's) and
    /// `record_save_failure` (the save's) rather than one. It joins rather
    /// than getting a store of its own for the reason the module header gives
    /// about the two disclosures: a second mechanism beside this one would give
    /// the bar two ways to learn the same *kind* of thing, and the second would
    /// be the one that forgot to retire itself.
    ///
    /// # It is retired by the operator's next act, and by nothing else
    ///
    /// [`Self::still_true`] answers `true` for it unconditionally, and that is a
    /// decision rather than a gap. Its two neighbours have a live predicate to
    /// re-ask — *is something framable now?*, *has the page drawn?* — because
    /// their reasons can stop being true on their own, with the operator doing
    /// nothing. A failed write has no such state: the folder is not going to
    /// reappear on the next frame, and if it did, the sentence would still be a
    /// true report of what happened when the operator pressed Save.
    ///
    /// So it lives exactly as long as `retire` lets it — until the next
    /// command — which is `super::page_box`'s clamp-note rule and the one the
    /// module header names as the right precedent. Pressing `Ctrl+S` again
    /// retires it and then records it again, so two failed saves are two events.
    ///
    /// The engine's own reason is **not** carried. It goes to the trace, from
    /// `crate::app::save`; see [`crate::text::status::save_copy_failed`] for why
    /// a `Display` impl's prose is not operator copy.
    SaveFailed,
    /// **A verb was asked to act on something drawn inside a form XObject.**
    ///
    /// # The two states this keeps apart, and why merging them is expensive
    ///
    /// *"Nothing is selected"* and *"the thing you selected cannot be reached
    /// by this verb"* are the operator's mistake and the program's limit. An
    /// interface that reports the second as the first sends them looking for
    /// something they did not do wrong — and here it would contradict the
    /// outline they can see on screen, which is the most confusing shape a
    /// message can have.
    ///
    ///
    ///
    /// This variant carried no payload and its sentence said *"pdfcer cannot
    /// edit inside one yet"*. `pdfcer-core`'s `Pass 188.0` shipped six
    /// form-scoped geometry verbs, this shell wired all six, and the sentence
    /// went on telling the operator otherwise for ten days.
    ///
    /// [`crate::text::status::InsideFormRefusal`] carries the whole account.
    /// The part that belongs HERE is what it did to the retirement rule below:
    /// the two call sites this variant serves have **opposite** predicates, so
    /// one `still_true` arm could not be right for both, and the one that was
    /// written discarded the other site's sentence before the bar drew it.
    ///
    /// # Retired by the operator's next act, and by the selection changing
    ///
    /// Unlike [`Self::SaveFailed`], this one has a live predicate worth
    /// re-asking: the moment the operator selects something else, the sentence
    /// stops being about what they are looking at. [`Self::still_true`]
    /// therefore consults `selection_in_form`, gathered from the same
    /// accessor `crate::app::conditions` publishes `selection.in_form` from —
    /// so the greyed control and the sentence in the bar cannot come from
    /// different questions.
    ///
    /// That reasoning is still right, and it is right about
    /// [`crate::text::status::InsideFormRefusal::NotAPath`] only. See
    /// [`Self::still_true`].
    InsideForm(crate::text::status::InsideFormRefusal),
    /// **A restyle of existing text did not happen** — O37, 2026-08-27.
    ///
    /// The payload is [`crate::text::status::TextStyleRefusal`], which is the
    /// shell's own reading of which refusals an operator can act on. The
    /// reasoning for keeping the engine's own prose OFF the status bar is on
    /// that enum.
    ///
    ///
    /// It read: *"It is a `Copy` enum rather than a `String` so this whole type
    /// stays `Copy` and `Declined::line` stays `&'static str`."* **Both halves
    /// are now overtaken, and neither by accident.**
    ///
    /// `Declined::line` stopped being `&'static str` on 2026-09-10, when O141's
    /// *"pdfcer cannot type a `q` into this text"* needed to name the
    /// character; that arm's own docs record that the generic wording had been
    /// defended in three separate places as a design choice when it was a
    /// return type. And `Copy` went on 2026-09-11, when
    /// [`crate::text::status::TextStyleRefusal::FaceLacksCharacters`] took the
    /// engine's `Refusal::remedy_faces` — the faces that *would* show the run
    /// — which is a `Vec<String>`.
    ///
    /// The rule the old sentence was really protecting survives untouched:
    /// **no engine prose on this bar.** What travels is a list of `/BaseFont`
    /// names pdfcer computed. The words around them are this shell's.
    ///
    /// # It is retired by the operator's next act, and by nothing else
    ///
    /// [`Self::still_true`] answers `true` for it unconditionally, in the same
    /// ruling as `SaveFailed` and the two adopt refusals: a page is not going
    /// to grow a Bold face, and a run is not going to become pinnable, between
    /// one frame and the next. The remedy is always something the operator
    /// *does* — pick a different face, select different text, press the button
    /// again — and every one of those is a command, which `retire` catches.
    ///
    /// It is deliberately NOT keyed on the selection the way `InsideForm`
    /// is. `InsideForm`'s sentence sends the operator to select something else,
    /// so selecting something else is the remedy completing and the sentence
    /// must go. These sentences send the operator to press a *different
    /// control* on the *same* selection — so retiring on a selection change
    /// would be right, and retiring on it alone would let the sentence vanish
    /// while the operator was reading it.
    TextStyle(crate::text::status::TextStyleRefusal),
    /// **A rotation did not happen** — the ninth handle, 2026-08-28.
    ///
    ///
    /// # Why this variant exists for a gesture that should never refuse
    ///
    /// Because **this project's founding defect shape is a grip that is
    /// dragged, released, and does nothing with no explanation** — and a rotate
    /// handle is the newest grip on the canvas. Two of the four refusals it can
    /// carry describe routing failures that cannot happen while the routing
    /// holds (`canvas::rotating` matches on `AnnotKind`, and a widget is never
    /// an annotation selection at all). That is the argument for wording them,
    /// not against: a routing bug with a sentence is a bug report, and one
    /// without is a handle that does nothing.
    ///
    /// The genuinely reachable case is a **certified** document, and it is
    /// the one an operator cannot possibly guess at — a signed drawing looks
    /// exactly like an unsigned one on the canvas.
    ///
    /// # Recorded from three places, which is unusual and is correct
    ///
    /// `canvas::rotating` records `RotateRefusal::NoDimensionRecord` before
    /// any verb is called, because that condition is a **query** the shell can
    /// answer itself — the same placement `record_flatten_certified` uses.
    /// `app::actions::annots::{rotate, rotate_dimension}` record the other
    /// three from **inside** the `vector_edit` closure, because whether the
    /// engine will refuse is not knowable before the call — the same placement
    /// `record_resize_not_rebuildable` uses, and for the reason its own docs
    /// give.
    ///
    /// # Retired by the operator's next act
    ///
    /// [`Self::still_true`] answers `true` unconditionally, with `TextStyle`
    /// and the others whose state cannot change between two frames: a document
    /// does not stop being signed, and a sidecar does not grow a record, while
    /// the operator reads the status bar. What retires it is their next
    /// command, which `retire` catches.
    Rotate(crate::text::rotating::RotateRefusal),
    /// **"Give this page its own copy" did not happen** — the form-XObject
    /// verb, 2026-08-28.
    ///
    ///
    /// # Why this refusal matters more than any other in this enum
    ///
    /// Because **a successful unshare and a refused one look identical on the
    /// canvas**, and no other decline here has that property: the copy is
    /// byte-identical until it is edited, so the page renders pixel-for-pixel
    /// the same either way. Silence therefore does not read as *"nothing
    /// happened"* — it reads as *"it worked"*, and the operator's very next act
    /// is to edit the drawing they believe they have just privatised. On this
    /// operator's documents that is a title block shared by thirty-six sheets.
    /// `crate::text::unshare`'s header carries the full account and is why every
    /// one of the verb's refusals is worded where `resize` words one of six.
    ///
    /// Recorded from three positions, unlike [`Self::Rotate`]'s two — see
    /// `record_unshare` for the split — and retired by the operator's next
    /// act: [`Self::still_true`] answers `true` unconditionally, deliberately
    /// **not** on `selection_in_form` the way [`Self::InsideForm`] does.
    Unshare(crate::text::unshare::UnshareRefusal),
    /// **`format.merge_text_runs` was refused**, by the preflight on the press
    /// or by the engine. Nothing changes on the page, so the sentence is the
    /// only report. Retired by the operator's next act.
    RunMerge(crate::text::runmerge::RunMergeRefusal),
    /// **A Find bar Replace rewrote nothing.** Retired by the next act.
    Replace(crate::text::replace::ReplaceRefusal),
    /// **The Settings window's Save wrote nothing.**
    ///
    /// # Why this is not [`Self::SaveFailed`], although both are failed writes
    ///
    /// Because the two sentences have to say opposite things about what
    /// happened to the operator's work.
    ///
    /// A failed `file.save_copy` produced **no file**: the operator asked for
    /// something and got nothing, and there is no partial state to explain.
    ///
    /// A failed settings save is the opposite shape. The application **adopted
    /// the configuration anyway** — deliberately, because the operator asked
    /// for it and a disk that refuses should not cost them a choice they made —
    /// so what is true is *"this is in force now and will be gone when pdfcer
    /// restarts"*. Reusing the save-a-copy sentence would tell them their
    /// choice did not take, which is false, and they would make it again.
    ///
    /// Sharing a variant would also make the two indistinguishable in the one
    /// place it matters: an operator who pressed Save in two different windows
    /// in one minute.
    ///
    /// # Retired by the operator's next command, like its neighbour
    ///
    /// [`Self::still_true`] answers `true` unconditionally, for the same reason
    /// `SaveFailed` does: the folder is not going to become writable on the
    /// next frame, and if it did, the sentence would still be a true report of
    /// what happened when Save was pressed.
    ///
    /// The engine's own reason is **not** carried — it goes to the trace from
    /// `crate::app::settings_window`, which is where the store location is also
    /// recorded. A `Display` impl's prose is not operator copy.
    SettingsNotSaved,
    /// **`edit.undo` was invoked with an empty command log.**
    ///
    /// # Why this is worded when the control that raises it is greyed
    ///
    /// Because the control is not the only route, and the other route is the
    /// one where greying explains nothing. `edit.undo` is gated on
    /// `undo.available`, so the quick-access button is un-pressable with an
    /// empty log — and it is *also* bound to `Ctrl+Z`, and
    /// `crate::app::modes::capability::offers_command` lets it through in
    /// **every** mode because it sits on no tab. So the reachable case is a
    /// chord, fired by an operator whose eyes are on the page rather than on an
    /// 18 pt icon in the title bar.
    ///
    /// That is [`Self::NothingToFrame`]'s *"reached by a chord"* argument with
    /// the reflexes of a whole industry behind it: `Ctrl+Z` is the keystroke an
    /// operator presses without deciding to, and answering the commonest
    /// keystroke in editing with nothing at all is the exact "the button does
    /// nothing" state this project was founded on.
    ///
    /// # It has a live predicate, and it is the one that produced it
    ///
    /// [`Self::still_true`] re-asks `EditSession::can_undo` — the same question
    /// `PdfcerApp::conditions` publishes `undo.available` from and the same one
    /// the apply arm declined on. So authoring anything at all retires the
    /// sentence on the next frame, without the operator invoking a command,
    /// which is [`Self::NothingToFrame`]'s shape exactly: *the remedy happened,
    /// so the sentence is history.*
    /// **A widget could not be registered: another field already has the name.**
    ///
    /// `EditError::FieldNameTaken`, raised by `EditSession::adopt_widget`.
    ///
    /// # Why this is refused rather than auto-renamed, which is the engine's
    /// ruling and this shell agrees with it
    ///
    /// ISO 32000-2 SS12.7.3.1 makes the **fully qualified name the field's
    /// identity**. Two top-level fields called `Address` are not two fields —
    /// they are *one field with two widgets*, so typing in either fills both.
    /// No viewer reports this. The operator discovers it by typing into one box
    /// and watching another change, which is the worst possible way to learn it.
    ///
    /// `pageops::assemble` auto-renames on merge because it has nobody to ask.
    /// This surface **does** have somebody to ask, and the engine put it
    /// plainly: *"`Address_2` is a name nobody chose."* So the edit declines and
    /// the operator retypes, with what they typed still in the box in front of
    /// them.
    ///
    /// The clashing name is **not** carried into the sentence, and the reason
    /// is that **the name is already on screen** — the operator typed it
    /// seconds ago and it is still in the field they typed it into.
    ///
    /// ⚠ This paragraph used to open *"It is a `Copy` enum"*. That was never
    /// the reason and is not even true: this enum derives
    /// `Clone, Debug, PartialEq, Eq` and has carried non-`Copy` payloads since
    /// `CustomStampUnavailable` arrived. The argument that matters survives the
    /// correction intact — and see `FieldPathCrossesTerminal` immediately
    /// below, which is the case where it does **not** hold and so does carry
    /// its name.
    ///
    ///
    /// | engine variant | verb | the fact |
    /// |---|---|---|
    /// | `EditError::FieldNameTaken` | `adopt_widget` | the name the widget would take is already borne |
    /// | `FormAuthorError::RenameCollision` | `rename_field` | the name being renamed TO is already borne |
    ///
    /// One decline for both, deliberately, under *one fact, one wording*: from
    /// the operator's chair these are the same sentence — *something already
    /// has that name* — with the same remedy. Two variants here would be two
    /// spellings of one fact, which is how a surface comes to tell the same
    /// truth two ways depending on which control produced it.
    ///
    /// And the rename one is **the only correctable refusal the rename
    /// surface can actually reach.** Everything else `rename_field` refuses is
    /// pre-empted by the Rename button's own gate —
    /// `!typed.is_empty() && !typed.contains('.')` covers a dotted name, an
    /// empty name and a too-deep path — but a collision would need a walk of
    /// the field tree to predict, so the panel cannot grey on it and the
    /// engine decides. It is also the common case: rename `Rev1` to `Rev2` on
    /// a form that has a `Rev2`.
    ///
    /// ⚠ It reached the operator as the funnel floor's generic *"That change
    /// was refused"* from the day the rename surface shipped until
    /// 2026-09-12, because `actions::forms::rename` mapped only `Ok`.
    FieldNameTaken,
    /// **A field name was refused because a dot in it points through a
    /// field that already exists** — `FormAuthorError::FieldPathCrossesTerminal`,
    /// raised by every `EditSession::add_*` verb and by `paste_field`.
    ///
    /// A period separates levels of the field-name tree (§12.7.3.2), so
    /// `Order.Total` asks for a field `Total` inside a group `Order` — and
    /// §12.7.3.1 does not let one dictionary be both a terminal field and a
    /// group. If `Order` is already an ordinary field, the request cannot be
    /// granted without destroying it, so the engine refuses.
    ///
    /// # Why this one CARRIES its name when its neighbour deliberately does
    /// not
    ///
    /// `FieldNameTaken` above argues that the clashing name is not worth
    /// carrying because it is the name the operator just typed and it is still
    /// in the box in front of him. That argument is sound there and does not
    /// hold here, which is the whole reason this variant has a payload.
    ///
    /// The name in this sentence is a **different field**. He typed
    /// `Order.Total`; the field in the way is `Order`. Nothing on screen says
    /// which prefix of what he typed is the problem, and on a form with eight
    /// fields "that name was refused" leaves him guessing. The engine already
    /// worked it out — `fully_qualified_name` walked the `/Parent` chain to
    /// build it — so the only thing standing between that answer and the
    /// operator is whether this shell bothers to carry it.
    ///
    /// # This replaced a shell-side pre-check, and the difference matters
    ///
    ///
    /// The pre-check was then not merely redundant — it was **wrong**, and in
    /// the direction a duplicate model always goes wrong. It refused on any
    /// prefix present in `AcroForm::fields`, where the engine refuses only on a
    /// **terminal** (`child_field_count == 0`). Those differ on the mixed node
    /// — child fields *and* its own bare widget kids, a shape pdfcer's own
    /// same-name merge can generate — which the engine correctly allows and the
    /// shell refused, with a sentence claiming a field would be destroyed when
    /// none would be.
    ///
    /// ⇒ Reading the engine's variant cannot drift from the engine, because it
    /// *is* the engine's answer. That is the property the pre-check could never
    /// have.
    FieldPathCrossesTerminal(String),
    /// **A field name was refused because it is a PATH rather than a
    /// name** — `FormAuthorError::DottedPartialName`.
    ///
    /// Raised by three engine verbs (`rename_field`, `adopt_widget`, `sign`)
    /// and reachable from **one** of this shell's surfaces: the Tab-order
    /// register panel's adopt boxes, which take free text and gate only on
    /// non-empty. The rename box greys its commit button on a period and the
    /// signature window offers only names that already exist, so neither can
    /// produce the input. See `crate::app::actions::forms::correctable`,
    /// which carries the route table.
    ///
    /// # The distinction from [`Self::FieldPathCrossesTerminal`] directly
    /// # above, which is the thing a reader will get backwards
    ///
    /// They are both about a period and they are **not** the same refusal. The
    /// difference is which side of the period the verb was looking at:
    ///
    /// | | the verb was given | the period means | refused because |
    /// |---|---|---|---|
    /// | [`Self::FieldPathCrossesTerminal`] | a **path**, legitimately | *put `Total` inside a group `Order`* | `Order` already exists and is an ordinary field, so granting it would destroy `Order` |
    /// | this one | a **partial name** | nothing — a partial name is one segment by §12.7.3.2's construction | the period cannot be honoured at all, whatever is or is not in the document |
    ///
    /// So the first depends on the document and the second does not. `A.B`
    /// given to `add_text_field` is a perfectly good request that may or may
    /// not be grantable; `A.B` given to `adopt_widget` is not a request the
    /// verb can express, because a `/T` is the one segment its node
    /// contributes to the fully-qualified name.
    ///
    /// # What the refusal prevents — and it is NOT data loss
    ///
    /// Worth stating because the obvious guess is wrong and the sentence this
    /// variant words has to be true. Neither verb touches an existing field's
    /// `/Kids`; a pre-existing `Text` survives an adopt of `Text.2` completely
    /// intact. What would be produced is a field **nobody can address**:
    /// §12.7.3.2 makes its FQN that same dotted string, and every resolver
    /// splits on `.` first, looks for `2` inside a group `Text`, finds a
    /// terminal there, and stops. It renders. It accepts a click. And
    /// `fill_text_field`, FDF/XFDF import, a `/CO` calculation-order entry and
    /// a reset-form `/Fields` array can none of them reach it — **pdfcer's own
    /// fill verbs included**.
    ///
    /// ⇒ Which is why the sentence says *can be clicked but never filled*
    /// rather than warning about a loss. See `crate::text::fieldclip`.
    ///
    /// # Why it carries its name, and the first answer was wrong
    ///
    /// Not because the box has closed — it has not. The adopt panel's drafts
    /// survive a refused adopt, so the typed name is still on screen.
    ///
    /// It carries the name because **that panel shows a name box per unclaimed
    /// widget**, several at once, and the bar has one sentence. Without the
    /// name, *"that name is a path"* is true of whichever row the operator was
    /// in and says nothing about which one. [`Self::FieldNameTaken`]'s
    /// opposite argument — *the name is in the box in front of him* — holds on
    /// the Properties panel, where there is exactly one box.
    DottedPartialName(String),
    /// **A widget could not be registered because it carries no name of its
    /// own, and none was supplied.**
    ///
    /// `EditError::WidgetHasNoFieldIdentity`.
    ///
    /// # What this actually means, and why the sentence must not say
    /// "recovered"
    ///
    /// It is a **bare kid**: a widget whose `/Parent` pointed at its field, in a
    /// document where that `/Parent` is gone. The engine measured a real form
    /// and found 2 of 13 in this shape after an insert, and named its own cause
    /// — `insert_pages` drops `/Parent` from every dictionary it copies, which
    /// is correct for a page and destroys a widget's only link to its identity.
    ///
    /// What was lost is not just the name. It was the name **and** the field
    /// type, the radio flags and the value. Nothing in this document holds any
    /// of it, so a name typed here **creates a new field**; it does not recover
    /// the old one. The sentence says so, because an operator told they had
    /// "restored" a radio button would go looking for its group.
    WidgetHasNoName,
    /// **A resize was refused because the appearance cannot be rebuilt**
    /// (`OPERATOR_REQUESTS.md` O51, engine `Pass 151.0`).
    ///
    /// The one decline in this enum that names a **remedy the operator can
    /// reach in one click**, and it exists because of a fact about the format
    /// rather than about pdfcer.
    ///
    /// An annotation's artwork is placed through §12.5.5's matrix, which a
    /// resize makes a scale, and that matrix is applied *after* stroking. So
    /// the drawn stroke scales with it whatever `/BS /W` says — and **no
    /// per-axis stroke width exists** in PDF or in SVG. Two states are
    /// therefore unsatisfiable when pdfcer did not author the appearance:
    ///
    /// | scale | *Scale line weight* | why |
    /// |---|---|---|
    /// | uniform | **off** | the matrix scales it anyway, against the request |
    /// | non-uniform | either | the stroke is anisotropic; no scalar describes it |
    ///
    /// ⇒ `uniform` is carried so the sentence can name the remedy that
    /// actually applies. Under a uniform scale, turning *Scale line weight* on
    /// makes the resize **exact**; under a non-uniform one it does not help and
    /// only *Allow the artwork to distort* will proceed.
    ///
    /// Inkscape hit the identical limit in SVG (Launchpad #1335376) and
    /// closed it **Invalid** — correct spec behaviour — and its response is to
    /// silently produce a distorted stroke. This is the sentence that makes
    /// pdfcer better than the parity reference rather than equal to it, which is
    /// what the engine's own note recommended.
    ResizeNotRebuildable {
        /// Whether the drag was proportional, which decides which remedy the
        /// sentence names.
        uniform: bool,
    },
    /// **A resize was refused because the annotation is a fixed-size marker.**
    ///
    ///
    /// Reachable from this shell, which is why it is worded: the sticky's
    /// canvas grips are move-only, but the Properties panel's geometry
    /// fields raise the same `AnnotAction::Resize` the grips do.
    ///
    /// Two sentences, not one, and the split is the engine's: for a
    /// `/Text` the rule is the subtype's own and nothing the operator does
    /// changes it; for anything else it is the `NoZoom` flag. This shell has
    /// no flag editor yet, so neither sentence names a switch — but the
    /// distinction is kept here so the day one exists the second sentence
    /// can name it without re-deriving which case it is in.
    ResizeFixedSizeMarker {
        /// `true` when the `NoZoom` flag, not the `/Text` subtype, makes it
        /// fixed-size.
        by_flag: bool,
    },
    /// **`edit.form_flatten` was invoked and the document's certification
    /// forbids it.**
    ///
    /// The ribbon control is `enabled_when("doc.pages")` and is therefore live
    /// on any open document, where the Forms panel's own Flatten button greys
    /// itself from `EditSession::flatten_refusal`. The two disagree on
    /// *appearance* and agree on *behaviour*, which is the intended shape:
    /// publishing a certification condition would cost a query per frame for a
    /// control that is almost never pressed, so the ribbon asks at the moment
    /// of the press and answers in a sentence.
    ///
    /// It asks `flatten_refusal` and **not** `fill_refusal`, which is the
    /// distinction the panel's own comment spent twenty lines earning: flatten
    /// removes the form, so it takes the strict structural gate, and on a
    /// certified fillable form at `/P 2` filling is permitted while flattening
    /// is not. An operator who has just typed into the form and then finds
    /// Flatten refusing is meeting a real rule rather than a broken control,
    /// and the sentence says which.
    FlattenCertified,
    /// **A form field or one of its boxes was asked to be deleted and the
    /// document's structure is frozen** — `EditSession::deletion_refusal`
    /// answered `Some`.
    ///
    /// [`Self::FlattenCertified`]'s sibling with the same gate underneath
    /// (`structural_form_refusal`: `/Encrypt`, then the strict certification
    /// check), asked about a different verb.
    ///
    /// They share a gate and are still different questions — flatten
    /// additionally creates page content and carries a `/Size`-suppression
    /// guard deletion does not — which is why core exposes them as two
    /// functions and why this is a second variant rather than a second caller
    /// of the first.
    ///
    /// # Why it exists when all four doors are already gated
    ///
    /// It should be unreachable, and that is exactly why it is worded. Every
    /// route an operator has to `delete_field` / `delete_widget` now consults
    /// `crate::panels::properties::formfield::refuses_delete` before offering
    /// anything: the Properties panel's two buttons, the `canvas.field` menu
    /// item (through `selection.delete_permitted`), the Delete key's rung 0,
    /// and the dispatcher's arm.
    ///
    /// What is left is the residue those four cannot cover, because **a gate
    /// is a forecast of the engine's guard and the guard is the authority**:
    ///
    /// * a **chord** bound to `format.delete` — a chord consults no
    ///   `visible_when`, so no menu or ribbon condition reaches it;
    /// * a condition that went stale inside a frame;
    /// * a refusal `deletion_refusal` does not predict;
    /// * and the case the *panel* cannot cover either — a delete reaching the
    ///   verb with **no field selected**, where there is no properties section
    ///   drawing a sentence at all.
    ///
    ///
    /// It is deliberately **not** the wording the Properties panel draws.
    /// That one is a standing *description* of the document, drawn from the
    /// moment a field is selected; this is a *decline*, reporting that a
    /// gesture just happened and took no effect. This module's header insists
    /// the two speech acts must not wear the same words in the same place, and
    /// [`crate::text::status::field_delete_declined_structural`] carries the
    /// full argument for every word the two do not share.
    FieldDeleteRefused,
    /// **The Points tool was pressed in a mode that cannot author.**
    ///
    /// `OPERATOR_REQUESTS.md` row **O69**. The arm has always declined — an
    /// anchor is selected in order to be *dragged*, and a mode that refuses
    /// the drag must refuse the tool rather than arm it and say no to every
    /// gesture afterwards — but it declined into the trace alone, so the
    /// operator pressed a control and the program did nothing and said
    /// nothing. That silence is half of why he reported the route as
    /// unreliable.
    ///
    /// The ribbon item is now withheld outside Edit, so the only surviving
    /// route to this decline is the bare `A` chord: chords are filtered by
    /// **tab** visibility and View is in every mode, so the key still reaches
    /// the arm. A key that does nothing has no control to hover, which makes
    /// it the case that most needs a sentence rather than the least.
    NodeToolNeedsEditMode,
    /// **A corner of a ce dimension could not be added or taken away** —
    /// the operator's report of 2026-09-05, in his own words:
    ///
    /// > *"I also can't edit or delete nodes of a markup shape once it is
    /// > drawn."*
    ///
    ///
    /// # Why a gesture with a preflight still needs a decline
    ///
    /// `canvas::dimdrag::count_edit` asks `EditSession::vertex_edit_preview`
    /// before it draws anything, so a refused edit is never previewed and never
    /// raised as an action — the engine is never asked to refuse. That is the
    /// right design and it makes this sentence **the only report of the
    /// refusal that exists**: no action, no funnel, no `EditRefused`. Without
    /// it the operator drags a corner of a triangle out of the shape, releases,
    /// and the triangle is still a triangle with nothing anywhere saying why.
    /// That silence is precisely the shape of the report this whole surface
    /// answers.
    ///
    /// # Recorded from the gesture, before any verb
    ///
    /// From `canvas::dimdrag` on the release frame — the placement
    /// `record_flatten_certified` uses and for its stated reason: the
    /// condition is a **query** the shell can answer itself, so it is answered
    /// where the gesture is rather than inside a funnel the gesture never
    /// enters.
    ///
    /// # Retired by the operator's next act
    ///
    /// [`Self::still_true`] answers `true`, joining the group whose reason is
    /// *nothing happened*: the edit was refused before it began, so the epoch
    /// did not move, the sidecar is as it was, and there is no state for a
    /// later frame to find the sentence stale against. Deliberately **not**
    /// re-asked through `vertex_edit_preview` — that is a sidecar read, and
    /// putting it in the per-frame path that decides whether a status line is
    /// still true would pay for it sixty times a second to learn an answer that
    /// cannot change without a command.
    VertexEditRefused(crate::text::measure::VertexEditRefusal),
    /// **A node of a MARKUP shape could not be moved, added or taken
    /// away** — the other half of the operator's report of 2026-09-05:
    ///
    /// > *"I also can't edit or delete nodes of a markup shape once it is
    /// > drawn."*
    ///
    /// [`Self::VertexEditRefused`] answers for a **ce dimension** and this for
    /// a comment shape, and they are two variants rather than one for R8b rule
    /// 15's reason: the ce-dimension sentences say *"measurement"*, which is
    /// the wrong word for a polygon somebody drew as a comment, and one enum
    /// serving both would have to say something vague enough to be true of
    /// either. See [`crate::text::markup::NodeEditRefusal`].
    ///
    /// # Why a gesture with a preflight still needs a decline
    ///
    /// `canvas::annotnodes` asks `EditSession::reshape_annotation_preview`
    /// before it draws anything, so a refused edit is never previewed and never
    /// raised — the engine is never asked to refuse. That is the right design
    /// and it makes this sentence **the only report of the refusal that
    /// exists**: no action, no funnel, no `EditRefused`. Without it the
    /// operator drags a corner of a triangle out of the shape, releases, and
    /// the triangle is still a triangle with nothing anywhere saying why.
    ///
    /// # It is also raised where there was never a gesture
    ///
    ///
    /// # Retired by the operator's next act
    ///
    /// [`Self::still_true`] answers `true`, joining the group whose reason is
    /// *nothing happened*: the edit was refused before it began, so the epoch
    /// did not move and there is no state for a later frame to find the
    /// sentence stale against.
    MarkupNodeRefused(crate::text::markup::NodeEditRefusal),
    /// **The field-group deletion PREVIEW refused**, so the operator was never
    /// offered the confirmation.
    ///
    /// Its own variant rather than folded into [`Self::FieldGroupDeleteRefused`]
    /// because they are different moments with different remedies: this one
    /// means pdfcer could not work out what the deletion would remove, and the
    /// other means it worked that out, showed the operator, and was then
    /// refused. An operator who reads the second after pressing the first
    /// learns nothing about which half failed.
    FieldGroupPreviewRefused,
    /// **The field-group deletion refused**, after the operator confirmed it.
    FieldGroupDeleteRefused,
    /// **One of the operator's own stamps could not be placed** — the
    /// collection file is gone, or no longer holds that page
    /// (`OPERATOR_REQUESTS.md` O172).
    ///
    /// A decline rather than a note, and the correction is worth keeping:
    /// this route recorded both cases through `record_note` for the length of
    /// one afternoon, which draws them under **`⚑ About your last edit:`** —
    /// after a gesture that edited nothing. The reason type carries the whole
    /// argument; see `crate::text::stamps::CustomStampUnavailable`.
    CustomStampUnavailable(crate::text::stamps::CustomStampUnavailable),
    /// **A bookmark was dropped on itself, or somewhere inside itself** — the
    /// shell's own forecast of `EditError::OutlineMoveIntoOwnSubtree`,
    /// 2026-08-29.
    ///
    /// # Why a drag needs this more than a button does
    ///
    /// A drag that is released and does nothing is **this project's founding
    /// defect shape** — the sentence [`Self::Rotate`] carries about the ninth
    /// handle, and the reason the eight resize grips were the shell's longest
    /// standing complaint. A bookmark drag is worse than a grip, because the
    /// row genuinely leaves the operator's pointer during the gesture: what a
    /// silence looks like from their side is *"it went somewhere"*, and this
    /// very feature can put a bookmark somewhere they cannot see (see
    /// [`crate::text::panels::bookmarks::bookmark_move_into_collapsed`]).
    ///
    /// ⇒ So the two readings a silence invites — *"the drag did not register"*
    /// and *"it moved and I have lost it"* — are both wrong, and one of them is
    /// a state the panel can genuinely produce. R83's rule is not *gate the
    /// control*; it is **a refusal must be a sentence, never a silence.**
    ///
    /// The caret is already dimmed over such a landing before the press,
    /// which is this panel's preferred channel. This is what is owed to the
    /// operator who released anyway — and they will, because the mark is faint
    /// by design and a hand that has committed to a drag finishes it.
    ///
    /// # Recorded from the VERB, although the shell saw it coming
    ///
    /// The panel forecasts this landing — it is a question about the tree it
    /// has already drawn — and uses the forecast to draw the faintest of its
    /// three carets. It does **not** use it to skip the call.
    ///
    /// The reason is this module's own boundary. `decline` is `pub(super)`
    /// inside `crate::app` on a stated argument — *"a decline is written by the
    /// one dispatcher and read by the one bar"* — and a panel is outside it.
    /// The two ways round are worse than going through: a `record_note` from
    /// the panel would render the sentence under `⚑ About your last edit:`,
    /// which [`crate::text::status`]' own rule forbids for a decline, and
    /// widening this module would trade a real invariant for one call site.
    ///
    /// ⇒ So the move is raised, `EditSession::move_outline_item` refuses it by
    /// name (`EditError::OutlineMoveIntoOwnSubtree`, *"refused unconditionally
    /// … a cycle is a defect whatever Acrobat does"*), and
    /// `crate::app::actions::bookmarks::move_to` records this from inside the
    /// `vector_edit` closure. The guard runs before the verb plans anything, so
    /// nothing is written, no epoch moves and no undo entry appears.
    ///
    /// It also puts the authority in one place. The forecast decides what the
    /// **caret** looks like; the engine decides what **happens**. They cannot
    /// drift into disagreeing about the outcome, because only one of them
    /// produces it.
    BookmarkMoveIntoOwnSubtree,
    /// **The engine refused a bookmark move**, 2026-08-29 — the residue the
    /// shell's forecast cannot cover.
    ///
    /// [`Self::BookmarkMoveIntoOwnSubtree`]'s sibling at the other end of the
    /// call, and its own variant for exactly the reason the two field-group
    /// declines are two: *"they are different moments with different remedies"*.
    /// This one means the shell asked and pdfcer said no; the other means the
    /// shell never asked. An operator who read one after the other would learn
    /// nothing about which half refused.
    ///
    /// What can reach it: `/Encrypt`, the certification gate, and an id that
    /// stopped resolving between the frame that drew the row and the apply that
    /// moved it — which is the ordinary state one frame after an undo. None is
    /// guessable from the screen, which is
    /// [`crate::text::status::field_delete_declined_structural`]'s argument for
    /// its own verb.
    ///
    /// Recorded from **inside** the `vector_edit` closure —
    /// `record_resize_not_rebuildable`'s placement, and its stated reason:
    /// whether the engine will refuse is not knowable before the call.
    BookmarkMoveRefused,
    NothingToUndo,
    /// **`edit.redo` was invoked with an empty redo stack.**
    ///
    /// Distinct from [`Self::NothingToUndo`] for the reason the module header
    /// gives about the disclosures and the declines generally — the operator
    /// gets **one** line, and these two describe different states with
    /// different remedies. An empty undo log means nothing has been changed at
    /// all; an empty redo stack is the ordinary state of a document that has
    /// been edited and never undone, and it is *also* what a fresh edit after an
    /// undo produces, because `EditSession::commit` clears the redo stack when a
    /// new command is recorded (*"the redone future no longer exists once
    /// history diverges"*).
    ///
    /// Its live predicate is `EditSession::can_redo`, for
    /// [`Self::NothingToUndo`]'s reason and asked the same way.
    NothingToRedo,
    /// **The engine refused an edit and this shell cannot say why** —
    /// `OPERATOR_REQUESTS.md` **O116**, 2026-09-04.
    ///
    /// The **last** variant in this enum in every sense: it is what the
    /// operator is told when no other variant applies, recorded from the one
    /// funnel every document change passes through
    /// (`super::super::actions::funnel`) rather than from any verb.
    ///
    /// # It is the deferral this file's neighbours kept naming, taken
    ///
    /// Six variants above cite `vector_edit`'s error arm by name and say some
    /// version of *"before this, that residue was a **silence**"* —
    /// [`Self::FieldDeleteRefused`], [`Self::BookmarkMoveRefused`],
    /// [`Self::ResizeNotRebuildable`] among them. Each of them worded **one**
    /// verb's residue. The arm itself stayed silent for every other verb, and
    /// its own comment said so deliberately: *"That is `FEATURES.md`'s 'Worded
    /// decline' row, which wants its own decision about wording and placement;
    /// this arm is where it lands when it is taken."* This is it, taken.
    ///
    /// What made it urgent rather than tidy is that the silence became
    /// reachable on **an ordinary CAD drawing with an ordinary embedded font**:
    /// Edit ▸ Edit text arms, a caret lands, characters are typed, Enter
    /// commits, `EditSession::edit_text` refuses a symbolic font it cannot
    /// re-encode, and nothing whatever appears. That is this project's founding
    /// defect class — *"I did the thing and nothing happened and nothing said
    /// why"* — reproduced by the driven check `text_edit_on_a_real_drawing`.
    ///
    /// # It carries NO payload, unlike every other refusal variant here
    ///
    /// [`Self::TextStyle`], [`Self::Rotate`] and [`Self::Unshare`] each carry a
    /// small enum of this shell's own saying *which* refusal, because in those
    /// three cases the
    /// shell can tell: the verb has a small, closed set of engine errors and a
    /// hand-written `refusal_for` maps them. This one deliberately has none,
    /// and adding one would be the exact mistake those three narrowly avoid at
    /// scale — a second copy of `pdfcer-core`'s whole taxonomy, in this crate,
    /// drifting from theirs. [`crate::text::status::edit_declined_by_engine`]
    /// carries the full argument, including why the sentence points nowhere.
    ///
    /// ⇒ A payload arrives the day `EditError` exposes a coarse `kind()`. Until
    /// then the honest arity is zero.
    ///
    /// # Retirement: the `retire`-only class, and NOT for its usual reason
    ///
    /// [`Self::still_true`] answers `true` unconditionally, joining
    /// [`Self::SaveFailed`], [`Self::FlattenCertified`] and the rest — but the
    /// argument those variants use **does not hold here**, and copying it would
    /// be recording a fact this shell does not have.
    ///
    /// Their argument is *stability*: a document does not stop being certified,
    /// a folder does not become writable, an appearance does not become
    /// rebuildable, between one frame and the next. **This decline cannot claim
    /// that.** Its causes are unknown by construction, and the set certainly
    /// contains conditions that change under the operator — the residue
    /// [`Self::BookmarkMoveRefused`] names is *"an id that stopped resolving
    /// between the frame that drew the row and the apply that moved it, which
    /// is the ordinary state one frame after an undo"*, and that is squarely
    /// inside what an unexplained refusal can be.
    ///
    /// So it is not in the stable class. Nor can it be in the live-predicate
    /// class ([`Self::NothingToFrame`], [`Self::NothingToUndo`],
    /// [`Self::InsideForm`]), and the reason is structural rather than
    /// awkward: **that class's entry requirement is that the sentence be
    /// re-asked through the same predicate that produced it**, and there is no
    /// predicate here to re-ask. Inventing a plausible one would be the
    /// "second spelling that drifts" this module's header forbids, with a
    /// failure mode worse than drift — a guessed predicate that answered
    /// `false` would retire a **true** sentence while the operator was reading
    /// it, which is the silence all over again with an extra step.
    ///
    /// ⇒ What actually earns the `true` is the **tense**. This sentence is a
    /// report of a past moment — *that change was refused, and the document is
    /// unchanged* — and it was true when it was written whatever the frame does
    /// afterwards. That is [`Self::SaveFailed`]'s second clause, the one its
    /// docs add after the stability claim: *"and if it did, the sentence would
    /// still be a true report of what happened when the operator pressed
    /// Save."* Here that clause is not the supporting argument; it is the whole
    /// of it. A sentence in the past tense can go stale, and `retire` — the
    /// operator's next command — is what handles stale.
    EditRefused,
    /// **A reflow that did not happen, and which of its eight causes it
    /// was** — `OPERATOR_REQUESTS.md` **O127**, defect 3.
    ///
    /// All eight were **already being reported** before O127, and none of them
    /// reached the operator: four went through
    /// `crate::app::actions::record_note`, which draws under `⚑ About your last
    /// edit:` for a press where nothing happened, and four collapsed into
    /// [`Self::EditRefused`]'s nine cause-free words. His verdict was *"I
    /// haven't seen the reflow option actually work with anything when I press
    /// it."* `decline/textedit.rs` carries the whole argument.
    ///
    /// It carries the cause rather than being eight variants, on
    /// [`Self::Rotate`]'s and [`Self::TextStyle`]'s precedent: the catalog owns
    /// the wording and this enum owns only which sentence.
    Reflow(crate::text::textedit::ReflowRefusal),
    /// **Enter was pressed in text that is already on the page, where a
    /// line break cannot go** — `OPERATOR_REQUESTS.md` **O127**, defect 2.
    ///
    /// Enter means *a new line* in every draft this shell has; in an existing
    /// show operator the FILE forbids one, so it declines by name instead. It
    /// used to **commit**, silently — the operator asked *"can the enter key
    /// create new lines?"* and was answered by an edit finishing under him.
    /// See `decline/textedit.rs`.
    EnterCannotSplit(crate::text::textedit::EnterRefusal),
    /// **A cut or a paste the active MODE does not do** — 2026-09-05, and
    /// it is the second half of the defect the driven sweep found as A1.
    ///
    /// The first half was that `edit.paste` could not be *reached* in Review at
    /// all: `app::modes::capability::offers_command` refused the chord because
    /// Paste lives on the Edit tab, so an operator in the mode whose entire
    /// purpose is marking up somebody else's drawing could copy a comment and
    /// had nowhere to put it. Two independent driven checks traced
    /// `chord-not-offered id=edit.paste mode=review`.
    ///
    /// ⇒ The chord now reaches the dispatcher, which was **already** gating the
    /// effect correctly on what is on the clipboard. This variant is what makes
    /// that safe: the moment a chord is allowed through blind, every refusal it
    /// can meet has to be worded, or the fix trades *"the key does nothing and
    /// the trace says why"* for *"the key does nothing and nothing says why"* —
    /// which is worse, because the second has no trace line either.
    ///
    /// It carries [`crate::text::clipboard::ModeRefusal`] rather than being
    /// six variants, on [`Self::Rotate`]'s and [`Self::Reflow`]'s precedent: the
    /// catalog owns the wording and this enum owns only which sentence.
    ///
    /// ## Retirement: the `retire`-only class, on the TENSE argument
    ///
    /// [`Self::still_true`] answers `true`, and the reason is
    /// [`Self::EditRefused`]'s rather than [`Self::SaveFailed`]'s. It **cannot**
    /// claim stability — the operator can change the mode, and changing the mode
    /// is precisely the remedy the sentence names, so the condition it reports
    /// is one they are being invited to falsify. Nor can it be re-asked through
    /// a live predicate: the fact recorded is *what the clipboard held at the
    /// moment of the press*, and the clipboard can change under it.
    ///
    /// ⇒ What earns the `true` is that the sentence is a **report of a past
    /// moment** — *that press did nothing, and the document is unchanged* — and
    /// it was true when it was written whatever the next frame does. An
    /// operator who reads it, moves the selector and presses again retires it
    /// with that press, through `retire`, which is the honest lifetime.
    ClipboardMode(crate::text::clipboard::ModeRefusal),
    /// **A text edit the engine refused, and WHICH KIND of refusal it was**
    /// — `OPERATOR_REQUESTS.md` **O140**, 2026-09-05.
    ///
    /// The operator: *"on page 2 there is a spelling mistake — clien instead of
    /// client. if I try to edit the edit is not accepted."*
    ///
    /// # What this replaces, and it is not a silence
    ///
    /// [`Self::EditRefused`] was already reaching him — O116 shipped it on
    /// 2026-09-04 and a driven run on his own file confirms the `⊗` slot draws
    /// one frame after the refusal. What he read was *"That change was refused,
    /// and the document is unchanged."* True, complete about the document, and
    /// **silent about the one thing he wanted**: why, and whether he can do
    /// anything.
    ///
    /// ⇒ So this is not the founding defect class a second time. It is the
    /// *next* rung of it: a sentence that says nothing actionable is not the
    /// same as no sentence, and it is not good enough either.
    ///
    /// # It exists because [`Self::EditRefused`]'s stated blocker LIFTED
    ///
    /// That variant's documentation is explicit — *"It carries NO payload,
    /// unlike every other refusal variant here… adding one would be the exact
    /// mistake those three narrowly avoid at scale — a second copy of
    /// `pdfcer-core`'s whole taxonomy… ⇒ A payload arrives the day `EditError`
    /// exposes a coarse `kind()`. Until then the honest arity is zero."*
    ///
    /// **`pdfcer-core` shipped `text_edit::RefusalKind` at `b1033ab`**, in
    /// answer to this project's own request, deliberately not
    /// `#[non_exhaustive]` so a front end may match it exhaustively. The
    /// condition that variant named is met, so this one exists — and it carries
    /// a payload for [`Self::Reflow`]'s and [`Self::Rotate`]'s reason: the
    /// catalog owns the wording and this enum owns only which sentence.
    ///
    /// [`Self::EditRefused`] is **not** deleted, and that is deliberate
    /// rather than an oversight. It is the funnel's floor for **every other
    /// verb** — ~78 call sites — and only `edit_text` has been given a
    /// classifier. Deleting it would silence the other seventy-seven.
    ///
    /// # Retirement: the `retire`-only class, on the TENSE argument
    ///
    /// [`Self::still_true`] answers `true`, and the argument is
    /// [`Self::EditRefused`]'s exactly: this is a **report of a past moment** —
    /// *that commit was refused, and the document is unchanged* — true when it
    /// was written whatever the next frame does. It cannot claim stability (an
    /// operator may unlock a protected document, and `DocumentProtected` names
    /// that as the remedy), and it cannot be re-asked through a live predicate,
    /// because the fact recorded is *what the engine answered about a request
    /// that no longer exists*. The operator's next command retires it.
    EditText(crate::text::textedit::EditRefusal),
    /// **A drag on one line inside a block of text, refused because this
    /// document does not write that line's position down** —
    /// `OPERATOR_REQUESTS.md` **O188**.
    ///
    /// The operator draws a box round one label in a title block, presses inside
    /// it, drags it across the sheet. The engine will not move that line, and
    /// this is the sentence that says so before the outline is ever drawn.
    ///
    /// # Why this one, when most of `Refusal`'s variants stay silent
    ///
    /// `canvas::moving::decline`'s standing argument is good and is not being
    /// overturned: *nothing selected*, *the drag never travelled* and *no part
    /// entered* describe states the operator put themselves in and can see, and a
    /// bar that narrates the obvious stops being read at all.
    ///
    /// This refusal fails that test on every clause. The operator did not make a
    /// mistake; they cannot see the cause; and from where they sit the drag
    /// gesture is simply broken. That is this project's founding defect shape —
    /// a control that works everywhere else doing nothing here, with no way on
    /// screen to learn why — arriving on the newest gesture in the program.
    ///
    /// # No payload, and it is not [`Self::EditRefused`]'s kind of no-payload
    ///
    /// That one carries nothing because its taxonomy was too wide to copy. This
    /// one carries nothing because **there is exactly one way to be it**: it is
    /// raised from the single `RunMoveBlock::NoPositionOfItsOwn` arm of
    /// `canvas::moving::Refusal::worded`, and the block it answers has already
    /// been narrowed to one cause by the engine. A payload would have one
    /// inhabitant.
    ///
    /// The wording is
    /// [`crate::text::arrange::run_has_no_position_of_its_own`], which carries
    /// the argument for the order of its clauses, for why there are two of
    /// these variants rather than one, and the citation for the single
    /// capability claim it makes.
    ///
    /// # Retirement: the `retire`-only class, on the TENSE argument
    ///
    /// [`Self::still_true`] answers `true`, joining [`Self::ClipboardMode`] and
    /// [`Self::EditText`] — and here the tense argument is not merely the best
    /// available, it is the only safe one.
    ///
    /// The obvious alternative is a live predicate: keep the sentence only while
    /// the selection is still a run at the Part rung. ⇒ **That would delete the
    /// instruction at the instant the operator began to follow it**, because the
    /// remedy the sentence names — *press Escape to select the whole block* —
    /// changes the selection. It is [`Self::ClipboardMode`]'s trap exactly: *the
    /// condition this reports is the one the sentence asks the operator to
    /// change.*
    ///
    /// What earns the `true` is that the wording states a **property of the
    /// document** — *this line's position is not written down here* — which
    /// nothing but an edit can change. The operator's next command retires it
    /// through `retire`.
    TextRunHasNoPositionOfItsOwn,
    /// **A drag on a line that the NEXT line's position is measured from**
    /// — O188's second refusal, 2026-09-15.
    ///
    /// The twin of [`Self::TextRunHasNoPositionOfItsOwn`] and everything above
    /// applies to it unchanged: same founding-defect shape, same `retire`-only
    /// class, same reason it earns a sentence where most canvas refusals do not.
    ///
    /// # Why the pair is two variants and not one with a payload
    ///
    /// Because the payload would be read exactly once, to choose between two
    /// fixed strings, and a decline's whole contract here is *one variant, one
    /// sentence, no two alike* — which `decline::tests::no_two_declines_share_a_
    /// sentence` enforces by iterating over the variants. A payload would hide
    /// one of the two sentences from that loop, and the loop is the only thing
    /// in the program that can catch a paraphrase.
    ///
    /// **The distinction is real to the operator**, which is the test that
    /// decides this. One says *there is nothing here to change*; the other says
    /// *changing it would move something you did not select*. Same remedy,
    /// different fact, and a single variant would have to pick one of them to
    /// tell him.
    ///
    /// The wording is [`crate::text::arrange::run_would_drag_the_next_line`].
    TextRunWouldDragTheNextLine,
    /// An OCR-layer write or File ▸ Remove OCR text did nothing: a layer was
    /// already present under a Refuse policy, one went missing mid-removal, or
    /// the document has none. Worded in [`crate::text::ocr::OcrLayerRefusal`].
    OcrLayer(crate::text::ocr::OcrLayerRefusal),
    /// A layer create, change or delete wrote nothing. Worded in
    /// [`crate::text::panels::layeredit::LayerRefusal`].
    Layer(crate::text::panels::layeredit::LayerRefusal),
    /// Making a markup part of the page was refused; worded in
    /// [`crate::text::flattenannot::refused`].
    MarkupFlatten(pdfcer_core::edit::AnnotFlattenRefusalReason),
    /// Edit ▸ Forms ▸ Repair fonts found no inline font to move. Worded in
    /// [`crate::text::formfonts::nothing_to_repair`].
    FormFontsNothingToRepair,
    /// A text-tool click opened no caret. Worded by
    /// [`crate::text::textedit::refusal`]; a picture of text offers
    /// Recognise text through [`Declined::remedy`].
    TextClick(crate::editmodel::refusal::Refusal),
    /// A paste of another program's copy placed nothing. Worded in
    /// [`crate::text::ospaste::OsPasteRefusal`].
    OsPaste(crate::text::ospaste::OsPasteRefusal),
}
