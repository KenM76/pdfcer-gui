//! Unit tests for `app::actions::split`: the preview's refusals and the
//! written files.

use std::path::PathBuf;

use super::*;
use crate::app::state::{FOUR_PAGES, open_fixture, open_local_fixture};

/// An empty scratch folder under the OS temporary directory.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("pdfcer-gui-split-tests-{}", std::process::id()))
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the temporary directory must be creatable");
    dir
}

fn request(criterion: SplitCriterion, template: &str, folder: &Path) -> SplitRequest {
    SplitRequest {
        criterion,
        template: template.to_owned(),
        folder: folder.to_path_buf(),
        labels: ExtractedPageLabels::Keep,
    }
}

fn names(preview: &Preview) -> Vec<String> {
    match preview {
        Preview::Parts { parts, .. } => parts.iter().map(|p| p.name.clone()).collect(),
        Preview::Refused(why) => panic!("the preview refused: {why}"),
    }
}

/// **The preview lists the engine's names and ranges for each rule.**
#[test]
fn the_preview_names_each_part() {
    let doc = open_fixture(FOUR_PAGES);
    let view = doc.session.view();
    let folder = scratch("preview");
    let every = request(SplitCriterion::EveryN(2), "{stem}_{n}.pdf", &folder);
    assert_eq!(
        names(&plan(&view, &every, "s", None)),
        ["s_1.pdf", "s_2.pdf"]
    );
    let after = request(
        SplitCriterion::AfterPages(vec![0, 2]),
        "p-{start}-{end}.pdf",
        &folder,
    );
    assert_eq!(
        names(&plan(&view, &after, "s", None)),
        ["p-1-1.pdf", "p-2-3.pdf", "p-4-4.pdf"]
    );
}

/// **A pattern the file system would refuse is refused before the engine.**
#[test]
fn a_pattern_naming_a_folder_or_no_pdf_is_refused() {
    let doc = open_fixture(FOUR_PAGES);
    let view = doc.session.view();
    let folder = scratch("pattern");
    for (template, want) in [
        ("sub/{n}.pdf", t::template_bad_character()),
        ("{stem}_{n}", t::template_not_pdf()),
    ] {
        let r = request(SplitCriterion::EveryN(1), template, &folder);
        assert_eq!(
            plan(&view, &r, "s", None),
            Preview::Refused(want.to_owned()),
            "{template}"
        );
    }
}

/// **A missing folder, a rule that divides nothing and duplicate names each
/// get their own sentence.**
#[test]
fn each_refusal_has_its_sentence() {
    let doc = open_fixture(FOUR_PAGES);
    let view = doc.session.view();
    let folder = scratch("refusals");
    let missing = request(SplitCriterion::EveryN(1), "{n}.pdf", &folder.join("absent"));
    assert_eq!(
        plan(&view, &missing, "s", None),
        Preview::Refused(t::folder_not_found().to_owned())
    );
    let whole = request(SplitCriterion::AfterPages(vec![3]), "{n}.pdf", &folder);
    assert_eq!(
        plan(&view, &whole, "s", None),
        Preview::Refused(t::no_split_points().to_owned())
    );
    let same = request(SplitCriterion::EveryN(1), "same.pdf", &folder);
    assert_eq!(
        plan(&view, &same, "s", None),
        Preview::Refused(t::ambiguous_names(1, 2))
    );
}

/// **No part may be written over the open document, and files already in
/// the folder are counted.**
#[test]
fn the_source_is_never_a_target_and_existing_files_are_counted() {
    let doc = open_fixture(FOUR_PAGES);
    let view = doc.session.view();
    let folder = scratch("source");
    let source = folder.join("s_1.pdf");
    std::fs::write(&source, b"%PDF-1.7").expect("scratch file");
    let r = request(SplitCriterion::EveryN(2), "{stem}_{n}.pdf", &folder);
    assert_eq!(
        plan(&view, &r, "s", Some(&source)),
        Preview::Refused(t::would_overwrite_source("s_1.pdf"))
    );
    match plan(&view, &r, "s", None) {
        Preview::Parts { existing, .. } => assert_eq!(existing, 1),
        Preview::Refused(why) => panic!("{why}"),
    }
}

/// **Every planned file is written as a PDF holding its pages, and the
/// labels choice reaches each.** `labelled-pages.pdf` shows `i ii 1 2`.
#[test]
fn the_split_writes_every_part_with_or_without_labels() {
    use pdfcer_core::document::Document;
    use pdfcer_core::page_labels::page_labels;

    let doc = open_local_fixture("labelled-pages.pdf");
    for (labels, want) in [
        (ExtractedPageLabels::Keep, [["i", "ii"], ["1", "2"]]),
        (ExtractedPageLabels::Drop, [["1", "2"], ["1", "2"]]),
    ] {
        let folder = scratch(labels_token(labels));
        let mut r = request(SplitCriterion::EveryN(2), "{n}.pdf", &folder);
        r.labels = labels;
        let written =
            write(&doc, &r, SeparationPolicy::default()).unwrap_or_else(|(_, why)| panic!("{why}"));
        assert_eq!(written.files, 2);
        for (name, want) in ["1.pdf", "2.pdf"].iter().zip(want) {
            let reopened = Document::load(&folder.join(name)).expect("the part must open");
            assert_eq!(
                page_labels(&reopened).expect("a page tree"),
                want,
                "{name} under {labels:?}"
            );
        }
        assert_eq!(doc.pages.len(), 4, "the source is unchanged");
    }
}
