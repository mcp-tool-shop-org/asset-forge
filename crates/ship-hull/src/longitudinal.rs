use ship_schema::spec::{BowStyle, HullProfile, HullSpec, ShipDimensions};
use ship_schema::style::StyleEnforcer;

use crate::curves::{ControlPoint, EndpointConfig, LongitudinalCurve};

/// The five longitudinal curves that define the hull envelope.
#[derive(Debug)]
pub struct HullCurves {
    /// Bottom spine: z-values of the keel along length.
    pub keel: LongitudinalCurve,
    /// Top spine: z-values of the deck sheer line.
    pub sheer: LongitudinalCurve,
    /// Maximum half-beam at each station (in meters).
    pub beam: LongitudinalCurve,
    /// Tumblehome factor at each station (0..1).
    pub tumblehome: LongitudinalCurve,
    /// Bow closure rate (how quickly beam collapses at bow). Higher = sharper.
    pub bow_closure: LongitudinalCurve,
}

impl HullCurves {
    /// Build the five longitudinal curves from enforced spec values.
    pub fn from_spec(
        dims: &ShipDimensions,
        hull: &HullSpec,
        enforcer: &StyleEnforcer,
    ) -> Result<Self, crate::curves::CurveError> {
        let dims = enforcer.enforce_dimensions(dims);
        let hull = enforcer.enforce_hull(hull);
        let se = enforcer.law.silhouette_exaggeration;

        let keel = build_keel_curve(&dims, &hull, se)?;
        let sheer = build_sheer_curve(&dims, &hull, se)?;
        let beam = build_beam_curve(&dims, &hull)?;
        let tumblehome = build_tumblehome_curve(&hull)?;
        let bow_closure = build_bow_closure_curve(&hull)?;

        Ok(Self { keel, sheer, beam, tumblehome, bow_closure })
    }
}

fn build_keel_curve(
    dims: &ShipDimensions,
    hull: &HullSpec,
    se: f64,
) -> Result<LongitudinalCurve, crate::curves::CurveError> {
    let d = dims.draft_visual * hull.keel_depth;
    let rake_lift = hull.rake * 0.3 * d * (1.0 + se * 0.5);
    let bow_rise = d * 0.9 * (1.0 + se * 0.3);

    // Profile-based fullness affects where the keel is deepest
    let deep_u = match hull.profile {
        HullProfile::Swift => 0.50,
        HullProfile::Balanced => 0.48,
        HullProfile::CargoLight => 0.45,
        HullProfile::NavyLight => 0.47,
    };

    let points = vec![
        cp(0.00, -d * 0.15 - rake_lift),    // stern
        cp(0.08, -d * 0.35),
        cp(0.18, -d * 0.72),
        cp(0.32, -d * 0.92),
        cp(deep_u, -d),                      // deepest point
        cp(0.64, -d * 0.90),
        cp(0.78, -d * 0.68),
        cp(0.90, -d * 0.32),
        cp(1.00, bow_rise * 0.1),            // bow tip rises above waterline
    ];

    let endpoint_config = EndpointConfig {
        stern_tangent_scale: 0.8,
        stern_slope_cap: d * 2.0,
        bow_tangent_scale: match hull.bow_style {
            BowStyle::Needle => 1.4,
            BowStyle::Raked => 1.2,
            BowStyle::Rounded => 0.8,
            BowStyle::Reinforced => 0.9,
        },
        bow_slope_cap: d * 3.0,
    };

    LongitudinalCurve::new(points, endpoint_config)
}

