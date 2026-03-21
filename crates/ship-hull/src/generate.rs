use ship_schema::spec::SloopAssetSpec;
use ship_schema::style::StyleEnforcer;

use crate::caps::{bow_cap, stern_cap};
use crate::longitudinal::HullCurves;
use crate::loft::loft_hull;
use crate::mesh::MeshData;
use crate::section::{config_from_style, sample_section, SectionProfile};
use crate::station::{compute_stations, StationLandmarks};
use crate::validate::{validate_mesh, MeshWarning};

/// Result of hull generation.
#[derive(Debug)]
pub struct HullResult {
    /// The hull body mesh (lofted sides).
    pub hull_mesh: MeshData,
    /// The bow cap mesh.
    pub bow_cap: MeshData,
    /// The stern cap mesh.
    pub stern_cap: MeshData,
    /// Station landmarks used during generation.
    pub stations: Vec<StationLandmarks>,
    /// Section profiles used during generation.
    pub sections: Vec<SectionProfile>,
    /// Validation warnings (empty = clean).
    pub warnings: Vec<MeshWarning>,
}

/// Generate a complete hull from a sloop asset spec.
pub fn generate_hull(spec: &SloopAssetSpec) -> Result<HullResult, HullError> {
    // Validate spec first
    let spec_errors = ship_schema::validate::validate_spec(spec);
    if !spec_errors.is_empty() {
        return Err(HullError::InvalidSpec(
            spec_errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; ")
        ));
    }

    let enforcer = StyleEnforcer::new(&spec.style);

    // Build longitudinal curves
    let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer)
        .map_err(|e| HullError::CurveError(e.to_string()))?;

    // Compute stations
    let stations = compute_stations(&curves, spec.dimensions.overall_length);

    // Build section config from style
    let config = config_from_style(
        enforcer.section_resolution(),
        spec.hull.fullness,
        spec.style.edge_chunkiness,
    );

    // Sample sections at each station
    let sections: Vec<SectionProfile> = stations.iter()
        .map(|s| sample_section(s, &config))
        .collect();

    // Loft the hull body
    let hull_mesh = loft_hull(&stations, &sections);

    // Generate caps
    let bow = bow_cap(
        stations.last().unwrap(),
        sections.last().unwrap(),
        hull_mesh.vertex_count(),
    );
    let stern = stern_cap(
        stations.first().unwrap(),
        sections.first().unwrap(),
        hull_mesh.vertex_count() + bow.vertex_count(),
    );

    // Validate all meshes
    let mut warnings = Vec::new();
    warnings.extend(validate_mesh(&hull_mesh));
    warnings.extend(validate_mesh(&bow));
    warnings.extend(validate_mesh(&stern));

    Ok(HullResult {
        hull_mesh,
        bow_cap: bow,
        stern_cap: stern,
        stations,
        sections,
        warnings,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum HullError {
    #[error("invalid spec: {0}")]
    InvalidSpec(String),
    #[error("curve generation failed: {0}")]
    CurveError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_schema::defaults::classic_runner;
    use ship_schema::spec::SloopVariantArchetype;

    #[test]
    fn generate_classic_runner_succeeds() {
        let spec = classic_runner();
        let result = generate_hull(&spec).unwrap();

        assert!(result.hull_mesh.vertex_count() > 100);
        assert!(result.hull_mesh.triangle_count() > 100);
        assert!(result.bow_cap.vertex_count() > 3);
        assert!(result.stern_cap.vertex_count() > 3);
        assert!(result.warnings.is_empty(), "should have no warnings: {:?}", result.warnings);
    }

    #[test]
    fn generate_all_archetypes_succeeds() {
        for arch in SloopVariantArchetype::all() {
            let spec = arch.default_spec();
            let result = generate_hull(&spec);
            assert!(result.is_ok(), "{:?} failed: {:?}", arch, result.err());

            let result = result.unwrap();
            assert!(result.warnings.is_empty(),
                "{:?} has warnings: {:?}", arch, result.warnings);
        }
    }

    #[test]
    fn invalid_spec_rejected() {
        let mut spec = classic_runner();
        spec.dimensions.overall_length = -5.0;
        let result = generate_hull(&spec);
        assert!(matches!(result, Err(HullError::InvalidSpec(_))));
    }

    #[test]
    fn hull_result_has_nine_stations() {
        let spec = classic_runner();
        let result = generate_hull(&spec).unwrap();
        assert_eq!(result.stations.len(), 9);
        assert_eq!(result.sections.len(), 9);
    }

    #[test]
    fn total_geometry_is_substantial() {
        let spec = classic_runner();
        let result = generate_hull(&spec).unwrap();

        let total_verts = result.hull_mesh.vertex_count()
            + result.bow_cap.vertex_count()
            + result.stern_cap.vertex_count();
        let total_tris = result.hull_mesh.triangle_count()
            + result.bow_cap.triangle_count()
            + result.stern_cap.triangle_count();

        assert!(total_verts > 150, "total vertices: {total_verts}");
        assert!(total_tris > 150, "total triangles: {total_tris}");
    }
}
