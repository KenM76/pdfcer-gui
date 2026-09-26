//! # `canvas::tool` — which pointer tool the canvas is in, and the space bar that borrows it
//!
//! ## What this module is for
//!
//! `GUI_ROADMAP.md` Phase 3.2: *"There is no hand tool at all; panning is
//! middle-drag only."* This is the hand tool, and the space bar that borrows
//! it for as long as it is held.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/tool/mod.md`.

use egui::CursorIcon;

use crate::canvas::markup::MarkupKind;
use crate::canvas::measure::MeasureKind;
use crate::canvas::textedit::TextEditKind;

/// `egui::Memory` key for the operator's chosen pointer tool.
pub(super) const TOOL_MEMORY_KEY: &str = "pdfcer-canvas-tool"; // ui-text-exempt: internal memory id, never displayed

/// What the primary button does over the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasTool {
    /// Click selects, drag rubber-bands. The shipped behaviour, and the
    /// default.
    #[default]
    Select,
    /// **Direct selection** — the white arrow. Click an object and every one of
    /// its anchors appears **immediately**; click an anchor to select it,
    /// Shift-click to add, drag to move the set.
    ///
    /// # Why this exists, and what it replaces
    ///
    /// It replaces a **ritual this project invented**: reaching an anchor by
    /// clicking an object, double-clicking to descend to its subpath and
    /// double-clicking again to descend to a node — three gestures, none of
    /// them signposted, with nothing on screen at any stage saying a deeper rung
    /// exists. The operator's verdict, and it is the correct one:
    ///
    /// > *"How do I get to see the end points of an object and select them to
    /// > drag and move? This doesn't work either. … The selector should be
    /// > predictable like other programs. It seems a lot of ideas are getting
    /// > invented instead of just using the … most common method expected."*
    ///
    /// Every vector editor an operator has ever used solves this with **a second
    /// arrow**: Illustrator's Direct Selection (`A`), Inkscape's Node tool
    /// (`N`), Figma's double-click-into-vector-edit, CorelDRAW's Shape tool
    /// (`F10`). The tool *is* the rung. There is no hidden state to descend
    /// through and no way to be at a rung you did not choose.
    ///
    /// # What the rung ladder is still for
    ///
    /// [`crate::canvas::selection::SelectionLevel`] is a good **model**, and it
    /// is what `move_node`, `move_subpath` and `move_nodes` are addressed
    /// through. Arming this tool *puts* the operator at the Node rung instead of
    /// requiring them to find their way there. Double-click descent still works
    /// and is still tested; it is simply not the only route, and being the only
    /// route is what made it a trap.
    Node,
    /// Click does nothing, drag moves the paper under the viewport.
    Hand,
    /// Drag authors a markup annotation of the carried kind.
    ///
    /// # One variant carrying a kind, not one variant per shape
    ///
    /// All markup kinds live in the carried [`MarkupKind`] rather than becoming
    /// separate `CanvasTool` entries, because separate entries would put
    /// mutually-exclusive states into a type that can express all their
    /// combinations.
    ///
    /// That last clause is the whole argument, and it is a statement about
    /// *types* rather than about tidiness: the operator is drawing exactly one
    /// shape, so a type that can say `Rectangle` and `Ellipse` at once — which
    /// four booleans, or four variants plus a "which is active" rule spread
    /// across call sites, both can — is a type whose illegal states have to be
    /// prevented by discipline. Carrying the kind makes them unrepresentable.
    ///
    /// It also makes the *tool* branch total: every rule this enum owns —
    /// [`Self::pans_with_primary`], [`Self::cursor`], the press-kind decision in
    /// [`crate::canvas::gesture::press_kind`] — is written once for markup as a
    /// whole and cannot be written four times and forgotten once.
    ///
    /// **Changing the kind mid-drag is not possible**, and it costs nothing to
    /// guarantee. Arming is a command, commands are dispatched between frames,
    /// and a drag in flight is owned by
    /// [`crate::canvas::gesture::GestureState`], which carries the kind it
    /// started with on its own `DragKind` — so a kind change mid-drag cannot
    /// reach the drag at all. A design that held the kind outside the gesture
    /// would have to discard an in-progress drag by hand; this one gets the
    /// property from the gesture machine's "a drag keeps the kind it started
    /// with" rule.
    Markup(MarkupKind),
    /// Clicks author a **dimension** of the carried kind.
    ///
    /// One variant carrying a kind, for the argument spelled out on
    /// [`MeasureKind`] — the same argument `Markup` above makes. The
    /// alternative it avoids is three separate `CanvasTool` variants for the
    /// measure tools plus five helper predicates to ask which is active.
    ///
    /// # Unlike every other tool, this one works on CLICKS
    ///
    /// A ce dimension is picked, not dragged: point A, point B, and a third
    /// click saying how far off the geometry the dimension sits. So
    /// [`crate::canvas::gesture::press_kind`] returns a
    /// [`crate::canvas::gesture::PressMeaning`] with **no drag and a live
    /// click** for this tool, and the pick state machines in
    /// [`crate::canvas::measure::pick`] advance one click at a time.
    ///
    /// That is why the mode gate had to grow two answers rather than one. A
    /// gate that suppressed the click whenever it suppressed the drag would be
    /// correct in Read — where neither means anything — and would silently
    /// break **Review**, whose whole purpose is placing dimensions on a page
    /// whose content is not the reviewer's to select.
    Measure(MeasureKind),
    /// Places a **text-bearing** annotation of the carried kind — a text box,
    /// a sticky note or a stamp.
    ///
    /// # A fourth family, and why it is not `Markup(kind)`
    ///
    /// `Markup` and this one look alike from outside — both draw a rectangle
    /// on the page and both end in an annotation — and they are different in
    /// the one way that matters to this enum: **what completes the gesture.**
    ///
    /// A markup band authors on **release**, from geometry alone. These cannot:
    /// releasing produces an empty box, and an empty box is not an annotation.
    /// The release opens a dialog and the operator types; nothing reaches the
    /// document until they accept. Folding them into `Markup(kind)` would put
    /// two different completion rules behind one variant, so every site that
    /// asks *"does the release author?"* would need a second predicate to ask
    /// *"…but not for these three"* — which is the shape
    /// `MarkupKind`'s own docs reject for mutually exclusive states.
    ///
    /// The kind is carried for the same reason `Markup` carries one: the
    /// operator is placing exactly one annotation, and a type that could say
    /// *text box* and *sticky* at once would need discipline to keep honest.
    TextAnnot(crate::canvas::textannot::TextAnnotKind),
    /// A drag sweeps a **range of text**, and a click resolves one — in every
    /// mode, including the ones whose primary button is otherwise the content
    /// marquee.
    ///
    /// # The variant that carries nothing, and why that is the point
    ///
    /// `Markup` and `Measure` each carry a kind because the operator is drawing
    /// exactly one shape or placing exactly one dimension, and a type that could
    /// say two at once would need discipline to keep honest. There is no
    /// corresponding choice here: text selection has **one** meaning, so there is
    /// nothing to carry, and the module header's admission bar — *"arrives with
    /// its own state"* — is met by state that lives everywhere except in this
    /// enum. The range is on [`crate::app::state::OpenDoc`] beside the object
    /// selection; the sweep's anchor is re-derived from the press origin every
    /// frame; there is no caret. **Armed or not armed** is the whole of what has
    /// to persist, and that is exactly one bit in a value this module already
    /// stores in `egui::Memory`.
    ///
    /// # What arming it changes, and what it deliberately does not
    ///
    /// It changes exactly one predicate:
    /// [`crate::canvas::textsel::takes_the_press`] gains a disjunct, so a press
    /// means text when this is armed **or** under the pre-existing rule (the
    /// select tool, in a mode that cannot select content). Read and Review are
    /// therefore **unchanged** — their select tool already swept text, and an
    /// operator who never presses this control will not notice it exists. What
    /// changes is Edit, where the select tool keeps the content marquee and this
    /// tool is how an editor reaches a text range at all.
    ///
    /// It changes **no capability**. Selecting text authors nothing — it reads
    /// the page and writes to the clipboard, which is the operator's own
    /// *copying is not authoring* ruling — so this tool is
    /// permitted in every mode, and [`retire_forbidden`] says so explicitly
    /// rather than by omission.
    ///
    /// # It is exclusive with the content marquee by PRECEDENCE, not by
    /// construction
    ///
    /// This is the one property this variant genuinely weakens, so it is stated
    /// here rather than left to be discovered. A pure mode rule has the two
    /// meanings reading the same flag on both sides of one branch —
    /// `edit_content` true means content, false means text — so no state can
    /// produce both and there is no ordering to get wrong. With this tool armed
    /// in Edit, *both* underlying facts are true: the mode can select content,
    /// and the operator has asked for text.
    ///
    /// The tie is broken where every other armed tool's is, in
    /// [`crate::canvas::gesture::press_kind`], by the rung that already reads
    /// **an armed tool takes the press**. That is not a new rule invented for
    /// this variant; it is the rule `Markup` relies on, and the alternative —
    /// leaving the mode branch to win — is a control that arms, shows an I-beam,
    /// and marquees objects.
    ///
    /// One consequence follows and is real: an object selection and a text
    /// selection can both be non-empty at once, in Edit. That is why they are
    /// two fields on the document rather than one enum. ⚠ `canvas::textsel` §3
    /// argues the two are mutually exclusive; this variant makes that untrue,
    /// and the shape is right for a reason that argument gets wrong. See
    /// [`crate::canvas::keys`]'s rung 5, which orders them.
    Text,
    /// A click puts a **caret** on the page — in an existing run for
    /// [`TextEditKind::Edit`], at a fresh origin for [`TextEditKind::Add`] — and
    /// the keyboard then edits the page's own content.
    ///
    /// # The argument the module header demands of every new variant
    ///
    /// Made here against the bar the header sets — *"arrives with its own
    /// state"* — and against the sharper objection text editing specifically
    /// raises: that it is a caret in a re-laid-out box and would drag a whole
    /// subsystem's state through this type.
    ///
    /// **The bar is cleared**, in the same five columns
    /// [`Self::Text`]'s table uses:
    ///
    /// | it arrives with | where |
    /// |---|---|
    /// | a draft type, with its own page and kind synchronisation | [`crate::canvas::textedit::Draft`] |
    /// | a [`PressMeaning`](crate::canvas::gesture::PressMeaning) — a live click and **no drag** | [`crate::canvas::gesture::press_kind`]'s first rung |
    /// | a resolver — one hit test producing the run, through the same bridge the text sweep uses | `canvas::textedit::click` |
    /// | a commit path, and a real one: two `Action`s and two `EditSession` verbs | `Action::CommitTextEdit`, `Action::CommitAddText` |
    /// | a refusal vocabulary of its own | [`crate::canvas::textedit::Refusal`] |
    ///
    /// **The objection is answered rather than outvoted.** It is not "text
    /// editing is big"; it is *"it would drag a whole subsystem's state through
    /// this type"* — a claim about **this enum**, not about the feature. And the
    /// state does not come through. What crosses is one [`TextEditKind`], which
    /// is exactly what [`Self::Markup`] and [`Self::Measure`] each carry and one
    /// value more than [`Self::Text`] carries. The draft, the caret, the anchor
    /// and the original text live in `egui::Memory` — where `canvas::measure`
    /// already keeps a half-finished pick, on the reasoning that *"a
    /// half-finished pick is not part of the document and a document saved
    /// mid-gesture must not carry one"*, which is true of a half-typed word in
    /// exactly the same way and for exactly the same reason.
    ///
    /// The second clause of the objection — *"a caret in a re-laid-out box"* —
    /// is a correct prediction about **ghost text**, and it is answered by not
    /// drawing any: see
    /// `canvas::textedit::preview`, which paints a caret and an extent bracket
    /// and no glyphs, and argues why a better ghost is the wrong fix rather than
    /// a deferred one.
    ///
    /// # What arming it changes
    ///
    /// One predicate and one rung. [`crate::canvas::gesture::press_kind`] gains
    /// a first rung — this tool takes the click and leaves the drag alone,
    /// beside the measure and vertex-markup rungs it is shaped like — and
    /// nothing else moves. In particular
    /// [`crate::canvas::textsel::takes_the_press`] is **untouched**: it asks
    /// [`Self::is_text`], which is `matches!(self, Self::Text)` and is therefore
    /// false here by construction, not by a condition someone added. So the text
    /// *sweep* and the text *caret* cannot both claim one press, and Read and
    /// Review are unchanged — the tool cannot be armed there at all.
    ///
    /// # It changes a capability, where `Text` changed none
    ///
    /// Selecting text authors nothing; this authors page content. So both
    /// dispatch arms decline unless `Capabilities::edit_content`, and
    /// [`retire_forbidden`] disarms it on the way into a mode that cannot
    /// author — which also abandons any draft, because a draft that survived
    /// into Read would be a keystroke buffer aimed at a document the mode says
    /// is not the operator's to change.
    ///
    /// [`TextEditKind`]: crate::canvas::textedit::TextEditKind
    /// [`TextEditKind::Edit`]: crate::canvas::textedit::TextEditKind::Edit
    /// [`TextEditKind::Add`]: crate::canvas::textedit::TextEditKind::Add
    TextEdit(TextEditKind),
    /// **Placing a new form field**, armed from Edit ▸ Forms.
    ///
    /// Geometrically identical to a markup rectangle — arm, put a rectangle
    /// on a page, commit once — which is why it borrows
    /// [`crate::canvas::markup::band`]'s drag rather than growing a second one.
    ///
    /// It differs in exactly one way, and the difference is the feature:
    /// **the release authors nothing.** It opens a dialog, and the field exists
    /// only once the operator presses OK. That is what makes Escape free and
    /// what stops a mis-drag leaving a stray control on the page — which
    /// matters more here than for markup, because an unwanted annotation is
    /// obvious and an unwanted invisible form field is not.
    Form(crate::canvas::formfield::FormFieldKind),
    /// **A window is waiting for the operator to point at the page** —
    /// `OPERATOR_REQUESTS.md` O66.
    ///
    /// > *"anything we are inserting like this should have an option in its
    /// > dialogue box to place it with the mouse instead of by positional
    /// > co-ordinates."*
    ///
    /// It clears this enum's admission bar the same way [`Self::Form`] does: it
    /// carries exactly one value, it has a `DragKind`, a `GestureOutcome`, a
    /// press meaning of its own, and a commit path — which is the requesting
    /// dialog's own Insert rather than anything on this canvas.
    ///
    /// The dialog that armed it is **not on screen** while this is active,
    /// and that is derived rather than stored — see
    /// [`crate::canvas::placing`]'s header for why a stored flag rebuilds a
    /// stranding bug the Set-scale round trip already has.
    Place(crate::canvas::placing::PlaceKind),
}