fn build_sheer_curve(
    dims: &ShipDimensions,
    hull: &HullSpec,
    se: f64,
) -> Result<LongitudinalCurve, crate::curves::CurveError> {
    let f = dims.freeboard;
    let sheer_amp = hull.sheer * f * (1.0 + se * 0.5);
    let qd = hull.quarterdeck_height;

    let points = vec![
        cp(0.00, f + sheer_amp * 0.8 + qd),   // stern with quarterdeck
        cp(0.08, f + sheer_amp * 0.5 + qd * 0.7),
        cp(0.18, f + sheer_amp * 0.1),
        cp(0.32, f - sheer_amp * 0.15),
        cp(0.48, f - sheer_amp * 0.2),          // lowest point (waist)
        cp(0.64, f - sheer_amp * 0.08),
        cp(0.78, f + sheer_amp * 0.3),
        cp(0.90, f + sheer_amp * 0.9),
        cp(1.00, f + sheer_amp * 1.2),          // bow rise
    ];

    let endpoint_config = EndpointConfig {
        stern_tangent_scale: 0.6,
        stern_slope_cap: f * 2.0,
        bow_tangent_scale: 1.0 + se * 0.3,
        bow_slope_cap: f * 3.0,
    };

    LongitudinalCurve::new(points, endpoint_config)
}

fn build_beam_curve(
    dims: &ShipDimensions,
    hull: &HullSpec,
) -> Result<LongitudinalCurve, crate::curves::CurveError> {
    let half_beam = dims.beam / 2.0;

    // Fullness affects how wide the body stays
    let full = hull.fullness;
    let stern_w = hull.transom_width * 0.12;

    // Profile affects where peak beam sits
    let peak_u = match hull.profile {
        HullProfile::Swift => 0.46,
        HullProfile::Balanced => 0.44,
        HullProfile::CargoLight => 0.42,
        HullProfile::NavyLight => 0.45,
    };

    let bow_tip = match hull.bow_style {
        BowStyle::Needle => 0.01,
        BowStyle::Raked => 0.02,
        BowStyle::Rounded => 0.04,
        BowStyle::Reinforced => 0.03,
    };

    let points = vec![
        cp(0.00, half_beam * stern_w),
        cp(0.08, half_beam * lerp(0.40, 0.55, full)),
        cp(0.18, half_beam * lerp(0.72, 0.85, full)),
        cp(0.32, half_beam * lerp(0.90, 0.97, full)),
        cp(peak_u, half_beam),                              // max beam
        cp(0.64, half_beam * lerp(0.85, 0.95, full)),
        cp(0.78, half_beam * lerp(0.60, 0.78, full)),
        cp(0.90, half_beam * lerp(0.28, 0.45, full)),
        cp(1.00, half_beam * bow_tip),
    ];

    let endpoint_config = EndpointConfig {
        stern_tangent_scale: 0.7,
        stern_slope_cap: half_beam * 4.0,
        bow_tangent_scale: 1.0,
        bow_slope_cap: half_beam * 5.0,
    };

    LongitudinalCurve::new(points, endpoint_config)
}

fn build_tumblehome_curve(
    hull: &HullSpec,
) -> Result<LongitudinalCurve, crate::curves::CurveError> {
    let th = hull.tumblehome;

    let points = vec![
        cp(0.00, th * 1.2),     // stronger at stern
        cp(0.08, th * 1.1),
        cp(0.18, th * 0.9),
        cp(0.32, th * 0.8),
        cp(0.48, th),           // reference at midship
        cp(0.64, th * 0.85),
        cp(0.78, th * 0.7),
        cp(0.90, th * 0.5),
        cp(1.00, th * 0.3),     // lighter at bow
    ];

    LongitudinalCurve::new(points, EndpointConfig::default())
}

fn build_bow_closure_curve(
    hull: &HullSpec,
) -> Result<LongitudinalCurve, crate::curves::CurveError> {
    let closure_rate = match hull.bow_style {
        BowStyle::Needle => 0.9,
        BowStyle::Raked => 0.7,
        BowStyle::Rounded => 0.4,
        BowStyle::Reinforced => 0.5,
    };

    let points = vec![
        cp(0.00, 0.0),          // stern: no closure effect
        cp(0.50, 0.0),
        cp(0.78, closure_rate * 0.3),
        cp(0.90, closure_rate * 0.7),
        cp(1.00, closure_rate),  // full bow closure
    ];

    LongitudinalCurve::new(points, EndpointConfig::default())
}

