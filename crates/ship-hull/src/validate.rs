use crate::mesh::MeshData;

#[derive(Debug)]
pub enum MeshWarning {
    DegenerateTriangle { index: usize, area: f64 },
    NonFinitePosition { vertex: usize },
    NonFiniteNormal { vertex: usize },
    IndexOutOfBounds { tri_index: usize, vertex_index: u32, vertex_count: u32 },
    TooFewVertices(usize),
    TooFewTriangles(usize),
}

/// Validate a mesh and return all warnings found.
pub fn validate_mesh(mesh: &MeshData) -> Vec<MeshWarning> {
    let mut warnings = Vec::new();

    if mesh.vertex_count() < 4 {
        warnings.push(MeshWarning::TooFewVertices(mesh.vertex_count()));
    }
    if mesh.triangle_count() < 2 {
        warnings.push(MeshWarning::TooFewTriangles(mesh.triangle_count()));
    }

    // Check positions
    for (i, p) in mesh.positions.iter().enumerate() {
        if p.iter().any(|v| !v.is_finite()) {
            warnings.push(MeshWarning::NonFinitePosition { vertex: i });
        }
    }

    // Check normals
    for (i, n) in mesh.normals.iter().enumerate() {
        if n.iter().any(|v| !v.is_finite()) {
            warnings.push(MeshWarning::NonFiniteNormal { vertex: i });
        }
    }

    // Check indices
    let vc = mesh.vertex_count() as u32;
    for (i, &idx) in mesh.indices.iter().enumerate() {
        if idx >= vc {
            warnings.push(MeshWarning::IndexOutOfBounds {
                tri_index: i / 3,
                vertex_index: idx,
                vertex_count: vc,
            });
        }
    }

    // Check degenerate triangles
    for (ti, tri) in mesh.indices.chunks(3).enumerate() {
        if tri.len() < 3 { continue; }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a >= mesh.positions.len() || b >= mesh.positions.len() || c >= mesh.positions.len() {
            continue; // already reported as IndexOutOfBounds
        }
        let pa = mesh.positions[a];
        let pb = mesh.positions[b];
        let pc = mesh.positions[c];

        let edge1 = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let edge2 = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let cx = edge1[1] * edge2[2] - edge1[2] * edge2[1];
        let cy = edge1[2] * edge2[0] - edge1[0] * edge2[2];
        let cz = edge1[0] * edge2[1] - edge1[1] * edge2[0];
        let area = (cx * cx + cy * cy + cz * cz).sqrt() * 0.5;

        if area < 1e-12 {
            warnings.push(MeshWarning::DegenerateTriangle { index: ti, area });
        }
    }

    warnings
}
