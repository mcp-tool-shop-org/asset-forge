use ship_schema::spec::{CamberStyle, SailSpec};

use crate::mesh::{MeshData, MeshGroup};
use crate::rig::RigAnchors;

/// Sail anchor points — computed from rig anchors for downstream use.
#[derive(Debug, Clone)]
pub struct SailAnchors {
    pub main_tack: [f64; 3],   // boom pivot (foot-mast junction)
    pub main_clew: [f64; 3],   // boom tip (foot outer corner)
    pub main_head: [f64; 3],   // gaff throat (head-mast junction)
    pub main_peak: [f64; 3],   // gaff peak (head outer corner)
    pub jib_tack: [f64; 3],    // bowsprit tip / lower forestay
    pub jib_clew: [f64; 3],    // deck level near mast
    pub jib_head: [f64; 3],    // forestay top
}

/// Compute sail anchor points from rig anchors.
pub fn compute_sail_anchors(rig: &RigAnchors) -> SailAnchors {
    SailAnchors {
        main_tack: rig.boom_pivot,
        main_clew: rig.boom_tip,
        main_head: rig.gaff_throat,
        main_peak: rig.gaff_peak,
        jib_tack: rig.bowsprit_tip,
        jib_clew: [
            rig.mast_base[0] - 0.3, // slightly aft of mast base
            0.0,
            rig.mast_base[2] + 0.2, // slightly above deck
        ],
        jib_head: rig.forestay_top,
    }
}

/// Generate mainsail mesh from sail anchors.
/// The gaff-sloop mainsail is a quadrilateral bounded by:
/// - luff: mast side (tack → head)
/// - foot: boom (tack → clew)
/// - head: gaff (head → peak)
/// - leech: free trailing edge (clew → peak)
pub fn generate_mainsail(
    sail_anchors: &SailAnchors,
    sail_spec: &SailSpec,
) -> MeshData {
    let mut mesh = MeshData::new(MeshGroup::SailMain);

    if sail_spec.reefed {
        return generate_reefed_mainsail(sail_anchors, sail_spec);
    }

    let segments_u = 8; // across sail (luff → leech)
    let segments_v = 6; // up sail (foot → head)

    let fullness = sail_spec.mainsail_fullness;
    let camber_peak = match sail_spec.camber_style {
        CamberStyle::Flat => 0.0,
        CamberStyle::LightBillow => 0.4,
        CamberStyle::Full => 0.35,
    };

    // Four corner anchors
    let tack = sail_anchors.main_tack;  // bottom-mast (luff-foot)
    let clew = sail_anchors.main_clew;  // bottom-leech (foot-leech)
    let head = sail_anchors.main_head;  // top-mast (luff-head)
    let peak = sail_anchors.main_peak;  // top-leech (head-leech)

    // Generate vertices via bilinear interpolation with camber
    for j in 0..=segments_v {
        let v = j as f64 / segments_v as f64;
        // Interpolate along luff (mast side)
        let luff_point = lerp3(&tack, &head, v);
        // Interpolate along leech (trailing edge)
        let leech_point = lerp3(&clew, &peak, v);

        for i in 0..=segments_u {
            let u = i as f64 / segments_u as f64;
            // Bilinear position
            let mut pos = lerp3(&luff_point, &leech_point, u);

            // Apply camber (belly displacement toward +Y / starboard)
            let camber = fullness * camber_peak
                * (1.0 - (2.0 * u - 1.0).powi(2))  // peak at middle
                * (1.0 - (2.0 * v - 0.5).powi(2).min(1.0)); // stronger in lower-mid
            pos[1] += camber;

            mesh.positions.push(pos);
            // Normal roughly perpendicular to sail plane (pointing starboard)
            mesh.normals.push([0.0, 1.0, 0.0]);
        }
    }

    // Generate triangles
    let cols = (segments_u + 1) as u32;
    for j in 0..segments_v as u32 {
        for i in 0..segments_u as u32 {
            let a = j * cols + i;
            let b = j * cols + i + 1;
            let c = (j + 1) * cols + i + 1;
            let d = (j + 1) * cols + i;

            mesh.indices.push(a); mesh.indices.push(b); mesh.indices.push(c);
            mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(d);
        }
    }

    // Also generate backface (visible from port side)
    let backface_base = mesh.positions.len() as u32;
    let pos_count = mesh.positions.len();
    for i in 0..pos_count {
        mesh.positions.push(mesh.positions[i]);
        mesh.normals.push([0.0, -1.0, 0.0]); // opposite normal
    }
    let idx_count = mesh.indices.len();
    for i in (0..idx_count).step_by(3) {
        // Reverse winding for backface
        mesh.indices.push(backface_base + mesh.indices[i]);
        mesh.indices.push(backface_base + mesh.indices[i + 2]);
        mesh.indices.push(backface_base + mesh.indices[i + 1]);
    }

    mesh
}

