use ship_schema::spec::SloopAssetSpec;
use ship_schema::style::StyleEnforcer;

use crate::caps::{bow_cap, stern_cap};
use crate::deck::{extract_deck_rim, generate_deck_surface, DeckRimPoint};
use crate::longitudinal::HullCurves;
use crate::loft::loft_hull;
use crate::mesh::MeshData;
use crate::rail::{generate_quarterdeck, generate_rail};
use crate::rig::{
    compute_rig_anchors, generate_boom, generate_bowsprit, generate_gaff, generate_mast,
    generate_rigging, RigAnchors,
};
use crate::sails::{compute_sail_anchors, generate_jib, generate_mainsail, SailAnchors};
use crate::section::{config_from_style, sample_section, SectionProfile};
use crate::station::{compute_stations, StationLandmarks};
use crate::superstructure::{generate_cabin, generate_hatches, generate_helm};
use crate::validate::{validate_mesh, MeshWarning};

/// Result of full ship generation.
#[derive(Debug)]
pub struct ShipResult {
    /// All mesh groups that make up the ship.
    pub meshes: Vec<MeshData>,
    /// Station landmarks used during generation.
    pub stations: Vec<StationLandmarks>,
    /// Section profiles used during generation.
    pub sections: Vec<SectionProfile>,
    /// Deck rim points.
    pub deck_rim: Vec<DeckRimPoint>,
    /// Rig anchor contract.
    pub rig_anchors: RigAnchors,
    /// Sail anchor contract.
    pub sail_anchors: SailAnchors,
    /// Validation warnings (empty = clean).
    pub warnings: Vec<MeshWarning>,
}

/// Legacy alias — generates hull only.
pub type HullResult = ShipResult;

/// Generate a complete ship from a sloop asset spec.
/// Produces hull + deck + rail + quarterdeck + cabin + hatches + helm + rig + sails.
pub fn generate_ship(spec: &SloopAssetSpec) -> Result<ShipResult, HullError> {
    // Validate spec first
    let spec_errors = ship_schema::validate::validate_spec(spec);
    if !spec_errors.is_empty() {
        return Err(HullError::InvalidSpec(
            spec_errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; ")
        ));
    }

    let enforcer = StyleEnforcer::new(&spec.style);

    // -- Hull --
    let curves = HullCurves::from_spec(&spec.dimensions, &spec.hull, &enforcer)
        .map_err(|e| HullError::CurveError(e.to_string()))?;
    let stations = compute_stations(&curves, spec.dimensions.overall_length);
    let config = config_from_style(
        enforcer.section_resolution(),
        spec.hull.fullness,
        spec.style.edge_chunkiness,
    );
    let sections: Vec<SectionProfile> = stations.iter()
        .map(|s| sample_section(s, &config))
        .collect();

    let hull_mesh = loft_hull(&stations, &sections);
    let bow = bow_cap(stations.last().unwrap(), sections.last().unwrap(), hull_mesh.vertex_count());
    let stern = stern_cap(stations.first().unwrap(), sections.first().unwrap(), 0);

    // -- Deck --
    let deck_rim = extract_deck_rim(&stations);
    let deck_surface = generate_deck_surface(&deck_rim, &spec.deck, &spec.dimensions, &enforcer);
    let rail = generate_rail(&deck_rim, &spec.deck);
    let quarterdeck = generate_quarterdeck(
        &deck_rim, &spec.deck, spec.hull.quarterdeck_height, spec.dimensions.overall_length,
    );

    // -- Superstructure --
    let cabin = generate_cabin(&spec.deck, &stations, spec.dimensions.overall_length);
    let hatches = generate_hatches(&spec.deck, &stations, spec.dimensions.overall_length);
    let helm = generate_helm(
        &spec.deck, &stations, spec.dimensions.overall_length, spec.hull.quarterdeck_height,
    );

    // -- Rig --
    let rig_anchors = compute_rig_anchors(&spec.rig, &spec.deck, &spec.dimensions, &stations);
    let mast = generate_mast(&rig_anchors, &spec.rig);
    let bowsprit = generate_bowsprit(&rig_anchors, &spec.rig);
    let boom = generate_boom(&rig_anchors, &spec.rig);
    let gaff = generate_gaff(&rig_anchors, &spec.rig);
    let rigging = generate_rigging(
        &rig_anchors, &spec.rig, &stations, &spec.dimensions, spec.deck.mast_offset,
    );

    // -- Sails --
    let sail_anchors = compute_sail_anchors(&rig_anchors);
    let mainsail = generate_mainsail(&sail_anchors, &spec.sails);
    let jib = generate_jib(&sail_anchors, &spec.sails);

    // Collect all meshes (skip empty ones)
    let all_meshes = vec![
        hull_mesh, bow, stern,
        deck_surface, rail, quarterdeck,
        cabin, hatches, helm,
        mast, bowsprit, boom, gaff, rigging,
        mainsail, jib,
    ];
    let meshes: Vec<MeshData> = all_meshes.into_iter()
        .filter(|m| m.vertex_count() > 0)
        .collect();

    // Validate
    let mut warnings = Vec::new();
    for mesh in &meshes {
        warnings.extend(validate_mesh(mesh));
    }

    Ok(ShipResult {
        meshes,
        stations,
        sections,
        deck_rim,
        rig_anchors,
        sail_anchors,
        warnings,
    })
}

