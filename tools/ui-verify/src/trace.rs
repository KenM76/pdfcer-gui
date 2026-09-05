//! Parse the application's `key=value` diagnostic trace.
//!
//! ## The format, and why it is worth a parser
//!
//! With its diagnostic environment variable set, the application writes one
//! line per event to **stderr**:
//!
//! ```text
//! pdfcer-diag start argv1=Some("a.pdf") viewport=ViewportBuilder { .. }
//! pdfcer-diag canvas tool=None rect=[[240.0 96.0] - [1560.0 968.0]] zoom=1.5 sel=0
//! pdfcer-diag vector-click screen=[820.0 514.0] canvas=[580.0 418.0] hits=1 newsel=1
//! pdfcer-diag delete-objects n=1 indices=[7]
//! ```
//!
//! Those four are the **old** binary's. The one this project is building
//! speaks a wider dialect, and the three lines below are what
//! `PROJECT_PLAN.md` §4.3 asked it for:
//!
//! ```text
//! pdfcer-diag canvas rect=[[16.0 22.8] - [1084.0 777.2]] zoom=0.4480 page=0 pages=1 off=[0.0 0.0]
//! pdfcer-diag ui-rect name=canvas-viewport rect=[[8.0 8.0] - [1092.0 792.0]]
//! pdfcer-diag objects n=28 page=0 paths=13 text=15 images=0 forms=0
//! ```
//!
//! Note what does **not** appear in the second set: `sel=`. A field the
//! application omits because it has nothing to say is not a field with the
//! value zero, and this parser preserves that distinction rather than
//! flattening it — [`TraceLine::get_usize`] returns `None` for an absent field
//! and for a literal `None`, and every caller is expected to have an answer
//! for that case that is not "assume 0". The same rule governs failure: this
//! application reports a failure as a *different event*
//! (`canvas-unavailable reason=…`, `objects-unavailable page=… reason=…`),
//! never as the success event with a field missing. So a missing field on a
//! success line is a **parse bug in this module**, not a zero, and it should
//! be chased here rather than absorbed at a call site.
//!
//! stderr because it needs no path, no open handle, no failure mode of its own,
//! and redirects with `2>`. `key=value` because the consumer is a grep, an LLM
//! or this parser — never a person reading a log.
//!
//! A regex would nearly work and then fail on the interesting lines. The values
//! are Rust `Debug` output, so they contain spaces inside brackets
//! (`rect=[[0.0 0.0] - [16.0 9.0]]`), inside parentheses (`tool=Some(Obj)`) and
//! inside quotes (`argv1=Some("my file.pdf")`). Splitting on whitespace gives
//! `rect=[[0.0` and four fragments; splitting on `=` gives nonsense the moment
//! a value contains one. So the splitter below tracks bracket depth and string
//! state, and a key boundary only counts at depth zero.
//!
//! That is not hypothetical fussiness: `rect=` and `zoom=` are exactly the two
//! fields [`crate::coords`] needs to convert a document point into a click, and
//! `rect=` is the one whose value contains spaces.
//!
//! ## What this module deliberately does not do
//!
//! It does not interpret. `hits=1` is a string `"1"` until someone asks for it
//! as a number, and an event name is a string, not an enum. The vocabulary —
//! which event carries the selection count, which field holds it — lives in
//! [`crate::profile`], because it differs between the binary this project is
//! building and the binary it is replacing, and a parser that hard-coded one
//! of them could not be pointed at the other.

use std::collections::BTreeMap;

use crate::error::{Error, Result};

/// One parsed trace line.
#[derive(Clone, Debug)]
pub struct TraceLine {
    /// 1-based line number within the stderr capture, for error messages that
    /// a human can go and look at.
    pub lineno: usize,
    /// The event name — the first token after the prefix.
    pub event: String,
    /// The whole line as it was written, kept so a failure report can quote
    /// the evidence rather than a reconstruction of it.
    pub raw: String,
    fields: BTreeMap<String, String>,
}

