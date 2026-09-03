//! # `app::keyboard` — the keyboard map, and the guard that must not be wrong
//!
//! ## ★ `DEFECTS.md` D1 — read this before touching the guard
//!
//! The old GUI's keyboard map guarded its unmodified-key bindings with:
//!
//! ```ignore
//! let typing = ctx.egui_wants_keyboard_input();
//! ```
//!
//! **That predicate does not mean what its name says.** Verified in the
//! vendored source at `egui-0.35.0/src/context.rs:2884-2886`:
//!
//! ```ignore
//! pub fn egui_wants_keyboard_input(&self) -> bool {
//!     self.memory(|m| m.focused().is_some())
//! }
//! ```
//!
//! — *any* focused widget, including the canvas. Its own doc comment
//! immediately above says *"egui is currently listening on text input (e.g.
//! typing text in a `TextEdit`)"*, which is what the name and the comment
//! both promise and what the implementation does not deliver. This is an
//! egui API footgun, not a careless read.
//!
//! The consequence was the defect the operator reported as *"I can't even
//! click on an object and delete it by hitting the delete key."* The canvas
//! calls `request_focus()` on click, and because the widget is recreated
//! every frame its id stays live — so from the **first canvas click
//! onward**, `typing` was permanently `true` and every unmodified binding
//! was suppressed. Delete, Backspace, PageUp, PageDown, Home, End and the
//! rotate keys all died from one click, and the deletion logic downstream
//! was correct and simply unreachable.
//!
//! **The fix, applied here from the first line of code:**
//!
//! ```ignore
//! let typing = ctx.text_edit_focused();
//! ```
//!
//! `text_edit_focused()` (`egui-0.35.0/src/context.rs:2889-2895`) resolves
//! the focused id and checks whether a `TextEditState` exists *for that id*.
//! It therefore preserves the guard's real intent exactly — a focused text
//! field keeps its unmodified keys — while a focused canvas, button or tab
//! does not steal them. A `DragValue` in keyboard-edit mode registers its
//! `TextEdit` under the *same* id it focuses, so numeric property fields
//! still count as typing, which is the case the original guard was written
//! for.
//!
//! ### Why the old test did not catch it, and what replaced it
//!
//! The original had exactly one test, and it built a bare
//! `egui::Context::default()` with **no widgets** — so `memory.focused()`
//! was always `None`, `typing` was always `false`, and the single property
//! that breaks in the real application was structurally absent from the
//! only harness that exercised the function.
//!
//! [`tests::a_focused_non_text_widget_does_not_suppress_unmodified_keys`]
//! is the test that would have caught it: it drives a real `Context`
//! through two frames, takes focus on a plain widget id in the first,
//! asserts in the second that `egui_wants_keyboard_input()` really is
//! `true` (so the test is known to be exercising the failing condition,
//! not a vacuous one), and then asserts the unmodified bindings still fire.
//! Swapping the guard back to `egui_wants_keyboard_input()` fails it.
//!
//! ## Why this is a pure-ish function taking a page count
//!
//! [`collect`] takes `page_count: Option<usize>` rather than the app's
//! `Status`, so it can be tested against a real `egui::Context` without
//! constructing a `Document`. `None` means "no document open", and the
//! whole map is then not installed — a binding that fires with nothing open
//! is a binding whose action has to defend itself.
//!
//! ## ★ Two owners for one chord — the defect this module was split for
//!
//! This module used to bind `Ctrl+0` to fit page and `Ctrl+2` to fit width,
//! **while `crate::shell::manifest`'s keymap bound the same two chords to
//! `view.zoom_actual` and `mode.review`.** Both statements were compiled in,
//! both were operator-visible, and they disagreed:
//!
//! - The manifest keymap is what `egui_shell::menu::Shortcuts` inverts to
//!   draw a context menu's right-aligned chord hint, so a right-click on
//!   blank paper offered *Actual size — Ctrl+0*.
//! - `crate::text::commands::view_zoom_actual`'s ribbon tooltip named
//!   `Ctrl+0` too, and `crate::text::commands::mode_review`'s named `Ctrl+2`.
//! - This module got there first and did neither, because nothing dispatches
//!   the manifest keymap: `egui-shell` deliberately does not own key
//!   handling (`egui_shell::ribbon`'s header: *"the application owns the
//!   question of what has focus and what a chord means"*).
//!
//! The visible cost was `crate::text::status::fit_actual_size_tooltip`,
//! which had to advertise **no chord at all** — with a test pinning the
//! omission — because it could not honestly name one.
//!
//! ### The fix: the manifest binds, this module enacts
//!
//! There is now exactly one place a chord is *bound to a meaning*, and it is
//! the manifest keymap. [`commands`] spells the key it saw the way a
//! manifest writes it, looks the spelling up in the keymap, and returns the
//! **command id** — which `crate::app::PdfcerApp::dispatch_command` then
//! dispatches through the same arm a ribbon click reaches. Rebind `Ctrl+0`
//! in a customization layer and the keyboard follows, with nothing here to
//! edit. [`tests::the_derived_chords_follow_the_keymap_rather_than_this_module`]
//! is the proof: it hands `commands` an invented keymap and watches the
//! chord change meaning.
//!
//! ### And the chords this module still owns outright
//!
//! [`collect`] keeps the *viewer* chords, which the manifest documents as
//! deliberately absent from its keymap — *"Viewer navigation, handled in the
//! app's own keyboard layer against the view state. They are not ribbon
//! commands and putting them here would give them a second owner."*
//! [`OWNED`] names every one of them, in every spelling a manifest might
//! write, and [`tests::no_chord_has_two_owners`] fails — naming the chord
//! and both claimants — the moment the keymap claims one back.
//!
//! ## The bindings, and why these
//!
//! | keys | action | owner |
//! |---|---|---|
//! | Ctrl+`+` / Ctrl+`=` | zoom in one rung | here ([`OWNED`]) — what browsers, Acrobat and every PDF reader do |
//! | Ctrl+`-` | zoom out one rung | here ([`OWNED`]) |
//! | PageDown / PageUp | next / previous page | here ([`OWNED`]) — the unmodified keys D1 killed |
//! | Home / End | first / last page | here ([`OWNED`]) |
//! | Ctrl+`N` | `file.new` → a blank document | the manifest keymap |
//! | Ctrl+`O` | `file.open` → the file picker | the manifest keymap |
//! | Ctrl+`F` | `edit.find` → open or close the Find bar | the manifest keymap |
//! | Ctrl+`0` | `view.zoom_actual` → actual size | the manifest keymap |
//! | Ctrl+`1` / Ctrl+`2` / Ctrl+`3` | `mode.read` / `mode.review` / `mode.edit` | the manifest keymap |
//!
//! Ctrl+`=` is bound alongside Ctrl+`+` because `+` is a shifted key on
//! most layouts and requiring the shift makes "zoom in" a three-finger
//! chord. Every browser accepts both; so does this.
//!
//! ### Why `Ctrl+0` is actual size and not fit page
//!
//! Acrobat numbers these chords `0` = fit page, `1` = actual size, `2` = fit
//! width, `3` = fit visible, and this module used to follow it. **That
//! numbering is no longer available as a whole**: `MODES_AND_PANELS.md`
//! Part 1 §6 specifies `Ctrl+1` / `Ctrl+2` / `Ctrl+3` for the Read / Review
//! / Edit selector, and the manifest binds them. Taking Acrobat's `0` while
//! `1`, `2` and `3` mean something else entirely would teach the operator
//! half of a numbering that then stops working.
//!
//! What is left is the browser convention — `Ctrl+0` returns to 100 % — and
//! it is also what two operator-visible strings already claimed before
//! anything reached them. So the manifest's reading wins, and the two fit
//! modes keep their status-bar buttons, their View ▸ Zoom controls and their
//! `canvas.empty` context-menu entries as the routes in. **`FitMode::Width`
//! is still reachable**, which was the structural reason this module bound
//! `Ctrl+2` at S0, back when no ribbon and no status bar existed.
//!
//! Note that these chords require egui's own `zoom_with_keyboard` to be
//! switched **off**, or it consumes them to rescale the entire user
//! interface — see [`crate::app::configure_context`]. Without that, the
//! chords would silently do the wrong thing.

