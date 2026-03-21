use ship_schema::spec::{BulwarkStyle, DeckSpec};

use crate::deck::DeckRimPoint;
use crate::mesh::{MeshData, MeshGroup};

/// Generate rail or bulwark geometry from the deck rim.
///
/// Open rail: thin posts + top rail (simplified as extruded strip).
/// Solid bulwark: thicker wall rising from deck edge.
pub fn generate_rail(
    rim: &[DeckRimPoint],
    deck: &DeckSpec,
) -> MeshData {
    let mut mesh = MeshData::new(MeshGroup::Rail);

    if rim.len() < 2 {
        return mesh;
    }

    let height = deck.rail_height;
    let thickness = deck.rail_thickness;

    if height <= 0.0 {
        return mesh;
    }

    // Generate rail as two extruded strips (port and starboard)
    // Each strip has inner and outer faces

    match deck.bulwark_style {
        BulwarkStyle::OpenRail => {
            // Simple top rail strip on each side
            generate_rail_strip(&mut mesh, rim, height, thickness, true);  // starboard
            generate_rail_strip(&mut mesh, rim, height, thickness, false); // port
        }
        BulwarkStyle::SolidLow | BulwarkStyle::SolidMedium => {
            // Solid wall from deck edge up
            let wall_height = match deck.bulwark_style {
                BulwarkStyle::SolidLow => height * 0.6,
                BulwarkStyle::SolidMedium => height * 0.85,
                _ => height,
            };
            generate_bulwark_wall(&mut mesh, rim, wall_height, thickness, true);
            generate_bulwark_wall(&mut mesh, rim, wall_height, thickness, false);
        }
    }

    mesh
}

/// Generate a thin rail strip along one side of the deck.
fn generate_rail_strip(
    mesh: &mut MeshData,
    rim: &[DeckRimPoint],
    height: f64,
    thickness: f64,
    starboard: bool,
) {
    let base_idx = mesh.positions.len() as u32;

    // Two rows of vertices: bottom (deck edge) and top (rail top)
    for point in rim {
        let y = if starboard { point.y_starboard } else { point.y_port };
        let outward = if starboard { thickness } else { -thickness };

        // Bottom outer
        mesh.positions.push([point.x, y + outward, point.z]);
        mesh.normals.push(if starboard { [0.0, 1.0, 0.0] } else { [0.0, -1.0, 0.0] });

        // Top outer
        mesh.positions.push([point.x, y + outward, point.z + height]);
        mesh.normals.push(if starboard { [0.0, 1.0, 0.0] } else { [0.0, -1.0, 0.0] });
    }

    // Connect strips between stations
    for s in 0..rim.len() as u32 - 1 {
        let a = base_idx + s * 2;
        let b = base_idx + s * 2 + 1;
        let c = base_idx + (s + 1) * 2 + 1;
        let d = base_idx + (s + 1) * 2;

        if starboard {
            mesh.indices.push(a); mesh.indices.push(d); mesh.indices.push(c);
            mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(b);
        } else {
            mesh.indices.push(a); mesh.indices.push(b); mesh.indices.push(c);
            mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(d);
        }
    }
}

