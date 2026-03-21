use crate::mesh::MeshData;
use crate::section::SectionProfile;
use crate::station::StationLandmarks;

/// Generate a bow cap (fan from bow tip to the forward-most section ring).
pub fn bow_cap(
    station: &StationLandmarks,
    section: &SectionProfile,
    _existing_vertex_count: usize,
) -> MeshData {
    fan_cap(station, section, true)
}

/// Generate a stern cap (fan from stern centerline to the aft-most section ring).
pub fn stern_cap(
    station: &StationLandmarks,
    section: &SectionProfile,
    _existing_vertex_count: usize,
) -> MeshData {
    fan_cap(station, section, false)
}

/// Generate a triangle fan cap.
/// For bow: center point is at station x, y=0, z=midpoint. Triangles fan inward.
/// For stern: same logic but winding is reversed.
fn fan_cap(
    station: &StationLandmarks,
    section: &SectionProfile,
    is_bow: bool,
) -> MeshData {
    use crate::mesh::MeshGroup;

    let mut mesh = MeshData::new(MeshGroup::Hull);
    let ring = section.full_section();

    if ring.len() < 3 {
        return mesh;
    }

    // Center point: at the station's x, y=0, z = midpoint of keel-to-deck
    let center_z = (station.z_keel + station.z_deck) / 2.0;
    let center = [station.x, 0.0, center_z];
    mesh.positions.push(center);

    // Add ring points as vertices in this cap mesh
    for p in &ring {
        mesh.positions.push([station.x, p.y, p.z]);
    }

    let center_idx = 0u32;
    let ring_start = 1u32;
    let ring_count = ring.len() as u32;

    // Fan triangles
    for j in 0..ring_count - 1 {
        let a = ring_start + j;
        let b = ring_start + j + 1;

        if is_bow {
            // Bow: normals should point forward (+x)
            mesh.indices.push(center_idx);
            mesh.indices.push(b);
            mesh.indices.push(a);
        } else {
            // Stern: normals should point backward (-x)
            mesh.indices.push(center_idx);
            mesh.indices.push(a);
            mesh.indices.push(b);
        }
    }

    // Close the fan
    let last = ring_start + ring_count - 1;
    let first = ring_start;
    if is_bow {
        mesh.indices.push(center_idx);
        mesh.indices.push(first);
        mesh.indices.push(last);
    } else {
        mesh.indices.push(center_idx);
        mesh.indices.push(last);
        mesh.indices.push(first);
    }

    // Compute simple normals (face direction)
    let normal = if is_bow { [1.0, 0.0, 0.0] } else { [-1.0, 0.0, 0.0] };
    mesh.normals = vec![normal; mesh.positions.len()];

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::longitudinal::HullCurves;
    use crate::section::{config_from_style, sample_section};
    use crate::station::compute_stations;
    use ship_schema::defaults::classic_runner;
    use ship_schema::style::StyleEnforcer;

    fn make_test_data() -> (Vec<StationLandmarks>, Vec<crate::section::SectionProfile>) {
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
        (stations, sections)
    }

    #[test]
    fn bow_cap_has_vertices_and_triangles() {
        let (stations, sections) = make_test_data();
        let cap = bow_cap(stations.last().unwrap(), sections.last().unwrap(), 0);
        assert!(cap.vertex_count() > 3, "bow cap needs vertices");
        assert!(cap.triangle_count() > 2, "bow cap needs triangles");
    }

    #[test]
    fn stern_cap_has_vertices_and_triangles() {
        let (stations, sections) = make_test_data();
        let cap = stern_cap(stations.first().unwrap(), sections.first().unwrap(), 0);
        assert!(cap.vertex_count() > 3, "stern cap needs vertices");
        assert!(cap.triangle_count() > 2, "stern cap needs triangles");
    }

    #[test]
    fn cap_indices_in_bounds() {
        let (stations, sections) = make_test_data();
        let bow = bow_cap(stations.last().unwrap(), sections.last().unwrap(), 0);
        let vc = bow.vertex_count() as u32;
        for &idx in &bow.indices {
            assert!(idx < vc, "bow cap index {idx} out of bounds (vc={vc})");
        }

        let stern = stern_cap(stations.first().unwrap(), sections.first().unwrap(), 0);
        let vc = stern.vertex_count() as u32;
        for &idx in &stern.indices {
            assert!(idx < vc, "stern cap index {idx} out of bounds (vc={vc})");
        }
    }

    #[test]
    fn cap_normals_point_correct_direction() {
        let (stations, sections) = make_test_data();
        let bow = bow_cap(stations.last().unwrap(), sections.last().unwrap(), 0);
        // Bow normals should point in +x direction
        for n in &bow.normals {
            assert!(n[0] > 0.0, "bow normal should point +x");
        }

        let stern = stern_cap(stations.first().unwrap(), sections.first().unwrap(), 0);
        // Stern normals should point in -x direction
        for n in &stern.normals {
            assert!(n[0] < 0.0, "stern normal should point -x");
        }
    }
}
