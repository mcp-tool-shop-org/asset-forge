use ship_schema::spec::{DeckSpec, ShipDimensions};
use ship_schema::style::StyleEnforcer;

use crate::mesh::{MeshData, MeshGroup};
use crate::station::StationLandmarks;

/// Deck rim point at a station: the top edge of the hull on each side.
#[derive(Debug, Clone, Copy)]
pub struct DeckRimPoint {
    pub x: f64,
    pub y_port: f64,
    pub y_starboard: f64,
    pub z: f64,
}

/// Extract deck rim points from hull stations.
/// The deck rim is the top boundary where port/starboard deck edges meet the hull.
pub fn extract_deck_rim(stations: &[StationLandmarks]) -> Vec<DeckRimPoint> {
    stations.iter().map(|s| DeckRimPoint {
        x: s.x,
        y_port: -s.half_beam_deck,
        y_starboard: s.half_beam_deck,
        z: s.z_deck,
    }).collect()
}

/// Generate the deck surface mesh from deck rim points.
/// Creates a flat(ish) surface bridging port to starboard at deck height.
pub fn generate_deck_surface(
    rim: &[DeckRimPoint],
    _deck: &DeckSpec,
    _dims: &ShipDimensions,
    _enforcer: &StyleEnforcer,
) -> MeshData {
    let mut mesh = MeshData::new(MeshGroup::Deck);

    if rim.len() < 2 {
        return mesh;
    }

    // For each station, create port and starboard vertices on the deck plane
    // Deck is a simple strip from port edge to starboard edge, connected between stations
    let segments = 4; // cross-deck segments for slight curvature

    for point in rim {
        for j in 0..=segments {
            let t = j as f64 / segments as f64;
            let y = point.y_port + (point.y_starboard - point.y_port) * t;
            // Slight crown (higher in center) for visual appeal
            let crown = 0.01 * (1.0 - (2.0 * t - 1.0).powi(2));
            mesh.positions.push([point.x, y, point.z + crown]);
            // Deck normal points up
            mesh.normals.push([0.0, 0.0, 1.0]);
        }
    }

    let ring_size = (segments + 1) as u32;

    // Quad-loft between adjacent stations
    for s in 0..rim.len() as u32 - 1 {
        let base_curr = s * ring_size;
        let base_next = (s + 1) * ring_size;

        for j in 0..segments as u32 {
            let a = base_curr + j;
            let b = base_curr + j + 1;
            let c = base_next + j + 1;
            let d = base_next + j;

            // CCW winding (visible from above)
            mesh.indices.push(a);
            mesh.indices.push(d);
            mesh.indices.push(c);

            mesh.indices.push(a);
            mesh.indices.push(c);
            mesh.indices.push(b);
        }
    }

    mesh
}

/// Placement bands for deck features.
/// Returns (u_start, u_end) normalized position ranges.
pub struct PlacementBands;

impl PlacementBands {
    pub const STERN: (f64, f64) = (0.00, 0.18);
    pub const AFT_DECK: (f64, f64) = (0.18, 0.38);
    pub const MID_DECK: (f64, f64) = (0.38, 0.62);
    pub const FORE_DECK: (f64, f64) = (0.62, 0.84);
    pub const BOW: (f64, f64) = (0.84, 1.00);

    /// Convert a normalized u position to x coordinate.
    pub fn u_to_x(u: f64, overall_length: f64) -> f64 {
        (u - 0.5) * overall_length
    }

    /// Find the deck width (full beam at deck level) at a given u position
    /// by interpolating between stations.
    pub fn deck_width_at_u(u: f64, stations: &[StationLandmarks]) -> f64 {
        if stations.is_empty() { return 0.0; }
        if stations.len() == 1 { return stations[0].half_beam_deck * 2.0; }

        // Find bracketing stations
        for i in 0..stations.len() - 1 {
            if u <= stations[i + 1].u {
                let t = if (stations[i + 1].u - stations[i].u).abs() < 1e-12 {
                    0.0
                } else {
                    (u - stations[i].u) / (stations[i + 1].u - stations[i].u)
                };
                let w = stations[i].half_beam_deck * (1.0 - t) + stations[i + 1].half_beam_deck * t;
                return w * 2.0;
            }
        }
        stations.last().unwrap().half_beam_deck * 2.0
    }

