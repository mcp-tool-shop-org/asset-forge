use ship_hull::generate::HullResult;
use ship_hull::mesh::MeshData;
use ship_schema::spec::SloopAssetSpec;

use crate::groups::{material_for_group, resolve_material_color};
use crate::manifest::{AssetManifest, BoundingBoxInfo};

/// Export a hull result as a binary GLB file.
/// Returns the GLB bytes and a manifest.
pub fn export_glb(
    spec: &SloopAssetSpec,
    result: &HullResult,
) -> Result<(Vec<u8>, AssetManifest), ExportError> {
    let meshes = collect_meshes(result);

    if meshes.is_empty() {
        return Err(ExportError::NoMeshes);
    }

    let glb_bytes = build_glb(&meshes, spec)?;

    let mut total_verts = 0;
    let mut total_tris = 0;
    let mut group_names = Vec::new();
    let mut global_min = [f64::MAX; 3];
    let mut global_max = [f64::MIN; 3];

    for m in &meshes {
        total_verts += m.vertex_count();
        total_tris += m.triangle_count();
        group_names.push(m.group.node_name().to_string());

        if let Some((min, max)) = m.bounding_box() {
            for i in 0..3 {
                global_min[i] = global_min[i].min(min[i]);
                global_max[i] = global_max[i].max(max[i]);
            }
        }
    }

    let manifest = AssetManifest {
        version: spec.version,
        name: spec.name.clone(),
        class: format!("{:?}", spec.class).to_lowercase(),
        archetype: spec.variant.archetype.display_name().to_string(),
        origin: format!("{:?}", spec.origin).to_lowercase(),
        glb_file: format!("{}.glb", slug(&spec.name)),
        vertex_count: total_verts,
        triangle_count: total_tris,
        mesh_groups: group_names,
        bounding_box: Some(BoundingBoxInfo {
            min: global_min,
            max: global_max,
        }),
    };

    Ok((glb_bytes, manifest))
}

fn collect_meshes(result: &HullResult) -> Vec<&MeshData> {
    result.meshes.iter().filter(|m| m.vertex_count() > 0).collect()
}

