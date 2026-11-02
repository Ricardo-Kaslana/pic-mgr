# Contributing to PicMgr

Thanks for taking the time to contribute. This document covers the development setup,
the checks we run in CI, and the conventions we follow.

## Reporting bugs and requesting features

- Search [existing issues](https://github.com/Ricardo-Kaslana/pic-mgr/issues) first.
- Use the bug report or feature request template when opening a new issue.
- Include your OS, PicMgr version, and the steps needed to reproduce the problem.
- Please do **not** include personal photos, file paths or database files in issues.

Security issues should follow [SECURITY.md](SECURITY.md) instead.

## Development setup

Prerequisites: Node.js 20+, Rust stable, and the [Tauri 2 system dependencies](https://v2.tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri dev
```

Before submitting a PR, run:

```bash
npm run typecheck
cd src-tauri && cargo fmt --check
cd src-tauri && cargo test
```

## Pull requests

- Keep PRs focused. One logical change per PR is easier to review than a mixed bag.
- Follow the existing commit style: `type(scope): summary` (for example `fix(sync): ...`).
- Add or update tests when changing backend behavior. The Rust unit tests live next to the
  code in `src-tauri/src/`.
- All user-facing text must go through `vue-i18n` and be added to both `src/i18n/en.ts` and
  `src/i18n/zh-CN.ts`.
- Update `CHANGELOG.md` for user-visible changes.
- Read [AGENTS.md](AGENTS.md) before touching the scanner, database schema, tag model or
  thumbnail pipeline — it documents invariants that are easy to break.

## Releasing

Maintainers publish a release by pushing a version tag:

1. Update the version in `package.json`, `src-tauri/Cargo.toml` and
   `src-tauri/tauri.conf.json`, then refresh the lockfiles
   (`npm install --package-lock-only`, `cd src-tauri && cargo check`).
2. Add a `CHANGELOG.md` entry.
3. Commit, tag (`git tag vX.Y.Z`) and push the tag.
4. The `Release` workflow builds installers for Windows, macOS and Linux and opens a
   draft GitHub release. Review the assets and publish it.

The installers are unsigned, so Windows SmartScreen and macOS Gatekeeper may warn users.

## Code style

- Rust: `cargo fmt` (default rustfmt settings).
- TypeScript/Vue: 2-space indentation, single quotes, no semicolons where the surrounding
  code already omits them. `vue-tsc` must pass.
- Comments should explain *why*, not *what*. Keep them short.

## License

By contributing, you agree that your contributions are licensed under the
[MIT License](LICENSE).
