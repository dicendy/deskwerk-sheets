//! The font pack of the web build and the desktop app: open fonts that stand in for the Office fonts a workbook names, plus
//! popular open fonts offered by their own names, and the fonts of the user's computer where the
//! browser lets us read them (Local Font Access).
//!
//! The files live next to the web build's `index.html` in `fonts/` (`apps/deskwerk-sheets-web/fonts`,
//! with `MANIFEST.tsv` naming each file's source, sha256 and licence, and the licence texts in
//! `fonts/licenses/`). Nothing is fetched from a third-party server: the web shell fetches a file
//! from the app's own origin when a workbook (or the user, through the font list) uses a family
//! that needs it, and hands the bytes to [`deliver`]. Only [`EAGER`] is fetched at startup.
//!
//! A substitute is bound under the name the workbook uses: a cell in `Times New Roman` draws with
//! Liberation Serif, while the workbook (and the xlsx it saves) keeps `Times New Roman`. The font
//! list marks such names (see [`substituted`]).
//!
//! Order of preference for a name, best first: an installed font (native, [`crate::fonts`]); a
//! font of this computer read through Local Font Access; a pack family of that name; the name's
//! substitute from [`SUBSTITUTES`]; else the built-in cell roles of [`crate::theme::cell_family`].
//!
//! The desktop app embeds the same files (`apps/deskwerk-sheets/src/font_pack.rs`) and hands them over
//! the same way; there the installed fonts come first and Local Font Access doesn't apply.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use crate::fonts::{Face, bind_faces_with};

mod files;
pub use files::FILES;

#[cfg(test)]
mod tests;

/// One file of the pack, as recorded in `fonts/MANIFEST.tsv`.
#[derive(Debug)]
pub struct PackFile {
    /// File name inside `fonts/`.
    pub file: &'static str,
    /// The file's sha256 (also its cache key in the URL).
    pub sha256: &'static str,
    pub bytes: u64,
    /// SPDX licence expression.
    pub license: &'static str,
}

/// One face of a [`FaceSet`]: the file that draws a cell role.
#[derive(Debug)]
pub struct PackFace {
    pub file: &'static str,
    pub bold: bool,
    pub italic: bool,
}

/// A family the pack can draw, under the name the pack knows it by.
#[derive(Debug)]
pub struct FaceSet {
    pub family: &'static str,
    pub faces: &'static [PackFace],
    /// Offered in the font list under its own name (the popular open fonts). The metric twins and
    /// stand-ins are not: the list offers the Office name they draw for.
    pub listed: bool,
}

/// How close a substitute is to the font it stands in for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Closeness {
    /// Metric-compatible: same advance widths, so text wraps and fits as in the original.
    Twin,
    /// The closest open design; widths differ, so text may wrap differently.
    Similar,
}

/// An Office font name and the pack family that draws it.
#[derive(Debug)]
pub struct Substitute {
    pub name: &'static str,
    pub family: &'static str,
    pub closeness: Closeness,
}

const fn face(file: &'static str, bold: bool, italic: bool) -> PackFace {
    PackFace { file, bold, italic }
}

macro_rules! four {
    ($stem:literal) => {
        &[
            face(concat!($stem, "-Regular.ttf"), false, false),
            face(concat!($stem, "-Bold.ttf"), true, false),
            face(concat!($stem, "-Italic.ttf"), false, true),
            face(concat!($stem, "-BoldItalic.ttf"), true, true),
        ]
    };
}

macro_rules! two {
    ($stem:literal) => {
        &[face(concat!($stem, "-Regular.ttf"), false, false), face(concat!($stem, "-Bold.ttf"), true, false)]
    };
}

const fn set(family: &'static str, faces: &'static [PackFace], listed: bool) -> FaceSet {
    FaceSet { family, faces, listed }
}