impl CanvasTool {
    /// Whether a primary-button drag pans the view rather than reaching the
    /// gesture machine.
    #[must_use]
    pub fn pans_with_primary(self) -> bool {
        matches!(self, Self::Hand)
    }

    /// The cursor this tool shows, or `None` to leave the cursor to whatever
    /// else the canvas is doing with it (a grip, a marquee, a move drag).
    #[must_use]
    pub fn cursor(self, dragging: bool) -> Option<CursorIcon> {
        match self {
            // The same crosshair a markup rectangle uses, because it is the
            // same gesture: the operator is about to put a rectangle on the
            // page. A different cursor would imply a different act.
            Self::Form(_) => Some(CursorIcon::Crosshair),
            // Crosshair, joining the Form / Markup / TextAnnot group for
            // their stated reason: the gesture is *put something here*, and a
            // crosshair is the one cursor that says so without implying what.
            Self::Place(_) => Some(CursorIcon::Crosshair),
            Self::Select => None,
            // **The same answer as `Select` — `None` — and that is the whole
            // point.** The Node tool's feedback is the anchors it draws, not a
            // cursor, and returning an icon here would suppress the anchor and
            // handle cursors underneath exactly as `Text`'s I-beam suppresses
            // the grip cursors (see the note above). The operator learns which
            // tool is armed from the Tool panel and from the marks on the page,
            // which is where every other editor puts that information.
            Self::Node => None,
            Self::Hand if dragging => Some(CursorIcon::Grabbing),
            Self::Hand => Some(CursorIcon::Grab),
            // The text-annotation tools join the crosshair group. They place
            // something on the page, which is what the crosshair says, and the
            // fact that the placing is followed by typing changes nothing
            // about the gesture the cursor is describing.
            Self::TextAnnot(_) => Some(CursorIcon::Crosshair),
            // A crosshair for both authoring tools, and for the same reason:
            // it says "this canvas places something now" and it suppresses the
            // grip cursors, which is correct because a click with a measure
            // tool armed picks a point rather than grabbing a handle.
            Self::Markup(_) | Self::Measure(_) => Some(CursorIcon::Crosshair),
            // …and an I-beam for the one tool that places nothing. The pointer
            // says which of the two things a drag on this page is about to do,
            // which is the whole reason the tool exists.
            Self::Text => Some(CursorIcon::Text),
            // …and the same I-beam for the caret tool, which is the one place
            // this file gives two variants one answer on purpose. The pointer
            // says *what a press on this page is about to do*, and for both of
            // these it is about to do something to text — sweep it or put a
            // caret in it. Acrobat, Inkscape and SolidWorks all show one I-beam
            // for both, and a second glyph invented to distinguish them would be
            // a distinction the operator has to learn in order to read something
            // the pressed ribbon control already tells them.
            Self::TextEdit(_) => Some(CursorIcon::Text),
        }
    }

