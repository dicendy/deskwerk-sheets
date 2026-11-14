//! Fonts for the web build: the font pack's files from the app's own origin (`fonts/` next to
//! `index.html`, see [`gridcraft_ui_egui::font_pack`]) and, where the browser offers Local Font
//! Access, the fonts installed on the user's computer. Nothing is fetched from another server.
//!
//! The pack starts with Calibri's twin; every other file is fetched when a workbook uses its
//! family. Local fonts are listed at startup when the permission is already granted, else when the
//! user picks "Use this computer's fonts" in the font list; a face's bytes are read only when a
//! workbook uses its family.
//!
//! The Local Font Access part follows Deskwerk Docs' `load_system_fonts` (MIT OR Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;

use gridcraft_ui_egui::font_pack::{self, Fetch, LocalFace};
use wasm_bindgen::{JsCast as _, JsValue};
use wasm_bindgen_futures::JsFuture;

thread_local! {
    /// Local Font Access's `FontData` objects by PostScript name, to read a face's bytes later.
    static HANDLES: RefCell<HashMap<String, JsValue>> = RefCell::new(HashMap::new());
    /// Where `fonts/` lives, found once.
    static BASE: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn get(v: &JsValue, key: &str) -> Option<JsValue> {
    js_sys::Reflect::get(v, &JsValue::from_str(key)).ok().filter(|x| !x.is_undefined() && !x.is_null())
}

fn get_string(v: &JsValue, key: &str) -> Option<String> {
    get(v, key).and_then(|s| s.as_string())
}

/// Turns the pack on, and lists this computer's fonts straight away when the browser has already
/// been allowed to.
pub fn start(ctx: &egui::Context) {
    font_pack::enable();
    let Some(window) = web_sys::window() else { return };
    let api = get(&window, "queryLocalFonts").is_some_and(|f| f.is_function());
    font_pack::set_local_api(api);
    if api {
        list_local_if_granted(ctx.clone());
    }
    ctx.request_repaint();
}

/// Each frame: ask for this computer's fonts if the user just chose to, and fetch the files the
/// pack wants.
pub fn pump(ctx: &egui::Context) {
    if font_pack::take_local_access_request() {
        // Called now, within the click's user activation, which the browser requires.
        list_local(ctx.clone());
    }
    for fetch in font_pack::take_fetches() {
        let ctx = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let bytes = match &fetch {
                Fetch::Pack { file, rev, .. } => fetch_pack(file, rev).await,
                Fetch::Local { id, .. } => read_local(id).await,
            };
            match bytes {
                Some(bytes) => font_pack::deliver(fetch.key(), bytes),
                None => font_pack::failed(fetch.key()),
            }
            ctx.request_repaint();
        });
    }
}

/// The URL `fonts/` is under: the canvas's `data-fonts-base` when the page sets it (a page hosting
/// the app in its own document), else the folder the app's script came from, else the page's.
fn base() -> String {
    if let Some(b) = BASE.with(|b| b.borrow().clone()) {
        return b;
    }
    let found = find_base().unwrap_or_else(|| "./".to_string());
    BASE.with(|b| *b.borrow_mut() = Some(found.clone()));
    found
}

fn find_base() -> Option<String> {
    let document = web_sys::window()?.document()?;
    if let Some(set) = document.get_element_by_id(crate::web::CANVAS_ID).and_then(|c| c.get_attribute("data-fonts-base")) {
        return Some(if set.ends_with('/') { set } else { format!("{set}/") });
    }
    let el = document.query_selector("link[href*='gridcraft-web'], script[src*='gridcraft-web']").ok()??;
    let el = JsValue::from(el);
    let url = get_string(&el, "href").filter(|u| !u.is_empty()).or_else(|| get_string(&el, "src"))?;
    url.rfind('/').map(|i| url[..=i].to_string())
}