/// Generate a reefed (shortened) mainsail.
fn generate_reefed_mainsail(
    sail_anchors: &SailAnchors,
    sail_spec: &SailSpec,
) -> MeshData {
    // Reefed sail uses reduced height (60% of full)
    let mut reduced = sail_anchors.clone();
    reduced.main_head = lerp3(&sail_anchors.main_tack, &sail_anchors.main_head, 0.6);
    reduced.main_peak = lerp3(&sail_anchors.main_clew, &sail_anchors.main_peak, 0.6);

    let mut spec = sail_spec.clone();
    spec.reefed = false; // prevent recursion
    spec.mainsail_fullness *= 0.7; // less belly when reefed
    generate_mainsail(&reduced, &spec)
}

/// Generate jib sail from sail anchors.
/// The jib is a triangle: tack (bowsprit) → clew (deck) → head (forestay top).
pub fn generate_jib(
    sail_anchors: &SailAnchors,
    sail_spec: &SailSpec,
) -> MeshData {
    let mut mesh = MeshData::new(MeshGroup::SailJib);

    if sail_spec.furled_jib {
        return generate_furled_jib(sail_anchors, sail_spec);
    }

    let segments_u = 6; // across jib
    let segments_v = 6; // up jib

    let fullness = sail_spec.jib_fullness;
    let camber_peak = match sail_spec.camber_style {
        CamberStyle::Flat => 0.0,
        CamberStyle::LightBillow => 0.35,
        CamberStyle::Full => 0.3,
    };

    // Triangle: tack at bottom-forward, clew at bottom-aft, head at top
    let tack = sail_anchors.jib_tack;
    let clew = sail_anchors.jib_clew;
    let head = sail_anchors.jib_head;

    for j in 0..=segments_v {
        let v = j as f64 / segments_v as f64;
        // As v increases (going up), the sail narrows toward head
        let bottom_left = lerp3(&tack, &head, v);
        let bottom_right = lerp3(&clew, &head, v);

        let _effective_cols = ((1.0 - v * 0.8) * segments_u as f64).max(1.0) as usize;

        for i in 0..=segments_u {
            let u = i as f64 / segments_u as f64;
            let collapse = v * 0.8; // triangle narrows toward top
            let u_adjusted = u * (1.0 - collapse) + collapse * 0.5;

            let mut pos = lerp3(&bottom_left, &bottom_right, u_adjusted);

            // Camber
            let camber = fullness * camber_peak
                * (1.0 - (2.0 * u - 1.0).powi(2))
                * (1.0 - v * 0.5);
            pos[1] += camber;

            mesh.positions.push(pos);
            mesh.normals.push([0.0, 1.0, 0.0]);
        }
    }

    let cols = (segments_u + 1) as u32;
    for j in 0..segments_v as u32 {
        for i in 0..segments_u as u32 {
            let a = j * cols + i;
            let b = j * cols + i + 1;
            let c = (j + 1) * cols + i + 1;
            let d = (j + 1) * cols + i;

            mesh.indices.push(a); mesh.indices.push(b); mesh.indices.push(c);
            mesh.indices.push(a); mesh.indices.push(c); mesh.indices.push(d);
        }
    }

    // Backface
    let backface_base = mesh.positions.len() as u32;
    let pos_count = mesh.positions.len();
    for i in 0..pos_count {
        mesh.positions.push(mesh.positions[i]);
        mesh.normals.push([0.0, -1.0, 0.0]);
    }
    let idx_count = mesh.indices.len();
    for i in (0..idx_count).step_by(3) {
        mesh.indices.push(backface_base + mesh.indices[i]);
        mesh.indices.push(backface_base + mesh.indices[i + 2]);
        mesh.indices.push(backface_base + mesh.indices[i + 1]);
    }

    mesh
}