    /// Which markup kind is armed, if any.
    #[must_use]
    pub fn markup_kind(self) -> Option<MarkupKind> {
        match self {
            Self::Markup(kind) => Some(kind),
            _ => None,
        }
    }

    /// Which measure kind is armed, if any.
    #[must_use]
    pub fn measure_kind(self) -> Option<MeasureKind> {
        match self {
            Self::Measure(kind) => Some(kind),
            _ => None,
        }
    }

    /// Which form-field kind is armed, if any.
    ///
    /// [`Self::markup_kind`]'s sibling, for the ghost outline the canvas draws
    /// under the pointer while a form tool is up — `OPERATOR_REQUESTS.md` O203.
    #[must_use]
    pub fn form_kind(self) -> Option<crate::canvas::formfield::FormFieldKind> {
        match self {
            Self::Form(kind) => Some(kind),
            _ => None,
        }
    }

    /// Which placement is armed, if any — `OPERATOR_REQUESTS.md` O66.
    #[must_use]
    pub fn place_kind(self) -> Option<crate::canvas::placing::PlaceKind> {
        match self {
            Self::Place(kind) => Some(kind),
            _ => None,
        }
    }

    /// **Whether the text tool is armed.**
    #[must_use]
    pub fn is_text(self) -> bool {
        matches!(self, Self::Text)
    }

