//! The Print window's N-up and booklet answers, and their lines in
//! `preferences.txt`. A group of [`super::printing::PrintPrefs`]; its parser and
//! writer are called from that group's.

use pdfcer_print::imposition::{Binding, PageOrder};

use super::printing::{KeyOutcome, PrintPrefs};

/// The pages-per-sheet counts offered, in the order offered.
pub const NUP_COUNTS: [u32; 5] = [2, 4, 6, 9, 16];

/// Which imposition is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImposeMode {
    /// One page per sheet (or poster, which is its own switch).
    #[default]
    Off,
    /// Several pages per sheet.
    NUp,
    /// A folded booklet.
    Booklet,
}

/// The remembered N-up and booklet settings. The booklet's sheet range and
/// which sides print are about one job and are not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImposePrefs {
    /// Which imposition is on.
    pub mode: ImposeMode,
    /// Pages per sheet; one of [`NUP_COUNTS`].
    pub count: u32,
    /// The order cells fill in.
    pub order: PageOrder,
    /// Outline each cell.
    pub border: bool,
    /// Turn a page a quarter turn when it then fits larger.
    pub rotate: bool,
    /// Which edge the booklet binds on.
    pub binding: Binding,
}

impl Default for ImposePrefs {
    fn default() -> Self {
        Self {
            mode: ImposeMode::Off,
            count: 2,
            order: PageOrder::Horizontal,
            border: false,
            rotate: true,
            binding: Binding::Left,
        }
    }
}

// ui-text-exempt: file tokens, never displayed.
const MODES: [(ImposeMode, &str); 3] = [
    (ImposeMode::Off, "off"),
    (ImposeMode::NUp, "n-up"),
    (ImposeMode::Booklet, "booklet"),
];
// ui-text-exempt: file tokens, never displayed.
const ORDERS: [(PageOrder, &str); 4] = [
    (PageOrder::Horizontal, "horizontal"),
    (PageOrder::HorizontalReversed, "horizontal-reversed"),
    (PageOrder::Vertical, "vertical"),
    (PageOrder::VerticalReversed, "vertical-reversed"),
];
// ui-text-exempt: file tokens, never displayed.
const BINDINGS: [(Binding, &str); 4] = [
    (Binding::Left, "left"),
    (Binding::Right, "right"),
    (Binding::LeftTall, "left-tall"),
    (Binding::RightTall, "right-tall"),
];

fn token<T: PartialEq + Copy>(table: &[(T, &'static str)], value: T) -> &'static str {
    table
        .iter()
        .find(|(v, _)| *v == value)
        .map_or("", |(_, t)| t)
}

fn from_token<T: Copy>(table: &[(T, &str)], token: &str) -> Option<T> {
    table.iter().find(|(_, t)| *t == token).map(|(v, _)| *v)
}

/// Read one line into [`ImposePrefs`], if it is one of this group's.
pub(super) fn parse_key(prefs: &mut PrintPrefs, key: &str, value: &str) -> KeyOutcome {
    let p = &mut prefs.impose;
    let bool_from = super::opening::bool_from_key;
    let stored = match key {
        // ui-text-exempt: file KEYS, parsed out of preferences.txt.
        "print_impose" => from_token(&MODES, value).map(|v| p.mode = v),
        "print_nup_count" => value
            .parse::<u32>()
            .ok()
            .filter(|n| NUP_COUNTS.contains(n))
            .map(|v| p.count = v),
        "print_nup_order" => from_token(&ORDERS, value).map(|v| p.order = v),
        "print_nup_border" => bool_from(value).map(|v| p.border = v),
        "print_nup_rotate" => bool_from(value).map(|v| p.rotate = v),
        "print_booklet_binding" => from_token(&BINDINGS, value).map(|v| p.binding = v),
        _ => return KeyOutcome::NotMine,
    };
    if stored.is_some() {
        KeyOutcome::Accepted
    } else {
        KeyOutcome::BadValue
    }
}

/// Append this group's lines.
pub(super) fn write_block(prefs: &PrintPrefs, out: &mut String) {
    out.push_str(
        "# print_impose: off | n-up | booklet. Several pages per sheet, or a\n\
         # folded booklet.\n\
         # print_nup_count: 2 | 4 | 6 | 9 | 16. Pages per sheet.\n\
         # print_nup_order: horizontal | horizontal-reversed | vertical |\n\
         # vertical-reversed. The order the pages fill the sheet.\n\
         # print_nup_border, print_nup_rotate: true | false. A line round each\n\
         # page; turn a page when it then prints larger.\n\
         # print_booklet_binding: left | right | left-tall | right-tall.\n",
    );
    let p = &prefs.impose;
    let bool_key = super::opening::bool_key;
    // ui-text-exempt: file KEYS, as above.
    for (key, value) in [
        ("print_impose", token(&MODES, p.mode).to_owned()),
        ("print_nup_count", p.count.to_string()),
        ("print_nup_order", token(&ORDERS, p.order).to_owned()),
        ("print_nup_border", bool_key(p.border).to_owned()),
        ("print_nup_rotate", bool_key(p.rotate).to_owned()),
        (
            "print_booklet_binding",
            token(&BINDINGS, p.binding).to_owned(),
        ),
    ] {
        out.push_str(key);
        out.push_str(" = "); // ui-text-exempt: the file's key-value separator.
        out.push_str(&value);
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_setting_round_trips_through_the_file() {
        let prefs = PrintPrefs {
            impose: ImposePrefs {
                mode: ImposeMode::Booklet,
                count: 9,
                order: PageOrder::VerticalReversed,
                border: true,
                rotate: false,
                binding: Binding::RightTall,
            },
            ..PrintPrefs::default()
        };
        let mut out = String::new();
        write_block(&prefs, &mut out);
        let mut back = PrintPrefs::default();
        for line in out.lines().filter(|l| !l.starts_with('#')) {
            let (k, v) = line.split_once(" = ").expect("key = value");
            assert_eq!(parse_key(&mut back, k, v), KeyOutcome::Accepted, "{line}");
        }
        assert_eq!(back.impose, prefs.impose);
    }

    #[test]
    fn a_count_not_offered_is_refused() {
        let mut prefs = PrintPrefs::default();
        assert_eq!(
            parse_key(&mut prefs, "print_nup_count", "3"),
            KeyOutcome::BadValue
        );
        assert_eq!(prefs.impose.count, 2);
    }
}
