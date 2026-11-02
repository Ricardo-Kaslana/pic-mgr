<div align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="112" alt="PicMgr logo" />
  <h1>PicMgr</h1>
  <p>A lightweight, local-first image manager for your desktop.</p>

  <p>
    <a href="https://github.com/Ricardo-Kaslana/pic-mgr/actions/workflows/ci.yml"><img src="https://github.com/Ricardo-Kaslana/pic-mgr/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
    <a href="https://github.com/Ricardo-Kaslana/pic-mgr/releases"><img src="https://img.shields.io/github/v/release/Ricardo-Kaslana/pic-mgr?include_prereleases" alt="Latest release" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT" /></a>
    <img src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white" alt="Tauri 2" />
    <img src="https://img.shields.io/badge/Vue-3-42B883?logo=vue.js&logoColor=white" alt="Vue 3" />
  </p>

  <p>
    <a href="README.md">English</a> |
    <a href="README.zh-CN.md">简体中文</a>
  </p>
</div>

> **Your files stay yours.** PicMgr indexes local folders as workspaces and stores tags, ratings and notes in SQLite. It never modifies, moves or rewrites your original image files unless you explicitly ask it to.

## Features

- **Workspaces** — add folders as workspaces, rescan or reconcile them when files change on disk.
- **Fast indexing** — SHA-256 based incremental scan with duplicate detection, backed by SQLite and FTS5.
- **Tags** — a global tag catalogue shared across workspaces; filter by tags, attach/detach in bulk, or delete a tag from the sidebar.
- **Ratings and notes** — 0–5 stars plus free-form notes for every image.
- **Full-text search** — search file names, paths and notes through SQLite FTS5.
- **Thumbnail cache** — on-demand JPEG thumbnails in three sizes, cached on disk with an in-memory LRU for blob URLs.
- **Safe file operations** — batch delete goes to the system recycle bin; batch move supports cross-device moves with conflict-safe suffixes.
- **Duplicate detection** — groups images by SHA-256 hash so you can review redundant files.
- **Metadata export** — export the current library as JSON or CSV.
- **Comfortable browsing** — virtual scrolling, keyboard-driven lightbox, sort persistence, collapsible sidebars and a quick-locate button.
- **Bilingual UI** — complete English and Simplified Chinese translations.
- **Digital Darkroom theme** — a carefully tuned dark and light theme.

## Tech stack

| Layer | Choice |
| --- | --- |
| Desktop shell | [Tauri 2](https://v2.tauri.app/) |
| Frontend | Vue 3, TypeScript, Pinia, Vue Router |
| Storage | SQLite (bundled via `rusqlite`) with FTS5 |
| Thumbnails | [`image`](https://crates.io/crates/image) — JPEG, quality 80 |
| Hashing | SHA-256 via `sha2` |
| UI text | `vue-i18n` (`en`, `zh-CN`) |

## Getting started

### Prerequisites

- [Node.js](https://nodejs.org/) 20 or newer
- [Rust](https://www.rust-lang.org/tools/install) stable
- The platform dependencies required by Tauri 2 — see the official [prerequisites guide](https://v2.tauri.app/start/prerequisites/)

### Development

```bash
npm install
npm run tauri dev
```

### Production build

```bash
npm run tauri build
```

### Checks

```bash
npm run typecheck              # vue-tsc
cd src-tauri && cargo fmt --check
cd src-tauri && cargo test
```

## How it works

- Folders become **workspaces**; every scan records file metadata and a SHA-256 hash in SQLite.
- All metadata — tags, ratings, notes, derived thumbnails — lives outside the original files.
- The Rust backend exposes 29 Tauri commands split by domain (`commands/`, `db/`, `scanner/`).
- The grid renders `ImageSummary` rows only; the detail panel loads the full `ImageRecord` on demand.
- Thumbnails are generated lazily, wrapped in `catch_unwind` so malformed files cannot crash the app.

For deeper details, see [docs/design/architecture.md](docs/design/architecture.md) and the [ADRs](docs/README.md).

## Project structure

```text
pic-mgr/
├── src/                    # Vue 3 frontend
│   ├── components/         # UI components
│   ├── composables/        # Shared composables
│   ├── i18n/               # en + zh-CN translations
│   ├── router/             # Vue Router
│   ├── stores/             # Pinia stores
│   ├── types/              # Shared TypeScript types
│   └── views/              # Workspace home, browser, dedup, settings
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri command handlers
│       ├── db/             # Schema, migrations and queries
│       └── scanner/        # Filesystem scan + thumbnail cache
├── docs/                   # Design records + ADRs
├── AGENTS.md               # Contributor and architecture invariants
└── CHANGELOG.md
```

## Documentation

- [Architecture overview](docs/design/architecture.md)
- [Design records](docs/design/)
- [Architecture decision records](docs/adr/)
- [Changelog](CHANGELOG.md)
- [Contributor guide](CONTRIBUTING.md)

## Contributing

Bug reports, feature requests and pull requests are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a PR, and note that this project follows the [Contributor Covenant](CODE_OF_CONDUCT.md).

If you find a security issue, please follow [SECURITY.md](SECURITY.md) instead of opening a public issue.

## License

Released under the [MIT License](LICENSE).

Copyright © 2026 Dawn.Wang.