    /// Whether this tool's subject is **anchors** rather than whole objects.
    #[must_use]
    pub fn is_node(self) -> bool {
        matches!(self, Self::Node)
    }

    /// **Which text-edit kind is armed, if any.**
    #[must_use]
    pub fn text_edit_kind(self) -> Option<TextEditKind> {
        match self {
            Self::TextEdit(kind) => Some(kind),
            _ => None,
        }
    }
}

/// How a tool is **chosen** — the memory-backed selection, the toggles, the
/// arming entry points and the mode-driven retirement. Its header carries the
/// seam argument.
pub mod arm;

pub use arm::*;

#[cfg(test)]
mod tests {
    // Imported HERE rather than at module scope. Every production user of
    // these two is in `arm`, so a module-level import would be unused in the
    // non-test build and clippy would refuse it.
    // The tests still exercise the arming API through the re-export, which is
    // the point: they test the module's surface, not its file layout.
    use crate::app::modes::Capabilities;
    use egui::Key;

    use super::*;
    use egui::{Context, Event, Modifiers, RawInput};

    /// **Space borrows the hand and gives it back** — the requirement,
    /// stated as the pure rule it is implemented as.
    #[test]
    fn space_borrows_the_hand_and_releasing_returns_the_previous_tool() {
        assert_eq!(resolve(CanvasTool::Select, false), CanvasTool::Select);
        assert_eq!(resolve(CanvasTool::Select, true), CanvasTool::Hand);
        assert_eq!(resolve(CanvasTool::Select, false), CanvasTool::Select);
    }

