# Deskwerk Sheets: rebranding leftovers

Deskwerk Sheets is based on GridCraft by the ArtCraft team (MIT OR Apache-2.0). The user-facing
name, app id (`eu.deskwerk.Sheets`), binaries (`deskwerk-sheets`, `deskwerk-sheets-cli`),
packages, installers and settings folders are renamed. This file lists what still carries the
upstream look or name, and why.

## Art

The upstream placeholder icons are replaced by Deskwerk art (app icon, document icon, accent
`#12875C`; see `assets/app-icon/README.md`). Nothing left to replace.

## macOS DMG layout

- `packaging/macos/dmg/dmg-layout.DS_Store` still describes a volume named `GridCraft` with
  `GridCraft.app`. The volume and bundle are now `Deskwerk Sheets` / `Deskwerk Sheets.app`, so
  Finder falls back to default icon positions until it is regenerated with
  `packaging/macos/dmg/generate.py` (already updated) on a Mac.
- `packaging/macos/dmg/background.svg` / `background.tiff` show the new icon, but the
  background text still reads `GridCraft`; redraw it (outlined text) to say Deskwerk Sheets.

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

## Charts

Gaps closed on the `charts-2` branch (details in `docs/chart-parity.md`): a kept XLSX chart with a
new title, legend position or type within column/bar/line/area is patched in place instead of
rebuilt; surface charts are kept byte for byte (shown as lines); pie/doughnut percentage labels;
a logarithmic primary value axis; secondary line/area/column groups keep their kind. Still open:

- `ChartAxis.log_base` and `Chart.data_label_percent` are set from the Format Chart pane (Value
  Axis ▸ Logarithmic scale, Elements ▸ Show percentage) and `chart.set {logBase, dataLabelPercent}`;
  the other value-axis options (bounds, major unit, number format) still have no UI.
- Other edits of a kept chart (colours, axis options, data labels, combo or pie type changes)
  still write the chart from the model and lose its formatting.
- Log scale is drawn on the primary axis of category charts only; scatter/bubble and the
  secondary axis stay linear, and columns on a log axis rise from the bottom of the plot rather
  than from the crossing point.
- Surface charts are drawn as lines; there is no surface/contour renderer.
- Not checked in Excel: the patched XML is validated by reading it back, not by opening it in
  Excel.