fn cp(u: f64, value: f64) -> ControlPoint {
    ControlPoint { u, value }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_schema::defaults::classic_runner;
    use ship_schema::spec::StyleLaw;
    use ship_schema::style::StyleEnforcer;

    fn test_enforcer() -> (StyleLaw, ship_schema::spec::ShipDimensions, ship_schema::spec::HullSpec) {
        let spec = classic_runner();
        (spec.style, spec.dimensions, spec.hull)
    }

    #[test]
    fn hull_curves_build_from_classic_runner() {
        let (law, dims, hull) = test_enforcer();
        let enforcer = StyleEnforcer::new(&law);
        let curves = HullCurves::from_spec(&dims, &hull, &enforcer).unwrap();

        // Basic sanity: all curves have 9 points (or at least >= 5)
        assert!(curves.keel.len() >= 5);
        assert!(curves.sheer.len() >= 5);
        assert!(curves.beam.len() >= 5);
    }

    #[test]
    fn keel_below_sheer_at_all_stations() {
        let (law, dims, hull) = test_enforcer();
        let enforcer = StyleEnforcer::new(&law);
        let curves = HullCurves::from_spec(&dims, &hull, &enforcer).unwrap();

        for i in 0..=100 {
            let u = i as f64 / 100.0;
            let keel_z = curves.keel.eval(u);
            let sheer_z = curves.sheer.eval(u);
            assert!(keel_z < sheer_z,
                "keel ({keel_z}) must be below sheer ({sheer_z}) at u={u}");
        }
    }

    #[test]
    fn beam_peaks_near_midship() {
        let (law, dims, hull) = test_enforcer();
        let enforcer = StyleEnforcer::new(&law);
        let curves = HullCurves::from_spec(&dims, &hull, &enforcer).unwrap();

        let (peak_u, _peak_v) = (0..=100)
            .map(|i| {
                let u = i as f64 / 100.0;
                (u, curves.beam.eval(u))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();

        assert!(peak_u > 0.30 && peak_u < 0.60,
            "beam should peak near midship, got u={peak_u}");
    }

    #[test]
    fn beam_narrow_at_bow_and_stern() {
        let (law, dims, hull) = test_enforcer();
        let enforcer = StyleEnforcer::new(&law);
        let curves = HullCurves::from_spec(&dims, &hull, &enforcer).unwrap();

        let half_beam = dims.beam / 2.0;
        let stern_beam = curves.beam.eval(0.0);
        let bow_beam = curves.beam.eval(1.0);

        assert!(stern_beam < half_beam * 0.3,
            "stern should be narrow: {stern_beam} vs half_beam {half_beam}");
        assert!(bow_beam < half_beam * 0.1,
            "bow should be very narrow: {bow_beam}");
    }

    #[test]
    fn tumblehome_strongest_at_stern() {
        let (law, dims, hull) = test_enforcer();
        let enforcer = StyleEnforcer::new(&law);
        let curves = HullCurves::from_spec(&dims, &hull, &enforcer).unwrap();

        let stern_th = curves.tumblehome.eval(0.0);
        let mid_th = curves.tumblehome.eval(0.5);
        let bow_th = curves.tumblehome.eval(1.0);

        assert!(stern_th >= mid_th, "tumblehome should be >= at stern vs midship");
        assert!(mid_th >= bow_th, "tumblehome should be >= at midship vs bow");
    }

    #[test]
    fn all_archetypes_build_valid_curves() {
        use ship_schema::spec::SloopVariantArchetype;

        for arch in SloopVariantArchetype::all() {
            let spec = arch.default_spec();
            let enforcer = StyleEnforcer::new(&spec.style);
            let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer);
            assert!(curves.is_ok(), "{:?} should produce valid curves: {:?}", arch, curves.err());

            let curves = curves.unwrap();
            // Keel must be below sheer everywhere
            for i in 0..=20 {
                let u = i as f64 / 20.0;
                assert!(curves.keel.eval(u) < curves.sheer.eval(u),
                    "{:?}: keel must be below sheer at u={u}", arch);
            }
        }
    }
}
