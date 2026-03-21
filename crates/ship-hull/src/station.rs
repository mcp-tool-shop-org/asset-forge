use crate::longitudinal::HullCurves;

/// The 9 named stations along the hull.
pub const STATIONS: [f64; 9] = [
    0.00, // stern edge
    0.08, // stern shoulder
    0.18, // aft body
    0.32, // aft-mid
    0.48, // midship
    0.64, // fore-mid
    0.78, // fore body
    0.90, // bow shoulder
    1.00, // bow tip
];

/// Derived section landmarks at a single station.
#[derive(Debug, Clone)]
pub struct StationLandmarks {
    /// Normalized position along hull length [0,1]. 0=stern, 1=bow.
    pub u: f64,
    /// X position in meters (derived from u and overall_length).
    pub x: f64,
    /// Keel Z position (bottom of hull).
    pub z_keel: f64,
    /// Deck sheer Z position (top of hull).
    pub z_deck: f64,
    /// Z position where max beam occurs (beam band height).
    pub z_beam_band: f64,
    /// Half-beam at this station in meters.
    pub half_beam: f64,
    /// Half-beam at deck level after tumblehome in meters.
    pub half_beam_deck: f64,
    /// Tumblehome factor at this station.
    pub tumblehome: f64,
    /// Bow closure factor (0 = no closure, 1 = full closure).
    pub bow_closure: f64,
}

impl StationLandmarks {
    /// Verify the fundamental invariant: z_keel < z_beam_band < z_deck.
    pub fn is_valid(&self) -> bool {
        self.z_keel < self.z_beam_band
            && self.z_beam_band < self.z_deck
            && self.half_beam >= 0.0
            && self.half_beam_deck >= 0.0
            && self.half_beam.is_finite()
            && self.z_keel.is_finite()
            && self.z_deck.is_finite()
    }
}

/// Compute station landmarks for all 9 named stations.
pub fn compute_stations(
    curves: &HullCurves,
    overall_length: f64,
) -> Vec<StationLandmarks> {
    STATIONS.iter().map(|&u| compute_single_station(curves, u, overall_length)).collect()
}

/// Compute landmarks for a single station.
pub fn compute_single_station(
    curves: &HullCurves,
    u: f64,
    overall_length: f64,
) -> StationLandmarks {
    let z_keel = curves.keel.eval(u);
    let z_deck = curves.sheer.eval(u);
    let half_beam = curves.beam.eval(u).max(0.0);
    let tumblehome = curves.tumblehome.eval(u).max(0.0);
    let bow_closure = curves.bow_closure.eval(u).clamp(0.0, 1.0);

    // Beam band sits at ~48% of hull height above keel
    let z_beam_band = z_keel + (z_deck - z_keel) * 0.48;

    // Deck half-beam after tumblehome reduction
    let half_beam_deck = half_beam * (1.0 - tumblehome);

    // X position: u=0 is stern (-L/2), u=1 is bow (+L/2)
    let x = (u - 0.5) * overall_length;

    StationLandmarks {
        u,
        x,
        z_keel,
        z_deck,
        z_beam_band,
        half_beam,
        half_beam_deck,
        tumblehome,
        bow_closure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_schema::defaults::classic_runner;
    use ship_schema::spec::SloopVariantArchetype;
    use ship_schema::style::StyleEnforcer;

    fn make_stations() -> Vec<StationLandmarks> {
        let spec = classic_runner();
        let enforcer = StyleEnforcer::new(&spec.style);
        let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
        compute_stations(&curves, spec.dimensions.overall_length)
    }

    #[test]
    fn nine_stations_generated() {
        let stations = make_stations();
        assert_eq!(stations.len(), 9);
    }

    #[test]
    fn all_stations_valid_landmarks() {
        let stations = make_stations();
        for s in &stations {
            assert!(s.is_valid(),
                "station u={} invalid: z_keel={}, z_beam_band={}, z_deck={}, half_beam={}",
                s.u, s.z_keel, s.z_beam_band, s.z_deck, s.half_beam);
        }
    }

    #[test]
    fn x_positions_span_hull_length() {
        let stations = make_stations();
        let spec = classic_runner();
        let l = spec.dimensions.overall_length;

        let stern_x = stations.first().unwrap().x;
        let bow_x = stations.last().unwrap().x;

        assert!((stern_x - (-l / 2.0)).abs() < 0.01, "stern x should be -L/2");
        assert!((bow_x - (l / 2.0)).abs() < 0.01, "bow x should be +L/2");
    }

    #[test]
    fn deck_above_keel_everywhere() {
        let stations = make_stations();
        for s in &stations {
            assert!(s.z_deck > s.z_keel,
                "deck must be above keel at u={}: deck={}, keel={}",
                s.u, s.z_deck, s.z_keel);
        }
    }

    #[test]
    fn beam_band_between_keel_and_deck() {
        let stations = make_stations();
        for s in &stations {
            assert!(s.z_beam_band > s.z_keel && s.z_beam_band < s.z_deck,
                "beam band must be between keel and deck at u={}", s.u);
        }
    }

    #[test]
    fn deck_beam_less_than_or_equal_max_beam() {
        let stations = make_stations();
        for s in &stations {
            assert!(s.half_beam_deck <= s.half_beam + 1e-10,
                "deck beam ({}) should not exceed max beam ({}) at u={}",
                s.half_beam_deck, s.half_beam, s.u);
        }
    }

    #[test]
    fn all_archetypes_produce_valid_stations() {
        for arch in SloopVariantArchetype::all() {
            let spec = arch.default_spec();
            let enforcer = StyleEnforcer::new(&spec.style);
            let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer).unwrap();
            let stations = compute_stations(&curves, spec.dimensions.overall_length);

            assert_eq!(stations.len(), 9, "{:?} should have 9 stations", arch);
            for s in &stations {
                assert!(s.is_valid(),
                    "{:?} station u={} invalid", arch, s.u);
            }
        }
    }
}
