use crate::mesh::{MeshData, MeshGroup};
use crate::section::SectionProfile;
use crate::station::StationLandmarks;

/// Loft a hull mesh from a sequence of section profiles at their stations.
/// Generates quads (as triangle pairs) between adjacent sections.
/// Returns a single MeshData for the Hull group.
pub fn loft_hull(
    stations: &[StationLandmarks],
    sections: &[SectionProfile],
) -> MeshData {
    assert_eq!(stations.len(), sections.len(), "stations and sections must match");
    assert!(stations.len() >= 2, "need at least 2 stations to loft");

    let mut mesh = MeshData::new(MeshGroup::Hull);

    // Build full section rings (port + starboard) for each station
    let rings: Vec<Vec<[f64; 3]>> = stations.iter().zip(sections.iter())
        .map(|(station, section)| {
            section.full_section().iter()
                .map(|p| [station.x, p.y, p.z])
                .collect()
        })
        .collect();

    // All rings should have the same number of points
    let ring_len = rings[0].len();
    for (i, ring) in rings.iter().enumerate() {
        assert_eq!(ring.len(), ring_len,
            "ring {i} has {} points, expected {ring_len}", ring.len());
    }

    // Add all vertices
    for ring in &rings {
        for &pos in ring {
            mesh.positions.push(pos);
        }
    }

    // Loft quads between adjacent rings
    for s in 0..rings.len() - 1 {
        let base_curr = s * ring_len;
        let base_next = (s + 1) * ring_len;

        for j in 0..ring_len - 1 {
            let a = (base_curr + j) as u32;
            let b = (base_curr + j + 1) as u32;
            let c = (base_next + j + 1) as u32;
            let d = (base_next + j) as u32;

            // Two triangles per quad, consistent winding (CCW when viewed from outside)
            mesh.indices.push(a);
            mesh.indices.push(b);
            mesh.indices.push(c);

            mesh.indices.push(a);
            mesh.indices.push(c);
            mesh.indices.push(d);
        }

        // Close the ring: connect last point to first
        let a = (base_curr + ring_len - 1) as u32;
        let b = base_curr as u32;
        let c = base_next as u32;
        let d = (base_next + ring_len - 1) as u32;

        mesh.indices.push(a);
        mesh.indices.push(b);
        mesh.indices.push(c);

        mesh.indices.push(a);
        mesh.indices.push(c);
        mesh.indices.push(d);
    }

    // Compute normals
    compute_normals(&mut mesh);

    mesh
}

