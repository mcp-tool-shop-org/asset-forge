<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/asset-forge/readme.png" width="400" alt="Asset Forge">
</p>

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/asset-forge/actions/workflows/ci.yml"><img src="https://github.com/mcp-tool-shop-org/asset-forge/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
</p>

# Asset Forge

Procedural 3D ship generator for game asset pipelines. Defines ship archetypes as parameterized specs, generates geometry (hull, deck, rails, rig, sails, superstructure), and exports GLB files with named groups.

---

## What This Does

- **6 ship archetypes** — classic runner, courier, fishing, merchant light, patrol, smuggler
- **Parameterized generation** — hull curves, deck geometry, rail/bulwark styles, rig (mast, bowsprit, boom, gaff), sails (mainsail + jib with camber/reef/furl), superstructure (cabin, hatches, helm)
- **GLB export** — valid glTF Binary with named mesh groups per component
- **Material colors** — hull, deck, sail, and metal tones derived from archetype spec
- **Schema migration** — v1 → v2 spec migration with validation

## Crates

| Crate | Purpose |
|-------|---------|
| `ship-schema` | Ship spec types, archetype defaults, validation, style system, v1→v2 migration |
| `ship-hull` | Geometry generation — hull loft, deck, rails, rig, sails, superstructure, caps |
| `ship-export` | GLB export with named groups and manifest output |
| `ship-testkit` | Test fixtures and geometric invariant checkers |

## Quick Start

```bash
# Run all tests
cargo test

# Generate all archetypes
cargo run --example export_all
```

Output goes to `output/` — one `.glb` and one `.json` manifest per archetype.

## Requirements

- Rust 1.75+
- No runtime dependencies beyond serde/serde_json/thiserror

## Security & Data Scope

| Aspect | Detail |
|--------|--------|
| **Data touched** | Ship spec JSON (read), GLB + manifest files (write to `output/`) |
| **Data NOT touched** | No network, no user data, no credentials, no databases |
| **Telemetry** | None |

See [SECURITY.md](SECURITY.md) for vulnerability reporting.

## License

[MIT](LICENSE)

---

Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