/// Every family the pack can draw.
pub const SETS: &[FaceSet] = &[
    // Metric twins and stand-ins for Office fonts.
    set("Carlito", four!("Carlito"), false),
    set("Caladea", four!("Caladea"), false),
    set("Liberation Sans", four!("LiberationSans"), false),
    set("Liberation Serif", four!("LiberationSerif"), false),
    set("Liberation Mono", four!("LiberationMono"), false),
    set("Gelasio", four!("Gelasio"), false),
    set("Selawik", two!("Selawik"), false),
    set("DejaVu Sans", four!("DejaVuSans"), false),
    set("DejaVu Sans Condensed", four!("DejaVuSansCondensed"), false),
    set("DejaVu Sans Mono", four!("DejaVuSansMono"), false),
    set("Fira Sans", four!("FiraSans"), false),
    set("Anton", &[face("Anton-Regular.ttf", false, false)], false),
    set("Inconsolata", two!("Inconsolata"), false),
    set("Didact Gothic", &[face("DidactGothic-Regular.ttf", false, false)], false),
    set("Libre Franklin", four!("LibreFranklin"), false),
    set("PT Sans Narrow", two!("PTSansNarrow"), false),
    // Calibri Light: Lato Light (Carlito is Lato redrawn to Calibri's widths), with Lato's
    // Regular as its bold, as Office draws a bold Calibri Light.
    set(
        "Lato Light",
        &[
            face("Lato-Light.ttf", false, false),
            face("Lato-Regular.ttf", true, false),
            face("Lato-LightItalic.ttf", false, true),
            face("Lato-Italic.ttf", true, true),
        ],
        false,
    ),
    // Popular open fonts, offered under their own names.
    set("Comic Neue", four!("ComicNeue"), true),
    set("EB Garamond", four!("EBGaramond"), true),
    set("Inter", four!("Inter"), true),
    set("Lato", four!("Lato"), true),
    set("Lexend", two!("Lexend"), true),
    set("Lora", four!("Lora"), true),
    set("Merriweather", four!("Merriweather"), true),
    set("Montserrat", four!("Montserrat"), true),
    set("Noto Sans", four!("NotoSans"), true),
    set("Noto Serif", four!("NotoSerif"), true),
    set("Nunito", four!("Nunito"), true),
    set("Open Sans", four!("OpenSans"), true),
    set("Oswald", two!("Oswald"), true),
    set("Playfair Display", four!("PlayfairDisplay"), true),
    set("Poppins", four!("Poppins"), true),
    set("PT Sans", four!("PTSans"), true),
    set("PT Serif", four!("PTSerif"), true),
    set("Raleway", four!("Raleway"), true),
    set("Roboto", four!("Roboto"), true),
    set("Roboto Mono", four!("RobotoMono"), true),
    set("Roboto Slab", two!("RobotoSlab"), true),
    set("Source Sans 3", four!("SourceSans3"), true),
];

const fn twin(name: &'static str, family: &'static str) -> Substitute {
    Substitute { name, family, closeness: Closeness::Twin }
}

const fn similar(name: &'static str, family: &'static str) -> Substitute {
    Substitute { name, family, closeness: Closeness::Similar }
}

