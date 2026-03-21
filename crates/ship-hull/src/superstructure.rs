use ship_schema::spec::DeckSpec;

use crate::deck::PlacementBands;
use crate::mesh::{MeshData, MeshGroup};
use crate::station::StationLandmarks;

/// Generate cabin block on the deck.
/// Positioned in the aft-mid band, behind the mast.
pub fn generate_cabin(
    deck: &DeckSpec,
    stations: &[StationLandmarks],
    overall_length: f64,
) -> MeshData {
    let mut mesh = MeshData::new(MeshGroup::Cabin);

    if !deck.cabin || deck.cabin_length <= 0.0 || deck.cabin_height <= 0.0 {
        return mesh;
    }

    // Cabin center u position: aft-mid band, offset by cabin_offset
    let cabin_center_u = 0.28 + deck.cabin_offset; // aft-mid area
    let cabin_half_len = deck.cabin_length / (2.0 * overall_length);
    let u_start = (cabin_center_u - cabin_half_len).max(0.05);
    let u_end = (cabin_center_u + cabin_half_len).min(0.50);

    let x_start = PlacementBands::u_to_x(u_start, overall_length);
    let x_end = PlacementBands::u_to_x(u_end, overall_length);
    let z_base_start = PlacementBands::deck_z_at_u(u_start, stations);
    let z_base_end = PlacementBands::deck_z_at_u(u_end, stations);
    let z_base = (z_base_start + z_base_end) / 2.0;
    let z_top = z_base + deck.cabin_height;

    let deck_width_start = PlacementBands::deck_width_at_u(u_start, stations);
    let deck_width_end = PlacementBands::deck_width_at_u(u_end, stations);
    let deck_width = deck_width_start.min(deck_width_end);
    let half_width = (deck_width * deck.cabin_width_ratio / 2.0).max(0.2);

    // Generate a box: 6 faces, 24 vertices (unique normals per face)
    generate_box(
        &mut mesh,
        x_start, x_end,
        -half_width, half_width,
        z_base, z_top,
    );

    mesh
}

/// Generate hatch blocks on the deck.
/// Positioned in the mid/fore band.
pub fn generate_hatches(
    deck: &DeckSpec,
    stations: &[StationLandmarks],
    overall_length: f64,
) -> MeshData {
    let mut mesh = MeshData::new(MeshGroup::Hatches);

    if deck.hatch_count == 0 {
        return mesh;
    }

    let hatch_height = 0.08; // low profile
    let hatch_length = 0.4;  // compact
    let hatch_width_ratio = 0.25;

    // Place hatches in the mid-fore band
    let band_start_u = 0.45;
    let spacing_u = deck.hatch_spacing / overall_length;

    for i in 0..deck.hatch_count {
        let u = band_start_u + (i as f64) * spacing_u;
        if u > 0.80 { break; } // don't place in bow band

        let x_center = PlacementBands::u_to_x(u, overall_length);
        let z_base = PlacementBands::deck_z_at_u(u, stations);
        let deck_width = PlacementBands::deck_width_at_u(u, stations);
        let half_width = (deck_width * hatch_width_ratio / 2.0).max(0.15);

        generate_box(
            &mut mesh,
            x_center - hatch_length / 2.0, x_center + hatch_length / 2.0,
            -half_width, half_width,
            z_base, z_base + hatch_height,
        );
    }

    mesh
}

/// Generate helm (tiller or wheel) at the stern.
pub fn generate_helm(
    deck: &DeckSpec,
    stations: &[StationLandmarks],
    overall_length: f64,
    quarterdeck_height: f64,
) -> MeshData {
    let mut mesh = MeshData::new(MeshGroup::Helm);

    // Helm position: stern band
    let helm_u = 0.08;
    let x = PlacementBands::u_to_x(helm_u, overall_length);
    let z_base = PlacementBands::deck_z_at_u(helm_u, stations) + quarterdeck_height;

    if deck.tiller {
        // Tiller: a horizontal bar
        let tiller_length = 0.8;
        let tiller_thickness = 0.04;

        generate_box(
            &mut mesh,
            x - tiller_length, x,
            -tiller_thickness, tiller_thickness,
            z_base + 0.3, z_base + 0.3 + tiller_thickness,
        );
    } else if deck.wheel {
        // Wheel: simplified as a post + disc approximation (octagonal prism)
        let post_height = 0.7;
        let post_thickness = 0.05;

        // Post
        generate_box(
            &mut mesh,
            x - post_thickness, x + post_thickness,
            -post_thickness, post_thickness,
            z_base, z_base + post_height,
        );

        // Wheel disc (simplified as wider flat box)
        let wheel_radius = 0.25;
        let wheel_depth = 0.03;
        let wheel_center_z = z_base + post_height * 0.75;
        generate_box(
            &mut mesh,
            x - wheel_depth, x + wheel_depth,
            -wheel_radius, wheel_radius,
            wheel_center_z - wheel_radius, wheel_center_z + wheel_radius,
        );
    }

    mesh
}