    /// Holding space while the hand tool is already chosen changes nothing,
    /// and releasing it does not drop the operator back into Select.
    #[test]
    fn space_over_the_hand_tool_is_a_no_op_in_both_directions() {
        assert_eq!(resolve(CanvasTool::Hand, true), CanvasTool::Hand);
        assert_eq!(resolve(CanvasTool::Hand, false), CanvasTool::Hand);
    }

    /// Only the hand pans, and each tool's cursor is what it should be — the
    /// two halves of the branch, asserted together so a future fourth tool
    /// cannot answer one and forget the other.
    #[test]
    fn only_the_hand_pans_and_each_tool_paints_its_own_cursor() {
        assert!(!CanvasTool::Select.pans_with_primary());
        assert!(CanvasTool::Hand.pans_with_primary());
        assert_eq!(CanvasTool::Select.cursor(false), None);
        assert_eq!(CanvasTool::Select.cursor(true), None);
        assert_eq!(CanvasTool::Hand.cursor(false), Some(CursorIcon::Grab));
        assert_eq!(CanvasTool::Hand.cursor(true), Some(CursorIcon::Grabbing));
        for &kind in MarkupKind::ALL {
            let tool = CanvasTool::Markup(kind);
            assert!(!tool.pans_with_primary(), "{kind:?} must not pan");
            assert_eq!(tool.cursor(false), Some(CursorIcon::Crosshair), "{kind:?}");
            assert_eq!(tool.cursor(true), Some(CursorIcon::Crosshair), "{kind:?}");
            assert_eq!(tool.markup_kind(), Some(kind));
        }
        assert_eq!(CanvasTool::Select.markup_kind(), None);
        assert_eq!(CanvasTool::Hand.markup_kind(), None);
    }