/// Legacy compatibility: generate hull only.
pub fn generate_hull(spec: &SloopAssetSpec) -> Result<ShipResult, HullError> {
    generate_ship(spec)
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
        let result = generate_ship(&spec).unwrap();
        assert!(result.meshes.len() >= 10, "should have many mesh groups: {}", result.meshes.len());
        assert!(result.warnings.is_empty(), "should have no warnings: {:?}", result.warnings);
    }

    #[test]
    fn generate_all_archetypes_succeeds() {
        for arch in SloopVariantArchetype::all() {
            let spec = arch.default_spec();
            let result = generate_ship(&spec);
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
        let result = generate_ship(&spec);
        assert!(matches!(result, Err(HullError::InvalidSpec(_))));
    }

    #[test]
    fn ship_has_expected_groups() {
        let spec = classic_runner();
        let result = generate_ship(&spec).unwrap();

        let group_names: Vec<&str> = result.meshes.iter()
            .map(|m| m.group.node_name())
            .collect();

        assert!(group_names.contains(&"Hull"), "missing Hull");
        assert!(group_names.contains(&"Deck"), "missing Deck");
        assert!(group_names.contains(&"Rail"), "missing Rail");
        assert!(group_names.contains(&"Cabin"), "missing Cabin");
        assert!(group_names.contains(&"Mast_Main"), "missing Mast_Main");
        assert!(group_names.contains(&"Bowsprit"), "missing Bowsprit");
        assert!(group_names.contains(&"Sail_Main"), "missing Sail_Main");
        assert!(group_names.contains(&"Sail_Jib_01"), "missing Sail_Jib_01");
    }

    #[test]
    fn total_geometry_is_substantial() {
        let spec = classic_runner();
        let result = generate_ship(&spec).unwrap();

        let total_verts: usize = result.meshes.iter().map(|m| m.vertex_count()).sum();
        let total_tris: usize = result.meshes.iter().map(|m| m.triangle_count()).sum();

        assert!(total_verts > 500, "total vertices: {total_verts}");
        assert!(total_tris > 300, "total triangles: {total_tris}");
    }

    #[test]
    fn rig_anchors_available() {
        let spec = classic_runner();
        let result = generate_ship(&spec).unwrap();
        assert!(result.rig_anchors.mast_top[2] > 5.0, "mast should be tall");
        assert!(result.sail_anchors.jib_tack[0] > 3.0, "jib should be forward");
    }
}
