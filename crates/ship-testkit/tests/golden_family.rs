use ship_hull::generate::{generate_ship, generate_hull};
use ship_export::export_glb;
use ship_schema::spec::SloopVariantArchetype;
use ship_testkit::fixtures::{all_fixtures, reference_ship};
use ship_testkit::invariants::check_mesh_invariants;
use ship_testkit::metrics::aggregate_metrics;

#[test]
fn all_archetypes_generate_clean_ships() {
    for spec in all_fixtures() {
        let result = generate_ship(&spec)
            .unwrap_or_else(|e| panic!("{}: generation failed: {e}", spec.name));

        for mesh in &result.meshes {
            let violations = check_mesh_invariants(mesh);
            assert!(violations.is_empty(),
                "{}: {} violations: {:?}", spec.name, mesh.group.node_name(), violations);
        }

        assert!(result.warnings.is_empty(),
            "{}: generator warnings: {:?}", spec.name, result.warnings);
    }
}

#[test]
fn all_archetypes_export_valid_glb() {
    for spec in all_fixtures() {
        let result = generate_ship(&spec).unwrap();
        let (glb, manifest) = export_glb(&spec, &result)
            .unwrap_or_else(|e| panic!("{}: export failed: {e}", spec.name));

        // Valid GLB
        assert_eq!(&glb[0..4], b"glTF", "{}: bad magic", spec.name);
        let stated_len = u32::from_le_bytes(glb[8..12].try_into().unwrap()) as usize;
        assert_eq!(glb.len(), stated_len, "{}: length mismatch", spec.name);

        // Manifest coherent
        assert!(manifest.vertex_count > 200, "{}: too few vertices: {}", spec.name, manifest.vertex_count);
        assert!(manifest.triangle_count > 150, "{}: too few triangles: {}", spec.name, manifest.triangle_count);
        assert!(manifest.mesh_groups.contains(&"Hull".to_string()),
            "{}: missing Hull group", spec.name);
        assert!(manifest.mesh_groups.contains(&"Deck".to_string()),
            "{}: missing Deck group", spec.name);
        assert!(manifest.mesh_groups.contains(&"Mast_Main".to_string()),
            "{}: missing Mast_Main group", spec.name);
        assert!(manifest.mesh_groups.contains(&"Sail_Main".to_string()),
            "{}: missing Sail_Main group", spec.name);
    }
}

#[test]
fn reference_ship_metrics_in_expected_range() {
    let spec = reference_ship();
    let result = generate_ship(&spec).unwrap();

    let mesh_refs: Vec<&_> = result.meshes.iter().collect();
    let metrics = aggregate_metrics(&mesh_refs);

    // X span should roughly match overall_length (10m) plus bowsprit
    assert!(metrics.x_span > 8.0 && metrics.x_span < 16.0,
        "x span should be ~10-14m (hull + bowsprit): {}", metrics.x_span);

    // Y span should roughly match beam (3.2m)
    assert!(metrics.y_span > 2.0 && metrics.y_span < 5.0,
        "y span should be ~3.2m: {}", metrics.y_span);

    // Z span should include mast height
    assert!(metrics.z_span > 5.0,
        "z span should include mast (~8.6m): {}", metrics.z_span);

    // Vertex count should be substantial with all parts
    assert!(metrics.vertex_count > 500,
        "full ship vertex count: {}", metrics.vertex_count);
}

#[test]
fn archetype_dimensions_match_spec() {
    for arch in SloopVariantArchetype::all() {
        let spec = arch.default_spec();
        let result = generate_ship(&spec).unwrap();

        let mesh_refs: Vec<&_> = result.meshes.iter().collect();
        let metrics = aggregate_metrics(&mesh_refs);

        // X span should cover hull + bowsprit
        let expected_x = spec.dimensions.overall_length + spec.dimensions.bowsprit_length;
        assert!(metrics.x_span > expected_x * 0.6 && metrics.x_span < expected_x * 1.5,
            "{:?}: x span {:.1} vs expected ~{:.1}", arch, metrics.x_span, expected_x);

        // Z span should include mast height
        assert!(metrics.z_span > spec.dimensions.mast_height * 0.5,
            "{:?}: z span {:.1} should include mast {:.1}", arch, metrics.z_span, spec.dimensions.mast_height);
    }
}

#[test]
fn spec_to_glb_roundtrip_for_all_archetypes() {
    for arch in SloopVariantArchetype::all() {
        let spec = arch.default_spec();

        // Serialize spec to JSON
        let json = serde_json::to_string(&spec).unwrap();
        let parsed: ship_schema::SloopAssetSpec = serde_json::from_str(&json).unwrap();

        // Generate from parsed spec
        let result = generate_ship(&parsed).unwrap();

        // Export
        let (glb, manifest) = export_glb(&parsed, &result).unwrap();

        // Verify
        assert_eq!(&glb[0..4], b"glTF");
        assert!(manifest.vertex_count > 200);
        assert!(manifest.mesh_groups.len() >= 10,
            "{:?}: expected >= 10 mesh groups, got {}", arch, manifest.mesh_groups.len());

        // Manifest serializes cleanly
        let manifest_json = serde_json::to_string(&manifest).unwrap();
        assert!(!manifest_json.is_empty());
    }
}

#[test]
fn beam_peak_location_per_archetype() {
    for arch in SloopVariantArchetype::all() {
        let spec = arch.default_spec();
        let result = generate_ship(&spec).unwrap();

        let widest = result.stations.iter()
            .max_by(|a, b| a.half_beam.partial_cmp(&b.half_beam).unwrap())
            .unwrap();

        assert!(widest.u > 0.25 && widest.u < 0.65,
            "{:?}: beam peaks at u={}, expected 0.25-0.65", arch, widest.u);
    }
}

#[test]
fn full_ship_has_named_groups() {
    let spec = reference_ship();
    let result = generate_ship(&spec).unwrap();

    let groups: Vec<&str> = result.meshes.iter()
        .map(|m| m.group.node_name())
        .collect();

    let expected = ["Hull", "Deck", "Rail", "Quarterdeck", "Cabin",
                    "Mast_Main", "Bowsprit", "Boom", "Gaff", "Rigging",
                    "Sail_Main", "Sail_Jib_01"];

    for name in &expected {
        assert!(groups.contains(name),
            "missing expected group: {name}. Present: {:?}", groups);
    }
}

#[test]
fn rig_anchors_contract_valid() {
    for arch in SloopVariantArchetype::all() {
        let spec = arch.default_spec();
        let result = generate_ship(&spec).unwrap();

        let a = &result.rig_anchors;

        // Mast on centerline
        assert!((a.mast_base[1]).abs() < 1e-10, "{:?}: mast off centerline", arch);

        // Mast top above base
        assert!(a.mast_top[2] > a.mast_base[2] + 3.0,
            "{:?}: mast too short", arch);

        // Bowsprit forward
        assert!(a.bowsprit_tip[0] > a.bowsprit_base[0],
            "{:?}: bowsprit not forward", arch);

        // Boom aft
        assert!(a.boom_tip[0] < a.boom_pivot[0],
            "{:?}: boom not aft", arch);

        // Gaff upward
        assert!(a.gaff_peak[2] > a.gaff_throat[2],
            "{:?}: gaff not upward", arch);
    }
}
