//! # `dialogs::compact_images` — the compact window's picture options
//!
//! Contract: [`Pictures`] is what the operator asked done to the copy's
//! pictures, and [`Pictures::patch`] is the engine's [`ImagePatch`] for it,
//! `None` while the switch is off. [`switches`] draws the choices and greys
//! the pairings `ImagePatch::validate` refuses (JPEG with black and white or
//! fewer colours), each saying why on hover, so a refused patch is never
//! built. It also greys CCITT Group 4 and JBIG2 beside greyscale or fewer colours: the
//! engine accepts that patch but every picture it reaches is skipped, since
//! both store 1-bit pictures only. [`Tally`] counts what a plan did to pictures for the report.

use egui::Ui;
use pdfcer_core::optimize::{ImageConvert, ImageEncode, ImagePatch, OptimizePlan, QUALITY_LEVELS};

use crate::text::compact_images as t;

/// The fold's header, and the stem every other region here is named from.
pub const REGION_FOLD: &str = "compact.pictures";
/// The switch inside it.
pub const REGION_SWITCH: &str = "compact.pictures.on";
/// The full-size line saying the copy's pictures lose detail.
pub const REGION_LOSSY: &str = "compact.pictures.lossy";
// The storage list is `<stem>.format`, each choice `<stem>.format.<Format::key>`;
// the colour list `<stem>.colour`, each choice `<stem>.colour.<colour_key>`. The
// stem is `REGION_FOLD` for the one set of rows, a kind's own for the split.

/// The resolutions offered; `None` keeps every picture's size.
const RESOLUTIONS: [Option<u16>; 7] = [
    None,
    Some(72),
    Some(96),
    Some(150),
    Some(200),
    Some(300),
    Some(600),
];
/// The palette sizes offered for fewer colours.
const PALETTES: [u16; 3] = [256, 64, 16];

/// How the copy's pictures are stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// As each is stored now; a resized one keeps its kind.
    Keep,
    /// Flate at level 9.
    Lossless,
    /// The smallest exact storage the engine finds for each picture.
    Smallest,
    /// JPEG at [`Pictures::quality`].
    Jpeg,
    /// LZW: exact, for workflows that expect it; refused under PDF/A.
    Lzw,
    /// Run length: exact, small for flat areas of one colour.
    RunLength,
    /// CCITT Group 4: exact, 1-bit pictures only; others are left as stored.
    Ccitt,
    /// JBIG2: exact, 1-bit pictures only, PDF 1.4 and later.
    Jbig2,
}

impl Format {
    /// Every choice, in the order the list shows them.
    pub const ALL: [Self; 8] = [
        Self::Keep,
        Self::Lossless,
        Self::Smallest,
        Self::Jpeg,
        Self::Lzw,
        Self::RunLength,
        Self::Ccitt,
        Self::Jbig2,
    ];

    /// The choice's name in a region, never displayed.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::Lossless => "lossless",
            Self::Smallest => "smallest",
            Self::Jpeg => "jpeg",
            Self::Lzw => "lzw",
            Self::RunLength => "runlength",
            Self::Ccitt => "ccitt",
            Self::Jbig2 => "jbig2",
        }
    }
}

/// A colour choice's name in a region, never displayed.
pub const fn colour_key(colour: ImageConvert) -> &'static str {
    match colour {
        ImageConvert::Gray => "gray",
        ImageConvert::Bilevel => "bilevel",
        ImageConvert::Indexed => "indexed",
        _ => "keep",
    }
}

/// What the operator asked done to the copy's pictures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pictures {
    /// The switch; nothing below applies while it is off.
    pub on: bool,
    /// Target pixels per inch at a picture's largest placement.
    pub ppi: Option<u16>,
    /// How they are stored.
    pub format: Format,
    /// JPEG quality, one of the engine's named levels.
    pub quality: u8,
    /// Colour conversion.
    pub colour: ImageConvert,
    /// Palette size for [`ImageConvert::Indexed`].
    pub colors: u16,
}

