/// Named mesh group within a generated hull.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MeshGroup {
    Hull,
    Deck,
    Rail,
    Cabin,
    MastMain,
    Bowsprit,
    Boom,
    Gaff,
    Rigging,
    SailMain,
    SailJib,
    Anchor,
    Lantern,
    Pennant,
}

impl MeshGroup {
    /// GLB node name for this group.
    pub fn node_name(&self) -> &'static str {
        match self {
            Self::Hull => "Hull",
            Self::Deck => "Deck",
            Self::Rail => "Rail",
            Self::Cabin => "Cabin",
            Self::MastMain => "Mast_Main",
            Self::Bowsprit => "Bowsprit",
            Self::Boom => "Boom",
            Self::Gaff => "Gaff",
            Self::Rigging => "Rigging",
            Self::SailMain => "Sail_Main",
            Self::SailJib => "Sail_Jib_01",
            Self::Anchor => "Anchor_01",
            Self::Lantern => "Lantern_01",
            Self::Pennant => "Pennant",
        }
    }
}

/// Raw mesh data for a single named group.
#[derive(Debug, Clone)]
pub struct MeshData {
    /// Group this mesh belongs to.
    pub group: MeshGroup,
    /// Vertex positions [x, y, z] in meters.
    pub positions: Vec<[f64; 3]>,
    /// Vertex normals [nx, ny, nz].
    pub normals: Vec<[f64; 3]>,
    /// Triangle indices (every 3 = one triangle).
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn new(group: MeshGroup) -> Self {
        Self {
            group,
            positions: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Check if any positions contain NaN or infinity.
    pub fn has_invalid_positions(&self) -> bool {
        self.positions.iter().any(|p| {
            p.iter().any(|v| !v.is_finite())
        })
    }

    /// Check if any normals contain NaN or infinity.
    pub fn has_invalid_normals(&self) -> bool {
        self.normals.iter().any(|n| {
            n.iter().any(|v| !v.is_finite())
        })
    }

    /// Compute axis-aligned bounding box. Returns (min, max) or None if empty.
    pub fn bounding_box(&self) -> Option<([f64; 3], [f64; 3])> {
        if self.positions.is_empty() {
            return None;
        }
        let mut min = [f64::MAX; 3];
        let mut max = [f64::MIN; 3];
        for p in &self.positions {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }
        Some((min, max))
    }
}