    /// **The text tool does not pan, shows an I-beam in both states, and is
    /// the only tool `is_text` answers `true` for.**
    #[test]
    fn the_text_tool_sweeps_rather_than_pans_and_says_so_with_the_pointer() {
        let text = CanvasTool::Text;
        assert!(!text.pans_with_primary());
        assert_eq!(text.cursor(false), Some(CursorIcon::Text));
        assert_eq!(text.cursor(true), Some(CursorIcon::Text));
        assert!(text.is_text());
        assert_eq!(text.markup_kind(), None);
        assert_eq!(text.measure_kind(), None);

        for other in [
            CanvasTool::Select,
            CanvasTool::Hand,
            CanvasTool::Markup(MarkupKind::Rectangle),
            CanvasTool::Measure(MeasureKind::Linear),
        ] {
            assert!(!other.is_text(), "{other:?} is not the text tool");
        }

        // Rung 1 of `cursor_for`, on hover with no button down and no gesture
        // — which is the whole difference the tool makes to the pointer, and the
        // half the un-armed rule could not pay for. It also outranks a hovered
        // grip, which is load-bearing in Edit: a content selection can be on the
        // page while this tool is armed, and `press_kind` gives the armed tool
        // the press, so a grip that kept its resize cursor would promise a
        // gesture already decided against.
        assert_eq!(
            cursor_for(CanvasTool::Text, None, None, false, true),
            Some(CursorIcon::Text),
            "an armed text tool paints the I-beam ON HOVER, before any drag"
        );
        assert_eq!(
            cursor_for(
                CanvasTool::Text,
                None,
                Some(crate::canvas::handles::Grip::SouthEast),
                true,
                true,
            ),
            Some(CursorIcon::Text),
            "…and it suppresses the grip cursors, as every armed tool does"
        );
        assert_eq!(
            cursor_for(CanvasTool::Text, None, None, false, false),
            None,
            "but it does not claim the pointer over the ribbon"
        );
    }

    /// **Pressing the armed Text button again retires it; pressing it from
    /// another tool takes it.**
    #[test]
    fn toggling_the_text_tool_arms_it_retires_it_and_takes_it_from_another_tool() {
        let ctx = Context::default();
        assert_eq!(selected(&ctx), CanvasTool::Select);

        assert_eq!(toggle_text(&ctx), CanvasTool::Text);
        assert_eq!(selected(&ctx), CanvasTool::Text);
        assert_eq!(
            toggle_text(&ctx),
            CanvasTool::Select,
            "a second press retires"
        );

        arm_markup(&ctx, MarkupKind::Ellipse);
        assert_eq!(
            toggle_text(&ctx),
            CanvasTool::Text,
            "from a pen, Text takes the tool rather than returning to Select"
        );
        // …and Hand still takes it back from Text, which is `toggle_hand`'s
        // matching arm and would silently answer `Select` if the variant had been
        // added to the wrong side of that match.
        assert_eq!(toggle_hand(&ctx), CanvasTool::Hand);
        assert_eq!(toggle_hand(&ctx), CanvasTool::Select);
    }