impl Pictures {
    /// Off, with the choices it opens on: 150 ppi, stored as now, high
    /// quality, colours kept.
    pub const OFF: Self = Self {
        on: false,
        ppi: Some(150),
        format: Format::Keep,
        quality: QUALITY_LEVELS[3].1,
        colour: ImageConvert::Keep,
        colors: 256,
    };

    /// The engine patch these choices ask for; `None` while off.
    #[must_use]
    pub fn patch(self) -> Option<ImagePatch> {
        if !self.on {
            return None;
        }
        let mut patch = ImagePatch::new();
        patch.ppi = self.ppi;
        patch.encode = Some(match self.format {
            Format::Keep => ImageEncode::Keep,
            Format::Lossless => ImageEncode::Flate { level: 9 },
            Format::Smallest => ImageEncode::Auto,
            Format::Jpeg => ImageEncode::Jpeg,
            Format::Lzw => ImageEncode::Lzw,
            Format::RunLength => ImageEncode::RunLength,
            Format::Ccitt => ImageEncode::Ccitt,
            Format::Jbig2 => ImageEncode::Jbig2,
        });
        if self.format == Format::Jpeg {
            patch.quality = Some(self.quality);
        }
        if self.colour != ImageConvert::Keep {
            patch.convert = Some(self.colour);
        }
        if self.colour == ImageConvert::Indexed {
            patch.colors = Some(self.colors);
        }
        Some(patch)
    }

    /// Whether `colour` cannot be stored as `format`: the engine refuses it
    /// (JPEG), or would skip every picture (CCITT or JBIG2 beside a multi-bit result).
    pub(crate) const fn clashes(format: Format, colour: ImageConvert) -> bool {
        match format {
            Format::Jpeg => matches!(colour, ImageConvert::Bilevel | ImageConvert::Indexed),
            Format::Ccitt | Format::Jbig2 => {
                matches!(colour, ImageConvert::Gray | ImageConvert::Indexed)
            }
            _ => false,
        }
    }
}

/// What a plan did to pictures.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tally {
    /// Scaled down.
    pub resized: usize,
    /// Written as JPEG.
    pub jpeg: usize,
    /// Written as CCITT Group 4, JBIG2, LZW or run length, in that order.
    pub exact: [usize; 4],
    /// Pictures the engine stored every exact way and kept the smallest of.
    pub tried: usize,
    /// Colour-converted.
    pub converted: usize,
    /// Whether any change lost information, by the engine's own test.
    pub lossy: bool,
}

impl Tally {
    /// Count `plan`'s picture changes.
    #[must_use]
    pub fn of(plan: &OptimizePlan) -> Self {
        let mut tally = Self {
            lossy: plan.is_lossy(),
            ..Self::default()
        };
        for image in plan.changes.iter().filter_map(|c| c.image.as_ref()) {
            tally.resized += usize::from(image.size_after != image.size_before);
            tally.jpeg += usize::from(image.encoded_as == ImageEncode::Jpeg);
            let exact = [
                ImageEncode::Ccitt,
                ImageEncode::Jbig2,
                ImageEncode::Lzw,
                ImageEncode::RunLength,
            ];
            for (count, encode) in tally.exact.iter_mut().zip(exact) {
                *count += usize::from(image.encoded_as == encode);
            }
            tally.converted += usize::from(image.conversion.is_some());
            tally.tried += usize::from(!image.candidates.is_empty());
        }
        tally
    }

    /// The report's lines.
    #[must_use]
    pub fn lines(self) -> Vec<String> {
        let mut lines = t::done(self.resized, self.jpeg, self.converted);
        lines.extend(t::stored_exact(self.exact));
        lines
    }
}