impl TraceLine {
    /// A field's raw value, or `None` if the line does not carry it.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(|v| {
            // ★ A quoted value is returned WITHOUT its quotes.
            //
            // The application quotes any value that may contain a character
            // this parser gives structural meaning to — a chord spelled `[`
            // being the case that forced it. Every caller wants the string the
            // application meant, not its literal, and leaving the quotes on
            // would make `l.get("chord") == Some("Ctrl+Z")` silently false for
            // exactly the lines that needed quoting most.
            let v = v.as_str();
            v.strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(v)
        })
    }

    /// A field parsed as an integer.
    ///
    /// Tolerates the `Some(3)` wrapper, because `Debug` on an `Option<usize>`
    /// is what several of these fields actually are and requiring every call
    /// site to strip it would put the same three lines in five places.
    #[must_use]
    pub fn get_usize(&self, key: &str) -> Option<usize> {
        let v = self.get(key)?;
        let v = unwrap_debug_option(v)?;
        v.trim().parse::<usize>().ok()
    }

    /// A field parsed as a float, with the same `Some(..)` tolerance.
    #[must_use]
    pub fn get_f32(&self, key: &str) -> Option<f32> {
        let v = self.get(key)?;
        let v = unwrap_debug_option(v)?;
        v.trim().parse::<f32>().ok()
    }

    /// A field parsed as an egui-style rectangle: `[[x0 y0] - [x1 y1]]`.
    ///
    /// That is `egui::Rect`'s `Debug` shape, via `Pos2`'s `[x y]`. Accepts the
    /// `Some(..)` wrapper for the same reason as above.
    #[must_use]
    pub fn get_rect(&self, key: &str) -> Option<crate::geom::LRect> {
        parse_egui_rect(unwrap_debug_option(self.get(key)?)?)
    }

    /// A field parsed as an egui-style position or vector: `[x y]`.
    ///
    /// `egui::Pos2` and `egui::Vec2` share this `Debug` shape, which is why
    /// one accessor serves both.
    #[must_use]
    pub fn get_vec2(&self, key: &str) -> Option<crate::geom::Pt> {
        let (x, y) = parse_egui_pos(unwrap_debug_option(self.get(key)?)?)?;
        Some(crate::geom::Pt::new(x, y))
    }

    /// Every field on the line, in key order — for diagnostics that want to
    /// show what *was* there when the field being looked for was not.
    #[must_use]
    pub fn field_names(&self) -> Vec<&str> {
        self.fields.keys().map(String::as_str).collect()
    }
}

/// A whole captured trace.
#[derive(Clone, Debug, Default)]
pub struct Trace {
    /// Every line that carried the expected prefix, in order.
    pub lines: Vec<TraceLine>,
    /// Lines in the capture that did **not** carry the prefix. Kept rather
    /// than dropped: a panic message, a wgpu warning or a Rust backtrace lands
    /// here, and when a check fails the first useful question is usually
    /// "did the process say anything else?".
    pub other: Vec<String>,
}

impl Trace {
    /// Parse a captured stderr stream.
    ///
    /// `prefix` is the marker the application puts at the head of every
    /// diagnostic line (`"pdfcer-diag"`). Lines without it go to
    /// [`Trace::other`].
    #[must_use]
    pub fn parse(text: &str, prefix: &str) -> Self {
        let mut trace = Self::default();
        for (i, raw) in text.lines().enumerate() {
            let lineno = i + 1;
            let Some(rest) = raw.strip_prefix(prefix) else {
                if !raw.trim().is_empty() {
                    trace.other.push(raw.to_owned());
                }
                continue;
            };
            let rest = rest.trim_start();
            let (event, tail) = match rest.find(' ') {
                Some(idx) => (&rest[..idx], &rest[idx + 1..]),
                None => (rest, ""),
            };
            if event.is_empty() {
                continue;
            }
            trace.lines.push(TraceLine {
                lineno,
                event: event.to_owned(),
                raw: raw.to_owned(),
                fields: parse_fields(tail),
            });
        }
        trace
    }

    /// Read a captured stderr file and parse it.
    ///
    /// Reads lossily: a crashing process can leave a partial UTF-8 sequence at
    /// the tail of the file, and a harness that returned "invalid UTF-8"
    /// instead of the ninety good lines above it would be hiding the evidence
    /// at the exact moment it matters most.
    pub fn read(path: &std::path::Path, prefix: &str) -> Result<Self> {
        let bytes = std::fs::read(path)
            .map_err(|e| Error::new(format!("cannot read the trace at {}: {e}", path.display())))?;
        Ok(Self::parse(&String::from_utf8_lossy(&bytes), prefix))
    }