/// Generate an axis-aligned box with proper normals.
fn generate_box(
    mesh: &mut MeshData,
    x0: f64, x1: f64,
    y0: f64, y1: f64,
    z0: f64, z1: f64,
) {
    let base = mesh.positions.len() as u32;

    // 6 faces, 4 vertices each = 24 vertices
    // Front (+X)
    let normals_and_verts: &[([f64; 3], [[f64; 3]; 4])] = &[
        ([1.0, 0.0, 0.0], [[x1, y0, z0], [x1, y1, z0], [x1, y1, z1], [x1, y0, z1]]),
        ([-1.0, 0.0, 0.0], [[x0, y1, z0], [x0, y0, z0], [x0, y0, z1], [x0, y1, z1]]),
        ([0.0, 1.0, 0.0], [[x0, y1, z0], [x0, y1, z1], [x1, y1, z1], [x1, y1, z0]]),
        ([0.0, -1.0, 0.0], [[x1, y0, z0], [x1, y0, z1], [x0, y0, z1], [x0, y0, z0]]),
        ([0.0, 0.0, 1.0], [[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]]),
        ([0.0, 0.0, -1.0], [[x0, y1, z0], [x1, y1, z0], [x1, y0, z0], [x0, y0, z0]]),
    ];

    for (normal, verts) in normals_and_verts {
        let idx = mesh.positions.len() as u32;
        for v in verts {
            mesh.positions.push(*v);
            mesh.normals.push(*normal);
        }
        mesh.indices.push(idx);
        mesh.indices.push(idx + 1);
        mesh.indices.push(idx + 2);
        mesh.indices.push(idx);
        mesh.indices.push(idx + 2);
        mesh.indices.push(idx + 3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::longitudinal::HullCurves;
    use crate::station::compute_stations;
    use ship_schema::defaults::classic_runner;
    use ship_schema::spec::SloopVariantArchetype;
    use ship_schema::style::StyleEnforcer;

    fn make_stations() -> (Vec<StationLandmarks>, ship_schema::spec::SloopAssetSpec) {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        let stations = compute_stations(&curves, spec.dimensions.overall_length);
        (stations, spec)
    }

    #[test]
    fn cabin_has_geometry_when_enabled() {
        let (stations, spec) = make_stations();
        let mesh = generate_cabin(&spec.deck, &stations, spec.dimensions.overall_length);
        assert!(mesh.vertex_count() == 24, "box should have 24 verts: {}", mesh.vertex_count());
        assert!(mesh.triangle_count() == 12, "box should have 12 tris: {}", mesh.triangle_count());
    }

    #[test]
    fn cabin_empty_when_disabled() {
        let (stations, mut spec) = make_stations();
        spec.deck.cabin = false;
        let mesh = generate_cabin(&spec.deck, &stations, spec.dimensions.overall_length);
        assert_eq!(mesh.vertex_count(), 0);
    }

    #[test]
    fn cabin_fits_within_deck_width() {
        let (stations, spec) = make_stations();
        let mesh = generate_cabin(&spec.deck, &stations, spec.dimensions.overall_length);
        if let Some((min, max)) = mesh.bounding_box() {
            let cabin_width = max[1] - min[1];
            let mid_deck_width = PlacementBands::deck_width_at_u(0.28, &stations);
            assert!(cabin_width < mid_deck_width,
                "cabin width {} should be less than deck width {}", cabin_width, mid_deck_width);
        }
    }

    #[test]
    fn hatches_have_geometry() {
        let (stations, spec) = make_stations();
        let mesh = generate_hatches(&spec.deck, &stations, spec.dimensions.overall_length);
        assert!(mesh.vertex_count() > 0, "should have hatch geometry");
    }

    #[test]
    fn hatches_zero_when_count_zero() {
        let (stations, mut spec) = make_stations();
        spec.deck.hatch_count = 0;
        let mesh = generate_hatches(&spec.deck, &stations, spec.dimensions.overall_length);
        assert_eq!(mesh.vertex_count(), 0);
    }

    #[test]
    fn helm_tiller_has_geometry() {
        let (stations, spec) = make_stations();
        let mesh = generate_helm(&spec.deck, &stations, spec.dimensions.overall_length, spec.hull.quarterdeck_height);
        assert!(mesh.vertex_count() > 0, "tiller should have geometry");
    }

    #[test]
    fn helm_wheel_has_geometry() {
        let (stations, mut spec) = make_stations();
        spec.deck.tiller = false;
        spec.deck.wheel = true;
        let mesh = generate_helm(&spec.deck, &stations, spec.dimensions.overall_length, spec.hull.quarterdeck_height);
        assert!(mesh.vertex_count() > 24, "wheel should have more geometry than a single box");
    }

    #[test]
    fn all_superstructure_valid_per_archetype() {
        for arch in SloopVariantArchetype::all() {
            let spec = arch.default_spec();
            let enforcer = StyleEnforcer::new(&spec.style);
            let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
            let stations = compute_stations(&curves, spec.dimensions.overall_length);

            let cabin = generate_cabin(&spec.deck, &stations, spec.dimensions.overall_length);
            assert!(!cabin.has_invalid_positions(), "{:?} cabin invalid", arch);

            let hatches = generate_hatches(&spec.deck, &stations, spec.dimensions.overall_length);
            assert!(!hatches.has_invalid_positions(), "{:?} hatches invalid", arch);

            let helm = generate_helm(&spec.deck, &stations, spec.dimensions.overall_length, spec.hull.quarterdeck_height);
            assert!(!helm.has_invalid_positions(), "{:?} helm invalid", arch);
        }
    }

    #[test]
    fn cabin_does_not_overlap_mast_zone() {
        let (stations, spec) = make_stations();
        let cabin = generate_cabin(&spec.deck, &stations, spec.dimensions.overall_length);
        if let Some((_, max)) = cabin.bounding_box() {
            let mast_x = PlacementBands::u_to_x(0.5 + spec.deck.mast_offset * 0.5, spec.dimensions.overall_length);
            // Cabin max x should be behind mast position
            assert!(max[0] < mast_x + 0.5,
                "cabin max_x ({}) should be behind mast_x ({})", max[0], mast_x);
        }
    }
}