/// Build a minimal valid GLB binary from mesh data.
/// Uses raw JSON + binary buffer construction (no gltf-json dependency for flexibility).
fn build_glb(meshes: &[&MeshData], spec: &SloopAssetSpec) -> Result<Vec<u8>, ExportError> {
    // Build the binary buffer: all positions, normals, then indices for each mesh
    let mut bin_data: Vec<u8> = Vec::new();
    let mut accessors = Vec::new();
    let mut buffer_views = Vec::new();
    let mut mesh_nodes = Vec::new();
    let mut material_defs = Vec::new();
    let mut material_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    let mut accessor_idx = 0usize;
    let mut bv_idx = 0usize;

    for mesh in meshes.iter() {
        // -- Positions --
        let pos_offset = bin_data.len();
        let mut pos_min = [f64::MAX; 3];
        let mut pos_max = [f64::MIN; 3];
        for p in &mesh.positions {
            for i in 0..3 {
                pos_min[i] = pos_min[i].min(p[i]);
                pos_max[i] = pos_max[i].max(p[i]);
                bin_data.extend_from_slice(&(p[i] as f32).to_le_bytes());
            }
        }
        let pos_len = bin_data.len() - pos_offset;

        buffer_views.push(serde_json::json!({
            "buffer": 0,
            "byteOffset": pos_offset,
            "byteLength": pos_len,
            "target": 34962  // ARRAY_BUFFER
        }));
        let pos_accessor = accessor_idx;
        accessors.push(serde_json::json!({
            "bufferView": bv_idx,
            "componentType": 5126,  // FLOAT
            "count": mesh.positions.len(),
            "type": "VEC3",
            "min": [pos_min[0] as f32, pos_min[1] as f32, pos_min[2] as f32],
            "max": [pos_max[0] as f32, pos_max[1] as f32, pos_max[2] as f32]
        }));
        accessor_idx += 1;
        bv_idx += 1;

        // -- Normals --
        let norm_offset = bin_data.len();
        for n in &mesh.normals {
            for i in 0..3 {
                bin_data.extend_from_slice(&(n[i] as f32).to_le_bytes());
            }
        }
        let norm_len = bin_data.len() - norm_offset;

        buffer_views.push(serde_json::json!({
            "buffer": 0,
            "byteOffset": norm_offset,
            "byteLength": norm_len,
            "target": 34962
        }));
        let norm_accessor = accessor_idx;
        accessors.push(serde_json::json!({
            "bufferView": bv_idx,
            "componentType": 5126,
            "count": mesh.normals.len(),
            "type": "VEC3"
        }));
        accessor_idx += 1;
        bv_idx += 1;

        // -- Indices --
        let idx_offset = bin_data.len();
        for &idx in &mesh.indices {
            bin_data.extend_from_slice(&idx.to_le_bytes());
        }
        let idx_len = bin_data.len() - idx_offset;

        buffer_views.push(serde_json::json!({
            "buffer": 0,
            "byteOffset": idx_offset,
            "byteLength": idx_len,
            "target": 34963  // ELEMENT_ARRAY_BUFFER
        }));
        let idx_accessor = accessor_idx;
        accessors.push(serde_json::json!({
            "bufferView": bv_idx,
            "componentType": 5125,  // UNSIGNED_INT
            "count": mesh.indices.len(),
            "type": "SCALAR"
        }));
        accessor_idx += 1;
        bv_idx += 1;

        // -- Material --
        let mat_name = material_for_group(&mesh.group).to_string();
        let mat_idx = if let Some(&idx) = material_map.get(&mat_name) {
            idx
        } else {
            let idx = material_defs.len();
            let color = resolve_material_color(&mat_name, &spec.materials, &spec.sails);
            let is_metal = mat_name.contains("metal");
            material_defs.push(serde_json::json!({
                "name": mat_name,
                "pbrMetallicRoughness": {
                    "baseColorFactor": [color[0] as f32, color[1] as f32, color[2] as f32, color[3] as f32],
                    "metallicFactor": if is_metal { 0.6 } else { 0.0 },
                    "roughnessFactor": if is_metal { 0.4 } else { 0.8 }
                }
            }));
            material_map.insert(mat_name, idx);
            idx
        };

        // -- Mesh primitive --
        mesh_nodes.push((
            mesh.group.node_name().to_string(),
            serde_json::json!({
                "primitives": [{
                    "attributes": {
                        "POSITION": pos_accessor,
                        "NORMAL": norm_accessor
                    },
                    "indices": idx_accessor,
                    "material": mat_idx
                }]
            }),
        ));
    }

    // Build node tree
    let mut nodes = Vec::new();
    let mut child_indices: Vec<usize> = Vec::new();

    // Add mesh nodes
    for (i, (name, _mesh_json)) in mesh_nodes.iter().enumerate() {
        let mesh_def_idx = i;
        nodes.push(serde_json::json!({
            "name": name,
            "mesh": mesh_def_idx
        }));
        child_indices.push(i);
    }

    // Root node
    let root_idx = nodes.len();
    nodes.push(serde_json::json!({
        "name": "ShipRoot",
        "children": child_indices
    }));

    // Pad binary buffer to 4-byte alignment
    while bin_data.len() % 4 != 0 {
        bin_data.push(0);
    }

    let gltf_json = serde_json::json!({
        "asset": {
            "version": "2.0",
            "generator": "asset-forge"
        },
        "scene": 0,
        "scenes": [{
            "name": "Ship",
            "nodes": [root_idx]
        }],
        "nodes": nodes,
        "meshes": mesh_nodes.iter().map(|(_, m)| m.clone()).collect::<Vec<_>>(),
        "accessors": accessors,
        "bufferViews": buffer_views,
        "buffers": [{
            "byteLength": bin_data.len()
        }],
        "materials": material_defs
    });

    let json_str = serde_json::to_string(&gltf_json)
        .map_err(|e| ExportError::SerializationError(e.to_string()))?;
    let mut json_bytes = json_str.into_bytes();
    // Pad JSON to 4-byte alignment with spaces
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }

    // GLB structure:
    // Header (12 bytes): magic + version + total length
    // JSON chunk: length + type + data
    // BIN chunk: length + type + data
    let total_length = 12 + 8 + json_bytes.len() + 8 + bin_data.len();

    let mut glb = Vec::with_capacity(total_length);

    // Header
    glb.extend_from_slice(b"glTF");                         // magic
    glb.extend_from_slice(&2u32.to_le_bytes());             // version
    glb.extend_from_slice(&(total_length as u32).to_le_bytes()); // total length

    // JSON chunk
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes());    // chunk type: JSON
    glb.extend_from_slice(&json_bytes);

    // BIN chunk
    glb.extend_from_slice(&(bin_data.len() as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes());    // chunk type: BIN
    glb.extend_from_slice(&bin_data);

    Ok(glb)
}

