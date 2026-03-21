use ship_schema::spec::{DeckSpec, RigSpec, ShipDimensions};

use crate::deck::PlacementBands;
use crate::mesh::{MeshData, MeshGroup};
use crate::station::StationLandmarks;

/// Rig anchor points — computed geometry contract consumed by sails and rigging.
#[derive(Debug, Clone)]
pub struct RigAnchors {
    pub mast_base: [f64; 3],
    pub mast_top: [f64; 3],
    pub boom_pivot: [f64; 3],
    pub boom_tip: [f64; 3],
    pub gaff_throat: [f64; 3],
    pub gaff_peak: [f64; 3],
    pub bowsprit_base: [f64; 3],
    pub bowsprit_tip: [f64; 3],
    pub forestay_top: [f64; 3],
    pub forestay_bow: [f64; 3],
}

/// Compute rig anchor points from spec and deck geometry.
pub fn compute_rig_anchors(
    rig: &RigSpec,
    deck: &DeckSpec,
    dims: &ShipDimensions,
    stations: &[StationLandmarks],
) -> RigAnchors {
    // Mast position: use deck.mast_offset to place along hull
    let mast_u = 0.50 + deck.mast_offset * 0.25; // centered with slight offset
    let mast_x = PlacementBands::u_to_x(mast_u, dims.overall_length);
    let mast_z_base = PlacementBands::deck_z_at_u(mast_u, stations);

    let mast_base = [mast_x, 0.0, mast_z_base];

    // Mast top: height with rake (lean aft)
    let rake_dx = -dims.mast_height * rig.mast_rake.sin();
    let mast_top = [
        mast_x + rake_dx,
        0.0,
        mast_z_base + dims.mast_height * rig.mast_rake.cos(),
    ];

    // Boom pivot: on mast at boom_height above deck
    let boom_pivot_z = mast_z_base + rig.boom_height;
    let boom_pivot = [mast_x, 0.0, boom_pivot_z];

    // Boom tip: extends aft from pivot
    let boom_tip = [
        mast_x - rig.boom_length,
        0.0,
        boom_pivot_z - 0.05, // slight droop
    ];

    // Gaff throat: on mast above boom pivot
    let gaff_throat_z = boom_pivot_z + 0.3;
    let gaff_throat = [mast_x, 0.0, gaff_throat_z];

    // Gaff peak: extends aft and upward from throat
    let gaff_peak = [
        mast_x - rig.gaff_length * rig.gaff_angle.cos(),
        0.0,
        gaff_throat_z + rig.gaff_length * rig.gaff_angle.sin(),
    ];

    // Bowsprit base: at bow deck edge
    let bow_u = 0.92;
    let bow_x = PlacementBands::u_to_x(bow_u, dims.overall_length);
    let bow_z = PlacementBands::deck_z_at_u(bow_u, stations);
    let bowsprit_base = [bow_x, 0.0, bow_z];

    // Bowsprit tip: extends forward and slightly downward
    let bowsprit_tip = [
        bow_x + dims.bowsprit_length * rig.bowsprit_angle.cos(),
        0.0,
        bow_z - dims.bowsprit_length * rig.bowsprit_angle.sin(),
    ];

    // Forestay anchors
    let forestay_top = [
        mast_top[0] + (mast_top[0] - mast_base[0]) * 0.1, // near mast top
        0.0,
        mast_top[2] * 0.85, // slightly below mast top
    ];
    let forestay_bow = bowsprit_tip;

    RigAnchors {
        mast_base,
        mast_top,
        boom_pivot,
        boom_tip,
        gaff_throat,
        gaff_peak,
        bowsprit_base,
        bowsprit_tip,
        forestay_top,
        forestay_bow,
    }
}

/// Generate mast cylinder mesh.
pub fn generate_mast(
    anchors: &RigAnchors,
    rig: &RigSpec,
) -> MeshData {
    generate_cylinder(
        &anchors.mast_base,
        &anchors.mast_top,
        rig.mast_diameter / 2.0,
        8, // 8-sided for stylized look
        MeshGroup::MastMain,
    )
}

/// Generate bowsprit cylinder mesh.
pub fn generate_bowsprit(
    anchors: &RigAnchors,
    rig: &RigSpec,
) -> MeshData {
    let bowsprit_radius = rig.mast_diameter * 0.5; // thinner than mast
    generate_cylinder(
        &anchors.bowsprit_base,
        &anchors.bowsprit_tip,
        bowsprit_radius,
        6,
        MeshGroup::Bowsprit,
    )
}

/// Generate boom cylinder mesh.
pub fn generate_boom(
    anchors: &RigAnchors,
    rig: &RigSpec,
) -> MeshData {
    let boom_radius = rig.mast_diameter * 0.4;
    generate_cylinder(
        &anchors.boom_pivot,
        &anchors.boom_tip,
        boom_radius,
        6,
        MeshGroup::Boom,
    )
}

