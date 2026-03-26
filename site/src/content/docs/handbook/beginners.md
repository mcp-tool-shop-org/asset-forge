---
title: Beginners Guide
description: A complete introduction to Asset Forge for newcomers to procedural geometry or Rust tooling.
sidebar:
  order: 99
---

## What Is This Tool?

Asset Forge is a Rust library that generates 3D ship models from parameter files. Instead of hand-modeling each ship variant in Blender or Maya, you define a spec (dimensions, hull shape, rig layout, materials) and Asset Forge produces a ready-to-use GLB file with all geometry, named mesh groups, and PBR material colors.

It currently generates sloop-class sailing ships with six built-in archetypes, each with a distinct silhouette and character. The output is standard glTF Binary, compatible with any 3D engine or viewer.

## Who Is This For?

- **Game developers** who need consistent ship assets across multiple variants without hiring a 3D artist for each one
- **Prototypers** building naval or maritime games who want placeholder geometry that actually looks intentional
- **Pipeline engineers** integrating procedural assets into Godot, Unity, Unreal, or Three.js workflows
- **Rust learners** looking at a real-world workspace with multiple crates, validation, style enforcement, and binary export

## Prerequisites

| Requirement | Version | Why |
|-------------|---------|-----|
| Rust toolchain | 1.75+ stable | The workspace uses edition 2021 features |
| Git | Any recent | To clone the repository |
| A GLB viewer | Optional | Blender, Godot, VS Code glTF extension, or any glTF viewer to inspect output |

No other dependencies are needed. Asset Forge has zero runtime dependencies beyond serde for JSON serialization.

## Your First 5 Minutes

**Step 1: Clone the repository.**

```bash
git clone https://github.com/mcp-tool-shop-org/asset-forge.git
cd asset-forge
```

**Step 2: Run the tests to verify your setup.**

```bash
cargo test
```

You should see all tests pass across the four crates (ship-schema, ship-hull, ship-export, ship-testkit).

**Step 3: Generate all six ship archetypes.**

```bash
cargo run --example export_all
```

This creates `.glb` and `.json` files in the `output/` directory.

**Step 4: Open a GLB file in your viewer.** Load `output/sloop-classic-runner.glb` in Blender (File > Import > glTF) or drag it into an online glTF viewer. You should see a sloop with hull, deck, rails, cabin, mast, bowsprit, boom, gaff, rigging, mainsail, and jib -- all as separate named mesh groups.

**Step 5: Read the manifest.** Open `output/sloop-classic-runner.json` to see vertex/triangle counts, mesh group names, and the bounding box.

## Common Mistakes

**Running `cargo run` without `--example export_all`.** The workspace has no default binary target. You need the `--example export_all` flag to run the export example in the `ship-export` crate.

**Expecting textures in the GLB.** Asset Forge produces PBR material colors (base color factor), not textured models. The GLB files have solid colors derived from the archetype's wood, sail, and metal tones.

**Editing the output GLB directly.** The GLB files are generated artifacts. Edit the spec instead and re-run the export. Any manual changes to output files will be overwritten.

**Ignoring validation errors.** If you customize a spec and `generate_ship()` returns an error, the spec failed validation. Check that all dimensions are positive, factors are in 0--1 range, and the sloop has exactly 1 mast.

**Using version 1 specs without migration.** If you have old v1 spec JSON, pass it through `migrate_to_current()` before generation. This fills in new v2 fields with archetype-aware defaults.

## Next Steps

- Read the [Reference](/asset-forge/handbook/reference/) for the full spec field listing and enum values
- Browse the [archetype defaults](https://github.com/mcp-tool-shop-org/asset-forge/blob/main/crates/ship-schema/src/defaults.rs) to understand how each variant differs
- Study the [style enforcement system](https://github.com/mcp-tool-shop-org/asset-forge/blob/main/crates/ship-schema/src/style.rs) to see how style profiles clamp geometry parameters
- Try modifying an archetype's hull profile, bow style, or material tones and re-exporting

## Glossary

| Term | Definition |
|------|-----------|
| **Archetype** | A named ship variant preset (e.g., Classic Runner, Smuggler) with default values for all spec fields |
| **Beam** | The widest point of the hull, measured in meters |
| **Camber** | The curvature/belly of a sail surface (Flat, LightBillow, Full) |
| **Draft** | How deep the hull sits below the waterline |
| **Freeboard** | Distance from the waterline to the deck edge |
| **GLB** | glTF Binary -- a single-file 3D format containing geometry, materials, and scene structure |
| **Loft** | The process of generating a 3D hull surface by interpolating between cross-section profiles at multiple stations along the ship's length |
| **Manifest** | A JSON sidecar file listing metadata about the exported asset (vertex count, mesh groups, bounding box) |
| **Quarterdeck** | A raised deck section toward the stern, common on sailing ships |
| **Sheer** | The upward curve of the deck line from midship toward bow and stern |
| **Spec** | Short for SloopAssetSpec -- the JSON/Rust struct that fully defines a ship variant |
| **Station** | A cross-section sample point along the ship's length used during hull lofting |
| **Style Law** | A set of constraints (exaggeration, chunkiness, realism) that clamp geometry parameters to maintain visual consistency |
| **Tumblehome** | The inward lean of the upper hull above the waterline |