/// The Office fonts the pack stands in for, and with what. The one table of these choices; the
/// font list offers every name here.
///
/// | Office font | Drawn with | Closeness |
/// |---|---|---|
/// | Calibri | Carlito | metric twin |
/// | Cambria | Caladea | metric twin |
/// | Arial, Helvetica | Liberation Sans | metric twin |
/// | Times New Roman, Times | Liberation Serif | metric twin |
/// | Courier New, Courier | Liberation Mono | metric twin |
/// | Georgia | Gelasio | metric twin |
/// | Segoe UI | Selawik | metric twin (Latin only, no italics) |
/// | Helvetica Neue, Microsoft Sans Serif | Liberation Sans | similar |
/// | Arial Narrow, Aptos Narrow | PT Sans Narrow | similar (Liberation Sans Narrow is GPL) |
/// | Calibri Light | Lato Light | similar |
/// | Aptos, Aptos Display | Inter | similar |
/// | Verdana, Lucida Sans Unicode | DejaVu Sans | similar |
/// | Tahoma | DejaVu Sans Condensed | similar |
/// | Consolas | Inconsolata | similar |
/// | Lucida Console | DejaVu Sans Mono | similar |
/// | Trebuchet MS | Fira Sans | similar |
/// | Comic Sans MS | Comic Neue | similar |
/// | Impact | Anton | similar |
/// | Garamond | EB Garamond | similar |
/// | Palatino Linotype, Book Antiqua, Palatino | Lora | similar |
/// | Century Gothic | Didact Gothic | similar |
/// | Candara | PT Sans | similar |
/// | Constantia | PT Serif | similar |
/// | Corbel | Source Sans 3 | similar |
/// | Franklin Gothic Book, Franklin Gothic Medium | Libre Franklin | similar |
pub const SUBSTITUTES: &[Substitute] = &[
    twin("Calibri", "Carlito"),
    twin("Cambria", "Caladea"),
    twin("Arial", "Liberation Sans"),
    twin("Helvetica", "Liberation Sans"),
    twin("Times New Roman", "Liberation Serif"),
    twin("Times", "Liberation Serif"),
    twin("Courier New", "Liberation Mono"),
    twin("Courier", "Liberation Mono"),
    twin("Georgia", "Gelasio"),
    twin("Segoe UI", "Selawik"),
    similar("Helvetica Neue", "Liberation Sans"),
    similar("Microsoft Sans Serif", "Liberation Sans"),
    similar("Arial Narrow", "PT Sans Narrow"),
    similar("Aptos Narrow", "PT Sans Narrow"),
    similar("Calibri Light", "Lato Light"),
    similar("Aptos", "Inter"),
    similar("Aptos Display", "Inter"),
    similar("Verdana", "DejaVu Sans"),
    similar("Lucida Sans Unicode", "DejaVu Sans"),
    similar("Tahoma", "DejaVu Sans Condensed"),
    similar("Consolas", "Inconsolata"),
    similar("Lucida Console", "DejaVu Sans Mono"),
    similar("Trebuchet MS", "Fira Sans"),
    similar("Comic Sans MS", "Comic Neue"),
    similar("Impact", "Anton"),
    similar("Garamond", "EB Garamond"),
    similar("Palatino Linotype", "Lora"),
    similar("Book Antiqua", "Lora"),
    similar("Palatino", "Lora"),
    similar("Century Gothic", "Didact Gothic"),
    similar("Candara", "PT Sans"),
    similar("Constantia", "PT Serif"),
    similar("Corbel", "Source Sans 3"),
    similar("Franklin Gothic Book", "Libre Franklin"),
    similar("Franklin Gothic Medium", "Libre Franklin"),
];

/// The family fetched as the editor starts: Calibri's twin, as Calibri is the default font of a new
/// workbook and of most files. Everything else is fetched when a workbook uses it.
pub const EAGER: &str = "Carlito";

/// The tooltip of a substituted name in the font list (`{name}` is the name).
pub const SUBSTITUTE_TIP: &str = "Shown with a similar font; the file keeps “{name}”.";

/// The font list's row that reads the computer's fonts through Local Font Access.
pub const USE_LOCAL_FONTS: &str = "Use this computer's fonts";

/// A local font file larger than this is not read (wasm memory is small).
pub const MAX_LOCAL_FONT_BYTES: usize = 64 << 20;

/// Most local faces kept in the list (a designer's machine has thousands).
pub const MAX_LOCAL_FACES: usize = 4000;

/// A face of a font installed on the user's computer, as Local Font Access lists it. Its bytes are
/// read only when a workbook uses its family.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalFace {
    pub family: String,
    /// Style name, e.g. `"Bold Italic"`.
    pub style: String,
    /// The browser's handle for the face (its PostScript name).
    pub id: String,
}