/// Generate gaff cylinder mesh.
pub fn generate_gaff(
    anchors: &RigAnchors,
    rig: &RigSpec,
) -> MeshData {
    let gaff_radius = rig.mast_diameter * 0.35;
    generate_cylinder(
        &anchors.gaff_throat,
        &anchors.gaff_peak,
        gaff_radius,
        6,
        MeshGroup::Gaff,
    )
}

/// Generate structural rigging lines (forestay, shrouds).
pub fn generate_rigging(
    anchors: &RigAnchors,
    rig: &RigSpec,
    stations: &[StationLandmarks],
    dims: &ShipDimensions,
    deck_mast_offset: f64,
) -> MeshData {
    let mut mesh = MeshData::new(MeshGroup::Rigging);
    let line_radius = rig.stay_thickness / 2.0;
    let sides = 4; // square-ish for thin lines

    // Forestay: mast top area to bowsprit tip
    let forestay = generate_cylinder(
        &anchors.forestay_top,
        &anchors.forestay_bow,
        line_radius,
        sides,
        MeshGroup::Rigging,
    );
    append_mesh(&mut mesh, &forestay);

    // Shrouds: mast top to deck edges at mast station
    let mast_u = 0.50 + deck_mast_offset * 0.25;
    let deck_width = PlacementBands::deck_width_at_u(mast_u, stations);
    let half_beam = deck_width / 2.0;
    let deck_z = PlacementBands::deck_z_at_u(mast_u, stations);
    let mast_x = PlacementBands::u_to_x(mast_u, dims.overall_length);

    let shroud_top_z = anchors.mast_top[2] * 0.75; // shrouds attach lower than top

    for pair in 0..rig.shroud_pairs {
        let offset_x = -(pair as f64) * 0.3; // spread aft slightly

        // Starboard shroud
        let s_top = [mast_x + offset_x * 0.1, 0.02, shroud_top_z];
        let s_bottom = [mast_x + offset_x, half_beam * 0.95, deck_z];
        let starboard = generate_cylinder(&s_top, &s_bottom, line_radius, sides, MeshGroup::Rigging);
        append_mesh(&mut mesh, &starboard);

        // Port shroud
        let p_top = [mast_x + offset_x * 0.1, -0.02, shroud_top_z];
        let p_bottom = [mast_x + offset_x, -half_beam * 0.95, deck_z];
        let port = generate_cylinder(&p_top, &p_bottom, line_radius, sides, MeshGroup::Rigging);
        append_mesh(&mut mesh, &port);
    }

    mesh
}

