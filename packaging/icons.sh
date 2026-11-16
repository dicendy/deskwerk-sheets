#!/usr/bin/env bash
# Install a delivered icon set into assets/app-icon/ (the art is raster; nothing is rendered here).
#
#   packaging/icons.sh <folder>
#
# <folder> holds: app.ico (16-256 px), app.icns, app-1024.png (full tile, transparent corners),
# app-macos-512.png (on Apple's grid), hicolor/<N>x<N>.png for N in 16 24 32 48 64 128 256 512,
# file.ico and file.svg (the document icon). Needs python3 with Pillow for the web manifest
# icons. The outputs are committed, so builds and packaging never need this script.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/assets/app-icon"
ID="eu.deskwerk.Sheets"
SRC="${1:?usage: packaging/icons.sh <folder>}"

cp "$SRC/app.ico" "$DIR/deskwerk-sheets.ico"
cp "$SRC/app.icns" "$DIR/deskwerk-sheets.icns"
cp "$SRC/app-1024.png" "$DIR/deskwerk-sheets-1024.png"
cp "$SRC/app-macos-512.png" "$DIR/deskwerk-sheets-macos-512.png"
for s in 16 24 32 48 64 128 256 512; do
  mkdir -p "$DIR/hicolor/${s}x${s}/apps"
  cp "$SRC/hicolor/${s}x${s}.png" "$DIR/hicolor/${s}x${s}/apps/$ID.png"
done
cp "$SRC/file.ico" "$DIR/document.ico"
mkdir -p "$DIR/hicolor/scalable/mimetypes"
cp "$SRC/file.svg" "$DIR/hicolor/scalable/mimetypes/$ID.document.svg"
mkdir -p "$DIR/web"
python3 -I -c '
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert("RGBA")
for s in (192, 512):
    im.resize((s, s), Image.Resampling.LANCZOS).save(f"{sys.argv[2]}/icon-{s}.png", optimize=True)
' "$DIR/deskwerk-sheets-1024.png" "$DIR/web"
# document.icns: render file.svg at 16-1024 px (any SVG renderer) and pack the PNGs with Pillow
# or iconutil; not done here.
echo "warning: document.icns not regenerated (render $SRC/file.svg and pack it by hand)" >&2
echo "icons written to $DIR; update the DMG background with packaging/macos/dmg/generate.py"
