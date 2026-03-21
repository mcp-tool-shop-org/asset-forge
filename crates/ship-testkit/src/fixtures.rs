use ship_schema::defaults;
use ship_schema::spec::{SloopAssetSpec, SloopVariantArchetype};

/// All canonical fixture specs for testing.
pub fn all_fixtures() -> Vec<SloopAssetSpec> {
    SloopVariantArchetype::all()
        .iter()
        .map(|a| a.default_spec())
        .collect()
}

/// Get a specific fixture by archetype.
pub fn fixture(archetype: SloopVariantArchetype) -> SloopAssetSpec {
    archetype.default_spec()
}

/// The canonical reference ship for baseline tests.
pub fn reference_ship() -> SloopAssetSpec {
    defaults::classic_runner()
}