/// Compute per-vertex normals by averaging adjacent face normals.
fn compute_normals(mesh: &mut MeshData) {
    mesh.normals = vec![[0.0; 3]; mesh.positions.len()];

    for tri in mesh.indices.chunks(3) {
        if tri.len() < 3 { continue; }
        let (ia, ib, ic) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let a = mesh.positions[ia];
        let b = mesh.positions[ib];
        let c = mesh.positions[ic];

        let edge1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let edge2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let normal = cross(edge1, edge2);

        // Accumulate (area-weighted by cross product magnitude)
        for &idx in &[ia, ib, ic] {
            mesh.normals[idx][0] += normal[0];
            mesh.normals[idx][1] += normal[1];
            mesh.normals[idx][2] += normal[2];
        }
    }

    // Normalize
    for n in &mut mesh.normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-12 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        }
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::longitudinal::HullCurves;
    use crate::section::{config_from_style, sample_section};
    use crate::station::compute_stations;
    use ship_schema::defaults::classic_runner;
    use ship_schema::spec::SloopVariantArchetype;
    use ship_schema::style::StyleEnforcer;

    fn generate_classic_hull() -> MeshData {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        let stations = compute_stations(&curves, spec.dimensions.overall_length);
        let config = config_from_style(
            enforcer.section_resolution(),
            spec.hull.fullness,
            spec.style.edge_chunkiness,
        );
        let sections: Vec<_> = stations.iter().map(|s| sample_section(s, &config)).collect();
        loft_hull(&stations, &sections)
    }

    #[test]
    fn hull_has_vertices_and_triangles() {
        let mesh = generate_classic_hull();
        assert!(mesh.vertex_count() > 100, "hull should have substantial vertices: {}", mesh.vertex_count());
        assert!(mesh.triangle_count() > 100, "hull should have substantial triangles: {}", mesh.triangle_count());
    }

    #[test]
    fn no_invalid_positions() {
        let mesh = generate_classic_hull();
        assert!(!mesh.has_invalid_positions(), "no NaN/Inf in positions");
    }

    #[test]
    fn no_invalid_normals() {
        let mesh = generate_classic_hull();
        assert!(!mesh.has_invalid_normals(), "no NaN/Inf in normals");
    }

    #[test]
    fn normals_are_unit_length() {
        let mesh = generate_classic_hull();
        for (i, n) in mesh.normals.iter().enumerate() {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 0.01 || len < 1e-10,
                "normal {i} should be unit length, got {len}");
        }
    }

    #[test]
    fn indices_in_bounds() {
        let mesh = generate_classic_hull();
        let vc = mesh.vertex_count() as u32;
        for (i, &idx) in mesh.indices.iter().enumerate() {
            assert!(idx < vc, "index {i} out of bounds: {idx} >= {vc}");
        }
    }

    #[test]
    fn no_degenerate_triangles() {
        let mesh = generate_classic_hull();
        for tri in mesh.indices.chunks(3) {
            let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
            let pa = mesh.positions[a];
            let pb = mesh.positions[b];
            let pc = mesh.positions[c];

            let edge1 = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let edge2 = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let normal = cross(edge1, edge2);
            let area = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt() * 0.5;

            assert!(area > 1e-12, "degenerate triangle found: verts {a},{b},{c}, area={area}");
        }
    }

    #[test]
    fn bounding_box_matches_dimensions() {
        let mesh = generate_classic_hull();
        let (min, max) = mesh.bounding_box().unwrap();
        let spec = classic_runner();

        // X span should be close to overall_length
        let x_span = max[0] - min[0];
        assert!(x_span > spec.dimensions.overall_length * 0.8,
            "x span {x_span} too small for length {}", spec.dimensions.overall_length);
        assert!(x_span < spec.dimensions.overall_length * 1.2,
            "x span {x_span} too large for length {}", spec.dimensions.overall_length);

        // Y span should be close to beam (full width)
        let y_span = max[1] - min[1];
        assert!(y_span > spec.dimensions.beam * 0.5,
            "y span {y_span} too small for beam {}", spec.dimensions.beam);
        assert!(y_span < spec.dimensions.beam * 1.5,
            "y span {y_span} too large for beam {}", spec.dimensions.beam);
    }

    #[test]
    fn all_archetypes_produce_valid_hulls() {
        for arch in SloopVariantArchetype::all() {
            let spec = arch.default_spec();
            let enforcer = StyleEnforcer::new(&spec.style);
            let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
            let stations = compute_stations(&curves, spec.dimensions.overall_length);
            let config = config_from_style(
                enforcer.section_resolution(),
                spec.hull.fullness,
                spec.style.edge_chunkiness,
            );
            let sections: Vec<_> = stations.iter().map(|s| sample_section(s, &config)).collect();
            let mesh = loft_hull(&stations, &sections);

            assert!(!mesh.has_invalid_positions(), "{:?} has invalid positions", arch);
            assert!(!mesh.has_invalid_normals(), "{:?} has invalid normals", arch);
            assert!(mesh.vertex_count() > 50, "{:?} too few vertices", arch);
            assert!(mesh.triangle_count() > 50, "{:?} too few triangles", arch);

            let vc = mesh.vertex_count() as u32;
            for &idx in &mesh.indices {
                assert!(idx < vc, "{:?} index out of bounds", arch);
            }
        }
    }
}
