# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.0.x   | Yes       |

## Reporting a Vulnerability

Email: **64996768+mcp-tool-shop@users.noreply.github.com**

Include:
- Description of the vulnerability
- Steps to reproduce
- Version affected
- Potential impact

### Response timeline

| Action | Target |
|--------|--------|
| Acknowledge report | 48 hours |
| Assess severity | 7 days |
| Release fix | 30 days |

## Scope

Asset Forge is a **local-only procedural geometry generator**.

- **Data touched:** Ship spec parameters (in-memory), GLB and JSON manifest files (written to `output/`)
- **Data NOT touched:** No network access, no user data, no credentials, no databases
- **No telemetry** is collected or sent
- **No secrets handling** — does not read or store credentials
