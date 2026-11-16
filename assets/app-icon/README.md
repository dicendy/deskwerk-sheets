# Deskwerk Sheets app and document icons

The Deskwerk Sheets icon: a green tile with a white sheet (formula bar with *fx*, a grid with a
tinted header row and column, a selected cell) and a round **S** badge. It replaces the
placeholder ledger page inherited from upstream GridCraft. It was made for Deskwerk Sheets as
AI-assisted raster art, chosen and approved by the owner; the 16 and 24 px sizes are drawn by
hand. It is deliberately not modelled on any vendor's spreadsheet logo (no "X", no letter tile
over layered panels).

The document icon (files the app opens) is hand-written vector geometry: a white page with a
green folded corner and a green band carrying the **S**.

## Colour

App colour `#12875C` (the icon's green), ink `#0C6845` for text on light backgrounds, tint
`#D6F2E4`, and `#5CCB9A` in the dark theme. These are the UI accent tokens in
`crates/ui-egui/src/theme.rs` and the metainfo `<branding>`. The green is shifted off Excel's
`#107C41`.

## Files

| File | Use |
|---|---|
| `gridcraft-1024.png` | 1024 px master of the tile (transparent corners); the DMG background shows it |
| `gridcraft-macos-512.png` | Runtime Dock icon on macOS, on Apple's grid (embedded by `apps/gridcraft`) |
| `gridcraft.icns` | macOS bundle icon (`CFBundleIconFile`) |
| `gridcraft.ico` | Windows exe and MSI icon, 16–256 px (embedded by `apps/gridcraft/build.rs`; MSI `ARPPRODUCTICON` and shortcuts; web favicon) |
| `hicolor/<size>/apps/eu.deskwerk.Sheets.png` | Linux icon theme, 16–512 px; the 256 px one is also the runtime window icon on Windows and Linux, the 128 px one the update prompt's |
| `web/icon-192.png`, `web/icon-512.png` | Web app manifest icons (`apps/gridcraft-web/manifest.webmanifest`) |
| `hicolor/scalable/mimetypes/eu.deskwerk.Sheets.document.svg` | Document icon, source and Linux mime icon (`packaging/linux/eu.deskwerk.Sheets.mime.xml`) |
| `document.ico` | Document icon for Windows: installed as `deskwerk-sheets-document.ico`, the ProgIds' `DefaultIcon` |
| `document.icns` | Document icon for macOS: `Deskwerk Sheets Document.icns`, every `CFBundleTypeIconFile` |
| `LICENSE.txt` | MIT OR Apache-2.0 |

There is no scalable SVG of the app icon (the art is raster), so Linux gets the hicolor PNGs only.

The app ID is `eu.deskwerk.Sheets`. It's used for the hicolor icon names, the Wayland app
ID and `packaging/linux/eu.deskwerk.Sheets.desktop`.

## Replace

The files are committed; builds and packaging never generate them. To replace the art, put a new
icon set in a folder and run `packaging/icons.sh <folder>` (see the script for the file names),
then update this README and the rows in `ATTRIBUTION.md`.