/// A file the web shell should fetch and [`deliver`] (or report [`failed`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fetch {
    /// `fonts/<file>?v=<rev>` next to the app.
    Pack { key: String, file: &'static str, rev: &'static str },
    /// A local face's bytes, by [`LocalFace::id`].
    Local { key: String, id: String },
}

impl Fetch {
    /// The key to [`deliver`] the bytes under.
    pub fn key(&self) -> &str {
        match self {
            Self::Pack { key, .. } | Self::Local { key, .. } => key,
        }
    }
}

#[derive(Default)]
struct Registry {
    enabled: bool,
    loaded: HashMap<String, egui::epaint::text::Blob>,
    asked: HashSet<String>,
    queue: Vec<Fetch>,
    generation: u64,
    local_api: bool,
    local: Option<Vec<LocalFace>>,
    local_access_wanted: bool,
    list: Option<((u64, bool), FontList)>,
}

fn registry() -> std::sync::MutexGuard<'static, Registry> {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Registry::default())).lock().unwrap_or_else(PoisonError::into_inner)
}

fn pack_key(file: &str) -> String {
    format!("pack/{file}")
}

fn local_key(id: &str) -> String {
    format!("local/{id}")
}

/// The pack file named `file`.
pub fn pack_file(file: &str) -> Option<&'static PackFile> {
    FILES.iter().find(|f| f.file == file)
}

/// The pack family named `family`.
pub fn face_set(family: &str) -> Option<&'static FaceSet> {
    SETS.iter().find(|s| s.family.eq_ignore_ascii_case(family))
}

/// The substitute for an Office font name, by any capitalisation.
pub fn substitute(name: &str) -> Option<&'static Substitute> {
    SUBSTITUTES.iter().find(|s| s.name.eq_ignore_ascii_case(name.trim()))
}

/// What the pack draws `name` with: a listed family of that name, else the name's substitute.
pub fn set_for(name: &str) -> Option<&'static FaceSet> {
    let name = name.trim();
    SETS.iter().find(|s| s.listed && s.family.eq_ignore_ascii_case(name)).or_else(|| substitute(name).and_then(|s| face_set(s.family)))
}

/// Turn the pack on (the web shell does, at startup) and queue [`EAGER`].
pub fn enable() {
    let mut reg = registry();
    if reg.enabled {
        return;
    }
    reg.enabled = true;
    reg.generation += 1;
    if let Some(set) = face_set(EAGER) {
        for f in set.faces {
            reg.ask_pack(f.file);
        }
    }
}

/// Turn the pack off and forget everything it loaded (tests).
pub fn reset() {
    let generation = registry().generation;
    *registry() = Registry { generation: generation + 1, ..Registry::default() };
}

/// Whether the pack is in use (the web build).
pub fn enabled() -> bool {
    registry().enabled
}

/// Bumped whenever a delivery or a local font list changes what the pack can bind, so the app
/// knows to rebuild its fonts.
pub fn generation() -> u64 {
    registry().generation
}

/// The files the web shell should fetch now. Each is handed out once.
pub fn take_fetches() -> Vec<Fetch> {
    std::mem::take(&mut registry().queue)
}

/// The bytes of a fetched file. Data that isn't a font is dropped, as if the fetch had failed.
pub fn deliver(key: &str, bytes: Vec<u8>) {
    if !crate::theme::font_parses(&bytes, 0) {
        log::warn!("font pack: {key} is not a usable font");
        return;
    }
    let mut reg = registry();
    reg.loaded.insert(key.to_string(), Arc::new(bytes));
    reg.generation += 1;
}

/// A fetch that failed: the file is not asked for again this session.
pub fn failed(key: &str) {
    log::warn!("font pack: couldn't load {key}");
}

