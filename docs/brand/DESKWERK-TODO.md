# Deskwerk Sheets: rebranding leftovers

Deskwerk Sheets is based on GridCraft by the ArtCraft team (MIT OR Apache-2.0). The user-facing
name, app id (`eu.deskwerk.Sheets`), binaries (`deskwerk-sheets`, `deskwerk-sheets-cli`),
packages, installers and settings folders are renamed. This file lists what still carries the
upstream look or name, and why.

## Art to replace

These icons are the original placeholder art from the upstream repository (a hand-drawn ledger
page on a green tile, MIT OR Apache-2.0, see `ATTRIBUTION.md`). They are **not** ArtCraft
trademarks, so shipping them is allowed, but Deskwerk should get its own icon:

- `assets/app-icon/gridcraft.svg` (the source; everything below is generated from it by
  `packaging/icons.sh`)
- `assets/app-icon/gridcraft-1024.png`
- `assets/app-icon/gridcraft-macos-512.png` (macOS Dock icon, embedded by `apps/gridcraft`)
- `assets/app-icon/gridcraft.icns` (copied into the bundle as `Deskwerk Sheets.icns`)
- `assets/app-icon/gridcraft.ico` (Windows exe and MSI icon, `cargo xtask ico`)
- `assets/app-icon/hicolor/{16x16,24x24,32x32,48x48,64x64,128x128,256x256,512x512}/apps/eu.deskwerk.Sheets.png`
- `assets/app-icon/hicolor/scalable/apps/eu.deskwerk.Sheets.svg`
- The proposed app colour `#1f9d55` (`assets/app-icon/README.md`, metainfo `<branding>`).

After replacing the SVG: run `packaging/icons.sh`, update `assets/app-icon/README.md` and the
`ATTRIBUTION.md` rows (`cargo xtask assets` fails on any asset without a row).

## macOS DMG layout

- `packaging/macos/dmg/dmg-layout.DS_Store` still describes a volume named `GridCraft` with
  `GridCraft.app`. The volume and bundle are now `Deskwerk Sheets` / `Deskwerk Sheets.app`, so
  Finder falls back to default icon positions until it is regenerated with
  `packaging/macos/dmg/generate.py` (already updated) on a Mac.
- `packaging/macos/dmg/background.svg` / `background.tiff` embed the app icon; regenerate them
  with the new icon.

## Internal names kept on purpose

- Cargo packages, crate names, module paths and directories: `gridcraft`, `gridcraft-cli`,
  `gridcraft-web`, `gridcraft-*` crates, `apps/gridcraft*` (`cargo -p gridcraft…` keeps working,
  `Cargo.lock` is unchanged).
- The web build (`gridcraft-web` binary; the zip is renamed to `deskwerk-sheets-web-<version>.zip`) and its page messages
  (`gridcraft:ready`, `gridcraft:open`, …), which are a host-page API.
- `GRIDCRAFT_*` environment variables.
- Log targets (`gridcraft_*`; the app binary now logs as `deskwerk_sheets`, which the default
  filter includes), the log file name `gridcraft.log`, temp-file prefixes.
- The control-channel token folder (`$XDG_RUNTIME_DIR/gridcraft`, `~/.cache/gridcraft`,
  `%USERPROFILE%\.gridcraft`), shared by the app and the CLI.
- The `gridcraft-check/1` report schema id of `deskwerk-sheets-cli check`.
- File names `packaging/windows/gridcraft.wxs` and internal WiX ids (`GridcraftExe`, …).
- The XLSX `appName` written into saved files.

## Windows installer identity

- New UpgradeCode `CA81A1E0-D901-45D0-9E16-8859F2B95AF8` (fixed from now on; it does not upgrade
  upstream GridCraft installs). Manufacturer `Deskwerk`, folder `Deskwerk\Sheets`, ProgIds
  `DeskwerkSheets.Workbook` / `DeskwerkSheets.TextData`, registry `Software\Deskwerk\Sheets`.

## Release signing

- `.github/workflows/release.yml` (tag `v*` or manual run) builds Linux, web, Windows and macOS
  packages and drafts a GitHub Release. No signing identity is configured yet, so Windows files
  are unsigned and macOS is ad-hoc signed, not notarized. To sign, add these repository secrets
  (details in `packaging/RELEASING.md`):
  - Windows, either `WINDOWS_CERTIFICATE` (base64 `.pfx` code-signing certificate) +
    `WINDOWS_CERTIFICATE_PASSWORD`, or Azure Trusted Signing: `AZURE_TENANT_ID`,
    `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT`,
    `AZURE_CERT_PROFILE`.
  - macOS signing: `APPLE_CERTIFICATE` (base64 `.p12` "Developer ID Application" certificate with
    key), `APPLE_CERTIFICATE_PASSWORD`, optional `KEYCHAIN_PASSWORD`.
  - macOS notarization: `APPLE_ID`, `APPLE_PASSWORD` (app-specific password), `APPLE_TEAM_ID`.
- Not built by CI yet: Linux aarch64/riscv64, Flatpak, FreeBSD, Windows x86/arm64. No Linux
  package signing (upstream's `LINUX_SIGNING_KEY`); releases carry `SHA256SUMS.txt` only.

## Updates

- No startup update check. Help › Check for Updates… uses craft-update with `UPDATE_APP`
  (`crates/ui-egui/src/lib.rs`): id `deskwerk-sheets`, repo `dicendy/deskwerk-sheets`, so it only
  asks `https://api.github.com/repos/dicendy/deskwerk-sheets/releases`. The AppImage update
  information points at the same releases.
- craft-update (storytold/craft-libs) still assumes ArtCraft conventions when it looks for an
  installed copy to replace: publisher `Learning Machines LLC` for MSIs, bundle id
  `ai.storyteller.<id>` on macOS. So in-place install from the update prompt may not find
  Deskwerk installs and fall back to a download. Fixing it needs a craft-update change (or
  disabling the menu item).
