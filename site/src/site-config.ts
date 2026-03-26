import type { SiteConfig } from '@mcptoolshop/site-theme/types';

export const config: SiteConfig = {
  title: 'Asset Forge',
  description: 'Procedural 3D ship generator for game asset pipelines. Define ship archetypes as parameterized specs, generate geometry, and export GLB files.',
  logoBadge: 'v1.0.0',
  brandName: 'MCP Tool Shop',
  repoUrl: 'https://github.com/mcp-tool-shop-org/asset-forge',
  footerText: 'Built by MCP Tool Shop',
  hero: {
    heading: 'Procedural Ship Geometry',
    subheading: 'Define archetypes. Generate hulls, decks, rigs, and sails. Export GLB with named mesh groups.',
    primaryCta: { label: 'Quick Start', href: '#quickstart' },
    secondaryCta: { label: 'Handbook', href: '/asset-forge/handbook/' },
  },
  sections: [
    {
      id: 'features',
      title: 'Features',
      subtitle: 'Everything you need to generate ship assets for your game pipeline.',
      kind: 'features',
      features: [
        {
          title: '6 Ship Archetypes',
          description: 'Classic Runner, Courier, Patrol, Smuggler, Fishing, and Merchant Light sloop variants with distinct silhouettes and material palettes.',
          icon: 'ship',
        },
        {
          title: 'Parameterized Specs',
          description: 'Hull curves, deck geometry, rail styles, rig layout, sail camber, superstructure, and material tones all driven by a single JSON spec.',
          icon: 'settings',
        },
        {
          title: 'GLB Export',
          description: 'Valid glTF Binary output with named mesh groups per component (Hull, Deck, Rail, Mast, Sails) and PBR material colors.',
          icon: 'package',
        },
        {
          title: 'Style Law Enforcement',
          description: 'Four style profiles (PortlightClassic, StorybookLowpoly, NavalStylized, MerchantStylized) actively clamp geometry to keep ships visually consistent.',
          icon: 'shield',
        },
        {
          title: 'Schema Migration',
          description: 'Automatic v1 to v2 spec migration with archetype-aware defaults. Old specs gain new deck, rig, and sail fields without breaking.',
          icon: 'refresh',
        },
        {
          title: 'Zero Runtime Dependencies',
          description: 'Pure Rust with only serde, serde_json, and thiserror. No networking, no user data, no databases. Runs anywhere Rust compiles.',
          icon: 'zap',
        },
      ],
    },
    {
      id: 'quickstart',
      title: 'Quick Start',
      subtitle: 'Generate ship assets in two commands.',
      kind: 'code-cards',
      cards: [
        {
          title: 'Run Tests',
          description: 'Verify all 4 crates build and pass 73 tests across schema, hull generation, export, and golden-family invariants.',
          code: 'cargo test',
        },
        {
          title: 'Export All Archetypes',
          description: 'Generate GLB and JSON manifest files for all 6 ship archetypes into the output/ directory.',
          code: 'cargo run --example export_all',
        },
        {
          title: 'Inspect Output',
          description: 'Each archetype produces a .glb file (loadable in Blender, Godot, Three.js) and a .json manifest with vertex/triangle counts and mesh group names.',
          code: 'ls output/\n# sloop-classic-runner.glb\n# sloop-classic-runner.json\n# sloop-courier.glb\n# ...',
        },
      ],
    },
  ],
};