/// Generate a solid bulwark wall along one side.
fn generate_bulwark_wall(
    mesh: &mut MeshData,
    rim: &[DeckRimPoint],
    height: f64,
    thickness: f64,
    starboard: bool,
) {
    let base_idx = mesh.positions.len() as u32;
    let outward_sign: f64 = if starboard { 1.0 } else { -1.0 };
    let normal_out = if starboard { [0.0, 1.0, 0.0] } else { [0.0, -1.0, 0.0] };

    // Outer face: 2 verts per station (bottom, top)
    for point in rim {
        let y = if starboard { point.y_starboard } else { point.y_port };
        let y_outer = y + outward_sign * thickness;

        mesh.positions.push([point.x, y_outer, point.z]);
        mesh.normals.push(normal_out);

        mesh.positions.push([point.x, y_outer, point.z + height]);
        mesh.normals.push(normal_out);
    }

    // Connect outer face
    for s in 0..rim.len() as u32 - 1 {
        let a = base_idx + s * 2;
        let b = base_idx + s * 2 + 1;
        let c = base_idx + (s + 1) * 2 + 1;
        let d = base_idx + (s + 1) * 2;

        if starboard {
            mesh.indices.push(a); mesh.indices.push(d); mesh.indices.push(c);
            mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(b);
        } else {
            mesh.indices.push(a); mesh.indices.push(b); mesh.indices.push(c);
            mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(d);
        }
    }

    // Inner face (facing inward toward deck center)
    let inner_base = mesh.positions.len() as u32;
    let normal_in = [0.0, -outward_sign, 0.0];

    for point in rim {
        let y = if starboard { point.y_starboard } else { point.y_port };

        mesh.positions.push([point.x, y, point.z]);
        mesh.normals.push(normal_in);

        mesh.positions.push([point.x, y, point.z + height]);
        mesh.normals.push(normal_in);
    }

    for s in 0..rim.len() as u32 - 1 {
        let a = inner_base + s * 2;
        let b = inner_base + s * 2 + 1;
        let c = inner_base + (s + 1) * 2 + 1;
        let d = inner_base + (s + 1) * 2;

        if starboard {
            // Reverse winding for inner face
            mesh.indices.push(a); mesh.indices.push(b); mesh.indices.push(c);
            mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(d);
        } else {
            mesh.indices.push(a); mesh.indices.push(d); mesh.indices.push(c);
            mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(b);
        }
    }

    // Top cap
    let cap_base = mesh.positions.len() as u32;
    let normal_up = [0.0, 0.0, 1.0];

    for point in rim {
        let y = if starboard { point.y_starboard } else { point.y_port };
        let y_outer = y + outward_sign * thickness;

        mesh.positions.push([point.x, y, point.z + height]);
        mesh.normals.push(normal_up);

        mesh.positions.push([point.x, y_outer, point.z + height]);
        mesh.normals.push(normal_up);
    }

    for s in 0..rim.len() as u32 - 1 {
        let a = cap_base + s * 2;
        let b = cap_base + s * 2 + 1;
        let c = cap_base + (s + 1) * 2 + 1;
        let d = cap_base + (s + 1) * 2;

        if starboard {
            mesh.indices.push(a); mesh.indices.push(d); mesh.indices.push(c);
            mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(b);
        } else {
            mesh.indices.push(a); mesh.indices.push(b); mesh.indices.push(c);
            mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(d);
        }
    }
}

