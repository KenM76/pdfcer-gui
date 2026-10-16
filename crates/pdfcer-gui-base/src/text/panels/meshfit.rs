//! The 3D viewer's Mesh fit choice and what a model drawn under it inferred
//! about its compressed meshes.

/// The 3D viewer's mesh-fit choice's label.
#[must_use]
pub fn mesh_fit() -> &'static str {
    "Mesh fit"
}

/// Its tip.
#[must_use]
pub fn mesh_fit_tooltip() -> &'static str {
    "How far pdfcer goes to draw a compressed mesh the file does not pin down. Exact only draws the shapes the file determines; Best fit also draws a shape a search rebuilt; In part also draws, of a mesh nothing rebuilds, the triangles read before its numbers stopped fitting, and Continued carries on past that point. Everything past Exact only is reported below the picture."
}

/// The mesh-fit choices, in the viewer's order.
#[must_use]
pub fn mesh_fit_names() -> [&'static str; 4] {
    ["Best fit", "Exact only", "In part", "Continued"]
}

/// Why the viewer opened a model drawn in part.
#[must_use]
pub fn mesh_fit_fell_back() -> &'static str {
    "Best fit leaves this model with nothing to draw, so it opened In part; Mesh fit chooses again."
}

/// Why a mesh-fit choice was not applied: it leaves nothing to draw.
#[must_use]
pub fn mesh_fit_refused(choice: &str, why: &str) -> String {
    format!("{choice} was not applied, and the model is still drawn as before: {why}")
}

/// Compressed meshes drawn in part, as the engine words them.
#[must_use]
pub fn mesh_partial(n: usize, drawn: usize, total: usize) -> String {
    match n {
        1 => format!(
            "One compressed mesh nothing rebuilds is drawn in part ({drawn} of {total} triangles); the file does not confirm the shape past where its numbers stopped fitting, so a detail can be wrong or missing."
        ),
        n => format!(
            "{n} compressed meshes nothing rebuilds are drawn in part ({drawn} of {total} triangles); the file does not confirm the shape past where their numbers stopped fitting, so a detail can be wrong or missing."
        ),
    }
}

/// One reason partial meshes stopped fitting, with how many.
#[must_use]
pub fn mesh_partial_why(why: &str, n: usize) -> String {
    format!("{n} of those stopped fitting because {why}.")
}

/// One reason compressed meshes were left out, with how many.
#[must_use]
pub fn mesh_left_out_why(why: &str, n: usize) -> String {
    format!("{n} compressed mesh(es) left out: {why}.")
}

/// Parts missing a left-out mesh.
#[must_use]
pub fn mesh_left_out_parts(parts: &[String]) -> String {
    format!(
        "Parts missing a left-out mesh ({}): {}",
        parts.len(),
        parts.join(", ")
    )
}

/// Parts drawn by a best-fit mesh.
#[must_use]
pub fn mesh_best_fit_parts(parts: &[String]) -> String {
    format!(
        "Parts drawn by a best fit ({}): {}",
        parts.len(),
        parts.join(", ")
    )
}

/// Parts drawn by a partial mesh.
#[must_use]
pub fn mesh_partial_parts(parts: &[String]) -> String {
    format!(
        "Parts drawn in part ({}): {}",
        parts.len(),
        parts.join(", ")
    )
}