    /// Find the deck z-height at a given u position by interpolating between stations.
    pub fn deck_z_at_u(u: f64, stations: &[StationLandmarks]) -> f64 {
        if stations.is_empty() { return 0.0; }
        if stations.len() == 1 { return stations[0].z_deck; }

        for i in 0..stations.len() - 1 {
            if u <= stations[i + 1].u {
                let t = if (stations[i + 1].u - stations[i].u).abs() < 1e-12 {
                    0.0
                } else {
                    (u - stations[i].u) / (stations[i + 1].u - stations[i].u)
                };
                return stations[i].z_deck * (1.0 - t) + stations[i + 1].z_deck * t;
            }
        }
        stations.last().unwrap().z_deck
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

    fn make_deck_mesh() -> (MeshData, Vec<StationLandmarks>) {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        let stations = compute_stations(&curves, spec.dimensions.overall_length);
        let rim = extract_deck_rim(&stations);
        let mesh = generate_deck_surface(&rim, &spec.deck, &spec.dimensions, &enforcer);
        (mesh, stations)
    }

    #[test]
    fn deck_rim_has_nine_points() {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        let stations = compute_stations(&curves, spec.dimensions.overall_length);
        let rim = extract_deck_rim(&stations);
        assert_eq!(rim.len(), 9);
    }

    #[test]
    fn deck_rim_port_starboard_symmetric() {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        let stations = compute_stations(&curves, spec.dimensions.overall_length);
        let rim = extract_deck_rim(&stations);

        for p in &rim {
            assert!((p.y_port + p.y_starboard).abs() < 1e-10,
                "port ({}) and starboard ({}) should be symmetric", p.y_port, p.y_starboard);
        }
    }

    #[test]
    fn deck_mesh_has_geometry() {
        let (mesh, _) = make_deck_mesh();
        assert!(mesh.vertex_count() > 20, "deck should have vertices: {}", mesh.vertex_count());
        assert!(mesh.triangle_count() > 10, "deck should have triangles: {}", mesh.triangle_count());
    }

    #[test]
    fn deck_mesh_no_invalid_data() {
        let (mesh, _) = make_deck_mesh();
        assert!(!mesh.has_invalid_positions(), "no NaN/Inf positions");
        assert!(!mesh.has_invalid_normals(), "no NaN/Inf normals");
    }

    #[test]
    fn deck_normals_point_upward() {
        let (mesh, _) = make_deck_mesh();
        for n in &mesh.normals {
            assert!(n[2] > 0.9, "deck normals should point up: {:?}", n);
        }
    }

    #[test]
    fn deck_indices_in_bounds() {
        let (mesh, _) = make_deck_mesh();
        let vc = mesh.vertex_count() as u32;
        for &idx in &mesh.indices {
            assert!(idx < vc, "index {} out of bounds (vc={})", idx, vc);
        }
    }

    #[test]
    fn placement_band_deck_width_varies() {
        let (_, stations) = make_deck_mesh();
        let mid_width = PlacementBands::deck_width_at_u(0.5, &stations);
        let bow_width = PlacementBands::deck_width_at_u(0.95, &stations);
        let stern_width = PlacementBands::deck_width_at_u(0.05, &stations);

        assert!(mid_width > bow_width, "midship should be wider than bow");
        assert!(mid_width > stern_width, "midship should be wider than stern");
        assert!(mid_width > 1.0, "midship deck should be > 1m wide");
    }

    #[test]
    fn all_archetypes_generate_deck_surface() {
        for arch in SloopVariantArchetype::all() {
            let spec = arch.default_spec();
            let enforcer = StyleEnforcer::new(&spec.style);
            let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
            let stations = compute_stations(&curves, spec.dimensions.overall_length);
            let rim = extract_deck_rim(&stations);
            let mesh = generate_deck_surface(&rim, &spec.deck, &spec.dimensions, &enforcer);

            assert!(mesh.vertex_count() > 10, "{:?}: deck too few vertices", arch);
            assert!(!mesh.has_invalid_positions(), "{:?}: invalid deck positions", arch);
        }
    }
}