/// Whether the file under `key` has arrived.
pub fn is_loaded(key: &str) -> bool {
    registry().loaded.contains_key(key)
}

/// Whether the browser offers Local Font Access (the web shell sets this).
pub fn set_local_api(available: bool) {
    registry().local_api = available;
}

/// The faces Local Font Access listed. Their families join the font list, and win over the pack.
pub fn set_local_fonts(mut faces: Vec<LocalFace>) {
    faces.truncate(MAX_LOCAL_FACES);
    let mut reg = registry();
    reg.local = Some(faces);
    reg.generation += 1;
}

/// Whether the font list offers to read this computer's fonts: the browser can, and they aren't
/// read yet.
pub fn local_access_offered() -> bool {
    let reg = registry();
    reg.local_api && reg.local.is_none()
}

/// The user asked to read this computer's fonts (the font list's row).
pub fn request_local_access() {
    registry().local_access_wanted = true;
}

/// Whether the user asked to read this computer's fonts since the last call (the web shell asks
/// the browser, within the click's user activation).
pub fn take_local_access_request() -> bool {
    std::mem::take(&mut registry().local_access_wanted)
}

/// Whether `name` is drawn by a stand-in, so the font list marks it: the pack is on, it has a
/// substitute for `name`, and no font of this computer goes by it.
pub fn substituted(name: &str) -> bool {
    let reg = registry();
    reg.enabled && substitute(name).is_some() && !reg.has_local(name) && !installed(name)
}

/// Whether the desktop app's catalog of installed fonts has `name` (which then draws itself, see
/// [`bind_used`]'s `installed`). Never on the web, whose computer fonts are [`set_local_fonts`]'s,
/// nor in unit tests, which must not depend on the test machine's fonts.
fn installed(name: &str) -> bool {
    #[cfg(any(test, target_arch = "wasm32"))]
    {
        let _ = name;
        false
    }
    #[cfg(not(any(test, target_arch = "wasm32")))]
    {
        !crate::fonts::installed_among(&[name.trim().to_string()]).is_empty()
    }
}

/// [`font_list`]'s rows: a font name, and whether [`substituted`] marks it.
pub type FontList = Arc<Vec<(String, bool)>>;

/// The font list: the Office and open font names ([`crate::ribbon::FONTS`]), then the families of
/// this computer's fonts once they are read, A–Z; each with whether [`substituted`] marks it.
/// Cached until the next [`generation`].
pub fn font_list() -> FontList {
    let mut reg = registry();
    // The desktop catalog of installed fonts may finish after the list was first built.
    let key = (reg.generation, crate::fonts::scan_done());
    if let Some((built, list)) = &reg.list
        && *built == key
    {
        return Arc::clone(list);
    }
    let mut names: Vec<String> = crate::ribbon::FONTS.iter().map(|s| s.to_string()).collect();
    if let Some(local) = &reg.local {
        let mut seen: HashSet<String> = names.iter().map(|n| n.to_lowercase()).collect();
        let mut extra: Vec<String> = local.iter().filter(|f| seen.insert(f.family.to_lowercase())).map(|f| f.family.clone()).collect();
        extra.sort_by_key(|n| n.to_lowercase());
        names.extend(extra);
    }
    let list: Vec<(String, bool)> =
        names.into_iter().map(|n| (n.clone(), reg.enabled && substitute(&n).is_some() && !reg.has_local(&n) && !installed(&n))).collect();
    let list = Arc::new(list);
    reg.list = Some((key, Arc::clone(&list)));
    list
}

impl Registry {
    fn has_local(&self, name: &str) -> bool {
        self.local.as_ref().is_some_and(|l| l.iter().any(|f| f.family.eq_ignore_ascii_case(name.trim())))
    }

    fn ask_pack(&mut self, file: &'static str) {
        let key = pack_key(file);
        if self.loaded.contains_key(&key) || !self.asked.insert(key.clone()) {
            return;
        }
        let rev = pack_file(file).map_or("", |f| f.sha256.get(..16).unwrap_or(f.sha256));
        self.queue.push(Fetch::Pack { key, file, rev });
    }

