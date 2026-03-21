use ship_hull::mesh::MeshGroup;
use ship_schema::spec::{MaterialSpec, MetalTone, SailSpec, SailTone, WoodTone};

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
        MeshGroup::Quarterdeck => MAT_DECK_WOOD,
        MeshGroup::Cabin => MAT_DECK_WOOD,
        MeshGroup::Hatches => MAT_TRIM_METAL,
        MeshGroup::Helm => MAT_DECK_WOOD,
        MeshGroup::MastMain | MeshGroup::Bowsprit | MeshGroup::Boom | MeshGroup::Gaff => MAT_TRIM_METAL,
        MeshGroup::Rigging => MAT_TRIM_METAL,
        MeshGroup::SailMain | MeshGroup::SailJib => MAT_SAIL_CLOTH,
        MeshGroup::Anchor => MAT_TRIM_METAL,
        MeshGroup::Lantern => MAT_TRIM_METAL,
        MeshGroup::Pennant => MAT_SAIL_CLOTH,
    }
}

/// RGBA color (linear, 0..1).
pub type Color4 = [f64; 4];

/// Resolve a material slot name to an actual RGBA color from the spec.
pub fn resolve_material_color(slot: &str, mat: &MaterialSpec, sails: &SailSpec) -> Color4 {
    match slot {
        MAT_HULL_WOOD => wood_tone_color(&mat.hull_wood),
        MAT_DECK_WOOD => wood_tone_color(&mat.deck_wood),
        MAT_SAIL_CLOTH => sail_tone_color(&sails.sail_tone),
        MAT_TRIM_METAL => metal_tone_color(&mat.trim_metal),
        MAT_PAINT_STRIPE => {
            if let Some(ref hex) = mat.stripe_color {
                hex_to_color(hex)
            } else {
                [0.2, 0.2, 0.3, 1.0]
            }
        }
        _ => [0.5, 0.5, 0.5, 1.0],
    }
}

fn wood_tone_color(tone: &WoodTone) -> Color4 {
    match tone {
        WoodTone::LightOak => [0.76, 0.60, 0.42, 1.0],
        WoodTone::GoldenOak => [0.65, 0.45, 0.25, 1.0],
        WoodTone::Walnut => [0.40, 0.26, 0.15, 1.0],
        WoodTone::WeatheredBrown => [0.45, 0.35, 0.25, 1.0],
        WoodTone::PaintedDark => [0.15, 0.12, 0.10, 1.0],
    }
}

fn sail_tone_color(tone: &SailTone) -> Color4 {
    match tone {
        SailTone::Cream => [0.95, 0.92, 0.82, 1.0],
        SailTone::Tan => [0.82, 0.72, 0.55, 1.0],
        SailTone::WeatheredIvory => [0.88, 0.84, 0.75, 1.0],
        SailTone::Gray => [0.65, 0.63, 0.60, 1.0],
    }
}

fn metal_tone_color(tone: &MetalTone) -> Color4 {
    match tone {
        MetalTone::Iron => [0.35, 0.33, 0.30, 1.0],
        MetalTone::Brass => [0.72, 0.58, 0.25, 1.0],
        MetalTone::BlackenedSteel => [0.18, 0.17, 0.16, 1.0],
    }
}

fn hex_to_color(hex: &str) -> Color4 {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return [0.5, 0.5, 0.5, 1.0];
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f64 / 255.0;
    [r, g, b, 1.0]
}