    /// Every line with this event name, in order.
    pub fn events<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a TraceLine> + 'a {
        self.lines.iter().filter(move |l| l.event == name)
    }

    /// The last line with this event name.
    ///
    /// Written as an explicit reverse search rather than `events(..).last()`
    /// so it stops at the first match from the end instead of walking the
    /// whole trace. Traces from a real run are tens of thousands of lines and
    /// several checks ask this question per assertion.
    #[must_use]
    pub fn last(&self, name: &str) -> Option<&TraceLine> {
        self.lines.iter().rev().find(|l| l.event == name)
    }

    /// The first line with this event name.
    #[must_use]
    pub fn first(&self, name: &str) -> Option<&TraceLine> {
        self.lines.iter().find(|l| l.event == name)
    }

    /// The last line with this event name that the application traced **after**
    /// `after` — where `after` is a [`TraceLine::lineno`] taken earlier.
    ///
    /// # ★★★ Why this exists: [`Trace::last`] cannot tell "unchanged" from
    /// "stopped"
    ///
    /// A trace is an append-only log, so `last` answers *"what is the newest
    /// line this run ever produced?"* — which is the right question only while
    /// the thing producing it is still producing. The moment a surface stops
    /// emitting, its final line stands for ever, and a check that keeps reading
    /// it sees **a number that never changes** and reports the feature behind
    /// it as inert.
    ///
    /// That is not hypothetical, and it is the third recurrence of one shape in
    /// this crate. [`crate::checks::driving::declared_since`] carries the first
    /// two (a drop caret gone by the time it was read; a deleted row still
    /// counted). The third, on 2026-09-05, cost a full sweep two false defect
    /// reports:
    ///
    /// > `save_copy_round_trip` and `undo_redo_round_trip` both read
    /// > `comments-panel … listed=` with `last`, and both reported *"THE
    /// > COMMENTS PANEL DOES NOT SEE THE ANNOTATION THAT WAS JUST AUTHORED"*.
    /// > The panel saw it perfectly. It had been sent to the back of a tabbed
    /// > dock by a persisted layout, a dock draws only its active tab, and so
    /// > the panel had stopped tracing three hundred frames before the drag.
    /// > `last` handed both checks the census it published in the *previous
    /// > mode*.
    ///
    /// ⇒ **If a check compares a number to what that number was earlier, the
    /// later read must be anchored.** `after` is normally the `lineno` of the
    /// event that is supposed to have caused the change — a commit line, a
    /// gesture's start — so a value published before the cause cannot satisfy
    /// it, and `None` means *"the surface said nothing since"*, which is a
    /// different verdict from *"the surface said the same thing"* and must be
    /// reported differently.
    ///
    /// `TraceLine::lineno` is the line's position in the capture, so marks
    /// taken from any event are directly comparable with any other — the same
    /// property `declared_since` relies on.
    #[must_use]
    pub fn last_after(&self, name: &str, after: usize) -> Option<&TraceLine> {
        self.lines
            .iter()
            .rev()
            .take_while(|l| l.lineno > after)
            .find(|l| l.event == name)
    }

    /// The line number of the newest line in the capture, for use as an anchor
    /// with [`Trace::last_after`].
    ///
    /// Zero on an empty capture, which is the correct anchor for "everything
    /// from here on": no line can have `lineno` 0.
    #[must_use]
    pub fn mark(&self) -> usize {
        self.lines.last().map_or(0, |l| l.lineno)
    }

    /// Did the application emit anything at all under the prefix?
    ///
    /// The distinction this answers is the one that cost pdfcer's investigation
    /// a round trip on 2026-08-04: an empty trace means either "the process
    /// never saw the diagnostic environment variable" or "the process saw
    /// nothing worth reporting", and those need different fixes. The
    /// application emits an unconditional `start` line so the two can be told
    /// apart; [`Trace::started`] is the question that uses it.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Did the process reach its unconditional first trace line?
    ///
    /// `false` with a non-empty capture means the diagnostic variable did not
    /// reach the process. `false` with an empty capture means the process
    /// produced nothing at all — a bad binary, or a crash before start.
    #[must_use]
    pub fn started(&self, start_event: &str) -> bool {
        self.first(start_event).is_some()
    }

    /// Script steps the application rejected as unparseable.
    ///
    /// Always worth printing, whatever a check was looking for. pdfcer records
    /// two working features being declared broken because their scripts used
    /// step names that did not exist: the harness traced the rejection on every
    /// single run, and every filter in use matched only the traces the test
    /// *expected*, so the explanation was never seen. A filter that matches
    /// only your expectation cannot tell you your input was wrong.
    #[must_use]
    pub fn rejected_steps(&self) -> Vec<&TraceLine> {
        self.lines
            .iter()
            .filter(|l| l.event.contains("UNPARSEABLE") || l.raw.contains("UNPARSEABLE"))
            .collect()
    }
}