use egui::{Context, Key};
use egui_shell::manifest::Keymap;

use crate::app::actions::Action;

/// **Spell a manifest chord into the modifiers and key that fire it.**
///
/// The replacement for what used to be a hand-written `DERIVED` spelling
/// table, and the reason that table had to go: it listed eight chords while
/// the shipped keymap bound twenty-one, so **fourteen bindings were declared,
/// printed in menus and tooltips as shortcuts, and dispatched by nothing** —
/// `Ctrl+Z`, `Ctrl+Y`, `Ctrl+S`, `Ctrl+E`, `Ctrl+Shift+E`, `F11`, `[`, `]`
/// and six more. Undo had a keyboard shortcut everywhere except the keyboard.
///
/// A table that must be kept in step with a manifest by hand will fall out of
/// step with it, and the failure is silent in both directions: the keymap
/// entry looks bound, the menu hint looks true, and the key does nothing. So
/// nothing is kept in step any more — the manifest is parsed, and a chord
/// fires if and only if it can be spelled.
///
/// # The grammar
///
/// `Modifier+Modifier+Key`, modifiers in any order, matching what
/// `built_in.ron` already writes: `Ctrl`, `Shift`, `Alt` (and `Cmd`/`Command`
/// as aliases for `Ctrl`, since egui's `command` is Ctrl everywhere and Cmd on
/// macOS). The key is whatever [`Key::from_name`] accepts, which is egui's own
/// parser — so `[`, `Down`, `F11`, `0` and `E` all resolve, and the manifest
/// cannot invent a spelling this shell then fails to honour.
///
/// Returns `None` for a chord that cannot be spelled, which
/// [`tests::every_chord_the_manifest_binds_actually_fires`] turns into a build
/// failure rather than a dead key.
fn parse_chord(chord: &str) -> Option<(egui::Modifiers, Key)> {
    let mut modifiers = egui::Modifiers::NONE;
    let mut key = None;
    for part in chord.split('+') {
        match part {
            "Ctrl" | "Cmd" | "Command" => modifiers.command = true,
            "Shift" => modifiers.shift = true,
            "Alt" => modifiers.alt = true,
            // A trailing empty segment is the literal `+` of a chord spelled
            // `Ctrl++`. Splitting on the separator cannot tell the two apart,
            // so the empty string is read as the key it can only have been.
            "" => key = Some(Key::Plus),
            other => key = Some(Key::from_name(other)?),
        }
    }
    Some((modifiers, key?))
}