/// The Pictures fold; whether any choice changed. With `classes` split, the
/// one set of rows gives way to one per kind of picture.
pub fn switches(
    ui: &mut Ui,
    pictures: &mut Pictures,
    classes: &mut crate::dialogs::compact_classes::Classes,
) -> bool {
    let fold = egui::CollapsingHeader::new(t::heading(pictures.on))
        .id_salt("compact-pictures") // ui-text-exempt: an id, never displayed
        .show(ui, |ui| {
            let before = *pictures;
            let switch = ui
                .checkbox(&mut pictures.on, t::toggle())
                .on_hover_text(t::toggle_tooltip());
            crate::diag::ui_rect_visible(REGION_SWITCH, switch.rect, ui.clip_rect());
            let split_before = *classes;
            ui.add_enabled_ui(pictures.on, |ui| {
                if !crate::dialogs::compact_classes::show(ui, classes) {
                    egui::Grid::new("compact-pictures-grid") // ui-text-exempt: an id, never displayed
                        .show(ui, |ui| rows(ui, pictures));
                }
            });
            *pictures != before || *classes != split_before
        });
    crate::diag::ui_rect_visible(REGION_FOLD, fold.header_response.rect, ui.clip_rect());
    fold.body_returned.unwrap_or(false)
}

/// The resolution, format and quality rows, then [`colour_rows`].
pub(crate) fn rows(ui: &mut Ui, p: &mut Pictures) {
    rows_in(ui, p, REGION_FOLD, false);
}

/// [`rows`] with its regions named from `stem`. `one_bit` is for black-and-white
/// pictures: no colour row, and JPEG greyed, since the engine stores no
/// 1-bit picture as JPEG.
pub(crate) fn rows_in(ui: &mut Ui, p: &mut Pictures, stem: &str, one_bit: bool) {
    ui.label(t::resolution_label())
        .on_hover_text(t::resolution_tooltip());
    egui::ComboBox::from_id_salt("compact-pictures-ppi") // ui-text-exempt: an id, never displayed
        .selected_text(t::resolution(p.ppi))
        .show_ui(ui, |ui| {
            for ppi in RESOLUTIONS {
                ui.selectable_value(&mut p.ppi, ppi, t::resolution(ppi));
            }
        });
    ui.end_row();
    ui.label(t::format_label());
    let shown = t::format(format_word(p.format));
    let list = egui::ComboBox::from_id_salt("compact-pictures-format") // ui-text-exempt: an id, never displayed
        .selected_text(shown)
        // Every choice in view: the default height scrolls the last one out.
        .height(f32::INFINITY)
        .show_ui(ui, |ui| {
            for format in Format::ALL {
                let word = format_word(format);
                let open = if one_bit {
                    format != Format::Jpeg
                } else {
                    !Pictures::clashes(format, p.colour)
                };
                let ruled_out = if one_bit {
                    t::format_one_bit_ruled_out()
                } else {
                    t::format_ruled_out(word)
                };
                let item = ui
                    .add_enabled(
                        open,
                        egui::Button::selectable(p.format == format, t::format(word)),
                    )
                    .on_hover_text(t::format_tooltip(word))
                    .on_disabled_hover_text(ruled_out);
                let region = format!("{stem}.format.{}", format.key());
                crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                if item.clicked() {
                    p.format = format;
                }
            }
        });
    crate::diag::ui_rect_visible(
        &format!("{stem}.format"),
        list.response.rect,
        ui.clip_rect(),
    );
    ui.end_row();
    if p.format == Format::Jpeg {
        ui.label(t::quality_label())
            .on_hover_text(t::quality_tooltip());
        egui::ComboBox::from_id_salt("compact-pictures-quality") // ui-text-exempt: an id, never displayed
            .selected_text(t::quality(p.quality))
            .show_ui(ui, |ui| {
                for (_, q) in QUALITY_LEVELS {
                    ui.selectable_value(&mut p.quality, q, t::quality(q));
                }
            });
        ui.end_row();
    }
    if !one_bit {
        colour_rows(ui, p, stem);
    }
}

