# macOS Gatekeeper documentation (GitHub Releases)

## Purpose

CI-built macOS artifacts (`Ferrite.app` inside `.dmg` / `.tar.gz`) ship **without** Apple Developer ID signing or notarization through **v0.3.0**. Gatekeeper—especially on **macOS 15.x**—may block or warn. This document maps **where** that behavior is documented and **what** maintainers ship.

**Tracking:** [GitHub #130](https://github.com/OlaProeis/Ferrite/issues/130). Signed GitHub releases depend on Apple Developer Program enrollment and CI wiring — no release date committed.

## User-facing

| Asset | Role |
|-------|------|
| [`docs/install/macos.md`](../../install/macos.md) | Troubleshooting: quarantine (`xattr -dr com.apple.quarantine …`), Open Anyway, Homebrew; temporary workaround tone |
| Root [`README.md`](../../../README.md) | Collapsible macOS section + install notes linking the install doc |
| [`docs/building.md`](../../building.md) | Builder context + troubleshooting pointer |

## Maintainer / release

| Asset | Role |
|-------|------|
| [`docs/github-release-checklist.md`](../../github-release-checklist.md) | Copy-paste Markdown for GitHub Release body until signing lands |
| [`.github/workflows/release.yml`](../../../.github/workflows/release.yml) | Comment on `release` job reminding publishers to paste the Gatekeeper block |

## Related packaging doc

[`macos-app-bundle-ci.md`](./macos-app-bundle-ci.md) describes `.app` CI packaging; it links the install doc for Gatekeeper behavior (bundling ≠ notarization).
