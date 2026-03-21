use crate::spec::SloopAssetSpec;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpecError {
    #[error("invalid version: expected 1 or 2, got {0}")]
    InvalidVersion(u32),

    #[error("invalid kind: expected \"ship\", got \"{0}\"")]
    InvalidKind(String),

    #[error("{field} must be positive, got {value}")]
    NonPositive { field: &'static str, value: f64 },

    #[error("{field} must be in [{min}, {max}], got {value}")]
    OutOfRange {
        field: &'static str,
        value: f64,
        min: f64,
        max: f64,
    },

    #[error("sloop must have mast_count = 1, got {0}")]
    InvalidMastCount(u8),

    #[error("freeboard ({freeboard}) must be greater than zero with positive draft ({draft})")]
    InvalidFreeboardDraft { freeboard: f64, draft: f64 },

    #[error("{0}")]
    Custom(String),
}

/// Validate a spec at the schema level (before geometry).
/// Returns all errors found, not just the first.
pub fn validate_spec(spec: &SloopAssetSpec) -> Vec<SpecError> {
    let mut errors = Vec::new();

    // Version
    if spec.version != 1 && spec.version != 2 {
        errors.push(SpecError::InvalidVersion(spec.version));
    }

    // Kind
    if spec.kind != "ship" {
        errors.push(SpecError::InvalidKind(spec.kind.clone()));
    }

    // Dimensions: all must be positive
    let d = &spec.dimensions;
    check_positive(&mut errors, "overall_length", d.overall_length);
    check_positive(&mut errors, "beam", d.beam);
    check_positive(&mut errors, "draft_visual", d.draft_visual);
    check_positive(&mut errors, "freeboard", d.freeboard);
    check_positive(&mut errors, "mast_height", d.mast_height);
    check_range(&mut errors, "bowsprit_length", d.bowsprit_length, 0.0, 100.0);

    // Hull ranges (0..1 factors)
    let h = &spec.hull;
    check_range(&mut errors, "keel_depth", h.keel_depth, 0.0, 2.0);
    check_range(&mut errors, "tumblehome", h.tumblehome, 0.0, 1.0);
    check_range(&mut errors, "sheer", h.sheer, 0.0, 1.0);
    check_range(&mut errors, "fullness", h.fullness, 0.0, 1.0);
    check_range(&mut errors, "rake", h.rake, 0.0, 1.0);
    check_range(&mut errors, "quarterdeck_height", h.quarterdeck_height, 0.0, 3.0);
    check_range(&mut errors, "transom_width", h.transom_width, 0.0, 1.0);

    // Sloop must have exactly 1 mast
    if spec.rig.mast_count != 1 {
        errors.push(SpecError::InvalidMastCount(spec.rig.mast_count));
    }

    // Rig ranges
    check_range(&mut errors, "mast_rake", spec.rig.mast_rake, 0.0, 0.5);
    check_positive(&mut errors, "boom_length", spec.rig.boom_length);
    check_range(&mut errors, "gaff_length", spec.rig.gaff_length, 0.0, 20.0);
    check_range(&mut errors, "stay_thickness", spec.rig.stay_thickness, 0.0, 0.5);

    // Sail ranges
    check_range(&mut errors, "mainsail_fullness", spec.sails.mainsail_fullness, 0.0, 1.0);
    check_range(&mut errors, "jib_fullness", spec.sails.jib_fullness, 0.0, 1.0);
    check_range(&mut errors, "patchiness", spec.sails.patchiness, 0.0, 1.0);
    check_range(&mut errors, "edge_wear", spec.sails.edge_wear, 0.0, 1.0);

    // Style ranges
    let s = &spec.style;
    check_range(&mut errors, "silhouette_exaggeration", s.silhouette_exaggeration, 0.0, 1.0);
    check_range(&mut errors, "detail_density", s.detail_density, 0.0, 1.0);
    check_range(&mut errors, "edge_chunkiness", s.edge_chunkiness, 0.0, 1.0);
    check_range(&mut errors, "realism_bias", s.realism_bias, 0.0, 1.0);

    // Deck
    if spec.deck.cabin && spec.deck.cabin_length <= 0.0 {
        errors.push(SpecError::Custom(
            "cabin is enabled but cabin_length is not positive".into(),
        ));
    }
    check_range(&mut errors, "rail_height", spec.deck.rail_height, 0.0, 2.0);
    check_range(&mut errors, "mast_offset", spec.deck.mast_offset, -1.0, 1.0);

    // V2 deck fields
    if spec.version >= 2 {
        check_range(&mut errors, "cabin_width_ratio", spec.deck.cabin_width_ratio, 0.1, 1.0);
        check_range(&mut errors, "cabin_height", spec.deck.cabin_height, 0.0, 3.0);
        check_range(&mut errors, "cabin_offset", spec.deck.cabin_offset, -0.5, 0.5);
        check_range(&mut errors, "hatch_spacing", spec.deck.hatch_spacing, 0.3, 5.0);
        check_range(&mut errors, "rail_thickness", spec.deck.rail_thickness, 0.01, 0.2);
        check_range(&mut errors, "quarterdeck_length", spec.deck.quarterdeck_length, 0.0, 10.0);
        check_range(&mut errors, "mast_diameter", spec.rig.mast_diameter, 0.05, 0.5);
        check_range(&mut errors, "boom_height", spec.rig.boom_height, 0.3, 3.0);
        check_range(&mut errors, "gaff_angle", spec.rig.gaff_angle, 0.1, 1.2);
        check_range(&mut errors, "bowsprit_angle", spec.rig.bowsprit_angle, 0.0, 0.5);
    }

    errors
}

fn check_positive(errors: &mut Vec<SpecError>, field: &'static str, value: f64) {
    if value <= 0.0 || !value.is_finite() {
        errors.push(SpecError::NonPositive { field, value });
    }
}

fn check_range(errors: &mut Vec<SpecError>, field: &'static str, value: f64, min: f64, max: f64) {
    if !value.is_finite() || value < min || value > max {
        errors.push(SpecError::OutOfRange {
            field,
            value,
            min,
            max,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults::classic_runner;

    #[test]
    fn valid_classic_runner_passes() {
        let spec = classic_runner();
        let errors = validate_spec(&spec);
        assert!(errors.is_empty(), "classic runner should be valid: {:?}", errors);
    }

    #[test]
    fn invalid_version_rejected() {
        let mut spec = classic_runner();
        spec.version = 99;
        let errors = validate_spec(&spec);
        assert!(errors.iter().any(|e| matches!(e, SpecError::InvalidVersion(99))));
    }

    #[test]
    fn invalid_kind_rejected() {
        let mut spec = classic_runner();
        spec.kind = "wagon".into();
        let errors = validate_spec(&spec);
        assert!(errors.iter().any(|e| matches!(e, SpecError::InvalidKind(_))));
    }

    #[test]
    fn negative_dimensions_rejected() {
        let mut spec = classic_runner();
        spec.dimensions.overall_length = -5.0;
        spec.dimensions.beam = 0.0;
        let errors = validate_spec(&spec);
        assert!(errors.len() >= 2, "should catch both: {:?}", errors);
    }

    #[test]
    fn wrong_mast_count_rejected() {
        let mut spec = classic_runner();
        spec.rig.mast_count = 3;
        let errors = validate_spec(&spec);
        assert!(errors.iter().any(|e| matches!(e, SpecError::InvalidMastCount(3))));
    }

    #[test]
    fn out_of_range_style_rejected() {
        let mut spec = classic_runner();
        spec.style.silhouette_exaggeration = 1.5;
        spec.style.detail_density = -0.2;
        let errors = validate_spec(&spec);
        assert!(errors.len() >= 2, "should catch both OOR: {:?}", errors);
    }

    #[test]
    fn serde_round_trip() {
        let spec = classic_runner();
        let json = serde_json::to_string_pretty(&spec).unwrap();
        let parsed: SloopAssetSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, spec.name);
        assert_eq!(parsed.version, spec.version);
        assert_eq!(parsed.dimensions.overall_length, spec.dimensions.overall_length);
        assert_eq!(parsed.hull.profile, spec.hull.profile);
        assert_eq!(parsed.variant.archetype, spec.variant.archetype);
        // Re-validate the round-tripped spec
        let errors = validate_spec(&parsed);
        assert!(errors.is_empty(), "round-tripped spec should be valid: {:?}", errors);
    }

    #[test]
    fn all_archetypes_valid() {
        use crate::spec::SloopVariantArchetype;
        let archetypes = [
            SloopVariantArchetype::ClassicRunner,
            SloopVariantArchetype::Courier,
            SloopVariantArchetype::Patrol,
            SloopVariantArchetype::Smuggler,
            SloopVariantArchetype::Fishing,
            SloopVariantArchetype::MerchantLight,
        ];
        for arch in &archetypes {
            let spec = arch.default_spec();
            let errors = validate_spec(&spec);
            assert!(errors.is_empty(), "{:?} should be valid: {:?}", arch, errors);
        }
    }
}