/// **The chords this module binds outright — viewer navigation — and every
/// spelling a manifest might use for each.**
///
/// The manifest's keymap names these as deliberately absent, for the reason
/// stated there: they are not ribbon commands, and binding them in two
/// places would give them two owners. This table is what makes that
/// statement *checkable* rather than a comment —
/// [`tests::no_chord_has_two_owners`] walks it against the real keymap.
///
/// Several spellings per key because the guard has to catch a conflict
/// however the author of the keymap chose to write it: `Ctrl+Plus` and
/// `Ctrl++` are the same chord, and a test that knew only one of them would
/// pass while the defect it exists to prevent sat in the file.
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
///
/// Only the chords [`OWNED`] lists — the viewer's own. The chords the
/// manifest binds are [`commands`]' job, and the two sets are disjoint by
/// test rather than by good intentions.
///
/// `page_count` is `None` when no document is open, in which case no
/// binding is installed at all.
pub fn collect(ctx: &Context, page_count: Option<usize>) -> Vec<Action> {
    let mut actions = Vec::new();
    let Some(page_count) = page_count else {
        return actions;
    };

    // ★ D1. `text_edit_focused()`, NEVER `egui_wants_keyboard_input()`.
    // See the module docs for the whole story; the one-line version is that
    // the latter means "any widget has focus", the canvas takes focus on
    // click, and the difference cost the operator the Delete key and all
    // keyboard page navigation from the first click onward.
    // ★★ …and a CANVAS text draft counts as typing too, which
    // `text_edit_focused()` alone cannot see.
    //
    // These bindings are page keys, not characters, so the harm is not a
    // mistyped bracket - it is `PageDown` under a half-typed word. A page
    // change abandons the draft (see `canvas::textedit::load`), so without this
    // the operator's words are discarded by a key they pressed for navigation
    // and never told. The chord dispatcher in [`commands`] asks the same two
    // questions for its own reason; the predicate is shared, the argument is
    // not.
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

    let (modifiers, pressed) = ctx.input(|i| {
        (
            i.modifiers,
            [
                Key::Plus,
                Key::Equals,
                Key::Minus,
                Key::PageDown,
                Key::PageUp,
                Key::Home,
                Key::End,
            ]
            .map(|k| i.key_pressed(k)),
        )
    });
    let [plus, equals, minus, page_down, page_up, home, end] = pressed;

    // `command` rather than `ctrl`: it is Ctrl everywhere and Cmd on macOS,
    // which is what a Mac operator's fingers expect. pdfcer ships on Windows
    // first, but a hard-coded `ctrl` is a portability bug that costs nothing
    // to avoid now and is tedious to find later.
    if modifiers.command {
        if plus || equals {
            actions.push(Action::ZoomIn);
        }
        if minus {
            actions.push(Action::ZoomOut);
        }
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
/// ★ Named for the CHORD rather than the command, because the mapping from
/// chord to command is the keymap's and this function must not have an opinion
/// about it. What egui has taken away is the *keystroke*; this puts the
/// keystroke back and lets the keymap decide what it means.
///
/// The spellings match the manifest's own (`"Ctrl+C"`), and the comparison at
/// the call site is case-insensitive so a hand-edited keymap saying `"ctrl+c"`
/// is not silently dead — which is the failure mode this whole function is a
/// fix for, one layer up.
///
/// # ★★★ `shift` is a parameter because egui CANNOT tell the two pastes apart
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
/// ★ Measured against the source on 2026-08-29 rather than assumed. The
/// alternative reading — that egui excludes shift, so `Ctrl+Shift+V` arrives as
/// an ordinary `Event::Key` and the generic path below handles it with no work
/// at all — was the hypothesis, and it was **wrong**. It would have shipped a
/// chord that did nothing, or worse, one that pasted a NEW field every time the
/// operator asked for a duplicate.
///
/// # ★★ Why the frame's modifiers, when this file's own rule says per-event
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

    // ★★ **Typing beats every chord, and that is not the D1 predicate alone.**
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

    // ★★ **Read the modifiers CARRIED BY THE KEY EVENT, not the frame's.**
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
    // ★ The events AND the modifier state in ONE borrow — see
    // [`clipboard_chord`] for why the second is needed and why it cannot be a
    // per-event fact. Two separate `ctx.input` calls could straddle a frame.
    let (events, shift_held) = ctx.input(|i| (i.events.clone(), i.modifiers.shift));

    let mut out = Vec::new();
    for ev in events {
        // ★★★ CTRL+C, CTRL+X AND CTRL+V NEVER ARRIVE AS KEY EVENTS, AND THAT IS
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
        // ★ And `Ctrl+V` is worse than the other two: `Event::Paste` is pushed
        // **only if the OS clipboard has non-empty text**. With an empty
        // clipboard the keystroke vanishes entirely — no event of any kind — so
        // a paste of something pdfcer is holding in its own memory would depend
        // on whether the operator had recently copied text in another
        // application. `canvas::clipboard` puts a short marker on the OS
        // clipboard when it copies, for exactly that reason; its own note
        // carries the argument.
        //
        // ★★ The translation goes THROUGH THE KEYMAP rather than hard-coding
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
            // ★ EXACT, never `Modifiers::matches_logically`.
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
                // ★ The chord is QUOTED (`{chord:?}`), and it has to be. Two of
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
    ///
    /// `Context::run_ui` (egui 0.35 renamed `run`) hands the closure a root
    /// [`egui::Ui`] rather than the context, and returns a `#[must_use]`
    /// `FullOutput` this harness has no use for — hence the `let _`.
    fn actions_for(ctx: &Context, input: RawInput, page_count: Option<usize>) -> Vec<Action> {
        let mut out = Vec::new();
        let _ = ctx.run_ui(input, |ui| out = collect(ui.ctx(), page_count));
        out
    }

    /// ★ The D1 regression test.
    ///
    /// Drives a real `Context` through two frames. The first takes focus on
    /// a plain (non-text) widget id — which is exactly what the canvas does
    /// on click. The second asserts two things in order:
    ///
    /// 1. `egui_wants_keyboard_input()` is genuinely `true`, so the test is
    ///    known to be exercising the failing condition rather than passing
    ///    vacuously. This is the assertion the original test was missing,
    ///    and its absence is why the defect shipped.
    /// 2. The unmodified bindings still fire.
    ///
    /// Swap `text_edit_focused()` back to `egui_wants_keyboard_input()` in
    /// [`collect`] and this test fails.
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
    ///
    /// Same rule as the zoom chords, checked on the derived path: a bare `2`
    /// belongs to the page-number box, and a keymap entry spelled `Ctrl+2`
    /// must not be satisfied by a `2` with nothing held.
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
            // ★ **The chord's OWN id must be absent** — not the whole list.
            //
            // It asserted `ids.is_empty()` until 2026-08-19, which was the same
            // statement while no bare letter was bound to anything. The four
            // pointer tools took `V`, `A`, `T` and `H` that day (the layout
            // every program in this class uses), so pressing `H` now fires
            // `view.tool_hand` — correctly — and an empty-list assertion would
            // fail on `Ctrl+H` for a reason that has nothing to do with what it
            // is testing.
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
    // ★ The one-owner-per-chord guard, and the derivation it protects
    // -----------------------------------------------------------------------

    /// The real keymap, as the application will use it.
    ///
    /// Read from [`crate::shell::manifest::built_in`] rather than hand-built,
    /// because a guard that checks an invented keymap guards an invented
    /// defect.
    fn built_in_keymap() -> Keymap {
        crate::shell::manifest::built_in()
            .keymap
            .expect("the built-in manifest binds chords")
    }

    /// ★ **No chord has two owners.**
    ///
    /// This is the regression test for the defect in the module header. It
    /// walks every chord [`collect`] binds outright, in every spelling a
    /// manifest might write it, and asserts the keymap claims none of them.
    /// Reintroduce `Ctrl+0` — or `Ctrl+Plus`, or `PageDown` — on either side
    /// and this fails **naming the chord and both claimants**, which is the
    /// property that makes it useful: a failure that said only "keymap
    /// mismatch" would send the next person looking in the wrong file.
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

    /// ★★ **THE GATE. Every chord the manifest binds actually fires.**
    ///
    /// The test whose absence let fourteen shortcuts ship dead. Its
    /// predecessor asserted the right property and then swept a *third* of the
    /// keymap - `if !is_digit_chord { continue; }` - while its own doc comment
    /// stated the general rule: *"a chord this module cannot see would then be
    /// a keymap entry, a menu hint and a tooltip promising something no
    /// keypress delivers."* The reasoning was right and the enforcement was
    /// narrowed, so `Ctrl+Z`, `Ctrl+Y`, `Ctrl+S`, `Ctrl+E`, `Ctrl+Shift+E`,
    /// `F11`, `[`, `]`, `Alt+Up`, `Alt+Down` and four more sat in the manifest,
    /// printed themselves in menus, and did nothing.
    ///
    /// So this sweeps **every** entry, and it does not check that a chord can
    /// be *spelled* - it presses it and checks the command comes back. A
    /// spelling test would have passed on a dispatcher that spelled the chord
    /// correctly and then filtered it out for holding Shift, which is exactly
    /// how `Ctrl+Shift+E` died.
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

    /// ★★ **A chord survives its modifier being released in the SAME frame.**
    ///
    /// The regression test for a defect that only appears under load and reads,
    /// from outside, as harness flakiness.
    ///
    /// [`commands`] used to ask `i.key_pressed(key)` for the key and
    /// `i.modifiers` for the modifiers. Those are different clocks:
    /// `i.modifiers` is the state at the END of the frame, `Event::Key` carries
    /// the state at the KEYSTROKE. On a long frame — the application busy
    /// rasterizing a dense CAD sheet — an operator's fifty-millisecond
    /// `Ctrl+Z` arrives as press-and-release together, `i.modifiers` reports
    /// `Ctrl` already up, the chord matches nothing, and undo is silently
    /// dropped.
    ///
    /// This frame is exactly that: a `Ctrl+Z` key event carrying its own
    /// modifiers, followed by the modifiers going empty before the frame ends.
    /// Revert [`commands`] to the frame snapshot and it fails.
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

    /// ★ A chord with extra modifiers held does NOT fire the shorter one.
    ///
    /// `Ctrl+Z` is undo and `Ctrl+Shift+Z` is redo. egui's own
    /// `Modifiers::matches_logically` is permissive - it asks only that the
    /// pattern's modifiers are present - so a dispatcher built on it fires
    /// **both** on one keypress, and which of undo and redo wins is iteration
    /// order. This is why [`commands`] compares the three flags exactly.
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

    /// ★ A focused text field silences every chord.
    ///
    /// Both halves of the guard matter, and this is the `egui::TextEdit` one:
    /// `Ctrl+Z` inside a field is the field's undo, and a document-level undo
    /// firing underneath it would revert an edit the operator never touched.
    #[test]
    fn a_focused_text_field_silences_the_chords() {
        let ctx = Context::default();
        let keymap = built_in_keymap();
        let id = egui::Id::new("a-field");

        // Frame 1: register a real TextEdit and focus it, so
        // `text_edit_focused()` is genuinely true rather than assumed.
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            let mut text = String::new();
            let r = ui.add(egui::TextEdit::singleline(&mut text).id(id));
            r.request_focus();
        });

        let mut focused = false;
        let mut ids = Vec::new();
        let _ = ctx.run_ui(key_press(Key::Z, Modifiers::COMMAND), |ui| {
            let mut text = String::new();
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

    /// ★ ...and a CANVAS text draft silences them too.
    ///
    /// The half `text_edit_focused()` cannot see. The caret this shell paints
    /// on the page is not an `egui::TextEdit`, so egui reports no focused text
    /// field for an operator who is mid-word - and `[` is bound to
    /// `pages.rotate_left`. Without the draft term, typing a bracket rotates
    /// the drawing and inserts nothing.
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

    /// ★ **`Ctrl+0` is actual size, and it is the manifest that says so.**
    ///
    /// Both halves matter. The first is the decision — the browser
    /// convention, and what `view_zoom_actual`'s tooltip and the
    /// `canvas.empty` menu hint have claimed all along. The second is the
    /// structure: this asserts the id, not an [`Action`], because this module
    /// no longer knows what `view.zoom_actual` *does*.
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

    /// ★ **`Ctrl+O` reaches the Open command.**
    ///
    /// The chord was in the keymap and printed in `file_open`'s tooltip from
    /// the day the ribbon landed, and pressing it did **nothing**: the shell
    /// carried a hand-written spelling table that held only digits, so the key
    /// could not be spelled, so nothing was looked up. Two operator-visible
    /// surfaces named a chord that did not exist.
    ///
    /// It was fixed by adding a row to that table, which fixed exactly this
    /// chord and left thirteen others dead — the table is now gone and
    /// [`parse_chord`] reads the manifest, so the class is closed rather than
    /// the instance. See `every_chord_the_manifest_binds_actually_fires`.
    ///
    /// Asserted through the real keymap rather than an invented one, so the
    /// test fails if the binding is ever removed from the manifest as well as
    /// if the spelling is removed from here.
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
    ///
    /// `MODES_AND_PANELS.md` Part 1 §6 specifies these three, and all three
    /// tooltips in `crate::text::commands` name them. This is what makes
    /// those three sentences true.
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

    /// ★ **The meaning really is derived, not restated.**
    ///
    /// Hands `commands` a keymap that binds `Ctrl+0` to something else
    /// entirely and watches the chord follow it. This is the test that would
    /// fail if someone "simplified" the lookup back into a `match` here —
    /// which is exactly how the two-owner defect was written the first time.
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

    /// A chord bound to nothing produces nothing.
    ///
    /// `Ctrl+1` in a keymap that does not mention it must not fall back to
    /// some default this module remembers — there is no such memory, and the
    /// test is what keeps it that way.
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
