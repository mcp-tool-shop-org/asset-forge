use crate::station::StationLandmarks;

/// A 2D point in the Y/Z cross-section plane.
#[derive(Debug, Clone, Copy)]
pub struct SectionPoint {
    pub y: f64,
    pub z: f64,
}

/// A sampled cross-section profile at one station.
/// Points run from keel (bottom center) up the starboard side to deck.
/// Port side is the mirror (negate y).
#[derive(Debug, Clone)]
pub struct SectionProfile {
    /// Station u value [0,1].
    pub u: f64,
    /// Starboard-side points from keel to deck.
    pub points: Vec<SectionPoint>,
}

impl SectionProfile {
    /// Mirror to get port-side points (negate y, reverse order, skip keel which is shared).
    pub fn port_side(&self) -> Vec<SectionPoint> {
        self.points.iter()
            .rev()
            .skip(1) // skip the deck point (will be its own mirror)
            .map(|p| SectionPoint { y: -p.y, z: p.z })
            .collect()
    }

    /// Full section: port side (bottom-up) + starboard side (bottom-up).
    /// Returns points going around the section from port-deck → keel → starboard-deck.
    pub fn full_section(&self) -> Vec<SectionPoint> {
        let mut full = self.port_side();
        full.extend_from_slice(&self.points);
        full
    }
}

/// Configuration for section profile sampling.
#[derive(Debug, Clone, Copy)]
pub struct SectionConfig {
    /// Total samples per half-section (starboard side, keel to deck).
    pub half_section_samples: usize,
    /// Exponent for lower hull curve. Higher = fuller/rounder hull.
    pub lower_fullness_exponent: f64,
    /// Exponent for upper topside curve. Higher = straighter then sudden tuck.
    pub upper_taper_exponent: f64,
    /// Small keel half-width in meters (keel is not a perfect point).
    pub keel_half_width: f64,
}

impl Default for SectionConfig {
    fn default() -> Self {
        Self {
            half_section_samples: 12,
            lower_fullness_exponent: 1.8,
            upper_taper_exponent: 1.5,
            keel_half_width: 0.02,
        }
    }
}

/// Sample a cross-section profile at a station using the two-stage profile law.
///
/// Lower hull (keel to beam band): superellipse-ish easing.
/// Upper hull (beam band to deck): smooth topside taper.
pub fn sample_section(
    landmarks: &StationLandmarks,
    config: &SectionConfig,
) -> SectionProfile {
    let n = config.half_section_samples.max(4);

    // Split samples: ~60% lower hull, ~40% upper hull
    let lower_count = (n as f64 * 0.6).round() as usize;
    let upper_count = n - lower_count;

    let mut points = Vec::with_capacity(n + 1);

    // -- Lower hull: keel → beam band --
    for i in 0..=lower_count {
        let t = i as f64 / lower_count as f64;
        let z = lerp(landmarks.z_keel, landmarks.z_beam_band, t);

        // Superellipse-ish curve: y grows from keel width to max beam
        let shape = lower_curve(t, config.lower_fullness_exponent);
        let y = lerp(config.keel_half_width, landmarks.half_beam, shape);

        points.push(SectionPoint { y, z });
    }

    // -- Upper hull: beam band → deck --
    // Skip t=0 since it's the same as the last lower point
    for i in 1..=upper_count {
        let t = i as f64 / upper_count as f64;
        let z = lerp(landmarks.z_beam_band, landmarks.z_deck, t);

        // Topside taper: y goes from max beam to deck beam
        let shape = upper_curve(t, config.upper_taper_exponent);
        let y = lerp(landmarks.half_beam, landmarks.half_beam_deck, shape);

        points.push(SectionPoint { y, z });
    }

    SectionProfile { u: landmarks.u, points }
}

/// Lower hull curve: controls how quickly beam fills out from keel.
/// t ∈ [0,1], returns [0,1].
/// Higher exponent = fuller, rounder hull.
fn lower_curve(t: f64, exponent: f64) -> f64 {
    1.0 - (1.0 - t).powf(exponent)
}

