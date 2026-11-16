//! Emit `craft_fonts.rs`: the optional craft-fonts faces embedded for the fallback chains
//! (`emit_craft_fonts`). Nothing is read at run time.

use std::fmt::Write as _;

#[path = "src/craft_fonts_policy.rs"]
mod craft_fonts_policy;

/// Build script: emits `craft_fonts.rs`; a missing optional input leaves the table empty, while
/// required mode reports build errors.
fn main() {
    emit_craft_fonts();
}

/// The optional craft-fonts build input (<https://github.com/storytold/craft-fonts>, recipe from
/// its `docs/integration.md`). With `CRAFT_FONTS_DIR=<absolute checkout>` the permitted fonts in
/// its `fonts/manifest.txt` are embedded as `CRAFT_FONTS` for `theme`'s fallback chains; unset,
/// the table is empty. Release and acceptance builds also set `CRAFT_FONTS_REQUIRED=1` (exactly
/// `1`; any other value leaves required mode off), which turns a missing checkout, a missing
/// required face or a malformed row into a build error.
///
/// Deskwerk Sheets never copies font bytes into this repository and never reads a fixture from Cargo
/// configuration: the only input is the environment variable. Barred Adobe/Source/Noto CJK faces
/// stay in the shared repository for other consumers but are filtered here. A web build embeds
/// only the explicit `WEB_ALLOWED` faces (no CJK collection). Native builds additionally opt into
/// craft-fonts' `fonts/simplified-chinese-manifest.txt` (the OFL LXGW face DeckCraft uses): it is
/// optional for ordinary builds and required for `CRAFT_FONTS_REQUIRED=1` native builds.
fn emit_craft_fonts() {
    println!("cargo::rerun-if-env-changed=CRAFT_FONTS_DIR");
    println!("cargo::rerun-if-env-changed=CRAFT_FONTS_REQUIRED");
    let required = craft_fonts_policy::required_mode(std::env::var("CRAFT_FONTS_REQUIRED").ok().as_deref());
    let wasm = std::env::var("CARGO_CFG_TARGET_ARCH").is_ok_and(|a| a == "wasm32");
    let mut src = String::new();
    let _ = writeln!(src, "pub static CRAFT_FONTS_REQUIRED: bool = {required};");
    src.push_str("pub static CRAFT_FONTS: &[CraftFont] = &[\n");
    match std::env::var_os("CRAFT_FONTS_DIR").map(std::path::PathBuf::from) {
        Some(dir) => match craft_fonts(&dir, wasm, required) {
            Ok(entries) => src.push_str(&entries),
            Err(e) if required => {
                println!("cargo::error=CRAFT_FONTS_DIR={}: {e}", dir.display());
            }
            Err(e) => println!("cargo::warning=building without craft-fonts: CRAFT_FONTS_DIR={}: {e}", dir.display()),
        },
        None if required => {
            println!("cargo::error=CRAFT_FONTS_REQUIRED=1 requires CRAFT_FONTS_DIR (an absolute craft-fonts checkout)");
        }
        None => {}
    }
    src.push_str("];\n");
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default()).join("craft_fonts.rs");
    if let Err(e) = std::fs::write(&out, src) {
        println!("cargo::error=writing {}: {e}", out.display());
    }
}

/// One `CraftFont { .. }` initialiser per manifest row Deskwerk Sheets may embed.
fn craft_fonts(dir: &std::path::Path, wasm: bool, required: bool) -> Result<String, String> {
    let manifest = dir.join("fonts/manifest.txt");
    println!("cargo::rerun-if-changed={}", manifest.display());
    let text = std::fs::read_to_string(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
    let rows = craft_fonts_policy::parse_manifest(&text)?;
    craft_fonts_policy::check_declared_scripts(&rows)?;
    if required {
        craft_fonts_policy::check_required(&rows)?;
    }
    let chinese = if wasm { Vec::new() } else { chinese_font_rows(dir, required)? };
    for row in rows.iter().chain(&chinese) {
        if craft_fonts_policy::barred(&row.family) {
            println!("cargo::warning=craft-fonts: not embedding {} {} (repository asset policy)", row.family, row.style);
        }
    }
    let mut out = String::new();
    for row in craft_fonts_policy::embeddable(&rows, wasm).into_iter().chain(craft_fonts_policy::embeddable(&chinese, wasm)) {
        let path = dir.join(&row.file).canonicalize().map_err(|e| format!("{}: {e}", row.file))?;
        println!("cargo::rerun-if-changed={}", path.display());
        let scripts: Vec<String> = row.scripts.iter().map(|s| format!("{s:?}")).collect();
        let family = &row.family;
        let style = &row.style;
        let _ = writeln!(
            out,
            "    CraftFont {{ family: {family:?}, style: {style:?}, scripts: &[{}], bytes: include_bytes!({path:?}) }},",
            scripts.join(", "),
        );
    }
    Ok(out)
}

/// The opt-in native Simplified Chinese rows. Older craft-fonts checkouts may lack the manifest, so
/// an ordinary build warns and carries on; a required acceptance build fails rather than passing
/// Chinese coverage only because the face was silently absent.
fn chinese_font_rows(dir: &std::path::Path, required: bool) -> Result<Vec<craft_fonts_policy::ManifestRow>, String> {
    let manifest = dir.join("fonts/simplified-chinese-manifest.txt");
    println!("cargo::rerun-if-changed={}", manifest.display());
    let optional = |e: String| -> Result<Vec<craft_fonts_policy::ManifestRow>, String> {
        if required {
            Err(format!("{}: {e}", manifest.display()))
        } else {
            println!("cargo::warning=skipping optional Chinese fonts: {}: {e}", manifest.display());
            Ok(Vec::new())
        }
    };
    match std::fs::read_to_string(&manifest) {
        Ok(text) => match craft_fonts_policy::parse_manifest(&text).and_then(|rows| {
            craft_fonts_policy::check_declared_scripts(&rows)?;
            craft_fonts_policy::check_chinese_required(&rows)?;
            Ok(rows)
        }) {
            Ok(rows) => Ok(rows),
            Err(e) => optional(e),
        },
        Err(e) => optional(e.to_string()),
    }
}
