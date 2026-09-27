//! # `app::keyboard` — the keyboard map, and the guard that must not be wrong
//!
//! ## `DEFECTS.md` D1 — read this before touching the guard
//!
//! The old GUI's keyboard map guarded its unmodified-key bindings with:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/keyboard.md`.

use pdfcer_gui_base::keyscripted as scripted;

pub use scripted::scripted_press;

use egui::{Context, Key};
use egui_shell::manifest::Keymap;

use crate::app::actions::Action;

use pdfcer_gui_base::keychord::parse_chord;

/// **The chords this module binds outright — viewer navigation — and every
/// spelling a manifest might use for each.**
pub const OWNED: &[(Key, &[&str])] = &[
    (Key::Plus, &["Ctrl+Plus", "Ctrl++"]),
    (Key::Equals, &["Ctrl+Equals", "Ctrl+="]),
    (Key::Minus, &["Ctrl+Minus", "Ctrl+-"]),
    // ui-text-exempt: chord spellings compared against a manifest keymap, never displayed
    (Key::PageDown, &["PageDown", "Page Down"]),
    // ui-text-exempt: chord spellings compared against a manifest keymap, never displayed
    (Key::PageUp, &["PageUp", "Page Up"]),
    (Key::Home, &["Home"]),
    (Key::End, &["End"]),
];

/// Read this frame's key presses and turn them into actions.
pub fn collect(ctx: &Context, page_count: Option<usize>) -> Vec<Action> {
    let mut actions = Vec::new();
    let Some(page_count) = page_count else {
        return actions;
    };

    // D1. `text_edit_focused()`, NEVER `egui_wants_keyboard_input()`.
    // See the module docs for the whole story; the one-line version is that
    // the latter means "any widget has focus", the canvas takes focus on
    // click, and the difference cost the operator the Delete key and all
    // keyboard page navigation from the first click onward.
    // …and a CANVAS text draft counts as typing too, which
    // `text_edit_focused()` alone cannot see.
    //
    // These bindings are page keys, not characters, so the harm is not a
    // mistyped bracket - it is `PageDown` under a half-typed word. The draft is
    // painted against the page on screen, so a page turn the operator did not
    // ask for takes their caret off it mid-sentence. The chord dispatcher in
    // [`commands`] asks the same two questions for its own reason; the
    // predicate is shared, the argument is not.
    //
    // The caret this shell paints on the page is deliberately **not** an
    // `egui::TextEdit` — `canvas::textedit`'s header gives the reason, and it is
    // a good one: the caret sits in PDF space at the glyphs' own scale, which a
    // floating widget cannot do. The cost is that egui has no focused text field
    // to report, so the D1 guard above answers `false` for an operator who is
    // visibly mid-word.
    //
    // Two bindings in the built-in keymap are **bare characters** — `[` and `]`,
    // on `pages.rotate_left` / `pages.rotate_right`. Without this term, typing a
    // bracket into a draft rotates the page instead of inserting the character,
    // and the operator gets no bracket and a rotated drawing. Any bare-character
    // binding added later inherits the fix rather than re-discovering the bug.
    //
    // Asked as "is a draft in flight" rather than "is a caret tool armed",
    // because an armed tool that has not been clicked yet owns no keystrokes —
    // the page keys must keep working right up until the caret is placed.
    let typing = crate::canvas::textedit::composing(ctx);

    // **The zoom chords match the modifiers CARRIED BY THE KEY EVENT, not
    // the frame's.** [`commands`] carries the whole argument and it applies
    // here identically: `InputState::modifiers` is the state as of the END of
    // the frame, `Event::Key` carries the state as of the KEYSTROKE, and the
    // two disagree whenever one frame swallows the press and the modifier's
    // release together. A frame is long exactly when a dense sheet is
    // rasterizing — which is the moment the operator is reaching for zoom, so
    // the failure concentrates on the keystroke that would have relieved it.
    //
    // ⚠ A test that builds a `RawInput` with the same `Modifiers` in the event
    // and in the frame cannot see the difference, because in that input the
    // two clocks agree by construction. The regression test for this sets them
    // apart deliberately.
    //
    // `command` rather than `ctrl`: it is Ctrl everywhere and Cmd on macOS,
    // which is what a Mac operator's fingers expect. pdfcer ships on Windows
    // first, but a hard-coded `ctrl` is a portability bug that costs nothing
    // to avoid now and is tedious to find later.
    //
    // The page keys below stay on `key_pressed`: they take no modifier, so
    // they have no second clock to disagree with.
    let (zoom_in, zoom_out, pressed) = ctx.input(|i| {
        let chord = |wanted: &[Key]| {
            i.events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } if modifiers.command && wanted.contains(key)
                )
            })
        };
        (
            chord(&[Key::Plus, Key::Equals]),
            chord(&[Key::Minus]),
            [Key::PageDown, Key::PageUp, Key::Home, Key::End].map(|k| i.key_pressed(k)),
        )
    });
    let [page_down, page_up, home, end] = pressed;

    if zoom_in {
        actions.push(Action::ZoomIn);
    }
    if zoom_out {
        actions.push(Action::ZoomOut);
    }

    // The unmodified keys — the ones D1 suppressed. Installed only when a
    // text field genuinely has focus is FALSE.
    if !typing {
        if page_down {
            actions.push(Action::NextPage);
        }
        if page_up {
            actions.push(Action::PrevPage);
        }
        if home {
            actions.push(Action::GoToPage(0));
        }
        if end {
            // `saturating_sub` rather than `- 1`: a document with `/Count 0`
            // is legal, and an underflow here would ask for page
            // `usize::MAX`. The view clamps anyway, but relying on a clamp
            // to absorb an arithmetic bug is how the clamp stops being a
            // clamp and becomes load-bearing.
            actions.push(Action::GoToPage(page_count.saturating_sub(1)));
        }
    }

    actions
}

