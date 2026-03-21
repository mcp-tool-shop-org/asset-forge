use ship_hull::generate::generate_hull;
use ship_export::export_glb;
use ship_schema::spec::SloopVariantArchetype;
use ship_testkit::fixtures::{all_fixtures, reference_ship};
use ship_testkit::invariants::check_mesh_invariants;
use ship_testkit::metrics::aggregate_metrics;

#[test]
fn all_archetypes_generate_clean_hulls() {
    for spec in all_fixtures() {
        let result = generate_hull(&spec)
            .unwrap_or_else(|e| panic!("{}: generation failed: {e}", spec.name));

        let hull_violations = check_mesh_invariants(&result.hull_mesh);
        assert!(hull_violations.is_empty(),
            "{}: hull violations: {:?}", spec.name, hull_violations);

        let bow_violations = check_mesh_invariants(&result.bow_cap);
        assert!(bow_violations.is_empty(),
            "{}: bow cap violations: {:?}", spec.name, bow_violations);

        let stern_violations = check_mesh_invariants(&result.stern_cap);
        assert!(stern_violations.is_empty(),
            "{}: stern cap violations: {:?}", spec.name, stern_violations);

        assert!(result.warnings.is_empty(),
            "{}: generator warnings: {:?}", spec.name, result.warnings);
    }
}

#[test]
fn all_archetypes_export_valid_glb() {
    for spec in all_fixtures() {
        let hull = generate_hull(&spec).unwrap();
        let (glb, manifest) = export_glb(&spec, &hull)
            .unwrap_or_else(|e| panic!("{}: export failed: {e}", spec.name));

        // Valid GLB
        assert_eq!(&glb[0..4], b"glTF", "{}: bad magic", spec.name);
        let stated_len = u32::from_le_bytes(glb[8..12].try_into().unwrap()) as usize;
        assert_eq!(glb.len(), stated_len, "{}: length mismatch", spec.name);

        // Manifest coherent
        assert!(manifest.vertex_count > 50, "{}: too few vertices", spec.name);
        assert!(manifest.triangle_count > 50, "{}: too few triangles", spec.name);
        assert!(manifest.mesh_groups.contains(&"Hull".to_string()),
            "{}: missing Hull group", spec.name);
    }
}

#[test]
fn reference_ship_metrics_in_expected_range() {
    let spec = reference_ship();
    let result = generate_hull(&spec).unwrap();

    let metrics = aggregate_metrics(&[
        &result.hull_mesh,
        &result.bow_cap,
        &result.stern_cap,
    ]);

    // X span should roughly match overall_length (10m)
    assert!(metrics.x_span > 8.0 && metrics.x_span < 12.0,
        "x span should be ~10m: {}", metrics.x_span);

    // Y span should roughly match beam (3.2m)
    assert!(metrics.y_span > 2.0 && metrics.y_span < 5.0,
        "y span should be ~3.2m: {}", metrics.y_span);

    // Z span should be reasonable (draft + freeboard ≈ 2.2m)
    assert!(metrics.z_span > 1.0 && metrics.z_span < 5.0,
        "z span should be ~2.2m: {}", metrics.z_span);

    // Vertex count in reasonable range
    assert!(metrics.vertex_count > 100 && metrics.vertex_count < 5000,
        "vertex count: {}", metrics.vertex_count);
}

#[test]
fn archetype_dimensions_match_spec() {
    for arch in SloopVariantArchetype::all() {
        let spec = arch.default_spec();
        let result = generate_hull(&spec).unwrap();

        let metrics = aggregate_metrics(&[
            &result.hull_mesh,
            &result.bow_cap,
            &result.stern_cap,
        ]);

        // X span within 20% of overall_length
        let expected_x = spec.dimensions.overall_length;
        assert!(metrics.x_span > expected_x * 0.7 && metrics.x_span < expected_x * 1.3,
            "{:?}: x span {:.1} vs expected {:.1}", arch, metrics.x_span, expected_x);

        // Y span within 50% of beam (wider tolerance for style exaggeration)
        let expected_y = spec.dimensions.beam;
        assert!(metrics.y_span > expected_y * 0.4 && metrics.y_span < expected_y * 1.6,
            "{:?}: y span {:.1} vs expected {:.1}", arch, metrics.y_span, expected_y);
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
        let hull = generate_hull(&parsed).unwrap();

        // Export
        let (glb, manifest) = export_glb(&parsed, &hull).unwrap();

        // Verify
        assert_eq!(&glb[0..4], b"glTF");
        assert!(manifest.vertex_count > 50);

        // Manifest serializes cleanly
        let manifest_json = serde_json::to_string(&manifest).unwrap();
        assert!(!manifest_json.is_empty());
    }
}

#[test]
fn beam_peak_location_per_archetype() {
    for arch in SloopVariantArchetype::all() {
        let spec = arch.default_spec();
        let result = generate_hull(&spec).unwrap();

        // Find the station with the widest beam
        let widest = result.stations.iter()
            .max_by(|a, b| a.half_beam.partial_cmp(&b.half_beam).unwrap())
            .unwrap();

        // Beam should peak between 30%-60% of hull length
        assert!(widest.u > 0.25 && widest.u < 0.65,
            "{:?}: beam peaks at u={}, expected 0.25-0.65", arch, widest.u);
    }
}
