use crate::spec::*;

impl SloopVariantArchetype {
    /// Returns the canonical default spec for this archetype.
    pub fn default_spec(&self) -> SloopAssetSpec {
        match self {
            Self::ClassicRunner => classic_runner(),
            Self::Courier => courier(),
            Self::Patrol => patrol(),
            Self::Smuggler => smuggler(),
            Self::Fishing => fishing(),
            Self::MerchantLight => merchant_light(),
        }
    }
}

/// Canonical "Classic Runner" sloop — the reference ship.
pub fn classic_runner() -> SloopAssetSpec {
    SloopAssetSpec {
        version: 2,
        kind: "ship".into(),
        class: ShipClass::Sloop,
        origin: AssetOrigin::Procedural,
        name: "Sloop / Classic Runner".into(),
        variant: VariantIdentity {
            archetype: SloopVariantArchetype::ClassicRunner,
            preset_version: 1,
            tags: vec![],
        },
        dimensions: ShipDimensions {
            overall_length: 10.0,
            beam: 3.2,
            draft_visual: 1.0,
            freeboard: 1.2,
            mast_height: 8.6,
            bowsprit_length: 2.1,
        },
        hull: HullSpec {
            profile: HullProfile::Swift,
            bow_style: BowStyle::Raked,
            stern_style: SternStyle::Rounded,
            keel_depth: 0.9,
            tumblehome: 0.16,
            sheer: 0.22,
            fullness: 0.42,
            rake: 0.18,
            quarterdeck_height: 0.35,
            transom_width: 0.85,
        },
        deck: DeckSpec {
            cabin: true,
            cabin_length: 1.8,
            cabin_width_ratio: 0.55,
            cabin_height: 0.9,
            cabin_offset: -0.12,
            hatch_count: 1,
            hatch_spacing: 1.2,
            rail_height: 0.45,
            rail_thickness: 0.04,
            bulwark_style: BulwarkStyle::OpenRail,
            wheel: false,
            tiller: true,
            mast_offset: 0.4,
            quarterdeck_length: 2.0,
        },
        rig: RigSpec {
            plan: SailPlan::GaffSloop,
            mast_count: 1,
            mast_rake: 0.08,
            mast_diameter: 0.15,
            boom_length: 3.8,
            boom_height: 1.0,
            gaff_length: 2.6,
            gaff_angle: 0.52,
            bowsprit_angle: 0.17,
            jib_count: 1,
            shroud_pairs: 2,
            stay_thickness: 0.03,
            yard_count: 0,
        },
        sails: SailSpec {
            mainsail_fullness: 0.28,
            jib_fullness: 0.18,
            reefed: false,
            furled_jib: false,
            sail_tone: SailTone::WeatheredIvory,
            patchiness: 0.08,
            edge_wear: 0.1,
            camber_style: CamberStyle::LightBillow,
        },
        attachments: AttachmentSpec {
            pennant: PennantStyle::Short,
            lanterns: LanternStyle::SternSingle,
            cannons: CannonLayout::None,
            figurehead: FigureheadStyle::None,
            anchor_visible: true,
            lifeboat: false,
            cargo_visible: false,
            stern_banner: false,
        },
        materials: MaterialSpec {
            hull_wood: WoodTone::GoldenOak,
            deck_wood: WoodTone::LightOak,
            mast_wood: WoodTone::WeatheredBrown,
            trim_metal: MetalTone::Iron,
            painted_stripe: true,
            stripe_color: Some("#23364d".into()),
        },
        style: StyleLaw {
            profile: ShipStyleProfile::PortlightClassic,
            silhouette_exaggeration: 0.55,
            detail_density: 0.38,
            edge_chunkiness: 0.62,
            realism_bias: 0.35,
        },
        damage: DamageState::Clean,
        render_law: RenderLaw {
            bow_forward_axis: "+x".into(),
            up_axis: "+z".into(),
            waterline_z: 0.0,
            pivot: "midship-waterline".into(),
        },
    }
}

/// Fast courier — longer, narrower, less attachment clutter.
pub fn courier() -> SloopAssetSpec {
    let mut spec = classic_runner();
    spec.name = "Sloop / Courier".into();
    spec.variant.archetype = SloopVariantArchetype::Courier;
    spec.dimensions.overall_length = 11.0;
    spec.dimensions.beam = 2.8;
    spec.hull.profile = HullProfile::Swift;
    spec.hull.bow_style = BowStyle::Needle;
    spec.hull.fullness = 0.32;
    spec.hull.rake = 0.22;
    // V2 deck
    spec.deck.cabin_width_ratio = 0.45;
    spec.deck.cabin_height = 0.7;
    spec.deck.cabin_offset = -0.15;
    spec.deck.rail_thickness = 0.03;
    spec.deck.quarterdeck_length = 1.5;
    // V2 rig
    spec.rig.mast_diameter = 0.12;
    spec.attachments.pennant = PennantStyle::Navy;
    spec.attachments.lanterns = LanternStyle::None;
    spec.attachments.anchor_visible = false;
    spec.materials.hull_wood = WoodTone::Walnut;
    spec
}

