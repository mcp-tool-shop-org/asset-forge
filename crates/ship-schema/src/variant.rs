use crate::spec::{SloopVariantArchetype, VariantIdentity};

impl VariantIdentity {
    /// Create a new variant identity with the given archetype.
    pub fn new(archetype: SloopVariantArchetype) -> Self {
        Self {
            archetype,
            preset_version: 1,
            tags: vec![],
        }
    }

    /// Add a tag to this variant identity.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

impl SloopVariantArchetype {
    /// Human-readable display name for this archetype.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ClassicRunner => "Classic Runner",
            Self::Courier => "Courier",
            Self::Patrol => "Patrol",
            Self::Smuggler => "Smuggler",
            Self::Fishing => "Fishing",
            Self::MerchantLight => "Merchant Light",
        }
    }

    /// All archetype variants.
    pub fn all() -> &'static [Self] {
        &[
            Self::ClassicRunner,
            Self::Courier,
            Self::Patrol,
            Self::Smuggler,
            Self::Fishing,
            Self::MerchantLight,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_archetypes_have_display_names() {
        for arch in SloopVariantArchetype::all() {
            let name = arch.display_name();
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn variant_identity_tags() {
        let vi = VariantIdentity::new(SloopVariantArchetype::Smuggler)
            .with_tag("dark")
            .with_tag("low-profile");
        assert_eq!(vi.tags.len(), 2);
        assert_eq!(vi.archetype, SloopVariantArchetype::Smuggler);
    }
}