    /// **Space borrows the hand out of the text tool and gives it back**, and
    /// **Escape does not claim the text tool.**
    #[test]
    fn space_borrows_the_hand_out_of_the_text_tool_and_escape_leaves_it_alone() {
        assert_eq!(resolve(CanvasTool::Text, true), CanvasTool::Hand);
        assert_eq!(resolve(CanvasTool::Text, false), CanvasTool::Text);

        let ctx = Context::default();
        select(&ctx, CanvasTool::Text);
        assert!(
            !disarm_markup(&ctx),
            "the markup claimant must not take Escape for a tool that is not a pen"
        );
        assert!(!disarm_measure(&ctx));
        assert_eq!(
            selected(&ctx),
            CanvasTool::Text,
            "and the tool is still armed afterwards"
        );
    }

    /// **No mode retires the text tool** — the `retire_forbidden` decision,
    /// asserted over every capability combination rather than over the three
    /// shipped modes.
    #[test]
    fn the_text_tool_is_permitted_in_every_mode_where_a_pen_is_not() {
        for markup in [false, true] {
            for measure in [false, true] {
                for content in [false, true] {
                    let caps = Capabilities {
                        edit_content: content,
                        author_markup: markup,
                        author_measure: measure,
                    };
                    let ctx = Context::default();
                    select(&ctx, CanvasTool::Text);
                    assert!(
                        !retire_forbidden(&ctx, caps),
                        "the text tool authors nothing, so {caps:?} has nothing to forbid"
                    );
                    assert_eq!(selected(&ctx), CanvasTool::Text);

                    select(&ctx, CanvasTool::Markup(MarkupKind::Rectangle));
                    assert_eq!(
                        retire_forbidden(&ctx, caps),
                        !markup,
                        "a pen IS retired by a mode that cannot author markup: {caps:?}"
                    );
                }
            }
        }
    }

    /// **The cursor precedence**, all four rungs, in one test that would
    /// have caught each of them being reordered.
    #[test]
    fn the_cursor_precedence_runs_tool_then_gesture_then_grip() {
        use crate::canvas::gesture::{DragKind, MarqueeIntent};
        use crate::canvas::handles::Grip;

        // 1. The armed tool wins over a gesture AND a hovered grip.
        assert_eq!(
            cursor_for(
                CanvasTool::Markup(MarkupKind::Arrow),
                Some(DragKind::Move),
                Some(Grip::SouthEast),
                true,
                true,
            ),
            Some(CursorIcon::Crosshair),
        );
        assert_eq!(
            cursor_for(CanvasTool::Hand, Some(DragKind::Move), None, true, true),
            Some(CursorIcon::Grabbing),
        );
        // …but only while the pointer is over the canvas or a button is down,
        // so the hand does not claim the cursor over the ribbon.
        assert_eq!(cursor_for(CanvasTool::Hand, None, None, false, false), None);

        // 2. With the select tool, a gesture in flight wins over a grip the
        //    pointer happens to be over.
        assert_eq!(
            cursor_for(
                CanvasTool::Select,
                Some(DragKind::Marquee(MarqueeIntent::Select)),
                Some(Grip::SouthEast),
                true,
                true,
            ),
            Some(CursorIcon::Crosshair),
        );
        assert_eq!(
            cursor_for(
                CanvasTool::Select,
                Some(DragKind::Resize(Grip::NorthWest)),
                Some(Grip::SouthEast),
                true,
                true,
            ),
            Some(Grip::NorthWest.cursor()),
            "an in-flight resize keeps ITS grip's cursor, not the hovered one"
        );

        // 3. Then a hovered grip, and 4. then nothing.
        assert_eq!(
            cursor_for(CanvasTool::Select, None, Some(Grip::East), false, true),
            Some(Grip::East.cursor()),
        );
        assert_eq!(
            cursor_for(CanvasTool::Select, None, None, false, true),
            None
        );
    }

    /// **Pressing an armed markup button again retires the tool; pressing a
    /// different one changes kind.**
    #[test]
    fn arming_a_markup_kind_toggles_that_kind_and_switches_between_kinds() {
        let ctx = Context::default();
        assert_eq!(selected(&ctx), CanvasTool::Select);

        assert_eq!(
            arm_markup(&ctx, MarkupKind::Rectangle),
            CanvasTool::Markup(MarkupKind::Rectangle)
        );
        // A different kind re-arms rather than retiring.
        assert_eq!(
            arm_markup(&ctx, MarkupKind::Arrow),
            CanvasTool::Markup(MarkupKind::Arrow)
        );
        assert_eq!(selected(&ctx), CanvasTool::Markup(MarkupKind::Arrow));
        // The same kind again retires.
        assert_eq!(arm_markup(&ctx, MarkupKind::Arrow), CanvasTool::Select);
        assert_eq!(selected(&ctx), CanvasTool::Select);
    }

