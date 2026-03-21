use crate::spec::*;

/// Current canonical spec version.
pub const CURRENT_VERSION: u32 = 2;

/// Supported spec versions.
pub const SUPPORTED_VERSIONS: &[u32] = &[1, 2];

/// Migrate a spec to the current version.
/// V1 specs get default values for new v2 fields based on their archetype.
/// Returns the migrated spec with version set to CURRENT_VERSION.
pub fn migrate_to_current(mut spec: SloopAssetSpec) -> SloopAssetSpec {
    if spec.version == 1 {
        spec = migrate_v1_to_v2(spec);
    }
    spec
}

/// Migrate a v1 spec to v2 by filling in new deck/rig/sail fields
/// with archetype-appropriate defaults.
fn migrate_v1_to_v2(mut spec: SloopAssetSpec) -> SloopAssetSpec {
    spec.version = 2;

    // V2 deck fields — archetype-aware defaults
    match spec.variant.archetype {
        SloopVariantArchetype::ClassicRunner => {
            spec.deck.cabin_width_ratio = 0.55;
            spec.deck.cabin_height = 0.9;
            spec.deck.cabin_offset = -0.12;
            spec.deck.hatch_spacing = 1.2;
            spec.deck.rail_thickness = 0.04;
            spec.deck.bulwark_style = BulwarkStyle::OpenRail;
            spec.deck.quarterdeck_length = 2.0;
        }
        SloopVariantArchetype::Courier => {
            spec.deck.cabin_width_ratio = 0.45;
            spec.deck.cabin_height = 0.7;
            spec.deck.cabin_offset = -0.15;
            spec.deck.hatch_spacing = 1.0;
            spec.deck.rail_thickness = 0.03;
            spec.deck.bulwark_style = BulwarkStyle::OpenRail;
            spec.deck.quarterdeck_length = 1.5;
        }
        SloopVariantArchetype::Patrol => {
            spec.deck.cabin_width_ratio = 0.55;
            spec.deck.cabin_height = 1.0;
            spec.deck.cabin_offset = -0.10;
            spec.deck.hatch_spacing = 1.4;
            spec.deck.rail_thickness = 0.06;
            spec.deck.bulwark_style = BulwarkStyle::SolidLow;
            spec.deck.quarterdeck_length = 2.5;
        }
        SloopVariantArchetype::Smuggler => {
            spec.deck.cabin_width_ratio = 0.60;
            spec.deck.cabin_height = 0.8;
            spec.deck.cabin_offset = -0.15;
            spec.deck.hatch_spacing = 1.0;
            spec.deck.rail_thickness = 0.03;
            spec.deck.bulwark_style = BulwarkStyle::SolidLow;
            spec.deck.quarterdeck_length = 1.8;
        }
        SloopVariantArchetype::Fishing => {
            spec.deck.cabin_width_ratio = 0.50;
            spec.deck.cabin_height = 0.8;
            spec.deck.cabin_offset = -0.10;
            spec.deck.hatch_spacing = 1.5;
            spec.deck.rail_thickness = 0.05;
            spec.deck.bulwark_style = BulwarkStyle::SolidMedium;
            spec.deck.quarterdeck_length = 1.5;
        }
        SloopVariantArchetype::MerchantLight => {
            spec.deck.cabin_width_ratio = 0.55;
            spec.deck.cabin_height = 1.0;
            spec.deck.cabin_offset = -0.12;
            spec.deck.hatch_spacing = 1.3;
            spec.deck.rail_thickness = 0.05;
            spec.deck.bulwark_style = BulwarkStyle::SolidLow;
            spec.deck.quarterdeck_length = 2.2;
        }
    }

    // V2 rig fields — consistent across archetypes with minor variation
    spec.rig.mast_diameter = match spec.variant.archetype {
        SloopVariantArchetype::Courier => 0.12,
        SloopVariantArchetype::Fishing => 0.13,
        _ => 0.15,
    };
    spec.rig.boom_height = 1.0;
    spec.rig.gaff_angle = 0.52;
    spec.rig.bowsprit_angle = 0.17;

    // V2 sail field
    spec.sails.camber_style = CamberStyle::LightBillow;

    spec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_spec_migrates_to_v2() {
        // Simulate a v1 spec (serde defaults will fill new fields with defaults)
        let v1_json = serde_json::to_string(&crate::defaults::classic_runner()).unwrap();
        // Replace version to 1 to simulate loading old data
        let v1_json = v1_json.replacen("\"version\":1", "\"version\":1", 1);
        let mut spec: SloopAssetSpec = serde_json::from_str(&v1_json).unwrap();
        spec.version = 1;

        let migrated = migrate_to_current(spec);
        assert_eq!(migrated.version, CURRENT_VERSION);
        assert!(migrated.deck.cabin_width_ratio > 0.0);
        assert!(migrated.deck.cabin_height > 0.0);
        assert!(migrated.rig.mast_diameter > 0.0);
    }

    #[test]
    fn v2_spec_passes_through_unchanged() {
        let mut spec = crate::defaults::classic_runner();
        spec.version = 2;
        spec.deck.cabin_width_ratio = 0.75; // custom value

        let migrated = migrate_to_current(spec.clone());
        assert_eq!(migrated.version, 2);
        assert_eq!(migrated.deck.cabin_width_ratio, 0.75); // preserved
    }

    #[test]
    fn all_archetypes_migrate_cleanly() {
        for arch in SloopVariantArchetype::all() {
            let mut spec = arch.default_spec();
            spec.version = 1;
            let migrated = migrate_to_current(spec);
            assert_eq!(migrated.version, 2);

            // New fields should be populated
            assert!(migrated.deck.cabin_width_ratio > 0.0,
                "{:?}: cabin_width_ratio not set", arch);
            assert!(migrated.deck.rail_thickness > 0.0,
                "{:?}: rail_thickness not set", arch);
            assert!(migrated.rig.mast_diameter > 0.0,
                "{:?}: mast_diameter not set", arch);
        }
    }

    #[test]
    fn v1_json_deserializes_with_defaults() {
        // Minimal v1 JSON (missing all v2 fields)
        let v1_json = r##"{
            "version": 1,
            "kind": "ship",
            "class": "sloop",
            "origin": "procedural",
            "name": "Test",
            "variant": {"archetype": "classic-runner", "presetVersion": 1, "tags": []},
            "dimensions": {"overallLength": 10, "beam": 3, "draftVisual": 1, "freeboard": 1.2, "mastHeight": 8, "bowspritLength": 2},
            "hull": {"profile": "swift", "bowStyle": "raked", "sternStyle": "rounded", "keelDepth": 0.9, "tumblehome": 0.16, "sheer": 0.22, "fullness": 0.42, "rake": 0.18, "quarterdeckHeight": 0.35, "transomWidth": 0.85},
            "deck": {"cabin": true, "cabinLength": 1.8, "hatchCount": 1, "railHeight": 0.45, "wheel": false, "tiller": true, "mastOffset": 0.4},
            "rig": {"plan": "gaff-sloop", "mastCount": 1, "mastRake": 0.08, "boomLength": 3.8, "gaffLength": 2.6, "jibCount": 1, "shroudPairs": 2, "stayThickness": 0.03, "yardCount": 0},
            "sails": {"mainsailFullness": 0.28, "jibFullness": 0.18, "reefed": false, "furledJib": false, "sailTone": "weathered-ivory", "patchiness": 0.08, "edgeWear": 0.1},
            "attachments": {"pennant": "short", "lanterns": "stern-single", "cannons": "none", "figurehead": "none", "anchorVisible": true, "lifeboat": false, "cargoVisible": false, "sternBanner": false},
            "materials": {"hullWood": "golden-oak", "deckWood": "light-oak", "mastWood": "weathered-brown", "trimMetal": "iron", "paintedStripe": true, "stripeColor": "#23364d"},
            "style": {"profile": "portlight-classic", "silhouetteExaggeration": 0.55, "detailDensity": 0.38, "edgeChunkiness": 0.62, "realismBias": 0.35},
            "damage": "clean",
            "renderLaw": {"bowForwardAxis": "+x", "upAxis": "+z", "waterlineZ": 0, "pivot": "midship-waterline"}
        }"##;

        let spec: SloopAssetSpec = serde_json::from_str(v1_json).unwrap();
        assert_eq!(spec.version, 1);
        // New fields should have serde defaults
        assert!(spec.deck.cabin_width_ratio > 0.0, "serde default should provide cabin_width_ratio");
        assert!(spec.rig.mast_diameter > 0.0, "serde default should provide mast_diameter");
    }
}
