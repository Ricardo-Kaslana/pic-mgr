# Documentation

## Architecture decisions

| ADR | Decision |
| --- | --- |
| [0001](adr/0001-tauri-2-choice.md) | Use Tauri 2 as the desktop framework |
| [0002](adr/0002-sqlite-metadata.md) | Store metadata in SQLite |
| [0003](adr/0003-scanner-no-image-crate.md) | Scan with `std::fs`, not the `image` crate |

## Design records

These are historical design documents and are kept in their original language (Chinese).

| Document | Scope |
| --- | --- |
| [architecture.md](design/architecture.md) | Overall design and data model |
| [move-to-workspace.md](design/move-to-workspace.md) | Moving images between workspaces |
| [sidebar-collapse.md](design/sidebar-collapse.md) | Collapsible sidebars |
| [workspace-sync.md](design/workspace-sync.md) | Workspace reconciliation and sync |

## Contributor guidance

See [AGENTS.md](../AGENTS.md) for invariants, build commands and architecture gotchas.
