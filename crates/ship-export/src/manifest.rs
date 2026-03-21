use serde::{Deserialize, Serialize};

/// Asset manifest sidecar — metadata about the exported asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetManifest {
    pub version: u32,
    pub name: String,
    pub class: String,
    pub archetype: String,
    pub origin: String,
    pub glb_file: String,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub mesh_groups: Vec<String>,
    pub bounding_box: Option<BoundingBoxInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundingBoxInfo {
    pub min: [f64; 3],
    pub max: [f64; 3],
}
