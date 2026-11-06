//! # `app::actions::gltf` — **a glTF model becomes PRC before it is placed**
//!
//! Insert 3D model accepts glTF (`.gltf`, `.glb`): `pdfcer_3d::read_gltf`
//! reads it with the window's colour and up-axis switches, `write_prc`
//! writes the PRC that `ThreeDSpec` embeds, textures included, and every
//! note and inference both calls make reaches the status line with the
//! receipt.
//!
//! External buffers are read only from beside the model file: a URI that is
//! absolute, names a drive or scheme, or climbs out with `..` resolves to
//! nothing, so a model cannot make the program read an arbitrary file. The
//! rule is the CLI's (`threed_cmd::gltf::beside` in `pdfcer-cli`); a path it
//! allows is then read only when its real path, links resolved, is still
//! inside the model's real folder ([`read_inside`]).

use std::path::{Component, Path, PathBuf};

use pdfcer_3d::{GltfColours, GltfOptions, GltfUpAxis, read_gltf, write_prc};

use crate::text::panels::models as t;

/// Whether `data` read from `path` is glTF: a GLB signature, or a `.gltf` /
/// `.glb` extension in any case.
fn is_gltf(path: &Path, data: &[u8]) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    data.starts_with(b"glTF") || matches!(ext.as_deref(), Some("gltf" | "glb"))
}

/// `uri` inside `dir`, or `None` when it is absolute, names a drive or
/// scheme, or has a `..` component.
fn beside(dir: &Path, uri: &str) -> Option<PathBuf> {
    let rel = Path::new(uri);
    let plain = !uri.contains(':')
        && rel
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    plain.then(|| dir.join(rel))
}

/// The bytes of `uri` beside `dir`, read only when the file's real path,
/// every link resolved, lies inside `dir`'s real path; `None` otherwise.
fn read_inside(dir: &Path, uri: &str) -> Option<Vec<u8>> {
    let path = beside(dir, uri)?;
    let folder = std::fs::canonicalize(dir).ok()?;
    let real = std::fs::canonicalize(path).ok()?;
    if !real.starts_with(&folder) {
        return None;
    }
    std::fs::read(real).ok()
}

/// The bytes to embed for the model at `path`, the notes its conversion
/// owes the status line, and whether it was converted. Anything not glTF
/// passes through unchanged with no notes; a glTF the engine cannot read or
/// write is `Err` with the reason.
///
/// # Errors
///
/// The engine's `GltfError` or `PrcWriteError`, worded for the status line.
pub(super) fn to_embed(
    path: &Path,
    data: Vec<u8>,
    options: GltfOptions,
) -> Result<(Vec<u8>, Vec<String>, bool), String> {
    if !is_gltf(path, &data) {
        return Ok((data, Vec::new(), false));
    }
    let dir = path.parent().unwrap_or(Path::new("."));
    let mut resolve = |uri: &str| read_inside(dir, uri);
    let import =
        read_gltf(&data, &mut resolve, options).map_err(|e| t::gltf_refused(&e.to_string()))?;
    let report = write_prc(&import.scene).map_err(|e| t::gltf_refused(&e.to_string()))?;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "model-insert-gltf triangles={} notes={} inferred={} flat_normals={} empty={} textured={} colours={:?} up={:?}",
            report.triangles,
            import.notes.len(),
            import.inferred.len(),
            report.flat_normals,
            report.empty_primitives,
            report.textured,
            options.colours,
            options.up_axis
        )
    });
    let mut notes = vec![t::gltf_converted(
        report.triangles,
        options.colours == GltfColours::Linear,
        options.up_axis == GltfUpAxis::ZUp,
    )];
    if report.textured > 0 {
        notes.push(crate::text::model3d::gltf_textured(report.textured));
    }
    notes.extend(import.inferred.iter().map(|i| t::gltf_inferred(i)));
    notes.extend(import.notes.iter().cloned());
    if report.flat_normals > 0 {
        notes.push(t::gltf_flat_normals(report.flat_normals));
    }
    if report.empty_primitives > 0 {
        notes.push(t::gltf_empty_primitives(report.empty_primitives));
    }
    Ok((report.prc, notes, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A model's external buffers are read beside it and nowhere else.
    #[test]
    fn external_buffers_stay_beside_the_model() {
        let dir = Path::new("models");
        assert_eq!(beside(dir, "a/b.bin"), Some(dir.join("a/b.bin")));
        assert_eq!(beside(dir, "../secret.bin"), None);
        assert_eq!(beside(dir, "C:/x.bin"), None);
        assert_eq!(beside(dir, "/etc/passwd"), None);
    }

    /// A link beside the model that leads out of its folder is not followed;
    /// a plain file beside it is read. Skipped where links cannot be made.
    #[test]
    fn a_link_out_of_the_models_folder_is_refused() {
        let base = std::env::temp_dir().join(format!("pdfcer-gltf-link-{}", std::process::id()));
        let (models, outside) = (base.join("models"), base.join("outside"));
        std::fs::create_dir_all(&models).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("secret.bin"), b"secret").unwrap();
        std::fs::write(models.join("mesh.bin"), b"mesh").unwrap();
        #[cfg(windows)]
        let linked =
            std::os::windows::fs::symlink_file(outside.join("secret.bin"), models.join("link.bin"));
        #[cfg(unix)]
        let linked =
            std::os::unix::fs::symlink(outside.join("secret.bin"), models.join("link.bin"));
        assert_eq!(read_inside(&models, "mesh.bin"), Some(b"mesh".to_vec()));
        if linked.is_ok() {
            assert_eq!(read_inside(&models, "link.bin"), None);
        } else {
            eprintln!("skipped: no symbolic link can be made here");
        }
        let _ = std::fs::remove_dir_all(&base);
    }

    /// A PRC file passes through unchanged and adds no notes.
    #[test]
    fn a_prc_passes_through() {
        let (data, notes, converted) = to_embed(
            Path::new("m.prc"),
            b"PRC....".to_vec(),
            GltfOptions::default(),
        )
        .unwrap();
        assert!(!converted);
        assert_eq!(data, b"PRC....");
        assert!(notes.is_empty());
    }

    /// A GLB signature is glTF whatever the file is called.
    #[test]
    fn a_glb_signature_is_gltf() {
        assert!(is_gltf(Path::new("model.bin"), b"glTF\x02\0\0\0"));
        assert!(is_gltf(Path::new("MODEL.GLTF"), b"{"));
        assert!(!is_gltf(Path::new("model.u3d"), b"U3D\0"));
    }
}
