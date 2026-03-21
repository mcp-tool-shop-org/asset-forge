use ship_hull::mesh::MeshData;

/// Summary metrics for a generated mesh.
#[derive(Debug, Clone)]
pub struct MeshMetrics {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub bounding_box: Option<([f64; 3], [f64; 3])>,
    pub x_span: f64,
    pub y_span: f64,
    pub z_span: f64,
}

impl MeshMetrics {
    pub fn from_mesh(mesh: &MeshData) -> Self {
        let bb = mesh.bounding_box();
        let (x_span, y_span, z_span) = if let Some((min, max)) = bb {
            (max[0] - min[0], max[1] - min[1], max[2] - min[2])
        } else {
            (0.0, 0.0, 0.0)
        };

        Self {
            vertex_count: mesh.vertex_count(),
            triangle_count: mesh.triangle_count(),
            bounding_box: bb,
            x_span,
            y_span,
            z_span,
        }
    }
}

/// Aggregate metrics across multiple meshes.
pub fn aggregate_metrics(meshes: &[&MeshData]) -> MeshMetrics {
    let mut total_verts = 0;
    let mut total_tris = 0;
    let mut global_min = [f64::MAX; 3];
    let mut global_max = [f64::MIN; 3];
    let mut has_data = false;

    for mesh in meshes {
        total_verts += mesh.vertex_count();
        total_tris += mesh.triangle_count();
        if let Some((min, max)) = mesh.bounding_box() {
            has_data = true;
            for i in 0..3 {
                global_min[i] = global_min[i].min(min[i]);
                global_max[i] = global_max[i].max(max[i]);
            }
        }
    }

    let bb = if has_data { Some((global_min, global_max)) } else { None };
    let (x_span, y_span, z_span) = if let Some((min, max)) = bb {
        (max[0] - min[0], max[1] - min[1], max[2] - min[2])
    } else {
        (0.0, 0.0, 0.0)
    };

    MeshMetrics {
        vertex_count: total_verts,
        triangle_count: total_tris,
        bounding_box: bb,
        x_span,
        y_span,
        z_span,
    }
}