/// Upper hull curve: controls topside taper shape.
/// t ∈ [0,1], returns [0,1].
/// Higher exponent = straighter sides then sudden tuck near deck.
fn upper_curve(t: f64, exponent: f64) -> f64 {
    t.powf(exponent)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Derive section config from style law values.
pub fn config_from_style(
    half_section_samples: usize,
    fullness: f64,
    edge_chunkiness: f64,
) -> SectionConfig {
    // Fullness maps to lower exponent: higher fullness = higher exponent = rounder
    let lower_exp = lerp(1.2, 2.8, fullness);

    // Edge chunkiness maps to upper exponent: higher chunkiness = lower exponent = boxier
    let upper_exp = lerp(2.0, 1.0, edge_chunkiness);

    // Keel width: chunkier = slightly wider keel
    let keel_hw = lerp(0.01, 0.04, edge_chunkiness);

    SectionConfig {
        half_section_samples,
        lower_fullness_exponent: lower_exp,
        upper_taper_exponent: upper_exp,
        keel_half_width: keel_hw,
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

    fn make_classic_sections() -> Vec<SectionProfile> {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        let stations = compute_stations(&curves, spec.dimensions.overall_length);
        let config = config_from_style(
            enforcer.section_resolution(),
            spec.hull.fullness,
            spec.style.edge_chunkiness,
        );
        stations.iter().map(|s| sample_section(s, &config)).collect()
    }

    #[test]
    fn section_point_count_matches_config() {
        let sections = make_classic_sections();
        for section in &sections {
            let n = section.points.len();
            assert!(n >= 4, "section at u={} has too few points: {n}", section.u);
            assert!(n <= 30, "section at u={} has too many points: {n}", section.u);
        }
    }

    #[test]
    fn section_z_monotonically_increases() {
        let sections = make_classic_sections();
        for section in &sections {
            for w in section.points.windows(2) {
                assert!(w[1].z >= w[0].z - 1e-10,
                    "z should increase from keel to deck at u={}: {} -> {}",
                    section.u, w[0].z, w[1].z);
            }
        }
    }

    #[test]
    fn section_y_non_negative() {
        let sections = make_classic_sections();
        for section in &sections {
            for p in &section.points {
                assert!(p.y >= -1e-10,
                    "starboard y should be non-negative at u={}: y={}", section.u, p.y);
            }
        }
    }

    #[test]
    fn keel_point_is_narrow() {
        let sections = make_classic_sections();
        for section in &sections {
            let keel = &section.points[0];
            assert!(keel.y < 0.1,
                "keel should be narrow at u={}: y={}", section.u, keel.y);
        }
    }

    #[test]
    fn full_section_is_symmetric() {
        let sections = make_classic_sections();
        let section = &sections[4]; // midship
        let _full = section.full_section();

        // Port side points should mirror starboard
        let port = section.port_side();
        let starboard = &section.points;

        // Port and starboard should have matching z values
        for (i, pp) in port.iter().enumerate() {
            let si = starboard.len() - 2 - i; // reverse index, skip deck
            let sp = &starboard[si];
            assert!((pp.z - sp.z).abs() < 1e-10,
                "port z should match starboard z");
            assert!((pp.y + sp.y).abs() < 1e-10,
                "port y should negate starboard y");
        }
    }

    #[test]
    fn no_self_intersection_in_section() {
        let sections = make_classic_sections();
        for section in &sections {
            // Simple check: y values in lower hull should be monotonically increasing
            // (no sudden dip back inward before beam band)
            let lower_count = (section.points.len() as f64 * 0.6).round() as usize;
            for w in section.points[..lower_count.min(section.points.len())].windows(2) {
                assert!(w[1].y >= w[0].y - 1e-10,
                    "lower hull y should increase at u={}: {} -> {}",
                    section.u, w[0].y, w[1].y);
            }
        }
    }

    #[test]
    fn section_resolution_varies_with_detail_density() {
        let low = config_from_style(8, 0.5, 0.5);
        let high = config_from_style(20, 0.5, 0.5);
        assert_eq!(low.half_section_samples, 8);
        assert_eq!(high.half_section_samples, 20);
    }

    #[test]
    fn all_archetypes_produce_valid_sections() {
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

            for station in &stations {
                let section = sample_section(station, &config);
                // No NaN/Inf
                for p in &section.points {
                    assert!(p.y.is_finite() && p.z.is_finite(),
                        "{:?} section at u={} has non-finite point: y={}, z={}",
                        arch, station.u, p.y, p.z);
                }
                // Z monotonic
                for w in section.points.windows(2) {
                    assert!(w[1].z >= w[0].z - 1e-10,
                        "{:?} section z not monotonic at u={}", arch, station.u);
                }
            }
        }
    }
}