/// A pack file's bytes from `fonts/<file>?v=<rev>`.
async fn fetch_pack(file: &str, rev: &str) -> Option<Vec<u8>> {
    let window = web_sys::window()?;
    let url = format!("{}fonts/{file}?v={rev}", base());
    let resp = JsFuture::from(window.fetch_with_str(&url)).await.ok()?;
    let resp: web_sys::Response = resp.dyn_into().ok()?;
    if !resp.ok() {
        log::warn!("font pack: {url} answered {}", resp.status());
        return None;
    }
    let buffer = JsFuture::from(resp.array_buffer().ok()?).await.ok()?;
    Some(js_sys::Uint8Array::new(&buffer).to_vec())
}

/// A local face's bytes, by PostScript name. A face past the size cap is not read.
async fn read_local(id: &str) -> Option<Vec<u8>> {
    let handle = HANDLES.with(|h| h.borrow().get(id).cloned())?;
    let blob_fn = get(&handle, "blob").filter(JsValue::is_function)?;
    let promise = js_sys::Function::from(blob_fn).call0(&handle).ok()?;
    let blob: web_sys::Blob = JsFuture::from(js_sys::Promise::from(promise)).await.ok()?.dyn_into().ok()?;
    if blob.size() > font_pack::MAX_LOCAL_FONT_BYTES as f64 {
        log::warn!("local font {id} is too large to read");
        return None;
    }
    let buffer = JsFuture::from(blob.array_buffer()).await.ok()?;
    let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
    (bytes.len() <= font_pack::MAX_LOCAL_FONT_BYTES).then_some(bytes)
}

/// Lists this computer's fonts if the `local-fonts` permission is already granted (no prompt).
fn list_local_if_granted(ctx: egui::Context) {
    wasm_bindgen_futures::spawn_local(async move {
        let Some(window) = web_sys::window().map(JsValue::from) else { return };
        let Some(permissions) = get(&window, "navigator").and_then(|n| get(&n, "permissions")).filter(JsValue::is_object) else { return };
        let Some(query) = get(&permissions, "query").filter(JsValue::is_function) else { return };
        let what = js_sys::Object::new();
        if js_sys::Reflect::set(&what, &JsValue::from_str("name"), &JsValue::from_str("local-fonts")).is_err() {
            return;
        }
        // Browsers that don't know the permission reject the query: nothing to list then.
        let Ok(promise) = js_sys::Function::from(query).call1(&permissions, &what) else { return };
        let Ok(status) = JsFuture::from(js_sys::Promise::from(promise)).await else { return };
        if get_string(&status, "state").as_deref() == Some("granted") {
            list_local(ctx);
        }
    });
}

/// Asks the browser for this computer's fonts (`queryLocalFonts()`, which may prompt) and hands
/// their names to the pack. A refusal leaves the font list's offer in place.
fn list_local(ctx: egui::Context) {
    let Some(window) = web_sys::window() else { return };
    let Some(query) = get(&window, "queryLocalFonts").filter(JsValue::is_function) else { return };
    let promise = match js_sys::Function::from(query).call0(&window) {
        Ok(p) => js_sys::Promise::from(p),
        Err(e) => {
            log::warn!("queryLocalFonts failed: {e:?}");
            return;
        }
    };
    wasm_bindgen_futures::spawn_local(async move {
        let list = match JsFuture::from(promise).await {
            Ok(list) => js_sys::Array::from(&list),
            Err(e) => {
                log::warn!("local fonts were not allowed: {e:?}");
                return;
            }
        };
        let mut faces: Vec<LocalFace> = Vec::new();
        HANDLES.with(|h| {
            let mut handles = h.borrow_mut();
            for font in list.iter().take(font_pack::MAX_LOCAL_FACES) {
                let (Some(family), Some(id)) = (get_string(&font, "family"), get_string(&font, "postscriptName")) else { continue };
                let style = get_string(&font, "style").unwrap_or_default();
                handles.insert(id.clone(), font);
                faces.push(LocalFace { family, style, id });
            }
        });
        log::info!("local fonts: {} faces", faces.len());
        font_pack::set_local_fonts(faces);
        ctx.request_repaint();
    });
}
