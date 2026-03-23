# CLAUDE.md – FreeSynergy Apps

## What is this?

FreeSynergy Apps – standalone apps that run inside the FreeSynergy Desktop shell
or as independent windows.

Each app is a separate Cargo crate with its own `[[bin]]` and can run standalone
or be embedded by `fs-shell` in the desktop.

## Rules

- Language in files: **English** (comments, code, variable names)
- Language in chat: **German**
- OOP everywhere: traits over match blocks, types carry their own behavior
- No CHANGELOG.md
- After every feature: commit directly

## Apps

| Crate | Description |
|---|---|
| `fs-store-app` | Store frontend — package discovery, install, updates |
| `fs-browser` | Embedded web browser with tabs, bookmarks, S3 downloads |
| `fs-lenses` | Aggregated cross-service data views |
| `fs-theme-app` | Theme manager — colors, cursors, chrome |
| `fs-builder` | Container builder, Bridge builder, i18n editor, resource browser |
| `fs-tasks` | Task manager and pipeline editor |
| `fs-bots` | Bot management UI |
| `fs-ai` | AI assistant app |
| `fs-container-app` | Container, service and bot management |
| `fs-managers` | Unified panel for all managers (language, icons, cursor, theme, container) |

## Dependencies

- **fs-libs** (`../fs-libs/`) — all shared library crates
- **fs-desktop** (`../fs-desktop/crates/fs-db-desktop`) — shared desktop DB schemas
- **fs-managers** (`../fs-managers/`) — manager backends

## Architecture

Each app follows the Provider Pattern (OOP, Dioxus):
- `AppContext` via `provide_context` for shared state
- Business logic in domain structs, not in components
- `View` trait implementations on domain objects
- `AppShell` from `fs-shell` for consistent layout (when embedded)

## CSS Variables Prefix

Always `--fs-` (e.g., `--fs-color-primary`, `--fs-font-family`).