fn slug(name: &str) -> String {
    let mut s: String = name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    // Collapse multiple dashes
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    s.trim_matches('-').to_string()
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("no meshes to export")]
    NoMeshes,
    #[error("serialization error: {0}")]
    SerializationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_hull::generate::generate_hull;
    use ship_schema::defaults::classic_runner;
    use ship_schema::spec::SloopVariantArchetype;

    #[test]
    fn export_classic_runner_produces_valid_glb() {
        let spec = classic_runner();
        let hull = generate_hull(&spec).unwrap();
        let (glb, manifest) = export_glb(&spec, &hull).unwrap();

        // GLB header check
        assert_eq!(&glb[0..4], b"glTF", "should start with glTF magic");
        assert_eq!(u32::from_le_bytes(glb[4..8].try_into().unwrap()), 2, "version should be 2");

        // File should be non-trivial
        assert!(glb.len() > 1000, "GLB should be substantial: {} bytes", glb.len());

        // Manifest check
        assert_eq!(manifest.name, "Sloop / Classic Runner");
        assert!(manifest.vertex_count > 100);
        assert!(manifest.triangle_count > 100);
        assert!(!manifest.mesh_groups.is_empty());
        assert!(manifest.mesh_groups.contains(&"Hull".to_string()));
    }

    #[test]
    fn glb_total_length_matches_header() {
        let spec = classic_runner();
        let hull = generate_hull(&spec).unwrap();
        let (glb, _) = export_glb(&spec, &hull).unwrap();

        let stated_length = u32::from_le_bytes(glb[8..12].try_into().unwrap()) as usize;
        assert_eq!(glb.len(), stated_length, "GLB length should match header");
    }

    #[test]
    fn manifest_serializes_to_json() {
        let spec = classic_runner();
        let hull = generate_hull(&spec).unwrap();
        let (_, manifest) = export_glb(&spec, &hull).unwrap();

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        assert!(json.contains("Classic Runner"));
        assert!(json.contains("vertexCount"));

        // Round-trip
        let parsed: AssetManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, manifest.name);
    }

    #[test]
    fn all_archetypes_export_successfully() {
        for arch in SloopVariantArchetype::all() {
            let spec = arch.default_spec();
            let hull = generate_hull(&spec).unwrap();
            let result = export_glb(&spec, &hull);
            assert!(result.is_ok(), "{:?} export failed: {:?}", arch, result.err());

            let (glb, manifest) = result.unwrap();
            assert_eq!(&glb[0..4], b"glTF", "{:?} GLB magic wrong", arch);
            assert!(manifest.vertex_count > 50, "{:?} too few vertices", arch);
        }
    }

    #[test]
    fn slug_generation() {
        assert_eq!(slug("Sloop / Classic Runner"), "sloop-classic-runner");
        assert_eq!(slug("Ship #42 (Test)"), "ship-42-test");
    }
}