/// Generate a furled (rolled up) jib — compact cylinder at bowsprit.
fn generate_furled_jib(
    sail_anchors: &SailAnchors,
    _sail_spec: &SailSpec,
) -> MeshData {
    let mut mesh = MeshData::new(MeshGroup::SailJib);

    // Simple rolled cylinder along the forestay line
    let base = sail_anchors.jib_tack;
    let top = sail_anchors.jib_head;
    let radius = 0.08; // compact rolled jib
    let sides = 6;

    let dx = top[0] - base[0];
    let dy = top[1] - base[1];
    let dz = top[2] - base[2];
    let length = (dx * dx + dy * dy + dz * dz).sqrt();

    if length < 1e-10 { return mesh; }

    let dir = [dx / length, dy / length, dz / length];
    let up = if dir[2].abs() < 0.9 { [0.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0] };
    let right = cross(&dir, &up);
    let right_len = (right[0] * right[0] + right[1] * right[1] + right[2] * right[2]).sqrt();
    let right = [right[0] / right_len, right[1] / right_len, right[2] / right_len];
    let up = cross(&right, &dir);

    for ring in 0..2 {
        let center = if ring == 0 { &base } else { &top };
        for i in 0..sides {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (sides as f64);
            let nx = right[0] * angle.cos() + up[0] * angle.sin();
            let ny = right[1] * angle.cos() + up[1] * angle.sin();
            let nz = right[2] * angle.cos() + up[2] * angle.sin();
            mesh.positions.push([center[0] + nx * radius, center[1] + ny * radius, center[2] + nz * radius]);
            mesh.normals.push([nx, ny, nz]);
        }
    }

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

fn lerp3(a: &[f64; 3], b: &[f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
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
    use crate::rig::compute_rig_anchors;
    use crate::station::compute_stations;
    use ship_schema::defaults::classic_runner;
    use ship_schema::spec::SloopVariantArchetype;
    use ship_schema::style::StyleEnforcer;

    fn make_sail_anchors() -> (SailAnchors, ship_schema::spec::SloopAssetSpec) {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        let stations = compute_stations(&curves, spec.dimensions.overall_length);
        let rig_anchors = compute_rig_anchors(&spec.rig, &spec.deck, &spec.dimensions, &stations);
        let sail_anchors = compute_sail_anchors(&rig_anchors);
        (sail_anchors, spec)
    }

    #[test]
    fn mainsail_has_geometry() {
        let (anchors, spec) = make_sail_anchors();
        let mesh = generate_mainsail(&anchors, &spec.sails);
        assert!(mesh.vertex_count() > 50, "mainsail should have vertices: {}", mesh.vertex_count());
        assert!(mesh.triangle_count() > 50, "mainsail should have triangles: {}", mesh.triangle_count());
    }

    #[test]
    fn mainsail_no_invalid_data() {
        let (anchors, spec) = make_sail_anchors();
        let mesh = generate_mainsail(&anchors, &spec.sails);
        assert!(!mesh.has_invalid_positions());
        assert!(!mesh.has_invalid_normals());
    }

    #[test]
    fn jib_has_geometry() {
        let (anchors, spec) = make_sail_anchors();
        let mesh = generate_jib(&anchors, &spec.sails);
        assert!(mesh.vertex_count() > 30, "jib should have vertices: {}", mesh.vertex_count());
    }

    #[test]
    fn reefed_mainsail_smaller_than_full() {
        let (anchors, spec) = make_sail_anchors();
        let full = generate_mainsail(&anchors, &spec.sails);

        let mut reefed_spec = spec.sails.clone();
        reefed_spec.reefed = true;
        let reefed = generate_mainsail(&anchors, &reefed_spec);

        let full_bb = full.bounding_box().unwrap();
        let reefed_bb = reefed.bounding_box().unwrap();
        let full_z_span = full_bb.1[2] - full_bb.0[2];
        let reefed_z_span = reefed_bb.1[2] - reefed_bb.0[2];

        assert!(reefed_z_span < full_z_span,
            "reefed z-span ({}) should be less than full ({})", reefed_z_span, full_z_span);
    }

    #[test]
    fn furled_jib_is_compact() {
        let (anchors, spec) = make_sail_anchors();
        let full_jib = generate_jib(&anchors, &spec.sails);

        let mut furled_spec = spec.sails.clone();
        furled_spec.furled_jib = true;
        let furled = generate_jib(&anchors, &furled_spec);

        assert!(furled.vertex_count() < full_jib.vertex_count(),
            "furled jib ({}) should have fewer verts than full ({})",
            furled.vertex_count(), full_jib.vertex_count());
    }

    #[test]
    fn mainsail_has_backface() {
        let (anchors, spec) = make_sail_anchors();
        let mesh = generate_mainsail(&anchors, &spec.sails);

        // Should have doubled vertices (front + back)
        // Segments: (8+1)*(6+1) = 63 per side, 126 total
        assert!(mesh.vertex_count() > 100, "should have front+back: {}", mesh.vertex_count());
    }

    #[test]
    fn sail_anchors_derived_from_rig() {
        let (sail_anchors, _) = make_sail_anchors();

        // Main tack should be at boom pivot
        assert!(sail_anchors.main_tack[2] > 0.0, "main tack should be above waterline");
        // Jib tack should be forward
        assert!(sail_anchors.jib_tack[0] > 3.0, "jib tack should be forward");
        // Jib head should be high
        assert!(sail_anchors.jib_head[2] > 4.0, "jib head should be high");
    }

    #[test]
    fn all_archetypes_produce_valid_sails() {
        for arch in SloopVariantArchetype::all() {
            let spec = arch.default_spec();
            let enforcer = StyleEnforcer::new(&spec.style);
            let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
            let stations = compute_stations(&curves, spec.dimensions.overall_length);
            let rig_anchors = compute_rig_anchors(&spec.rig, &spec.deck, &spec.dimensions, &stations);
            let sail_anchors = compute_sail_anchors(&rig_anchors);

            let main = generate_mainsail(&sail_anchors, &spec.sails);
            assert!(!main.has_invalid_positions(), "{:?} mainsail invalid", arch);
            assert!(main.vertex_count() > 30, "{:?} mainsail too few verts", arch);

            let jib = generate_jib(&sail_anchors, &spec.sails);
            assert!(!jib.has_invalid_positions(), "{:?} jib invalid", arch);
            assert!(jib.vertex_count() > 10, "{:?} jib too few verts", arch);
        }
    }
}