/// Generate a cylinder mesh between two 3D points.
fn generate_cylinder(
    start: &[f64; 3],
    end: &[f64; 3],
    radius: f64,
    sides: usize,
    group: MeshGroup,
) -> MeshData {
    let mut mesh = MeshData::new(group);
    let sides = sides.max(4);

    // Direction vector
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let dz = end[2] - start[2];
    let length = (dx * dx + dy * dy + dz * dz).sqrt();

    if length < 1e-10 {
        return mesh;
    }

    // Build local coordinate frame
    let dir = [dx / length, dy / length, dz / length];

    // Find a perpendicular vector
    let up = if dir[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let right = cross(&dir, &up);
    let right_len = (right[0] * right[0] + right[1] * right[1] + right[2] * right[2]).sqrt();
    let right = [right[0] / right_len, right[1] / right_len, right[2] / right_len];
    let up = cross(&right, &dir);

    // Generate vertices: two rings
    for ring in 0..2 {
        let center = if ring == 0 { start } else { end };
        for i in 0..sides {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (sides as f64);
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            let nx = right[0] * cos_a + up[0] * sin_a;
            let ny = right[1] * cos_a + up[1] * sin_a;
            let nz = right[2] * cos_a + up[2] * sin_a;

            mesh.positions.push([
                center[0] + nx * radius,
                center[1] + ny * radius,
                center[2] + nz * radius,
            ]);
            mesh.normals.push([nx, ny, nz]);
        }
    }

    // Connect rings with quads
    for i in 0..sides {
        let next = (i + 1) % sides;
        let a = i as u32;
        let b = next as u32;
        let c = (sides + next) as u32;
        let d = (sides + i) as u32;

        mesh.indices.push(a); mesh.indices.push(b); mesh.indices.push(c);
        mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(d);
    }

    mesh
}

fn cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn append_mesh(target: &mut MeshData, source: &MeshData) {
    let base = target.positions.len() as u32;
    target.positions.extend_from_slice(&source.positions);
    target.normals.extend_from_slice(&source.normals);
    for &idx in &source.indices {
        target.indices.push(base + idx);
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

    fn make_anchors() -> (RigAnchors, ship_schema::spec::SloopAssetSpec, Vec<StationLandmarks>) {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        let stations = compute_stations(&curves, spec.dimensions.overall_length);
        let anchors = compute_rig_anchors(&spec.rig, &spec.deck, &spec.dimensions, &stations);
        (anchors, spec, stations)
    }

    #[test]
    fn mast_base_on_centerline() {
        let (anchors, _, _) = make_anchors();
        assert!((anchors.mast_base[1]).abs() < 1e-10, "mast base should be on centerline (y=0)");
    }

    #[test]
    fn mast_top_above_base() {
        let (anchors, _, _) = make_anchors();
        assert!(anchors.mast_top[2] > anchors.mast_base[2] + 5.0,
            "mast top should be well above base");
    }

    #[test]
    fn bowsprit_points_forward() {
        let (anchors, _, _) = make_anchors();
        assert!(anchors.bowsprit_tip[0] > anchors.bowsprit_base[0],
            "bowsprit tip should be forward (+x) of base");
    }

    #[test]
    fn boom_extends_aft() {
        let (anchors, _, _) = make_anchors();
        assert!(anchors.boom_tip[0] < anchors.boom_pivot[0],
            "boom should extend aft (-x) from pivot");
    }

    #[test]
    fn gaff_extends_aft_and_upward() {
        let (anchors, _, _) = make_anchors();
        assert!(anchors.gaff_peak[0] < anchors.gaff_throat[0], "gaff should extend aft");
        assert!(anchors.gaff_peak[2] > anchors.gaff_throat[2], "gaff should extend upward");
    }

    #[test]
    fn mast_mesh_has_geometry() {
        let (anchors, spec, _) = make_anchors();
        let mesh = generate_mast(&anchors, &spec.rig);
        assert!(mesh.vertex_count() >= 16, "mast should have vertices: {}", mesh.vertex_count());
        assert!(!mesh.has_invalid_positions(), "no NaN/Inf");
    }

    #[test]
    fn bowsprit_mesh_has_geometry() {
        let (anchors, spec, _) = make_anchors();
        let mesh = generate_bowsprit(&anchors, &spec.rig);
        assert!(mesh.vertex_count() >= 12, "bowsprit should have vertices");
        assert!(!mesh.has_invalid_positions(), "no NaN/Inf");
    }

    #[test]
    fn boom_and_gaff_meshes_valid() {
        let (anchors, spec, _) = make_anchors();
        let boom = generate_boom(&anchors, &spec.rig);
        let gaff = generate_gaff(&anchors, &spec.rig);
        assert!(boom.vertex_count() > 0);
        assert!(gaff.vertex_count() > 0);
        assert!(!boom.has_invalid_positions());
        assert!(!gaff.has_invalid_positions());
    }

    #[test]
    fn rigging_has_geometry() {
        let (anchors, spec, stations) = make_anchors();
        let mesh = generate_rigging(&anchors, &spec.rig, &stations, &spec.dimensions, spec.deck.mast_offset);
        assert!(mesh.vertex_count() > 20, "rigging should have vertices: {}", mesh.vertex_count());
        assert!(!mesh.has_invalid_positions());
    }

    #[test]
    fn all_archetypes_produce_valid_rig() {
        for arch in SloopVariantArchetype::all() {
            let spec = arch.default_spec();
            let enforcer = StyleEnforcer::new(&spec.style);
            let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
            let stations = compute_stations(&curves, spec.dimensions.overall_length);
            let anchors = compute_rig_anchors(&spec.rig, &spec.deck, &spec.dimensions, &stations);

            // All anchor points should be finite
            for p in &[anchors.mast_base, anchors.mast_top, anchors.boom_pivot, anchors.boom_tip,
                       anchors.gaff_throat, anchors.gaff_peak, anchors.bowsprit_base, anchors.bowsprit_tip] {
                assert!(p.iter().all(|v| v.is_finite()), "{:?} has non-finite anchor", arch);
            }

            // Mast on centerline
            assert!((anchors.mast_base[1]).abs() < 1e-10, "{:?} mast off centerline", arch);

            // All meshes valid
            let mast = generate_mast(&anchors, &spec.rig);
            let bowsprit = generate_bowsprit(&anchors, &spec.rig);
            let boom = generate_boom(&anchors, &spec.rig);
            let gaff = generate_gaff(&anchors, &spec.rig);
            let rigging = generate_rigging(&anchors, &spec.rig, &stations, &spec.dimensions, spec.deck.mast_offset);

            for (name, mesh) in [("mast", &mast), ("bowsprit", &bowsprit), ("boom", &boom),
                                  ("gaff", &gaff), ("rigging", &rigging)] {
                assert!(!mesh.has_invalid_positions(),
                    "{:?} {} has invalid positions", arch, name);
                assert!(mesh.vertex_count() > 0,
                    "{:?} {} has no vertices", arch, name);
            }
        }
    }
}
