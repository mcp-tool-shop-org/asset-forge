use ship_hull::mesh::MeshData;

/// Check all mesh invariants. Returns a list of violations (empty = clean).
pub fn check_mesh_invariants(mesh: &MeshData) -> Vec<String> {
    let mut violations = Vec::new();

    // No NaN/Inf positions
    if mesh.has_invalid_positions() {
        violations.push("mesh contains NaN/Inf positions".into());
    }

    // No NaN/Inf normals
    if mesh.has_invalid_normals() {
        violations.push("mesh contains NaN/Inf normals".into());
    }

    // Position/normal count match
    if mesh.positions.len() != mesh.normals.len() {
        violations.push(format!(
            "position count ({}) != normal count ({})",
            mesh.positions.len(), mesh.normals.len()
        ));
    }

    // Indices in bounds
    let vc = mesh.vertex_count() as u32;
    for (i, &idx) in mesh.indices.iter().enumerate() {
        if idx >= vc {
            violations.push(format!("index {i} ({idx}) >= vertex count ({vc})"));
            break; // Only report first
        }
    }

    // Triangle count is whole
    if mesh.indices.len() % 3 != 0 {
        violations.push(format!(
            "index count ({}) not divisible by 3", mesh.indices.len()
        ));
    }

    // No degenerate triangles
    let mut degen_count = 0;
    for tri in mesh.indices.chunks(3) {
        if tri.len() < 3 { continue; }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a >= mesh.positions.len() || b >= mesh.positions.len() || c >= mesh.positions.len() {
            continue;
        }
        let pa = mesh.positions[a];
        let pb = mesh.positions[b];
        let pc = mesh.positions[c];

        let e1 = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let e2 = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        let area = (cx * cx + cy * cy + cz * cz).sqrt() * 0.5;

        if area < 1e-12 {
            degen_count += 1;
        }
    }
    if degen_count > 0 {
        violations.push(format!("{degen_count} degenerate triangles"));
    }

    // Normals are roughly unit length
    let mut bad_normals = 0;
    for n in &mesh.normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-6 && (len - 1.0).abs() > 0.05 {
            bad_normals += 1;
        }
    }
    if bad_normals > 0 {
        violations.push(format!("{bad_normals} normals not unit length"));
    }

    violations
}
