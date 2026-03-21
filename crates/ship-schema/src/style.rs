use crate::spec::{HullSpec, ShipDimensions, StyleLaw};

/// Clamp a value into [min, max].
fn clamp(v: f64, min: f64, max: f64) -> f64 {
    v.max(min).min(max)
}

/// Lerp between two values.
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Active style-law enforcement.
/// Remaps hull and dimension parameters based on the style profile values.
/// This is not advisory — it actively clamps and adjusts.
pub struct StyleEnforcer<'a> {
    pub law: &'a StyleLaw,
}

impl<'a> StyleEnforcer<'a> {
    pub fn new(law: &'a StyleLaw) -> Self {
        Self { law }
    }

    /// Enforce style law on hull parameters. Returns a clamped copy.
    pub fn enforce_hull(&self, hull: &HullSpec) -> HullSpec {
        let mut h = hull.clone();
        let se = self.law.silhouette_exaggeration;
        let rb = self.law.realism_bias;

        // Sheer: higher exaggeration allows more, higher realism clamps it
        let sheer_max = lerp(0.15, 0.40, se) * lerp(1.0, 0.7, rb);
        h.sheer = clamp(h.sheer, 0.05, sheer_max);

        // Rake: higher exaggeration allows more lean
        let rake_max = lerp(0.10, 0.30, se) * lerp(1.0, 0.6, rb);
        h.rake = clamp(h.rake, 0.0, rake_max);

        // Tumblehome: higher realism keeps it conservative
        let tumblehome_max = lerp(0.30, 0.15, rb);
        h.tumblehome = clamp(h.tumblehome, 0.0, tumblehome_max);

        // Fullness: always in valid range
        h.fullness = clamp(h.fullness, 0.20, 0.80);

        // Keel depth factor
        h.keel_depth = clamp(h.keel_depth, 0.3, 1.0);

        // Transom width factor
        h.transom_width = clamp(h.transom_width, 0.0, 1.0);

        // Quarterdeck height
        h.quarterdeck_height = clamp(h.quarterdeck_height, 0.0, 1.0);

        h
    }

    /// Enforce style law on dimensions. Returns a clamped copy.
    pub fn enforce_dimensions(&self, dims: &ShipDimensions) -> ShipDimensions {
        let mut d = dims.clone();

        // Hard physical constraints
        d.overall_length = clamp(d.overall_length, 4.0, 30.0);
        d.beam = clamp(d.beam, 1.0, d.overall_length * 0.5);
        d.draft_visual = clamp(d.draft_visual, 0.3, d.beam);
        d.freeboard = clamp(d.freeboard, 0.3, 3.0);
        d.mast_height = clamp(d.mast_height, 2.0, d.overall_length * 1.5);
        d.bowsprit_length = clamp(d.bowsprit_length, 0.0, d.overall_length * 0.4);

        d
    }

    /// Compute section resolution from style law.
    /// Returns the number of sample points per half-section (one side, keel to deck).
    pub fn section_resolution(&self) -> usize {
        let dd = self.law.detail_density;
        let ec = self.law.edge_chunkiness;
        // Base from detail density: 8..20
        let base = lerp(8.0, 20.0, dd);
        // Chunkiness pulls resolution down slightly
        let adjusted = base * lerp(1.0, 0.85, ec);
        (adjusted.round() as usize).max(8).min(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::*;

    fn test_law() -> StyleLaw {
        StyleLaw {
            profile: ShipStyleProfile::PortlightClassic,
            silhouette_exaggeration: 0.55,
            detail_density: 0.38,
            edge_chunkiness: 0.62,
            realism_bias: 0.35,
        }
    }

    #[test]
    fn sheer_clamped_by_style() {
        let law = test_law();
        let enforcer = StyleEnforcer::new(&law);
        let mut hull = crate::defaults::classic_runner().hull;
        hull.sheer = 0.90; // way too high
        let enforced = enforcer.enforce_hull(&hull);
        assert!(enforced.sheer <= 0.40, "sheer should be clamped");
        assert!(enforced.sheer >= 0.05, "sheer should have minimum");
    }

    #[test]
    fn dimensions_beam_cannot_exceed_half_length() {
        let law = test_law();
        let enforcer = StyleEnforcer::new(&law);
        let mut dims = crate::defaults::classic_runner().dimensions;
        dims.beam = 20.0; // absurd
        let enforced = enforcer.enforce_dimensions(&dims);
        assert!(enforced.beam <= enforced.overall_length * 0.5);
    }

    #[test]
    fn section_resolution_in_range() {
        for dd in [0.0, 0.25, 0.5, 0.75, 1.0] {
            for ec in [0.0, 0.5, 1.0] {
                let law = StyleLaw {
                    detail_density: dd,
                    edge_chunkiness: ec,
                    ..test_law()
                };
                let enforcer = StyleEnforcer::new(&law);
                let res = enforcer.section_resolution();
                assert!(res >= 8 && res <= 20, "resolution {res} out of range for dd={dd} ec={ec}");
            }
        }
    }

    #[test]
    fn high_realism_tightens_constraints() {
        let low_law = StyleLaw {
            realism_bias: 0.1,
            ..test_law()
        };
        let high_law = StyleLaw {
            realism_bias: 0.9,
            ..test_law()
        };
        let low_realism = StyleEnforcer::new(&low_law);
        let high_realism = StyleEnforcer::new(&high_law);

        // Push sheer high and verify high realism clamps tighter
        let mut extreme = crate::defaults::classic_runner().hull;
        extreme.sheer = 1.0;
        let low_clamped = low_realism.enforce_hull(&extreme);
        let high_clamped = high_realism.enforce_hull(&extreme);
        assert!(high_clamped.sheer <= low_clamped.sheer,
            "high realism should clamp sheer tighter: {} vs {}", high_clamped.sheer, low_clamped.sheer);
    }
}
