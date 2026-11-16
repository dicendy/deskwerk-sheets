# Deskwerk Sheets: rebranding leftovers

Deskwerk Sheets is a fork of an upstream open-source spreadsheet (MIT OR Apache-2.0; the upstream
copyright and licence notices are in `LICENSE-MIT`, `NOTICE` and `ATTRIBUTION.md`). The user-facing
name, app id (`eu.deskwerk.Sheets`), binaries (`deskwerk-sheets`, `deskwerk-sheets-cli`),
packages, installers and settings folders are renamed. This file lists what is still open.

## Art

The upstream placeholder icons are replaced by Deskwerk art (app icon, document icon, accent
`#12875C`; see `assets/app-icon/README.md`). The macOS DMG background reads "Deskwerk Sheets" and
`dmg-layout.DS_Store` describes the `Deskwerk Sheets` volume with `Deskwerk Sheets.app` (0.16.4).

## Names (0.16.4)

Every upstream name is gone from the code, packaging and docs: Cargo packages and crates
(`deskwerk-sheets`, `deskwerk-sheets-cli`, `deskwerk-sheets-web`, `deskwerk-sheets-*`), directories
(`apps/deskwerk-sheets*`), `DESKWERK_SHEETS_*` environment variables, log targets and the log file
`deskwerk-sheets.log`, the control-token folder, the web page API (`window.deskwerk_sheets`,
`deskwerk-sheets:ready` …, canvas `#deskwerk_sheets_canvas`), MCP resource URIs
(`deskwerk-sheets://…`), the XLSX `appName`, WiX ids and file names. Upstream names remain only in
the legal files (`LICENSE-MIT`, `NOTICE`, `ATTRIBUTION.md`) and in the compatibility constants of
`deskwerk_sheets_core::legacy`, which read data written by the upstream builds: the old settings
folder is copied on first start, web settings are read from the old browser storage keys, and the
old equation marker in XLSX drawings and the old Solver option names are still read.

## About and licences

The About window is empty for now (its title and a Close button). The app shows no licence or
credits: `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, `ATTRIBUTION.md` and the font licences ship as
plain files in every package (MSI install folder, macOS `Contents/Resources`, Linux
`/usr/share/doc/deskwerk-sheets`, flatpak, and the web build's static files).

## Windows installer identity

- UpgradeCode `CA81A1E0-D901-45D0-9E16-8859F2B95AF8` (fixed; every Deskwerk Sheets MSI upgrades the
  previous one in place). Manufacturer `Deskwerk`, folder `Deskwerk\Sheets`, ProgIds
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
- craft-update (storytold/craft-libs) still assumes upstream conventions when it looks for an
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
