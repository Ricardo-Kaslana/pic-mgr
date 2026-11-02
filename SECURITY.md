# Security Policy

## Supported versions

| Version | Supported |
| --- | --- |
| 0.3.x | Yes |
| < 0.3 | No |

## Reporting a vulnerability

Please **do not** open a public issue for security problems.

Report vulnerabilities privately through GitHub Security Advisories:

<https://github.com/Ricardo-Kaslana/pic-mgr/security/advisories/new>

Please include:

- the affected version and platform,
- a description of the issue and its impact,
- reproduction steps or a proof of concept if available.

You can expect an initial response within seven days. Once the issue is confirmed, a fix will
be prepared and released as soon as practical, and you will be credited in the advisory unless
you prefer to remain anonymous.

## Scope

PicMgr is a local desktop application. It does not run a network service or upload data.
Relevant areas include:

- parsing of untrusted image files during thumbnail generation,
- SQLite query construction in the Rust backend,
- Tauri command permissions and the webview configuration,
- the update/release pipeline enabled by GitHub Actions.

Issues in third-party dependencies should be reported upstream; Dependabot keeps our
dependencies current.