/// Strip a `Debug`-printed `Some(..)` wrapper, and treat `None` as absent.
///
/// `None` mapping to `None` is the point: a field whose value is literally the
/// string `None` is the application saying "there is nothing here", and a
/// caller asking for it as a number wants the same answer as if the field had
/// been missing.
fn unwrap_debug_option(v: &str) -> Option<&str> {
    let v = v.trim();
    if v == "None" {
        return None;
    }
    match v.strip_prefix("Some(").and_then(|s| s.strip_suffix(')')) {
        Some(inner) => Some(inner.trim()),
        None => Some(v),
    }
}

/// `[[x0 y0] - [x1 y1]]` — `egui::Rect`'s `Debug`.
fn parse_egui_rect(v: &str) -> Option<crate::geom::LRect> {
    use crate::geom::{LRect, Pt};
    let v = v.trim();
    let inner = v.strip_prefix('[')?.strip_suffix(']')?;
    let (a, b) = inner.split_once(" - ")?;
    let a = parse_egui_pos(a)?;
    let b = parse_egui_pos(b)?;
    Some(LRect::new(Pt::new(a.0, a.1), Pt::new(b.0, b.1)))
}

/// `[x y]` — `egui::Pos2`'s and `egui::Vec2`'s `Debug`.
fn parse_egui_pos(v: &str) -> Option<(f32, f32)> {
    let v = v.trim().strip_prefix('[')?.strip_suffix(']')?;
    let mut it = v.split_whitespace();
    let x = it.next()?.parse::<f32>().ok()?;
    let y = it.next()?.parse::<f32>().ok()?;
    Some((x, y))
}

