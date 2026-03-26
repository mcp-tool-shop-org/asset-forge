---
title: Reference
description: Spec fields, enums, style profiles, and export format reference.
sidebar:
  order: 4
---

## SloopAssetSpec Structure

The top-level spec contains these sections:

| Section | Key Fields |
|---------|-----------|
| `dimensions` | `overall_length`, `beam`, `draft_visual`, `freeboard`, `mast_height`, `bowsprit_length` |
| `hull` | `profile`, `bow_style`, `stern_style`, `keel_depth`, `tumblehome`, `sheer`, `fullness`, `rake`, `quarterdeck_height`, `transom_width` |
| `deck` | `cabin`, `cabin_length`, `cabin_width_ratio`, `cabin_height`, `hatch_count`, `rail_height`, `bulwark_style`, `wheel`, `tiller`, `mast_offset` |
| `rig` | `plan`, `mast_count`, `mast_rake`, `mast_diameter`, `boom_length`, `gaff_length`, `gaff_angle`, `bowsprit_angle`, `jib_count`, `shroud_pairs` |
| `sails` | `mainsail_fullness`, `jib_fullness`, `reefed`, `furled_jib`, `sail_tone`, `patchiness`, `edge_wear`, `camber_style` |
| `attachments` | `pennant`, `lanterns`, `cannons`, `figurehead`, `anchor_visible`, `lifeboat`, `cargo_visible` |
| `materials` | `hull_wood`, `deck_wood`, `mast_wood`, `trim_metal`, `painted_stripe`, `stripe_color` |
| `style` | `profile`, `silhouette_exaggeration`, `detail_density`, `edge_chunkiness`, `realism_bias` |

## Hull Profiles

| Profile | Character |
|---------|-----------|
| `Swift` | Narrow, fast, low drag |
| `Balanced` | Middle ground |
| `CargoLight` | Wide beam for cargo capacity |
| `NavyLight` | Sturdy, broader, combat-ready |

## Bow Styles

`Needle` / `Raked` / `Rounded` / `Reinforced`

## Stern Styles

`Simple` / `Rounded` / `Square` / `OrnateLight`

## Sail Plans

| Plan | Description |
|------|-------------|
| `ForeAndAft` | Simple fore-and-aft rig |
| `GaffSloop` | Gaff-rigged mainsail with jib |
| `CutterLike` | Multiple headsails |

## Style Profiles

Four profiles control how the style enforcer clamps geometry:

| Profile | Behavior |
|---------|----------|
| `PortlightClassic` | Moderate exaggeration, chunky edges, low realism |
| `StorybookLowpoly` | High exaggeration, very chunky, minimal detail |
| `NavalStylized` | Low exaggeration, high realism bias |
| `MerchantStylized` | Moderate everything, higher realism |

The style enforcer uses `silhouette_exaggeration`, `detail_density`, `edge_chunkiness`, and `realism_bias` (all 0.0--1.0) to actively clamp hull sheer, rake, tumblehome, section resolution, and rig parameters.

## Material Tones

**Wood:** `LightOak`, `GoldenOak`, `Walnut`, `WeatheredBrown`, `PaintedDark`

**Sail:** `Cream`, `Tan`, `WeatheredIvory`, `Gray`

**Metal:** `Iron`, `Brass`, `BlackenedSteel`

## Damage States

`Clean` / `Worn` / `BattleScarred`

## GLB Export Format

- glTF 2.0 Binary (`.glb`)
- Root node: `ShipRoot` with child nodes for each mesh group
- PBR materials with `baseColorFactor` derived from spec material tones
- Metal groups get `metallicFactor: 0.6`, `roughnessFactor: 0.4`
- Non-metal groups get `metallicFactor: 0.0`, `roughnessFactor: 0.8`

## JSON Manifest

Each export produces a `.json` sidecar with:

```json
{
  "version": 2,
  "name": "Sloop / Classic Runner",
  "class": "sloop",
  "archetype": "Classic Runner",
  "origin": "procedural",
  "glbFile": "sloop-classic-runner.glb",
  "vertexCount": 1234,
  "triangleCount": 567,
  "meshGroups": ["Hull", "Deck", "Rail", "..."],
  "boundingBox": { "min": [-1, -2, -1], "max": [9, 2, 9] }
}
```

## Schema Migration

Specs at version 1 are automatically migrated to version 2 via `migrate_to_current()`. New v2 fields (cabin dimensions, rail thickness, bulwark style, mast diameter, gaff angle, bowsprit angle, camber style) receive archetype-appropriate defaults during migration.

## Validation

`validate_spec()` checks all fields before geometry generation:

- Version must be 1 or 2
- Kind must be "ship"
- Dimensions must be positive and finite
- Hull factors must be in valid 0--1 ranges
- Sloop requires exactly 1 mast
- V2-specific fields validated when version >= 2