/// The colour row, and the palette size under fewer colours.
fn colour_rows(ui: &mut Ui, p: &mut Pictures, stem: &str) {
    ui.label(t::colour_label());
    let list = egui::ComboBox::from_id_salt("compact-pictures-colour") // ui-text-exempt: an id, never displayed
        .selected_text(t::colour(p.colour))
        .show_ui(ui, |ui| {
            for colour in ImageConvert::ALL {
                let open = !Pictures::clashes(p.format, colour);
                let item = ui
                    .add_enabled(
                        open,
                        egui::Button::selectable(p.colour == colour, t::colour(colour)),
                    )
                    .on_disabled_hover_text(t::colour_ruled_out(format_word(p.format)));
                let region = format!("{stem}.colour.{}", colour_key(colour));
                crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                if item.clicked() {
                    p.colour = colour;
                }
            }
        });
    crate::diag::ui_rect_visible(
        &format!("{stem}.colour"),
        list.response.rect,
        ui.clip_rect(),
    );
    ui.end_row();
    if p.colour == ImageConvert::Indexed {
        ui.label(t::colors_label());
        egui::ComboBox::from_id_salt("compact-pictures-colors") // ui-text-exempt: an id, never displayed
            .selected_text(t::colors(p.colors))
            .show_ui(ui, |ui| {
                for n in PALETTES {
                    ui.selectable_value(&mut p.colors, n, t::colors(n));
                }
            });
        ui.end_row();
    }
}

/// The wording's name for a choice.
const fn format_word(format: Format) -> t::FormatWord {
    match format {
        Format::Keep => t::FormatWord::Keep,
        Format::Lossless => t::FormatWord::Lossless,
        Format::Smallest => t::FormatWord::Smallest,
        Format::Jpeg => t::FormatWord::Jpeg,
        Format::Lzw => t::FormatWord::Lzw,
        Format::RunLength => t::FormatWord::RunLength,
        Format::Ccitt => t::FormatWord::Ccitt,
        Format::Jbig2 => t::FormatWord::Jbig2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Off asks the engine for nothing.
    #[test]
    fn off_builds_no_patch() {
        assert_eq!(Pictures::OFF.patch(), None);
    }

    /// Every pairing the window lets the operator reach is one the engine
    /// accepts; every pairing it greys is one the engine refuses, or CCITT
    /// or JBIG2 beside a multi-bit colour result, which the engine accepts and skips.
    #[test]
    fn the_greyed_pairings_are_exactly_the_refused_ones() {
        for format in Format::ALL {
            for colour in ImageConvert::ALL {
                let p = Pictures {
                    on: true,
                    format,
                    colour,
                    ..Pictures::OFF
                };
                let accepted = p.patch().expect("on").validate().is_ok();
                let pointless = matches!(format, Format::Ccitt | Format::Jbig2)
                    && matches!(colour, ImageConvert::Gray | ImageConvert::Indexed);
                assert_eq!(
                    accepted && !pointless,
                    !Pictures::clashes(format, colour),
                    "{format:?} {colour:?}"
                );
            }
        }
    }

    /// Quality and palette size ride only with the choice that uses them.
    #[test]
    fn a_setting_rides_only_with_its_choice() {
        let lossless = Pictures {
            on: true,
            format: Format::Lossless,
            ..Pictures::OFF
        };
        let patch = lossless.patch().expect("on");
        assert_eq!(
            (patch.quality, patch.colors, patch.convert),
            (None, None, None)
        );
        let jpeg = Pictures {
            on: true,
            format: Format::Jpeg,
            ..Pictures::OFF
        };
        assert_eq!(jpeg.patch().expect("on").quality, Some(80));
    }

    /// Each exact format asks the engine for its own encoder, with no quality.
    #[test]
    fn each_exact_format_names_its_encoder() {
        for (format, encode) in [
            (Format::Smallest, ImageEncode::Auto),
            (Format::Lzw, ImageEncode::Lzw),
            (Format::RunLength, ImageEncode::RunLength),
            (Format::Ccitt, ImageEncode::Ccitt),
            (Format::Jbig2, ImageEncode::Jbig2),
        ] {
            let p = Pictures {
                on: true,
                format,
                ..Pictures::OFF
            };
            let patch = p.patch().expect("on");
            assert_eq!((patch.encode, patch.quality), (Some(encode), None));
        }
    }
}
