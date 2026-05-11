# GitHub Release checklist

Manual steps for maintainers when publishing Ferrite releases. The [`release.yml`](../.github/workflows/release.yml) workflow uploads artifacts and opens a GitHub Release with auto-generated notes.

---

## macOS — paste into release description (v0.3.0 until signing ships)

**Until [v0.3.1](../ROADMAP.md) adds Developer ID signing and notarization**, prepend or append the following block to the GitHub Release body so macOS downloaders see it immediately (see [#130](https://github.com/OlaProeis/Ferrite/issues/130)).

Copy everything inside the fence:

```markdown
### macOS (Gatekeeper)

GitHub **DMG / `.tar.gz`** builds for **v0.3.x** are **unsigned** and **not notarized**. On **macOS 15.x (Sequoia)** you may see Gatekeeper warnings or the app may refuse to open.

**Temporary workarounds:**

- **Terminal** (reliable): `xattr -dr com.apple.quarantine /Applications/Ferrite.app` — change the path if `Ferrite.app` is not in Applications.
- **Finder:** Control-click `Ferrite.app` → **Open** → **Open** (may not work on every 15.x build).
- **Homebrew:** `brew install --cask ferrite` often avoids quarantine friction.

**Proper fix:** Apple Developer ID signing + notarization is planned for **v0.3.1**.

Full detail: [docs/install/macos.md](https://github.com/OlaProeis/Ferrite/blob/master/docs/install/macos.md)
```

---

## After the workflow runs

1. Confirm all platform artifacts attached (Windows signed zip/MSI/PAF, Linux tar/deb/rpm, macOS DMG + tar per arch).
2. For macOS-heavy releases, paste the block above into the release description and save.
3. Spot-check links in the pasted section (issue #130, install doc).