    fn ask_local(&mut self, id: &str) {
        let key = local_key(id);
        if self.loaded.contains_key(&key) || !self.asked.insert(key.clone()) {
            return;
        }
        self.queue.push(Fetch::Local { key, id: id.to_string() });
    }

    /// The faces of the pack family for `name`, as [`Face`]s keyed `pack/<file>`.
    fn pack_faces(name: &str) -> Vec<Face> {
        let Some(set) = set_for(name) else { return Vec::new() };
        set.faces
            .iter()
            .map(|f| Face {
                family: set.family.to_string(),
                names: vec![set.family.to_string()],
                path: pack_key(f.file).into(),
                index: 0,
                weight: if f.bold { 700.0 } else { 400.0 },
                italic: f.italic,
            })
            .collect()
    }

    /// The local faces of `name` that draw its four roles, as [`Face`]s keyed `local/<id>`.
    fn local_faces(&self, name: &str) -> Vec<Face> {
        let Some(local) = &self.local else { return Vec::new() };
        let all: Vec<Face> = local
            .iter()
            .filter(|f| f.family.eq_ignore_ascii_case(name.trim()))
            .map(|f| Face {
                family: f.family.clone(),
                names: vec![f.family.clone()],
                path: local_key(&f.id).into(),
                index: 0,
                weight: crate::fonts::style_weight(&f.style),
                italic: crate::fonts::style_italic(&f.style),
            })
            .collect();
        crate::fonts::role_faces(&all)
    }

    /// Bind `faces` under `name` once the regular face has arrived; ask for every face not here yet.
    fn bind_or_ask(&mut self, fonts: &mut egui::FontDefinitions, name: &str, faces: &[Face]) -> Option<Vec<String>> {
        for f in faces {
            let key = f.path.to_string_lossy().to_string();
            if let Some(file) = key.strip_prefix("pack/").and_then(pack_file) {
                self.ask_pack(file.file);
            } else if let Some(id) = key.strip_prefix("local/") {
                self.ask_local(id);
            }
        }
        let here: Vec<Face> = faces.iter().filter(|f| self.loaded.contains_key(&*f.path.to_string_lossy())).cloned().collect();
        let regular = crate::fonts::role_faces(faces).into_iter().next();
        let regular_here = regular.is_some_and(|r| here.iter().any(|h| h.path == r.path));
        if !regular_here {
            return None;
        }
        let loaded = &self.loaded;
        let load = |f: &Face| loaded.get(&*f.path.to_string_lossy()).map(|b| egui::FontData::from_blob(Arc::clone(b), f.index));
        Some(bind_faces_with(fonts, name, &here, &load))
    }
}

/// Bind the pack and local faces for every name in `used` that isn't in `installed`, under the
/// name as the workbook spells it, and ask the web shell for the files that haven't arrived. A name
/// waits on the built-in roles until its regular face is here. Returns the bound role keys. Does
/// nothing while the pack is off.
pub fn bind_used(fonts: &mut egui::FontDefinitions, used: &[String], installed: &[String]) -> Vec<String> {
    let mut reg = registry();
    if !reg.enabled {
        return Vec::new();
    }
    let mut bound: Vec<String> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    for name in used {
        if name.trim().is_empty() || installed.contains(name) || !seen.insert(name.as_str()) {
            continue;
        }
        let local = reg.local_faces(name);
        if !local.is_empty()
            && let Some(keys) = reg.bind_or_ask(fonts, name, &local)
        {
            bound.extend(keys);
            continue;
        }
        let pack = Registry::pack_faces(name);
        if !pack.is_empty()
            && let Some(keys) = reg.bind_or_ask(fonts, name, &pack)
        {
            bound.extend(keys);
        }
    }
    bound
}