/// Patrol sloop — broader, sturdier, light armament.
pub fn patrol() -> SloopAssetSpec {
    let mut spec = classic_runner();
    spec.name = "Sloop / Patrol".into();
    spec.variant.archetype = SloopVariantArchetype::Patrol;
    spec.dimensions.beam = 3.6;
    spec.hull.profile = HullProfile::NavyLight;
    spec.hull.bow_style = BowStyle::Reinforced;
    spec.hull.fullness = 0.55;
    spec.hull.transom_width = 0.92;
    // V2 deck
    spec.deck.cabin_height = 1.0;
    spec.deck.rail_thickness = 0.06;
    spec.deck.bulwark_style = BulwarkStyle::SolidLow;
    spec.deck.quarterdeck_length = 2.5;
    spec.attachments.cannons = CannonLayout::LightPortPair;
    spec.attachments.pennant = PennantStyle::Navy;
    spec.materials.hull_wood = WoodTone::PaintedDark;
    spec.materials.painted_stripe = true;
    spec.materials.stripe_color = Some("#8b0000".into());
    spec
}

/// Smuggler — low profile, dark wood, minimal ornament.
pub fn smuggler() -> SloopAssetSpec {
    let mut spec = classic_runner();
    spec.name = "Sloop / Smuggler".into();
    spec.variant.archetype = SloopVariantArchetype::Smuggler;
    spec.dimensions.overall_length = 10.5;
    spec.dimensions.beam = 3.0;
    spec.hull.profile = HullProfile::Swift;
    spec.hull.bow_style = BowStyle::Raked;
    spec.hull.fullness = 0.38;
    spec.hull.rake = 0.20;
    spec.hull.sheer = 0.15;
    spec.deck.cabin = true;
    spec.deck.cabin_length = 2.2;
    // V2 deck
    spec.deck.cabin_width_ratio = 0.60;
    spec.deck.cabin_height = 0.8;
    spec.deck.cabin_offset = -0.15;
    spec.deck.rail_thickness = 0.03;
    spec.deck.bulwark_style = BulwarkStyle::SolidLow;
    spec.deck.quarterdeck_length = 1.8;
    spec.attachments = AttachmentSpec {
        pennant: PennantStyle::None,
        lanterns: LanternStyle::None,
        cannons: CannonLayout::None,
        figurehead: FigureheadStyle::None,
        anchor_visible: false,
        lifeboat: false,
        cargo_visible: true,
        stern_banner: false,
    };
    spec.materials.hull_wood = WoodTone::PaintedDark;
    spec.materials.painted_stripe = false;
    spec.materials.stripe_color = None;
    spec.style.silhouette_exaggeration = 0.40;
    spec
}

/// Fishing sloop — wide, sturdy, functional.
pub fn fishing() -> SloopAssetSpec {
    let mut spec = classic_runner();
    spec.name = "Sloop / Fishing".into();
    spec.variant.archetype = SloopVariantArchetype::Fishing;
    spec.dimensions.overall_length = 8.5;
    spec.dimensions.beam = 3.4;
    spec.hull.profile = HullProfile::Balanced;
    spec.hull.bow_style = BowStyle::Rounded;
    spec.hull.stern_style = SternStyle::Square;
    spec.hull.fullness = 0.60;
    spec.hull.sheer = 0.18;
    spec.hull.transom_width = 0.95;
    spec.deck.cabin = false;
    spec.deck.cabin_length = 0.0;
    spec.deck.hatch_count = 2;
    spec.deck.tiller = true;
    // V2 deck
    spec.deck.cabin_width_ratio = 0.50;
    spec.deck.cabin_height = 0.8;
    spec.deck.hatch_spacing = 1.5;
    spec.deck.rail_thickness = 0.05;
    spec.deck.bulwark_style = BulwarkStyle::SolidMedium;
    spec.deck.quarterdeck_length = 1.5;
    // V2 rig
    spec.rig.mast_diameter = 0.13;
    spec.rig.plan = SailPlan::ForeAndAft;
    spec.attachments = AttachmentSpec {
        pennant: PennantStyle::None,
        lanterns: LanternStyle::SternSingle,
        cannons: CannonLayout::None,
        figurehead: FigureheadStyle::None,
        anchor_visible: true,
        lifeboat: false,
        cargo_visible: true,
        stern_banner: false,
    };
    spec.materials.hull_wood = WoodTone::WeatheredBrown;
    spec.materials.deck_wood = WoodTone::WeatheredBrown;
    spec.style.detail_density = 0.28;
    spec.style.edge_chunkiness = 0.70;
    spec.damage = DamageState::Worn;
    spec
}

/// Light merchant — cargo-friendly, practical, visible trade goods.
pub fn merchant_light() -> SloopAssetSpec {
    let mut spec = classic_runner();
    spec.name = "Sloop / Merchant Light".into();
    spec.variant.archetype = SloopVariantArchetype::MerchantLight;
    spec.dimensions.overall_length = 10.5;
    spec.dimensions.beam = 3.8;
    spec.hull.profile = HullProfile::CargoLight;
    spec.hull.bow_style = BowStyle::Rounded;
    spec.hull.stern_style = SternStyle::Square;
    spec.hull.fullness = 0.62;
    spec.hull.sheer = 0.20;
    spec.hull.transom_width = 0.95;
    spec.deck.cabin = true;
    spec.deck.cabin_length = 2.0;
    spec.deck.hatch_count = 2;
    // V2 deck
    spec.deck.cabin_height = 1.0;
    spec.deck.hatch_spacing = 1.3;
    spec.deck.rail_thickness = 0.05;
    spec.deck.bulwark_style = BulwarkStyle::SolidLow;
    spec.deck.quarterdeck_length = 2.2;
    spec.rig.plan = SailPlan::GaffSloop;
    spec.attachments.cargo_visible = true;
    spec.attachments.pennant = PennantStyle::Merchant;
    spec.materials.hull_wood = WoodTone::GoldenOak;
    spec.style.silhouette_exaggeration = 0.45;
    spec.style.realism_bias = 0.45;
    spec
}
