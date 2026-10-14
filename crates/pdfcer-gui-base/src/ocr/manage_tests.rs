//! Tests for `ocr::manage`: removal, install staging and the write probe.
#![cfg(test)]

use super::*;

/// SHA-256 of `abc`.
const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pdfcer-ocr-manage-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    dir
}

/// An add-on folder `dir` named `name` holding `w.bin` = `contents`, its
/// manifest listing `abc`'s digest.
fn addon(dir: &Path, name: &str, contents: &[u8]) {
    std::fs::create_dir_all(dir).expect("addon dir");
    std::fs::write(dir.join("w.bin"), contents).expect("weights");
    std::fs::write(
        dir.join(MANIFEST_FILE),
        format!("name = {name}\nengine = tesseract\nsha256 = w.bin {ABC}\n"),
    )
    .expect("manifest");
}

fn found(root: &Path, name: &str) -> OcrModel {
    discover_ocr_models(&[root.to_path_buf()])
        .by_name(name)
        .cloned()
        .expect(name)
}

#[test]
fn removing_a_model_deletes_its_folder_and_reports_its_size() {
    let root = scratch("remove");
    addon(&root.join("tess"), "tess-eng", b"abc");
    std::fs::create_dir_all(root.join("paddle")).expect("bare");
    std::fs::write(root.join("paddle/rec.onnx"), [0u8; 10]).expect("rec");
    let before = generation();
    let freed = remove(&found(&root, "tess-eng")).expect("removed");
    let manifest = format!("name = tess-eng\nengine = tesseract\nsha256 = w.bin {ABC}\n");
    assert_eq!(freed, 3 + manifest.len() as u64);
    assert!(!root.join("tess").exists());
    assert!(generation() > before);
    assert_eq!(remove(&found(&root, "paddle")).expect("bare removed"), 10);
    assert!(
        discover_ocr_models(std::slice::from_ref(&root))
            .models
            .is_empty()
    );
    let leftovers: Vec<_> = std::fs::read_dir(&root).expect("root").collect();
    assert!(leftovers.is_empty(), "nothing is left beside the models");
}

#[test]
fn a_folder_that_changed_since_it_was_listed_is_not_deleted() {
    let root = scratch("changed");
    addon(&root.join("tess"), "tess-eng", b"abc");
    let listed = found(&root, "tess-eng");
    std::fs::write(
        root.join("tess").join(MANIFEST_FILE),
        "name = other\nengine = tesseract\n",
    )
    .expect("rewrite");
    assert!(matches!(remove(&listed), Err(RemoveError::Changed(_))));
    assert!(root.join("tess/w.bin").exists());
    let gone = OcrModel {
        folder: root.join("nowhere"),
        ..listed
    };
    assert!(matches!(remove(&gone), Err(RemoveError::Gone(_))));
}

#[test]
fn a_root_that_is_itself_a_model_is_never_deleted() {
    let root = scratch("isroot");
    addon(&root, "tess-eng", b"abc");
    let model = found(&root, "tess-eng");
    assert_eq!(model.folder, root);
    assert!(matches!(remove(&model), Err(RemoveError::IsRoot(_))));
    assert!(root.join("w.bin").exists());
}

#[test]
fn installing_a_folder_verifies_it_then_moves_it_into_models() {
    let dir = scratch("install");
    addon(&dir.join("src").join("tess"), "tess-eng", b"abc");
    let models = dir.join("app").join("models");
    let got = install(&Source::Folder(dir.join("src").join("tess")), &models).expect("installed");
    assert_eq!(got.checked, 1);
    assert_eq!(got.model.folder, models.join("tess"));
    assert_eq!(found(&models, "tess-eng").folder, models.join("tess"));
    let beside: Vec<_> = std::fs::read_dir(dir.join("app"))
        .expect("app")
        .filter_map(Result::ok)
        .map(|e| e.file_name())
        .collect();
    assert_eq!(
        beside,
        vec![std::ffi::OsString::from("models")],
        "staging is gone"
    );
    assert!(matches!(
        install(&Source::Folder(dir.join("src").join("tess")), &models),
        Err(InstallError::Exists(..))
    ));
}

#[test]
fn a_model_whose_file_fails_its_fingerprint_is_not_installed() {
    let dir = scratch("tampered");
    addon(&dir.join("src").join("tess"), "tess-eng", b"abd");
    let models = dir.join("models");
    assert!(matches!(
        install(&Source::Folder(dir.join("src").join("tess")), &models),
        Err(InstallError::Verify(_))
    ));
    assert!(discover_ocr_models(&[models]).models.is_empty());
}

#[test]
fn a_folder_with_no_model_says_so() {
    let dir = scratch("empty");
    std::fs::create_dir_all(dir.join("src").join("junk")).expect("junk");
    std::fs::write(dir.join("src/junk/readme.txt"), "hi").expect("readme");
    let models = dir.join("models");
    assert!(matches!(
        install(&Source::Folder(dir.join("src").join("junk")), &models),
        Err(InstallError::NoModel(_))
    ));
}

#[test]
fn a_writable_folder_is_writable_and_a_missing_one_asks_its_parent() {
    let dir = scratch("probe");
    assert!(writable(&dir));
    assert!(writable(&dir.join("not").join("yet")));
    assert!(
        std::fs::read_dir(&dir).expect("dir").next().is_none(),
        "the probe is removed"
    );
}
