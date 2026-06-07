# Context Map

## Contexts

| Context | Path | Description |
|---------|------|-------------|
| Backend (Rust) | `src-tauri/CONTEXT.md` | Tauri commands, SQLite, scanner, thumbnail pipeline |
| Frontend (Vue) | `src/CONTEXT.md` | Vue 3 + Pinia stores, components, routing, i18n |

## System-wide ADRs

- [ADR-0001](docs/adr/0001-tauri-2-choice.md) — Tauri 2 as desktop framework
- [ADR-0002](docs/adr/0002-sqlite-metadata.md) — SQLite for metadata storage
- [ADR-0003](docs/adr/0003-scanner-no-image-crate.md) — Scanner uses std::fs, not image crate

## Key design docs

- [Design Spec](docs/superpowers/specs/2026-06-07-pic-mgr-design.md)
- [Implementation Plan](docs/superpowers/plans/2026-06-07-pic-mgr-implementation.md)