/// **Read this frame's keypresses and return the command ids the
/// manifest keymap binds them to.**
///
/// The whole of the "one owner per chord" fix, in one function. It knows how
/// to *spell* a key and nothing else; the keymap says what the spelling
/// means, and `crate::app::PdfcerApp::dispatch_command` says what the meaning
/// does — the same arm a ribbon click, a QAT click and a context-menu click
/// all land in. A chord therefore cannot disagree with the control that
/// shares its command, because there is nothing left for it to disagree
/// with.
///
/// `keymap` is `None` when the manifest failed to validate, in which case
/// there is no ribbon either and no chord should reach a command the
/// operator has no other route to.
///
/// # Why the other modifiers are refused
///
/// `Shift` and `Alt` must be *up*. The manifest spells a shifted chord
/// separately (`Ctrl+Shift+Z` beside `Ctrl+Y`), so treating `Ctrl+Shift+0`
/// as `Ctrl+0` would fire a binding whose spelling is not in the keymap —
/// the same class of invisible second meaning this function exists to
/// remove. `command` rather than `ctrl` for the reason [`collect`] gives:
/// it is Ctrl everywhere and Cmd on macOS.
///
/// # Why there is no `page_count` guard
///
/// [`collect`] installs nothing without a document because its actions are
/// all about a page. These are commands, and some of them — the mode
/// selector — are meaningful with nothing open. The ones that are not
/// (`view.zoom_actual`) resolve to an [`Action`] that
/// `PdfcerApp::apply` drops when `Status` is not `Open`, which is where that
/// judgement already lives.
#[must_use]
/// The chord an `egui` clipboard event stands in for, or `None`.
///
/// Named for the CHORD rather than the command, because the mapping from
/// chord to command is the keymap's and this function must not have an opinion
/// about it. What egui has taken away is the *keystroke*; this puts the
/// keystroke back and lets the keymap decide what it means.
///
/// The spellings match the manifest's own (`"Ctrl+C"`), and the comparison at
/// the call site is case-insensitive so a hand-edited keymap saying `"ctrl+c"`
/// is not silently dead — which is the failure mode this whole function is a
/// fix for, one layer up.
///
/// # `shift` is a parameter because egui CANNOT tell the two pastes apart
///
/// `OPERATOR_REQUESTS.md` **O58** binds `Ctrl+Shift+V` to `edit.paste_duplicate`,
/// and `egui-winit-0.35.0`'s own predicate is:
///
/// ```rust
/// fn is_paste_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
///     keycode == egui::Key::Paste
///         || (modifiers.command && keycode == egui::Key::V)          // <-- shift NOT excluded
///         || (cfg!(target_os = "windows") && modifiers.shift && keycode == egui::Key::Insert)
/// }
/// ```
///
/// **`Ctrl+Shift+V` therefore becomes `Event::Paste` exactly like `Ctrl+V`**,
/// the raw key event is swallowed by the same `return` documented at the call
/// site, and `Event::Paste` carries no modifier field. So the shift is
/// unrecoverable from the event and must come from the input state.
///
///
/// # Why the frame's modifiers, when this file's own rule says per-event
///
/// Because there is no per-event answer to have. `Event::Paste` is a *semantic*
/// event synthesised by the platform layer; the keystroke that produced it was
/// discarded along with its modifier state. The frame's state is the only
/// source that exists, and the caller reads it in the **same `ctx.input`
/// borrow** as the event list so the two cannot describe different frames.
///
/// The hazard the per-event rule guards against is therefore still live here in
/// a narrow form: an operator who releases Shift within the same long frame as
/// the keypress gets an ordinary paste. That is a real limitation, it is
/// unfixable at this layer, and it is written down rather than left to be
/// rediscovered — `tools/ui-verify`'s `a_form_field_can_be_copied_and_pasted_both_ways`
/// is the check that would catch it becoming common.
fn clipboard_chord(ev: &egui::Event, shift: bool) -> Option<&'static str> {
    match ev {
        // ui-text-exempt: keymap chord spellings, never displayed.
        egui::Event::Copy => Some("Ctrl+C"),
        egui::Event::Cut => Some("Ctrl+X"),
        egui::Event::Paste(_) if shift => Some("Ctrl+Shift+V"),
        egui::Event::Paste(_) => Some("Ctrl+V"),
        _ => None,
    }
}

