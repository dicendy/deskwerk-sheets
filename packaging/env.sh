# shellcheck shell=bash
# Shared setup for the packaging scripts. Source it: `. "$(dirname "$0")/../env.sh"`.
#
# Exports:
#   ROOT                    workspace root
#   VERSION                 [workspace.package] version from Cargo.toml (override: DESKWERK_SHEETS_VERSION)
#   DIST                    output directory for release artifacts (default: $ROOT/dist/release)
#   DESKWERK_SHEETS_BUILD_SHA    git commit baked into the binaries (see crates/engine/src/build_info.rs)
#   DESKWERK_SHEETS_BUILD_DATE   UTC build date, YYYY-MM-DD
#   CARGO_TARGET_DIR        cargo's target dir (default: $ROOT/target)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export ROOT

# The version lives in exactly one place: `[workspace.package] version` in the root Cargo.toml.
# (`cargo xtask version` prints the same thing; awk avoids compiling xtask here.)
workspace_version() {
  awk '
    /^\[/ { in_pkg = ($0 == "[workspace.package]") ; next }
    in_pkg && $1 == "version" { gsub(/[" ]/, "", $3); print $3; exit }
  ' "$ROOT/Cargo.toml"
  return
}

VERSION="${DESKWERK_SHEETS_VERSION:-$(workspace_version)}"
if [[ -z "$VERSION" ]]; then
  echo "error: could not read [workspace.package] version from $ROOT/Cargo.toml" >&2
  exit 1
fi
export VERSION

DIST="${DIST:-$ROOT/dist/release}"
mkdir -p "$DIST"
export DIST

if [[ -z "${DESKWERK_SHEETS_BUILD_SHA:-}" ]]; then
  DESKWERK_SHEETS_BUILD_SHA="$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || true)"
fi
export DESKWERK_SHEETS_BUILD_SHA
export DESKWERK_SHEETS_BUILD_DATE="${DESKWERK_SHEETS_BUILD_DATE:-$(date -u +%Y-%m-%d)}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

# Emit a GitHub Actions warning (plain stderr outside Actions).
warn() {
  if [[ -n "${GITHUB_ACTIONS:-}" ]]; then echo "::warning::$*"; else echo "warning: $*" >&2; fi
  return
}

# Copy licence and readme files that exist into a package directory.
copy_docs() {
  local dest="$1" f
  for f in README.md LICENSE LICENSE-MIT LICENSE-APACHE COPYRIGHT NOTICE ATTRIBUTION.md; do
    if [[ -f "$ROOT/$f" ]]; then cp "$ROOT/$f" "$dest/"; fi
  done
  copy_font_licences "$dest"
  return
}

# Every build carries the font pack (apps/deskwerk-sheets-web/fonts: served by the web build, embedded
# in the desktop app), so every package carries its licences in font-licences/ with its manifest.
# Builds made with craft-fonts (CRAFT_FONTS_DIR, set by the release workflow) embed its fonts too,
# so the package also carries their licences as OFL-<family-dir>.txt.
copy_font_licences() {
  local dest="$1" f
  mkdir -p "$dest/font-licences"
  cp "$ROOT"/apps/deskwerk-sheets-web/fonts/licenses/*.txt "$ROOT/apps/deskwerk-sheets-web/fonts/MANIFEST.tsv" "$dest/font-licences/"
  [[ -n "${CRAFT_FONTS_DIR:-}" ]] || return 0
  for f in "$CRAFT_FONTS_DIR"/fonts/*/OFL.txt; do
    if [[ -f "$f" ]]; then cp "$f" "$dest/OFL-$(basename "$(dirname "$f")").txt"; fi
  done
}

# Portable SHA-256 of a file (prints just the hash).
sha256() {
  local file="$1"
  if command -v sha256sum >/dev/null; then sha256sum "$file" | cut -d' ' -f1; else shasum -a 256 "$file" | cut -d' ' -f1; fi
  return
}