    /// **Escape's claimant reports whether it took the key.**
    #[test]
    fn disarming_markup_reports_whether_there_was_anything_to_disarm() {
        let ctx = Context::default();
        assert!(!disarm_markup(&ctx), "nothing armed: the key is not ours");

        arm_markup(&ctx, MarkupKind::Ellipse);
        assert!(disarm_markup(&ctx));
        assert_eq!(selected(&ctx), CanvasTool::Select);
        assert!(!disarm_markup(&ctx), "and it is not claimed twice");

        // The hand tool is not ours to retire either — Escape must not silently
        // put an operator who chose the hand back into Select.
        select(&ctx, CanvasTool::Hand);
        assert!(!disarm_markup(&ctx));
        assert_eq!(selected(&ctx), CanvasTool::Hand);
    }

    /// **Space borrows the hand out of the markup tool and gives it back.**
    #[test]
    fn space_borrows_the_hand_out_of_the_markup_tool_and_returns_the_kind() {
        let armed = CanvasTool::Markup(MarkupKind::Rectangle);
        assert_eq!(resolve(armed, true), CanvasTool::Hand);
        assert_eq!(resolve(armed, false), armed);
    }

    /// The chosen tool survives a frame, and the toggle alternates rather
    /// than latching.
    #[test]
    fn the_chosen_tool_persists_and_the_toggle_alternates() {
        let ctx = Context::default();
        assert_eq!(selected(&ctx), CanvasTool::Select);
        assert_eq!(toggle_hand(&ctx), CanvasTool::Hand);
        assert_eq!(selected(&ctx), CanvasTool::Hand);
        assert_eq!(toggle_hand(&ctx), CanvasTool::Select);
        select(&ctx, CanvasTool::Hand);
        assert_eq!(selected(&ctx), CanvasTool::Hand);
    }

    /// **A focused text field keeps the space bar**, so typing a page
    /// number into the status bar does not pan the drawing under the
    /// operator.
    #[test]
    fn a_focused_text_field_keeps_the_space_bar() {
        let ctx = Context::default();
        let mut buffer = String::from("37");

        // Frame 1: build the field and take focus.
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            // escape-disposition: not-a-surface — a field this test builds to put egui
            // into a known focus state. It is never drawn for an operator.
            ui.add(egui::TextEdit::singleline(&mut buffer))
                .request_focus();
        });

        // Frame 2: the field holds focus and space is down.
        let input = RawInput {
            events: vec![Event::Key {
                key: Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        };
        let mut typing = false;
        let mut held = true;
        let _ = ctx.run_ui(input, |ui| {
            // escape-disposition: not-a-surface — a field this test builds to put egui
            // into a known focus state. It is never drawn for an operator.
            ui.add(egui::TextEdit::singleline(&mut buffer));
            // typing-guard-exempt: a TEST asserting the harness actually reached
            // the focused state. Reading the raw egui answer is the point - a
            // test that asked `composing()` could not tell a focused widget from
            // a canvas draft, and the thing being proved is that the widget half
            // is reachable at all. D1 shipped because its test could not reach it.
            typing = ui.ctx().text_edit_focused();
            held = space_held(ui.ctx());
        });

        assert!(
            typing,
            "the test is vacuous unless a TEXT field really holds focus"
        );
        assert!(!held, "a focused text field must keep the space bar");
        assert_eq!(
            resolve(selected(&ctx), held),
            CanvasTool::Select,
            "and the tool must therefore not have changed"
        );
    }

    /// With no text field in the way, a held space bar really does reach the
    /// canvas — the other direction of the guard above, without which the
    /// previous test would pass on a build where space-pan never worked at
    /// all.
    #[test]
    fn a_held_space_bar_reaches_the_canvas_when_nothing_is_typing() {
        let ctx = Context::default();
        let input = RawInput {
            events: vec![Event::Key {
                key: Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        };
        let mut tool = CanvasTool::Select;
        let _ = ctx.run_ui(input, |ui| {
            tool = active(ui.ctx());
        });
        assert_eq!(tool, CanvasTool::Hand);
    }
}
