---
title: Asset Forge Handbook
description: Complete guide to the procedural ship generator for game asset pipelines.
sidebar:
  order: 0
---

Asset Forge is a Rust workspace that generates 3D ship geometry from parameterized specs and exports the results as GLB files. It targets game asset pipelines where you need consistent, variant-rich ship models without hand-modeling each one.

## How It Works

1. **Define a spec** -- Each ship is described by a `SloopAssetSpec` struct covering dimensions, hull shape, deck layout, rig configuration, sail properties, attachments, materials, and style rules.
2. **Generate geometry** -- The `ship-hull` crate reads the spec, computes hull stations, lofts the hull surface, adds deck, rails, quarterdeck, cabin, hatches, helm, mast, bowsprit, boom, gaff, rigging, and sails.
3. **Export GLB** -- The `ship-export` crate packs all mesh groups into a single glTF Binary file with PBR material colors and produces a JSON manifest sidecar.

## Workspace Layout

| Crate | Role |
|-------|------|
| `ship-schema` | Spec types, archetype defaults, validation, style profiles, v1-to-v2 migration |
| `ship-hull` | Full geometry pipeline -- hull loft, deck, rails, rig, sails, superstructure |
| `ship-export` | GLB binary construction with named mesh groups and manifest output |
| `ship-testkit` | Test fixtures, geometric invariant checkers, golden-family tests |

## Ship Archetypes

Six sloop variants ship as built-in presets:

| Archetype | Character |
|-----------|-----------|
| Classic Runner | Balanced reference ship with gaff sloop rig |
| Courier | Longer, narrower, needle bow for speed |
| Patrol | Broader beam, reinforced bow, light cannon pair |
| Smuggler | Low profile, dark wood, no pennant or lanterns |
| Fishing | Wide and sturdy, open deck, worn damage state |
| Merchant Light | Cargo-friendly, visible trade goods, merchant pennant |

Each archetype provides a complete default spec that can be used directly or customized before generation.
