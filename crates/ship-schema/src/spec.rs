use serde::{Deserialize, Serialize};

// -- Coordinate primitive --

/// 3D vector in meters. Convention: +X forward, +Z up, ±Y port/starboard.
pub type Vec3 = [f64; 3];

// -- Enums --

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssetOrigin {
    Procedural,
    Imported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShipClass {
    Sloop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WoodTone {
    LightOak,
    GoldenOak,
    Walnut,
    WeatheredBrown,
    PaintedDark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SailTone {
    Cream,
    Tan,
    WeatheredIvory,
    Gray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MetalTone {
    Iron,
    Brass,
    BlackenedSteel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HullProfile {
    Swift,
    Balanced,
    CargoLight,
    NavyLight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SternStyle {
    Simple,
    Rounded,
    Square,
    OrnateLight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BowStyle {
    Needle,
    Raked,
    Rounded,
    Reinforced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SailPlan {
    ForeAndAft,
    GaffSloop,
    CutterLike,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrnamentLevel {
    Plain,
    Light,
    Rich,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DamageState {
    Clean,
    Worn,
    BattleScarred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PennantStyle {
    None,
    Short,
    Navy,
    Merchant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LanternStyle {
    None,
    SternSingle,
    SternDouble,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CannonLayout {
    None,
    LightPortPair,
    LightBroadside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FigureheadStyle {
    None,
    Simple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShipStyleProfile {
    PortlightClassic,
    StorybookLowpoly,
    NavalStylized,
    MerchantStylized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SloopVariantArchetype {
    ClassicRunner,
    Courier,
    Patrol,
    Smuggler,
    Fishing,
    MerchantLight,
}

// -- Spec structs --

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipDimensions {
    /// Overall length in meters.
    pub overall_length: f64,
    /// Maximum beam in meters.
    pub beam: f64,
    /// Visual draft depth in meters (below waterline).
    pub draft_visual: f64,
    /// Freeboard height in meters (waterline to deck).
    pub freeboard: f64,
    /// Main mast height in meters.
    pub mast_height: f64,
    /// Bowsprit length in meters.
    pub bowsprit_length: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HullSpec {
    pub profile: HullProfile,
    pub bow_style: BowStyle,
    pub stern_style: SternStyle,
    /// Keel depth factor (0..1 relative to draft_visual).
    pub keel_depth: f64,
    /// Inward taper of upper hull (0..1).
    pub tumblehome: f64,
    /// Deck rise toward bow/stern (0..1).
    pub sheer: f64,
    /// Body volume / chunkiness (0..1).
    pub fullness: f64,
    /// Fore-aft lean in silhouette (0..1).
    pub rake: f64,
    /// Quarterdeck rise in meters.
    pub quarterdeck_height: f64,
    /// Transom width factor (0..1 relative to beam).
    pub transom_width: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckSpec {
    pub cabin: bool,
    /// Cabin length in meters.
    pub cabin_length: f64,
    pub hatch_count: u8,
    /// Rail height in meters.
    pub rail_height: f64,
    pub wheel: bool,
    pub tiller: bool,
    /// Mast fore/aft offset from hull center (fraction of length, positive = forward).
    pub mast_offset: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RigSpec {
    pub plan: SailPlan,
    pub mast_count: u8,
    /// Mast backward lean (radians-ish, 0..0.3).
    pub mast_rake: f64,
    /// Boom length in meters.
    pub boom_length: f64,
    /// Gaff length in meters.
    pub gaff_length: f64,
    pub jib_count: u8,
    pub shroud_pairs: u8,
    /// Stay rope thickness in meters.
    pub stay_thickness: f64,
    pub yard_count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SailSpec {
    /// Mainsail belly depth (0..1).
    pub mainsail_fullness: f64,
    /// Jib belly depth (0..1).
    pub jib_fullness: f64,
    pub reefed: bool,
    pub furled_jib: bool,
    pub sail_tone: SailTone,
    /// Patch wear factor (0..1).
    pub patchiness: f64,
    /// Edge fraying factor (0..1).
    pub edge_wear: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentSpec {
    pub pennant: PennantStyle,
    pub lanterns: LanternStyle,
    pub cannons: CannonLayout,
    pub figurehead: FigureheadStyle,
    pub anchor_visible: bool,
    pub lifeboat: bool,
    pub cargo_visible: bool,
    pub stern_banner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialSpec {
    pub hull_wood: WoodTone,
    pub deck_wood: WoodTone,
    pub mast_wood: WoodTone,
    pub trim_metal: MetalTone,
    pub painted_stripe: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stripe_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleLaw {
    pub profile: ShipStyleProfile,
    /// How much to exaggerate bow/stern rise, narrowing, sheer (0..1).
    pub silhouette_exaggeration: f64,
    /// Geometry detail budget (0..1).
    pub detail_density: f64,
    /// Mesh chunkiness / hard-edge bias (0..1).
    pub edge_chunkiness: f64,
    /// How close to realistic proportions (0..1).
    pub realism_bias: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLaw {
    /// Forward axis convention.
    pub bow_forward_axis: String,
    /// Up axis convention.
    pub up_axis: String,
    /// Waterline Z position in meters.
    pub waterline_z: f64,
    /// Pivot point convention.
    pub pivot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariantIdentity {
    pub archetype: SloopVariantArchetype,
    pub preset_version: u32,
    pub tags: Vec<String>,
}

/// The canonical sloop asset specification.
/// All geometry, style, and export behavior derives from this spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SloopAssetSpec {
    pub version: u32,
    pub kind: String,
    pub class: ShipClass,
    pub origin: AssetOrigin,
    pub name: String,
    pub variant: VariantIdentity,
    pub dimensions: ShipDimensions,
    pub hull: HullSpec,
    pub deck: DeckSpec,
    pub rig: RigSpec,
    pub sails: SailSpec,
    pub attachments: AttachmentSpec,
    pub materials: MaterialSpec,
    pub style: StyleLaw,
    pub damage: DamageState,
    pub render_law: RenderLaw,
}