/// Split a trace line's tail into `key=value` pairs.
///
/// The algorithm, and why it is not a regex or a `split_whitespace`:
///
/// 1. Walk the characters, maintaining bracket depth over `[`, `(`, `{` and a
///    flag for being inside a double-quoted string (honouring `\` escapes).
/// 2. A **key boundary** is an identifier starting at depth zero, outside a
///    string, at the start of the tail or immediately after a space, and
///    followed by `=`.
/// 3. Each value runs from just after its `=` to just before the next key
///    boundary, or to end of line for the last one.
///
/// Step 2's depth condition is the whole reason this is a function rather than
/// a one-liner: `rect=[[0.0 0.0] - [16.0 9.0]] zoom=1.5` contains no key
/// boundary inside the brackets, and a splitter that did not know that would
/// produce a field called `0` and lose `rect` entirely.
fn parse_fields(tail: &str) -> BTreeMap<String, String> {
    let chars: Vec<char> = tail.chars().collect();
    let mut boundaries: Vec<(usize, usize)> = Vec::new(); // (key start, index of '=')
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        if in_string {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                i += 1;
                continue;
            }
            '[' | '(' | '{' => depth += 1,
            ']' | ')' | '}' => depth -= 1,
            _ => {}
        }
        let at_boundary = i == 0 || chars[i - 1] == ' ';
        if depth == 0 && at_boundary && is_ident_start(c) {
            let mut j = i;
            while j < chars.len() && is_ident(chars[j]) {
                j += 1;
            }
            if j < chars.len() && chars[j] == '=' {
                boundaries.push((i, j));
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }

    let mut out = BTreeMap::new();
    for (n, &(start, eq)) in boundaries.iter().enumerate() {
        let key: String = chars[start..eq].iter().collect();
        let end = boundaries
            .get(n + 1)
            .map_or(chars.len(), |&(next_start, _)| next_start);
        let value: String = chars[eq + 1..end].iter().collect();
        // Later wins. A trace line should not repeat a key, but if one does,
        // the rightmost is the one a human reading the line would take.
        out.insert(key, value.trim_end().to_owned());
    }
    out
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    const PREFIX: &str = "pdfcer-diag";

    #[test]
    fn parses_a_plain_line() {
        let t = Trace::parse("pdfcer-diag delete-objects n=1 indices=[7]", PREFIX);
        assert_eq!(t.lines.len(), 1);
        let l = &t.lines[0];
        assert_eq!(l.event, "delete-objects");
        assert_eq!(l.get_usize("n"), Some(1));
        assert_eq!(l.get("indices"), Some("[7]"));
    }

    /// The case a whitespace splitter gets wrong, and the reason this parser
    /// exists: `rect`'s value contains four spaces and a hyphen, and `zoom`
    /// immediately follows it.
    #[test]
    fn a_value_may_contain_spaces_inside_brackets() {
        let t = Trace::parse(
            "pdfcer-diag canvas rect=[[240.0 96.0] - [1560.0 968.0]] zoom=1.5 sel=2",
            PREFIX,
        );
        let l = t.last("canvas").expect("canvas line");
        let r = l.get_rect("rect").expect("a parsable rect");
        assert_eq!(r.width(), 1320.0);
        assert_eq!(r.height(), 872.0);
        assert_eq!(l.get_f32("zoom"), Some(1.5));
        assert_eq!(l.get_usize("sel"), Some(2));
    }

    #[test]
    fn a_value_may_contain_spaces_inside_quotes() {
        let t = Trace::parse(
            "pdfcer-diag start argv1=Some(\"my drawing a.pdf\") viewport=Some(1)",
            PREFIX,
        );
        let l = t.first("start").expect("start line");
        assert_eq!(l.get("argv1"), Some("Some(\"my drawing a.pdf\")"));
        assert_eq!(l.get_usize("viewport"), Some(1));
    }

    #[test]
    fn debug_option_wrappers_are_transparent_and_none_reads_as_absent() {
        let t = Trace::parse("pdfcer-diag canvas first=Some(3) second=None", PREFIX);
        let l = t.last("canvas").unwrap();
        assert_eq!(l.get_usize("first"), Some(3));
        assert_eq!(
            l.get_usize("second"),
            None,
            "a field whose value is None must read as absent, not as a parse failure"
        );
    }

    #[test]
    fn non_prefixed_output_is_kept_not_discarded() {
        let t = Trace::parse(
            "thread 'main' panicked at src/main.rs:1:1\npdfcer-diag start argv1=None",
            PREFIX,
        );
        assert_eq!(t.lines.len(), 1);
        assert_eq!(t.other.len(), 1);
        assert!(t.other[0].contains("panicked"));
    }

    #[test]
    fn started_distinguishes_no_diag_from_no_output() {
        assert!(!Trace::parse("", PREFIX).started("start"));
        assert!(!Trace::parse("some unrelated stderr", PREFIX).started("start"));
        assert!(Trace::parse("pdfcer-diag start argv1=None", PREFIX).started("start"));
    }

    #[test]
    fn rejected_script_steps_are_findable() {
        let t = Trace::parse("pdfcer-diag script-step-UNPARSEABLE step=nav:home", PREFIX);
        assert_eq!(t.rejected_steps().len(), 1);
    }

    /// The exact shape of the 2026-09-05 false report, in five lines.
    ///
    /// A surface publishes a census, something else happens, and the surface
    /// **stops publishing**. `last` still answers with the stale census — which
    /// is what made two checks report a working panel as broken — and
    /// `last_after`, anchored on the cause, correctly answers `None`.
    #[test]
    fn last_reads_a_fossil_where_last_after_reports_silence() {
        let t = Trace::parse(
            "pdfcer-diag comments-panel listed=12\n\
             pdfcer-diag mode-changed to=review\n\
             pdfcer-diag markup-commit kind=Rectangle\n\
             pdfcer-diag add-markup page=0\n\
             pdfcer-diag frame n=70",
            PREFIX,
        );
        let cause = t.last("add-markup").unwrap().lineno;
        assert_eq!(
            t.last("comments-panel").and_then(|l| l.get_usize("listed")),
            Some(12),
            "`last` reads the census the panel published before it went quiet — the fossil"
        );
        assert!(
            t.last_after("comments-panel", cause).is_none(),
            "anchored on the edit that should have moved it, the panel said NOTHING — which is a \
             different verdict from `it said 12` and must not be reported as one"
        );
    }

    #[test]
    fn last_after_finds_the_newest_line_past_the_anchor() {
        let t = Trace::parse(
            "pdfcer-diag comments-panel listed=12\n\
             pdfcer-diag add-markup page=0\n\
             pdfcer-diag comments-panel listed=13\n\
             pdfcer-diag comments-panel listed=13",
            PREFIX,
        );
        let cause = t.last("add-markup").unwrap().lineno;
        assert_eq!(
            t.last_after("comments-panel", cause)
                .and_then(|l| l.get_usize("listed")),
            Some(13)
        );
        assert!(
            t.last_after("comments-panel", 9_999).is_none(),
            "an anchor past the end of the capture can never be satisfied"
        );
    }

    #[test]
    fn mark_is_the_newest_line_and_zero_on_an_empty_capture() {
        assert_eq!(Trace::parse("", PREFIX).mark(), 0);
        let t = Trace::parse("noise\npdfcer-diag start argv1=None\nmore noise", PREFIX);
        assert_eq!(t.mark(), 2, "the anchor is the position in the FILE");
        assert!(
            t.last_after("start", t.mark()).is_none(),
            "a mark taken now must exclude every line that already exists"
        );
    }
}