pub fn commands(ctx: &Context, keymap: Option<&Keymap>) -> Vec<String> {
    let Some(keymap) = keymap else {
        return Vec::new();
    };

    // **Typing beats every chord, and that is not the D1 predicate alone.**
    //
    // Two claimants have to be asked about, because this shell composes text in
    // two different places:
    //
    // 1. `text_edit_focused()` — a real `egui::TextEdit`: a form field, the
    //    page-number box, a dialog's box, the Find bar. D1's predicate, never
    //    `egui_wants_keyboard_input()`, for the reason [`collect`] gives.
    // 2. A **canvas text draft** — the caret this shell paints on the page,
    //    which is deliberately *not* a `TextEdit` (`canvas::textedit`'s header
    //    says why: the caret sits in PDF space at the glyphs' own scale, which
    //    a floating widget cannot do). egui therefore reports no focused text
    //    field for an operator who is visibly mid-word, and asking only (1)
    //    would let `[` rotate the drawing instead of inserting a bracket.
    //
    // ALL chords yield, not just the unmodified ones, and that is the
    // conservative reading on purpose. `Ctrl+Z` inside a text field is the
    // field's undo; if this fired first, the operator's next keystroke would
    // revert the *document* instead of the word — destructive, silent, and
    // exactly backwards from what the key looked like it did. Every command a
    // chord reaches is also on the ribbon, so yielding costs a click; getting
    // it wrong costs an edit the operator did not ask for.
    //
    // Asked as *"is a draft in flight"* rather than *"is a caret tool armed"*,
    // because an armed tool that has not been clicked yet owns no keystrokes —
    // the page keys must keep working right up until the caret is placed.
    if crate::canvas::textedit::composing(ctx) {
        return Vec::new();
    }

    // **Read the modifiers CARRIED BY THE KEY EVENT, not the frame's.**
    //
    // The obvious shape — `i.key_pressed(key)` for the key and `i.modifiers`
    // for the modifiers — is subtly wrong, and wrong in a way that only shows
    // under load. `i.modifiers` is the modifier state as of the END of the
    // frame; `Event::Key` carries the state as of the KEYSTROKE. Those differ
    // whenever the modifier is released in the same frame the key was pressed
    // in, which is what happens when a frame is long: the operator taps
    // `Ctrl+Z` in fifty milliseconds and the application, busy rasterizing a
    // dense CAD sheet, sees press and release together with `Ctrl` already up.
    // The chord then matches nothing and the keystroke is silently dropped.
    //
    // It was found by driving: `tools/ui-verify`'s chord check reported a
    // different pair of chords dead on each run, and reordering the list moved
    // which ones. A per-frame snapshot compared against a per-event fact is
    // exactly the kind of defect that looks like harness flakiness — and the
    // last time something in this file looked like harness flakiness, the
    // conclusion drawn was "this machine cannot type", which cost the project
    // its entire keyboard surface for months. See `checks::chords`.
    //
    // Matching per event also removes the two-pass shape below it: each event
    // knows its own key and its own modifiers, so there is nothing to carry
    // between the `input` borrow and the filter.
    // The events AND the modifier state in ONE borrow — see
    // [`clipboard_chord`] for why the second is needed and why it cannot be a
    // per-event fact. Two separate `ctx.input` calls could straddle a frame.
    let (events, shift_held) = ctx.input(|i| (i.events.clone(), i.modifiers.shift));

    let mut out = Vec::new();
    for ev in events {
        // CTRL+C, CTRL+X AND CTRL+V NEVER ARRIVE AS KEY EVENTS, AND THAT IS
        // WHY THEY HAVE NEVER WORKED.
        //
        // The operator, twice: *"still no ctrl+c, ctrl+v, ctrl+x"*. On
        // 2026-08-20 they were bound in the manifest, which was necessary and
        // **not sufficient** — and the reason is fifteen lines of
        // `egui-winit-0.35.0/src/lib.rs`:
        //
        // ```rust
        // if is_cut_command(modifiers, active_key)   { events.push(Event::Cut);   return; }
        // if is_copy_command(modifiers, active_key)  { events.push(Event::Copy);  return; }
        // if is_paste_command(modifiers, active_key) { … events.push(Event::Paste(contents)); return; }
        // events.push(Event::Key { … });
        // ```
        //
        // **The `return` is before the `Event::Key` push.** So for these three
        // chords there is no key event at all, the loop below sees nothing, and
        // a keymap binding for `Ctrl+C` is a binding nothing can ever match. The
        // chord was dead the day it was written and every unit test agreed it
        // was bound, because a keymap lookup is not a keystroke.
        //
        // And `Ctrl+V` is worse than the other two: `Event::Paste` is pushed
        // **only if the OS clipboard has non-empty text**. With an empty
        // clipboard the keystroke vanishes entirely — no event of any kind — so
        // a paste of something pdfcer is holding in its own memory would depend
        // on whether the operator had recently copied text in another
        // application. `canvas::clipboard` puts a short marker on the OS
        // clipboard when it copies, for exactly that reason; its own note
        // carries the argument.
        //
        // The translation goes THROUGH THE KEYMAP rather than hard-coding
        // three ids. An operator who rebinds `Ctrl+C` gets the rebinding
        // honoured, and a manifest that binds these chords to something else
        // entirely still works — which is R8's whole posture: the registry
        // decides, not this file.
        if let Some(chord) = clipboard_chord(&ev, shift_held) {
            for (bound, id) in keymap.iter() {
                if bound.eq_ignore_ascii_case(chord) {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!("chord-command chord={chord:?} id={id} via=clipboard-event")
                    });
                    out.push(id.to_owned());
                }
            }
            continue;
        }
        let egui::Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } = ev
        else {
            continue;
        };
        for (chord, id) in keymap.iter() {
            let Some((wanted, wanted_key)) = parse_chord(chord) else {
                continue;
            };
            if wanted_key != key {
                continue;
            }
            // EXACT, never `Modifiers::matches_logically`.
            //
            // egui's own matcher is permissive — it asks whether the pattern's
            // modifiers are *present*, not whether the extras are *absent* — so
            // `Ctrl+Shift+Z` would satisfy the pattern `Ctrl+Z` as well as its
            // own. The keymap binds both, to **redo** and **undo**, so a
            // permissive match makes one keypress mean two opposite things and
            // the winner is whichever the iteration order reaches first.
            //
            // This is the generalisation of the rule the previous
            // implementation stated for two modifiers and enforced by refusing
            // them outright: *"the manifest spells a shifted chord separately,
            // so treating `Ctrl+Shift+0` as `Ctrl+0` would fire a binding whose
            // spelling is not in the keymap."* Refusing them outright is what
            // made `Ctrl+Shift+E` and `Ctrl+Shift+Z` undispatchable; comparing
            // them exactly keeps the property and drops the casualty.
            //
            // `command` rather than `ctrl` for the reason [`collect`] gives: it
            // is Ctrl everywhere and Cmd on macOS.
            if modifiers.command != wanted.command
                || modifiers.shift != wanted.shift
                || modifiers.alt != wanted.alt
            {
                continue;
            }
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // The chord is QUOTED (`{chord:?}`), and it has to be. Two of
                // the manifest's bindings are `[` and `]`, and the harness's
                // trace parser tracks bracket depth so that
                // `rect=[[0.0 0.0] - [16.0 9.0]] zoom=1.5` does not split into
                // nonsense. An unquoted `chord=[` therefore opened a bracket
                // that never closed, and every field after it on the line —
                // including `id` — was swallowed into the chord's value. The
                // check read `[` as dead while the line proving it alive sat in
                // the file it had just read.
                format!("chord-command chord={chord:?} id={id}")
            });
            out.push(id.to_owned());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, Modifiers, RawInput};

    /// Build a `RawInput` carrying one key press.
    fn key_press(key: Key, modifiers: Modifiers) -> RawInput {
        RawInput {
            events: vec![Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }],
            modifiers,
            ..Default::default()
        }
    }

    /// Run one frame and return whatever `collect` produced in it.
    fn actions_for(ctx: &Context, input: RawInput, page_count: Option<usize>) -> Vec<Action> {
        let mut out = Vec::new();
        let _ = ctx.run_ui(input, |ui| out = collect(ui.ctx(), page_count));
        out
    }

    /// The D1 regression test.
    #[test]
    fn a_focused_non_text_widget_does_not_suppress_unmodified_keys() {
        let ctx = Context::default();
        let id = egui::Id::new("a-plain-focusable-widget");

        // Frame 1: take focus, the way the canvas does on click.
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            ui.ctx().memory_mut(|m| m.request_focus(id));
        });

        // Frame 2: a focused widget is holding keyboard focus. Prove it,
        // then prove the guard is unaffected.
        let mut wants_keyboard = false;
        let mut text_focused = true;
        let mut actions = Vec::new();
        let _ = ctx.run_ui(key_press(Key::PageDown, Modifiers::NONE), |ui| {
            let ctx = ui.ctx();
            wants_keyboard = ctx.egui_wants_keyboard_input();
            // typing-guard-exempt: a TEST asserting the harness actually reached
            // the focused state. Reading the raw egui answer is the point - a
            // test that asked `composing()` could not tell a focused widget from
            // a canvas draft, and the thing being proved is that the widget half
            // is reachable at all. D1 shipped because its test could not reach it.
            text_focused = ctx.text_edit_focused();
            actions = collect(ctx, Some(5));
        });

        assert!(
            wants_keyboard,
            "the test is vacuous unless a widget really holds focus — this is the exact \
             condition D1's guard mistook for typing"
        );
        assert!(
            !text_focused,
            "a plain focusable widget is not a text field, and the guard must say so"
        );
        assert_eq!(actions, vec![Action::NextPage]);
    }

    /// With no document open, no binding is installed at all.
    #[test]
    fn nothing_is_bound_without_a_document() {
        let ctx = Context::default();
        let actions = actions_for(&ctx, key_press(Key::PageDown, Modifiers::NONE), None);
        assert!(actions.is_empty());
    }

    /// End goes to the last page, and does not underflow on an empty one.
    ///
    /// The empty-document case is legal PDF (`/Count 0`), and `usize`
    /// underflow here would ask the view for page `usize::MAX`.
    #[test]
    fn end_lands_on_the_last_page_and_survives_an_empty_document() {
        let ctx = Context::default();
        assert_eq!(
            actions_for(&ctx, key_press(Key::End, Modifiers::NONE), Some(7)),
            vec![Action::GoToPage(6)]
        );
        assert_eq!(
            actions_for(&ctx, key_press(Key::End, Modifiers::NONE), Some(0)),
            vec![Action::GoToPage(0)]
        );
    }

    /// Ctrl+`=` must zoom in as well as Ctrl+`+`.
    ///
    /// `+` is a shifted key on most layouts, so binding only `+` turns
    /// "zoom in" into a three-finger chord. Every browser accepts both.
    #[test]
    fn both_plus_and_equals_zoom_in_with_the_command_modifier() {
        let ctx = Context::default();
        let ctrl = Modifiers::COMMAND;
        assert_eq!(
            actions_for(&ctx, key_press(Key::Plus, ctrl), Some(3)),
            vec![Action::ZoomIn]
        );
        assert_eq!(
            actions_for(&ctx, key_press(Key::Equals, ctrl), Some(3)),
            vec![Action::ZoomIn]
        );
    }

    /// The modifier is read from the KEYSTROKE, not from the frame.
    #[test]
    fn a_zoom_chord_is_matched_on_the_events_own_modifiers() {
        let ctx = Context::default();

        let on_the_event = RawInput {
            events: vec![Event::Key {
                key: Key::Minus,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::COMMAND,
            }],
            modifiers: Modifiers::NONE,
            ..Default::default()
        };
        assert_eq!(
            actions_for(&ctx, on_the_event, Some(3)),
            vec![Action::ZoomOut],
            "the keystroke carried the modifier; only the end-of-frame \
             snapshot had lost it"
        );

        let on_the_frame = RawInput {
            events: vec![Event::Key {
                key: Key::Minus,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            modifiers: Modifiers::COMMAND,
            ..Default::default()
        };
        assert!(
            actions_for(&ctx, on_the_frame, Some(3)).is_empty(),
            "a bare `-` must not zoom because the frame happened to end with \
             the modifier held"
        );
    }

    /// **Every viewer chord is reachable through the scripted seam.**
    #[test]
    fn every_viewer_chord_has_a_spelling_the_scripted_seam_accepts() {
        for (key, spellings) in OWNED {
            assert!(
                spellings
                    .iter()
                    .any(|s| parse_chord(s).is_some_and(|(_, parsed)| parsed == *key)),
                "no spelling of {key:?} can be delivered by the seam, so that \
                 verb has no headless route at all"
            );
        }
    }

    /// The zoom chords require their modifier.
    ///
    /// A bare `0` or `-` belongs to whatever surface has focus — a page-number
    /// box, a text field — and a modifierless binding here would steal it.
    #[test]
    fn the_zoom_chords_do_not_fire_without_the_modifier() {
        let ctx = Context::default();
        for key in [Key::Plus, Key::Equals, Key::Minus] {
            assert!(
                actions_for(&ctx, key_press(key, Modifiers::NONE), Some(3)).is_empty(),
                "an unmodified key must not reach a zoom command"
            );
        }
    }

    /// A digit alone must not reach a command either.
    #[test]
    fn a_digit_reaches_no_command_without_its_modifier() {
        let ctx = Context::default();
        let keymap = built_in_keymap();
        for (chord, bound) in keymap.iter() {
            let Some((wanted, key)) = parse_chord(chord) else {
                continue;
            };
            if wanted == egui::Modifiers::NONE {
                continue; // `[` is spelled with no modifier; it needs none.
            }
            let mut ids = Vec::new();
            let _ = ctx.run_ui(key_press(key, Modifiers::NONE), |ui| {
                ids = commands(ui.ctx(), Some(&keymap));
            });
            // **The chord's OWN id must be absent** — not the whole list.
            //
            //
            // The property under test never was "a bare key does nothing". It is
            // **"a chord that names a modifier does not fire without it"**, and
            // that is what this now says.
            assert!(
                !ids.iter().any(|got| got == bound),
                "`{chord}` fired with no modifier held"
            );
        }
    }

    // -----------------------------------------------------------------------
    // The one-owner-per-chord guard, and the derivation it protects
    // -----------------------------------------------------------------------

    /// The real keymap, as the application will use it.
    fn built_in_keymap() -> Keymap {
        crate::shell::manifest::built_in()
            .keymap
            .expect("the built-in manifest binds chords")
    }

    /// **No chord has two owners.**
    #[test]
    fn no_chord_has_two_owners() {
        let keymap = built_in_keymap();
        for (key, spellings) in OWNED {
            for chord in *spellings {
                assert!(
                    keymap.get(chord).is_none(),
                    "the chord `{chord}` ({key:?}) has two owners: `app::keyboard::collect` binds \
                     it to a viewer action, and the manifest keymap binds it to `{}`. One chord, \
                     one owner — either drop the keymap entry or move the binding out of \
                     `collect` and into the manifest keymap.",
                    keymap.get(chord).unwrap_or_default(),
                );
            }
        }
    }

    /// **THE GATE. Every chord the manifest binds actually fires.**
    #[test]
    fn every_chord_the_manifest_binds_actually_fires() {
        let ctx = Context::default();
        let keymap = built_in_keymap();
        for (chord, command) in keymap.iter() {
            let (modifiers, key) = parse_chord(chord).unwrap_or_else(|| {
                panic!(
                    "the manifest binds `{chord}` to `{command}`, but no key can be spelled from \
                     it, so pressing it does nothing"
                )
            });
            let mut ids = Vec::new();
            let _ = ctx.run_ui(key_press(key, modifiers), |ui| {
                ids = commands(ui.ctx(), Some(&keymap));
            });
            assert!(
                ids.iter().any(|id| id == command),
                "the manifest binds `{chord}` to `{command}` and the menus print it as a \
                 shortcut, but pressing it dispatched {ids:?}"
            );
        }
    }

    /// **A chord survives its modifier being released in the SAME frame.**
    #[test]
    fn a_chord_survives_its_modifier_being_released_in_the_same_frame() {
        let ctx = Context::default();
        let keymap = built_in_keymap();

        let mut input = RawInput::default();
        input.events.push(egui::Event::Key {
            key: Key::Z,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        });
        // ...and the modifier is up again before the frame is read, which is
        // what `i.modifiers` would report.
        input.events.push(egui::Event::Key {
            key: Key::Z,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: Modifiers::NONE,
        });
        input.modifiers = Modifiers::NONE;

        let mut ids = Vec::new();
        let _ = ctx.run_ui(input, |ui| ids = commands(ui.ctx(), Some(&keymap)));
        assert_eq!(
            ids,
            vec!["edit.undo".to_owned()],
            "a quick Ctrl+Z on a long frame must still reach undo"
        );
    }

    /// A chord with extra modifiers held does NOT fire the shorter one.
    #[test]
    fn a_longer_chord_does_not_also_fire_the_shorter_one() {
        let ctx = Context::default();
        let keymap = built_in_keymap();
        let mut ids = Vec::new();
        let _ = ctx.run_ui(
            key_press(Key::Z, Modifiers::COMMAND.plus(Modifiers::SHIFT)),
            |ui| ids = commands(ui.ctx(), Some(&keymap)),
        );
        assert_eq!(
            ids,
            vec!["edit.redo".to_owned()],
            "Ctrl+Shift+Z is redo and must not also be undo"
        );
    }

    /// A focused text field silences every chord.
    #[test]
    fn a_focused_text_field_silences_the_chords() {
        let ctx = Context::default();
        let keymap = built_in_keymap();
        let id = egui::Id::new("a-field");

        // Frame 1: register a real TextEdit and focus it, so
        // `text_edit_focused()` is genuinely true rather than assumed.
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            let mut text = String::new();
            // escape-disposition: not-a-surface — a field this test builds to put egui
            // into a known focus state. It is never drawn for an operator.
            let r = ui.add(egui::TextEdit::singleline(&mut text).id(id));
            r.request_focus();
        });

        let mut focused = false;
        let mut ids = Vec::new();
        let _ = ctx.run_ui(key_press(Key::Z, Modifiers::COMMAND), |ui| {
            let mut text = String::new();
            // escape-disposition: not-a-surface — a field this test builds to put egui
            // into a known focus state. It is never drawn for an operator.
            ui.add(egui::TextEdit::singleline(&mut text).id(id));
            // typing-guard-exempt: a TEST asserting the harness actually reached
            // the focused state. Reading the raw egui answer is the point - a
            // test that asked `composing()` could not tell a focused widget from
            // a canvas draft, and the thing being proved is that the widget half
            // is reachable at all. D1 shipped because its test could not reach it.
            focused = ui.ctx().text_edit_focused();
            ids = commands(ui.ctx(), Some(&keymap));
        });
        assert!(focused, "precondition: the field really holds focus");
        assert!(
            ids.is_empty(),
            "a chord fired while the operator was typing"
        );
    }

    /// ...and a CANVAS text draft silences them too.
    #[test]
    fn a_canvas_text_draft_silences_the_bare_chords() {
        use crate::canvas::textedit::{Anchor, Draft, TextEditKind, store};
        let ctx = Context::default();
        let keymap = built_in_keymap();

        let press = || key_press(Key::OpenBracket, Modifiers::NONE);
        let mut ids = Vec::new();
        let _ = ctx.run_ui(press(), |ui| ids = commands(ui.ctx(), Some(&keymap)));
        assert_eq!(
            ids,
            vec!["pages.rotate_left".to_owned()],
            "precondition: `[` rotates when nothing is being composed"
        );

        store(
            &ctx,
            Draft {
                page: 0,
                kind: TextEditKind::Add,
                anchor: Anchor::Origin { x: 1.0, y: 1.0 },
                text: String::new(),
                caret: 0,
                mark: None,
                seeded: true,
            },
        );
        let mut ids = Vec::new();
        let _ = ctx.run_ui(press(), |ui| ids = commands(ui.ctx(), Some(&keymap)));
        assert!(
            ids.is_empty(),
            "a bracket typed into a draft must not rotate the page"
        );
    }

    /// **`Ctrl+0` is actual size, and it is the manifest that says so.**
    #[test]
    fn ctrl_0_names_the_actual_size_command() {
        let ctx = Context::default();
        let keymap = built_in_keymap();
        let mut ids = Vec::new();
        let _ = ctx.run_ui(key_press(Key::Num0, Modifiers::COMMAND), |ui| {
            ids = commands(ui.ctx(), Some(&keymap));
        });
        assert_eq!(ids, vec!["view.zoom_actual".to_owned()]);

        // And it raises no viewer action of its own — the whole point of the
        // split. A `Fit(Page)` here would be the defect, restored.
        assert!(
            actions_for(&ctx, key_press(Key::Num0, Modifiers::COMMAND), Some(3)).is_empty(),
            "`collect` must not bind a chord the manifest owns"
        );
    }

    /// **`Ctrl+O` reaches the Open command.**
    #[test]
    fn ctrl_o_names_the_open_command() {
        let ctx = Context::default();
        let keymap = built_in_keymap();
        let mut ids = Vec::new();
        let _ = ctx.run_ui(key_press(Key::O, Modifiers::COMMAND), |ui| {
            ids = commands(ui.ctx(), Some(&keymap));
        });
        assert_eq!(ids, vec!["file.open".to_owned()]);

        // A bare `O` is a letter somebody may be typing into the page box.
        let mut unmodified = Vec::new();
        let _ = ctx.run_ui(key_press(Key::O, Modifiers::NONE), |ui| {
            unmodified = commands(ui.ctx(), Some(&keymap));
        });
        assert!(unmodified.is_empty());
    }

    /// The mode chords reach the mode commands.
    #[test]
    fn the_mode_chords_name_the_mode_commands() {
        let ctx = Context::default();
        let keymap = built_in_keymap();
        for (key, expected) in [
            (Key::Num1, "mode.read"),
            (Key::Num2, "mode.review"),
            (Key::Num3, "mode.edit"),
        ] {
            let mut ids = Vec::new();
            let _ = ctx.run_ui(key_press(key, Modifiers::COMMAND), |ui| {
                ids = commands(ui.ctx(), Some(&keymap));
            });
            assert_eq!(ids, vec![expected.to_owned()]);
        }
    }

    /// **The meaning really is derived, not restated.**
    #[test]
    fn the_derived_chords_follow_the_keymap_rather_than_this_module() {
        let ctx = Context::default();
        let mut invented = std::collections::BTreeMap::new();
        invented.insert("Ctrl+0".to_owned(), "view.zoom_fit_width".to_owned());
        let keymap = Keymap(invented);

        let mut ids = Vec::new();
        let _ = ctx.run_ui(key_press(Key::Num0, Modifiers::COMMAND), |ui| {
            ids = commands(ui.ctx(), Some(&keymap));
        });
        assert_eq!(ids, vec!["view.zoom_fit_width".to_owned()]);
    }

    /// **Every chord the manifest binds is written down in `MANUAL.md`.**
    #[test]
    fn every_chord_the_manifest_binds_is_written_down_in_the_manual() {
        /// The operator-facing manual, compiled in so the test cannot be
        /// pointed at a copy that is not the shipped one.
        const MANUAL: &str = include_str!("../../../../MANUAL.md");

        // Every bolded run in the manual, in source order.
        let mut spellings: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        let mut rest = MANUAL;
        while let Some(open) = rest.find("**") {
            rest = &rest[open + 2..];
            let Some(close) = rest.find("**") else {
                break;
            };
            let run = rest[..close]
                .trim()
                .replace('↑', "Up")
                .replace('↓', "Down")
                .replace('←', "Left")
                .replace('→', "Right");
            rest = &rest[close + 2..];

            // Rewrite 2: distribute a modifier prefix over a slash list.
            let parts: Vec<&str> = run.split(" / ").map(str::trim).collect();
            let prefix = parts
                .first()
                .and_then(|first| first.rfind('+'))
                .map(|at| parts[0][..=at].to_owned())
                .unwrap_or_default();
            for part in &parts {
                if part.contains('+') || prefix.is_empty() {
                    spellings.insert((*part).to_owned());
                } else {
                    spellings.insert(format!("{prefix}{part}"));
                }
            }
            spellings.insert(run);
        }

        let keymap = built_in_keymap();
        let missing: Vec<String> = keymap
            .0
            .iter()
            .filter(|(chord, _)| !spellings.contains(chord.as_str()))
            .map(|(chord, id)| format!("{chord} -> {id}"))
            .collect();

        assert!(
            missing.is_empty(),
            "★ MANUAL.md does not mention {} of the {} chords built_in.ron binds. \
             An operator cannot use a shortcut nobody told them about, and no other \
             check in this repository reads MANUAL.md. Add a row to \"Every \
             keyboard shortcut\" for each:\n  {}",
            missing.len(),
            keymap.0.len(),
            missing.join("\n  ")
        );
    }

    /// A chord bound to nothing produces nothing.
    #[test]
    fn an_unbound_chord_and_an_absent_keymap_both_produce_nothing() {
        let ctx = Context::default();
        let empty = Keymap(std::collections::BTreeMap::new());
        for keymap in [Some(&empty), None] {
            let mut ids = Vec::new();
            let _ = ctx.run_ui(key_press(Key::Num1, Modifiers::COMMAND), |ui| {
                ids = commands(ui.ctx(), keymap);
            });
            assert!(ids.is_empty());
        }
    }
}
