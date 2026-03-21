use ship_hull::mesh::MeshGroup;

/// Material slot names for GLB export.
pub const MAT_HULL_WOOD: &str = "mat_hull_wood";
pub const MAT_DECK_WOOD: &str = "mat_deck_wood";
pub const MAT_SAIL_CLOTH: &str = "mat_sail_cloth";
pub const MAT_TRIM_METAL: &str = "mat_trim_metal";
pub const MAT_PAINT_STRIPE: &str = "mat_paint_stripe";

/// Map a mesh group to its default material slot.
pub fn material_for_group(group: &MeshGroup) -> &'static str {
    match group {
        MeshGroup::Hull => MAT_HULL_WOOD,
        MeshGroup::Deck => MAT_DECK_WOOD,
        MeshGroup::Rail => MAT_DECK_WOOD,
        MeshGroup::Cabin => MAT_DECK_WOOD,
        MeshGroup::MastMain | MeshGroup::Bowsprit | MeshGroup::Boom | MeshGroup::Gaff => MAT_DECK_WOOD,
        MeshGroup::Rigging => MAT_TRIM_METAL,
        MeshGroup::SailMain | MeshGroup::SailJib => MAT_SAIL_CLOTH,
        MeshGroup::Anchor => MAT_TRIM_METAL,
        MeshGroup::Lantern => MAT_TRIM_METAL,
        MeshGroup::Pennant => MAT_SAIL_CLOTH,
    }
}
