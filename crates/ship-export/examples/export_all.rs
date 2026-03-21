use ship_hull::generate::generate_hull;
use ship_export::export_glb;
use ship_schema::spec::SloopVariantArchetype;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = Path::new("output");
    fs::create_dir_all(out_dir).expect("failed to create output dir");

    for arch in SloopVariantArchetype::all() {
        let spec = arch.default_spec();
        let hull = generate_hull(&spec).expect("hull generation failed");
        let (glb, manifest) = export_glb(&spec, &hull).expect("export failed");

        let slug = spec.name.to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "-")
            .replace("--", "-")
            .trim_matches('-')
            .to_string();

        let glb_path = out_dir.join(format!("{slug}.glb"));
        let json_path = out_dir.join(format!("{slug}.json"));

        fs::write(&glb_path, &glb).expect("failed to write glb");
        fs::write(&json_path, serde_json::to_string_pretty(&manifest).unwrap())
            .expect("failed to write manifest");

        println!(
            "{:16} -> {} ({} verts, {} tris, {} bytes)",
            arch.display_name(),
            glb_path.display(),
            manifest.vertex_count,
            manifest.triangle_count,
            glb.len(),
        );
    }
}