/// Generate quarterdeck raised platform.
/// Sits in the aft portion of the deck, raised by quarterdeckHeight.
pub fn generate_quarterdeck(
    rim: &[DeckRimPoint],
    deck: &DeckSpec,
    quarterdeck_height: f64,
    overall_length: f64,
) -> MeshData {
    let mut mesh = MeshData::new(MeshGroup::Quarterdeck);

    if quarterdeck_height <= 0.0 || deck.quarterdeck_length <= 0.0 {
        return mesh;
    }

    // Quarterdeck occupies stern band: u = 0.0 to u_end
    let u_end = (deck.quarterdeck_length / overall_length).min(0.35);

    // Find rim points that fall within the quarterdeck zone
    let qd_points: Vec<&DeckRimPoint> = rim.iter()
        .filter(|p| {
            let u = (p.x / overall_length) + 0.5;
            u <= u_end + 0.01
        })
        .collect();

    if qd_points.len() < 2 {
        return mesh;
    }

    // Generate a raised deck surface
    let segments = 3;
    for point in &qd_points {
        for j in 0..=segments {
            let t = j as f64 / segments as f64;
            let y = point.y_port + (point.y_starboard - point.y_port) * t;
            mesh.positions.push([point.x, y, point.z + quarterdeck_height]);
            mesh.normals.push([0.0, 0.0, 1.0]);
        }
    }

    let ring_size = (segments + 1) as u32;

    for s in 0..qd_points.len() as u32 - 1 {
        let base_curr = s * ring_size;
        let base_next = (s + 1) * ring_size;

        for j in 0..segments as u32 {
            let a = base_curr + j;
            let b = base_curr + j + 1;
            let c = base_next + j + 1;
            let d = base_next + j;

            mesh.indices.push(a); mesh.indices.push(d); mesh.indices.push(c);
            mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(b);
        }
    }

    // Front face of quarterdeck step
    let step_base = mesh.positions.len() as u32;
    if let Some(front_point) = qd_points.last() {
        for j in 0..=segments {
            let t = j as f64 / segments as f64;
            let y = front_point.y_port + (front_point.y_starboard - front_point.y_port) * t;

            // Bottom edge (deck level)
            mesh.positions.push([front_point.x, y, front_point.z]);
            mesh.normals.push([1.0, 0.0, 0.0]);

            // Top edge (quarterdeck level)
            mesh.positions.push([front_point.x, y, front_point.z + quarterdeck_height]);
            mesh.normals.push([1.0, 0.0, 0.0]);
        }

        for j in 0..segments as u32 {
            let a = step_base + j * 2;
            let b = step_base + j * 2 + 1;
            let c = step_base + (j + 1) * 2 + 1;
            let d = step_base + (j + 1) * 2;

            mesh.indices.push(a); mesh.indices.push(b); mesh.indices.push(c);
            mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(d);
        }
    }

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::extract_deck_rim;
    use crate::longitudinal::HullCurves;
    use crate::station::compute_stations;
    use ship_schema::defaults::classic_runner;
    use ship_schema::spec::SloopVariantArchetype;
    use ship_schema::style::StyleEnforcer;

    fn make_rail_mesh() -> MeshData {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        let stations = compute_stations(&curves, spec.dimensions.overall_length);
        let rim = extract_deck_rim(&stations);
        generate_rail(&rim, &spec.deck)
    }

    fn make_quarterdeck_mesh() -> MeshData {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        let stations = compute_stations(&curves, spec.dimensions.overall_length);
        let rim = extract_deck_rim(&stations);
        generate_quarterdeck(&rim, &spec.deck, spec.hull.quarterdeck_height, spec.dimensions.overall_length)
    }

    #[test]
    fn rail_has_geometry() {
        let mesh = make_rail_mesh();
        assert!(mesh.vertex_count() > 20, "rail should have vertices: {}", mesh.vertex_count());
        assert!(mesh.triangle_count() > 10, "rail should have triangles: {}", mesh.triangle_count());
    }

    #[test]
    fn rail_no_invalid_data() {
        let mesh = make_rail_mesh();
        assert!(!mesh.has_invalid_positions(), "no NaN/Inf in rail positions");
        assert!(!mesh.has_invalid_normals(), "no NaN/Inf in rail normals");
    }

    #[test]
    fn rail_indices_in_bounds() {
        let mesh = make_rail_mesh();
        let vc = mesh.vertex_count() as u32;
        for &idx in &mesh.indices {
            assert!(idx < vc, "rail index {} >= vc {}", idx, vc);
        }
    }

    #[test]
    fn quarterdeck_has_geometry() {
        let mesh = make_quarterdeck_mesh();
        assert!(mesh.vertex_count() > 4, "quarterdeck should have vertices: {}", mesh.vertex_count());
        assert!(mesh.triangle_count() > 2, "quarterdeck should have triangles: {}", mesh.triangle_count());
    }

    #[test]
    fn quarterdeck_raised_above_deck() {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        let stations = compute_stations(&curves, spec.dimensions.overall_length);
        let rim = extract_deck_rim(&stations);
        let qd = generate_quarterdeck(&rim, &spec.deck, spec.hull.quarterdeck_height, spec.dimensions.overall_length);

        if let Some((_, max)) = qd.bounding_box() {
            let deck_z = rim[0].z; // stern deck height
            assert!(max[2] > deck_z, "quarterdeck should be above deck level");
        }
    }

    #[test]
    fn quarterdeck_stays_in_aft_region() {
        let mesh = make_quarterdeck_mesh();
        if let Some((min, max)) = mesh.bounding_box() {
            // Quarterdeck should be in aft half (x < 0 since stern is -L/2)
            assert!(max[0] < 2.0, "quarterdeck should stay aft, max_x={}", max[0]);
        }
    }

    #[test]
    fn bulwark_styles_produce_different_geometry() {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        let stations = compute_stations(&curves, spec.dimensions.overall_length);
        let rim = extract_deck_rim(&stations);

        let mut open_deck = spec.deck.clone();
        open_deck.bulwark_style = BulwarkStyle::OpenRail;
        let open_rail = generate_rail(&rim, &open_deck);

        let mut solid_deck = spec.deck.clone();
        solid_deck.bulwark_style = BulwarkStyle::SolidMedium;
        let solid_rail = generate_rail(&rim, &solid_deck);

        // Solid bulwark should have more geometry (inner + outer + cap)
        assert!(solid_rail.vertex_count() > open_rail.vertex_count(),
            "solid bulwark ({}) should have more verts than open rail ({})",
            solid_rail.vertex_count(), open_rail.vertex_count());
    }

    #[test]
    fn all_archetypes_produce_valid_rail_and_quarterdeck() {
        for arch in SloopVariantArchetype::all() {
            let spec = arch.default_spec();
            let enforcer = StyleEnforcer::new(&spec.style);
            let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
            let stations = compute_stations(&curves, spec.dimensions.overall_length);
            let rim = extract_deck_rim(&stations);

            let rail = generate_rail(&rim, &spec.deck);
            assert!(!rail.has_invalid_positions(), "{:?} rail has invalid positions", arch);
            assert!(rail.vertex_count() > 0, "{:?} rail has no vertices", arch);

            let qd = generate_quarterdeck(&rim, &spec.deck, spec.hull.quarterdeck_height, spec.dimensions.overall_length);
            assert!(!qd.has_invalid_positions(), "{:?} quarterdeck has invalid positions", arch);
        }
    }
}
