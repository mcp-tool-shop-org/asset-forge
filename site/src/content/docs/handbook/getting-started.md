---
title: Getting Started
description: Set up Asset Forge and generate your first ship.
sidebar:
  order: 1
---

## Requirements

- **Rust 1.75+** (stable toolchain)
- No external runtime dependencies -- the workspace uses only `serde`, `serde_json`, and `thiserror`

## Clone and Build

```bash
git clone https://github.com/mcp-tool-shop-org/asset-forge.git
cd asset-forge
cargo build
```

## Run the Tests

```bash
cargo test
```

This exercises all four crates: schema validation, hull geometry generation, GLB export, and golden-family invariant checks.

## Generate All Archetypes

```bash
cargo run --example export_all
```

Output lands in the `output/` directory. For each of the six archetypes you get:

- A `.glb` file -- valid glTF Binary loadable in Blender, Godot, Three.js, or any glTF viewer
- A `.json` manifest -- metadata including vertex count, triangle count, mesh group names, and bounding box

## Use the Output

The GLB files contain a `ShipRoot` node with named children for each component:

- `Hull`, `BowCap`, `SternCap` -- the hull shell
- `Deck`, `Rail`, `Quarterdeck` -- deck surfaces and railings
- `Cabin`, `Hatches`, `Helm` -- superstructure elements
- `Mast_Main`, `Bowsprit`, `Boom`, `Gaff`, `Rigging` -- rig spars and lines
- `Sail_Main`, `Sail_Jib_01` -- sail surfaces with camber

Each mesh group carries a PBR material with base color derived from the archetype's material spec (hull wood tone, deck wood tone, sail tone, metal tone).

## Customizing a Spec

To generate a custom ship, start from an archetype default and modify fields:

```rust
use ship_schema::defaults::classic_runner;

let mut spec = classic_runner();
spec.name = "My Custom Sloop".into();
spec.dimensions.overall_length = 12.0;
spec.hull.bow_style = BowStyle::Needle;
spec.materials.hull_wood = WoodTone::Walnut;

let result = ship_hull::generate::generate_ship(&spec)?;
let (glb, manifest) = ship_export::export_glb(&spec, &result)?;
```

The style law enforcer automatically clamps parameters to stay within the chosen style profile's visual bounds.
