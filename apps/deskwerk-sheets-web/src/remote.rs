//! Remote workbooks: a server (Deskwerk Server) opens a workbook of its own in Deskwerk Sheets and
//! stores what Save writes, over plain HTTP. Unlike the host modes (`host`), the page that frames
//! the editor may live on another origin.
//!
//! - `?src=<url>` turns it on: the workbook is fetched with `GET src`, sent with
//!   `Authorization: Bearer <token>` (`&token=`), and opens as `&name=` (else the last part of
//!   `src`'s path) like a dropped file. No sample meanwhile. Only `https:` URLs are taken, and
//!   `http:` to this machine (`localhost`) for development.
//! - `&save=<url>`: Save writes the workbook back with `PUT save` and `If-Match:` the `ETag` of the
//!   last load or save, so a file changed on the server since isn't overwritten. Save keeps the
//!   opened file's format (`Services::save_format`: a `.csv` stays a `.csv`, with its separator).
//!   The save finishes later ([`SaveQueue`]): the workbook as it was sent counts as saved once the
//!   server confirms it, so changes made during the upload stay unsaved, and a failure (412:
//!   changed on the server, 423: the edit lock is gone) leaves it unsaved with the reason in the
//!   bar under the ribbon. Without `save` the workbook is read-only on the server: saves download
//!   as usual. Only a file under the opened file's own name goes to the server; exports (PDF,
//!   web page, CSV of a workbook) download, so they never replace it. A workbook Deskwerk Sheets
//!   can only import (ODS, XLS, XLSB, SLK) is never written back: Save says so and downloads an
//!   .xlsx copy instead.
//! - `&origin=<origin>`: the framing page, told what happens with `postMessage` to that origin
//!   alone: `deskwerk:ready {app, version}`, `deskwerk:loaded`, `deskwerk:load-failed {status,
//!   error}`, `deskwerk:dirty {dirty}`, `deskwerk:saved {version, etag}` and `deskwerk:save-failed
//!   {status, error}`. A `deskwerk:save` message from exactly that origin runs File › Save.
//!
//! Leaving the page with unsaved changes, or with a save still on its way, asks first. The token is
//! kept in memory and taken out of the address bar. `?src=` wins over the host modes.
//!
//! The parsing, the messages and the save bookkeeping are plain code, tested natively; the browser
//! side is [`Remote`] (wasm only).
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use std::cell::{Cell, RefCell};
use std::sync::Arc;

use deskwerk_sheets_engine::model::Workbook;
use deskwerk_sheets_ui_egui::{Notice, NoticeKind, SheetApp};
use serde_json::{Value, json};

/// The app id in `deskwerk:ready`.
const APP: &str = "sheets";

/// The name a workbook gets when neither `name` nor `src` gives one.
const DEFAULT_NAME: &str = "Book.xlsx";

/// The longest file name taken from `name` or `src`.
const MAX_NAME_CHARS: usize = 255;

/// Formats Save writes back as they were opened (`Services::save_format`).
const WRITABLE: &[&str] = &["xlsx", "xlsm", "csv", "tsv", "json"];

/// Formats Deskwerk Sheets reads but can't write: (extension, what the message calls it).
const IMPORT_ONLY: &[(&str, &str)] =
    &[("ods", "ODS"), ("xls", "Excel 97-2003 (.xls)"), ("xlt", "Excel 97-2003 (.xlt)"), ("xlsb", "XLSB"), ("slk", "SYLK (.slk)")];

/// What the page's address asks for: the workbook's URL, where to save it, the server's token,
/// the workbook's name and the framing page's origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Params {
    pub src: String,
    pub save: Option<String>,
    pub token: Option<String>,
    pub name: String,
    pub origin: Option<String>,
}

impl Params {
    /// The parameters from the query string's values (`get("src")`…). `Ok(None)` without `src`
    /// (the normal app); an error for a URL or origin that isn't taken.
    pub fn parse(get: impl Fn(&str) -> Option<String>) -> Result<Option<Self>, String> {
        let value = |key: &str| get(key).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let Some(src) = value("src") else { return Ok(None) };
        if !allowed_url(&src) {
            return Err(format!("src is not an https URL: {src}"));
        }
        let save = value("save");
        if let Some(save) = save.as_deref().filter(|s| !allowed_url(s)) {
            return Err(format!("save is not an https URL: {save}"));
        }
        let origin = value("origin").map(|o| o.trim_end_matches('/').to_string());
        if let Some(origin) = origin.as_deref().filter(|o| !is_origin(o)) {
            return Err(format!("origin is not an origin: {origin}"));
        }
        let name = value("name").and_then(|n| file_name(&n)).or_else(|| file_name(url_path(&src))).unwrap_or_else(|| DEFAULT_NAME.into());
        Ok(Some(Self { src, save, token: value("token"), name, origin }))
    }

    /// Whether a file named `path` writes the opened workbook back (its own name; the extension in
    /// any case), rather than a copy (another name, an export), which downloads.
    pub fn saves_back(&self, path: &str) -> bool {
        self.save.is_some() && import_only(&self.name).is_none() && file_name(path).is_some_and(|name| same_file(&name, &self.name))
    }

    /// The refusal when Save would write back a workbook Deskwerk Sheets can only import: a file
    /// named `path` is that save when it is a workbook (the .xlsx Save makes of it).
    pub fn refused(&self, path: &str) -> Option<String> {
        let format = import_only(&self.name)?;
        (self.save.is_some() && is_workbook(path)).then(|| refused_message(&self.name, format))
    }
}

/// `scheme://authority` and the rest of `url`, when the scheme is one taken: `https`, or `http`
/// to this machine.
fn split_url(url: &str) -> Option<(&str, &str)> {
    let (scheme, rest) = url.split_once("://")?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = rest.get(..end)?;
    if authority.is_empty() || authority.contains(['@', ' ', '\\']) {
        return None;
    }
    let host = match authority.strip_prefix('[') {
        Some(v6) => v6.split(']').next().map(|h| format!("[{h}]"))?,
        None => authority.split(':').next().unwrap_or_default().to_string(),
    };
    let local = matches!(host.to_ascii_lowercase().as_str(), "localhost" | "127.0.0.1" | "[::1]");
    let path = rest.get(end..)?;
    match scheme.to_ascii_lowercase().as_str() {
        "https" => Some((authority, path)),
        "http" if local => Some((authority, path)),
        _ => None,
    }
}

fn allowed_url(url: &str) -> bool {
    split_url(url).is_some()
}

/// An origin as `postMessage` takes it: a scheme and host (and port), nothing after.
fn is_origin(origin: &str) -> bool {
    split_url(origin).is_some_and(|(_, rest)| rest.is_empty())
}

/// The path part of a URL (no query or fragment).
fn url_path(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let rest = rest.split(['?', '#']).next().unwrap_or_default();
    rest.find('/').and_then(|i| rest.get(i..)).unwrap_or_default()
}

/// The last part of a path, without the characters a file name can't hold, cut to
/// [`MAX_NAME_CHARS`]; `None` when nothing is left.
fn file_name(path: &str) -> Option<String> {
    let last = path.rsplit(['/', '\\']).next().unwrap_or_default();
    let name: String =
        last.chars().filter(|c| !c.is_control() && !matches!(c, ':' | '*' | '?' | '"' | '<' | '>' | '|')).take(MAX_NAME_CHARS).collect();
    let name = name.trim();
    (!name.is_empty() && name != "." && name != "..").then(|| name.to_string())
}

/// A file name's extension, lowercased (`""` without one).
fn extension(name: &str) -> String {
    name.rsplit_once('.').map(|(_, ext)| ext.to_ascii_lowercase()).unwrap_or_default()
}

/// The same name: the same stem, and the same extension in any case (`data.CSV` saves as
/// `data.csv`).
fn same_file(a: &str, b: &str) -> bool {
    match (a.rsplit_once('.'), b.rsplit_once('.')) {
        (Some((sa, ea)), Some((sb, eb))) => sa == sb && ea.eq_ignore_ascii_case(eb),
        _ => a == b,
    }
}

fn is_workbook(name: &str) -> bool {
    matches!(extension(name).as_str(), "xlsx" | "xlsm")
}

/// The format Save keeps for a workbook opened as `name` (`Services::save_format`), when it is
/// one Deskwerk Sheets writes.
pub fn save_format(name: &str) -> Option<&'static str> {
    let ext = extension(name);
    WRITABLE.iter().copied().find(|f| *f == ext)
}

/// What a message calls the format of `name` when Deskwerk Sheets can only import it.
pub fn import_only(name: &str) -> Option<&'static str> {
    let ext = extension(name);
    IMPORT_ONLY.iter().find(|(e, _)| *e == ext).map(|(_, label)| *label)
}

/// Why Save doesn't write `name` back to the server, and what happens instead.
pub fn refused_message(name: &str, format: &str) -> String {
    format!(
        "{name} can't be saved back to the server: Deskwerk Sheets opens {format} files but can't write them. Your changes download as an .xlsx copy instead."
    )
}

/// Why the server refused a save, as the bar under the ribbon says it.
pub fn save_error(status: u16) -> String {
    match status {
        412 => "the file was changed on the server since it was opened. Download a copy with File › Export to keep your changes.".into(),
        423 => "the file is no longer locked for you on the server (the lock was released, expired or taken over). Download a copy with File › Export to keep your changes.".into(),
        401 | 403 => "the server no longer accepts this editing session. Download a copy with File › Export to keep your changes.".into(),
        413 => "the file is too large for the server.".into(),
        415 => "the server doesn't take this file format.".into(),
        507 => "there is no storage space left on the server.".into(),
        0 => "the server couldn't be reached.".into(),
        s => format!("the server refused it (HTTP {s})."),
    }
}

/// Why the workbook couldn't be loaded.
pub fn load_error(status: u16) -> String {
    match status {
        401 | 403 => "the server didn't accept this editing session. Open the file from the server again.".into(),
        404 => "the file isn't on the server.".into(),
        0 => "the server couldn't be reached.".into(),
        s => format!("the server refused it (HTTP {s})."),
    }
}

/// The saved version's number: the server's `X-Version`, else the one in its `ETag`
/// (`"<file>.<version>"`, maybe weak).
pub fn version(x_version: Option<&str>, etag: Option<&str>) -> Option<u64> {
    if let Some(v) = x_version.and_then(|v| v.trim().parse().ok()) {
        return Some(v);
    }
    let tag = etag?.trim().trim_start_matches("W/").trim_matches('"');
    tag.rsplit_once('.')?.1.parse().ok()
}

/// A `deskwerk:*` message: `{type, ...fields}`.
pub fn message(ty: &str, fields: Value) -> Value {
    let mut msg = json!({ "type": ty });
    if let (Some(msg), Value::Object(fields)) = (msg.as_object_mut(), fields) {
        msg.extend(fields);
    }
    msg
}

/// Whether a message is the framing page asking for File › Save: `deskwerk:save` from exactly
/// the expected origin.
pub fn is_save_request(expected: &str, origin: &str, ty: Option<&str>) -> bool {
    origin == expected && ty == Some("deskwerk:save")
}

/// One save on its way to the server.
struct Save {
    ticket: u64,
    /// The workbook it sent (its document's uid and the workbook then), taken on the first frame
    /// after the save started, before the next edit.
    sent: Option<(u64, Arc<Workbook>)>,
    /// The server's answer, once it came.
    outcome: Option<Result<(), String>>,
}

/// The saves sent to the server, between the `download` hook that starts them and the frame that
/// settles them ([`SaveQueue::sync`], like `Bridge::sync`): `Services::download` returns nothing
/// and can't reach the workbook, so the bookkeeping waits for the frame.
#[derive(Default)]
pub struct SaveQueue {
    last_ticket: Cell<u64>,
    saves: RefCell<Vec<Save>>,
    /// Messages for the bar under the ribbon (a refused save, a failed load).
    alerts: RefCell<Vec<String>>,
    /// The text this queue last showed there, taken away again by a save that succeeds.
    shown: RefCell<Option<String>>,
    /// The unsaved state last reported.
    dirty: Cell<Option<bool>>,
}

impl SaveQueue {
    /// A save of the active workbook has been sent: its ticket for [`SaveQueue::finish`].
    pub fn start(&self) -> u64 {
        let ticket = self.last_ticket.get() + 1;
        self.last_ticket.set(ticket);
        self.saves.borrow_mut().push(Save { ticket, sent: None, outcome: None });
        ticket
    }

    /// The server's answer; the error is the reason the bar shows.
    pub fn finish(&self, ticket: u64, outcome: Result<(), String>) {
        if let Some(save) = self.saves.borrow_mut().iter_mut().find(|s| s.ticket == ticket) {
            save.outcome = Some(outcome);
        }
    }

    /// Show `text` in the bar under the ribbon on the next frame.
    pub fn alert(&self, text: String) {
        self.alerts.borrow_mut().push(text);
    }

    /// Saves the server hasn't answered yet.
    pub fn in_flight(&self) -> usize {
        self.saves.borrow().iter().filter(|s| s.outcome.is_none()).count()
    }

    /// After each frame: a save that started takes the workbook it sent; a confirmed one marks
    /// that workbook saved (once it is the active one again, if another is in front), so changes
    /// made since stay unsaved; a failed one says why, as `Couldn't save <name>: <reason>`.
    /// Returns whether any workbook has unsaved changes when that changed since the last frame.
    pub fn sync(&self, app: &mut SheetApp, name: &str) -> Option<bool> {
        let mut alerts = std::mem::take(&mut *self.alerts.borrow_mut());
        let saved = self.settle(app, name, &mut alerts);
        if let Some(text) = alerts.pop() {
            app.notice = Some(Notice { text: text.clone(), kind: NoticeKind::Warning, action: None });
            *self.shown.borrow_mut() = Some(text);
        } else if saved && app.notice.as_ref().is_some_and(|n| Some(&n.text) == self.shown.borrow().as_ref()) {
            app.notice = None;
        }
        let dirty = app.session.documents().iter().any(|d| d.is_dirty());
        (self.dirty.replace(Some(dirty)) != Some(dirty)).then_some(dirty)
    }

    /// Settles the answered saves; whether one of them succeeded.
    fn settle(&self, app: &mut SheetApp, name: &str, alerts: &mut Vec<String>) -> bool {
        let active = app.session.active().map(|d| (d.uid, d.wb.clone()));
        let mut saved = false;
        let mut saves = self.saves.borrow_mut();
        for save in saves.iter_mut().filter(|s| s.sent.is_none()) {
            save.sent.clone_from(&active);
        }
        saves.retain(|save| match (&save.outcome, &save.sent) {
            (None, _) => true,
            (Some(Err(error)), _) => {
                alerts.push(app.ui.language.tr("Couldn't save {name}: {error}").replace("{name}", name).replace("{error}", error));
                false
            }
            (Some(Ok(())), Some((uid, wb))) => {
                saved = true;
                match app.session.active_mut().filter(|d| d.uid == *uid) {
                    Some(doc) => {
                        doc.saved = wb.clone();
                        false
                    }
                    // Not in front: wait for it, unless it was closed.
                    None => app.session.document_index(*uid).is_some(),
                }
            }
            (Some(Ok(())), None) => {
                saved = true;
                false
            }
        });
        saved
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::Remote;

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::sync::mpsc::Sender;

    use deskwerk_sheets_ui_egui::{ControlRequest, Inbox, Services, SheetApp};
    use serde_json::{Value, json};
    use wasm_bindgen::JsCast as _;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;

    use super::{APP, Params, SaveQueue, load_error, message, save_error, save_format, version};

    /// The largest workbook loaded, as for one a host page sends.
    const MAX_LOAD_BYTES: u64 = 512 << 20;

    /// How often a save waiting for the one before it looks again, in milliseconds.
    const WAIT_MS: i32 = 50;

    /// A failed request: the HTTP status (0 when there was no answer) and the reason.
    type Failure = (u16, String);

    pub struct Remote {
        params: Params,
        /// The `ETag` of the last load or save, sent as `If-Match`.
        etag: RefCell<Option<String>>,
        /// A save is being sent: the next one waits for its new `ETag`.
        busy: Cell<bool>,
        saves: SaveQueue,
        /// Whether a workbook has unsaved changes, as of the last frame (the unload guard's).
        dirty: Cell<bool>,
    }

    impl Remote {
        /// The remote workbook this page's address asks for, if any. The token leaves the
        /// address bar (and so the history and a copied link) at once.
        pub fn from_query() -> Option<Rc<Self>> {
            let search = web_sys::window()?.location().search().ok()?;
            let query = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
            match Params::parse(|key| query.get(key)) {
                Ok(params) => {
                    let params = params?;
                    if params.token.is_some() {
                        hide_token();
                    }
                    Some(Rc::new(Self {
                        params,
                        etag: RefCell::new(None),
                        busy: Cell::new(false),
                        saves: SaveQueue::default(),
                        dirty: Cell::new(false),
                    }))
                }
                Err(e) => {
                    log::error!("remote workbook not opened: {e}");
                    None
                }
            }
        }

        /// Save keeps the opened format and writes back to the server; copies download.
        pub fn connect(self: &Rc<Self>, services: &mut Services, ctx: egui::Context) {
            services.save_format = save_format(&self.params.name).map(str::to_string);
            if self.params.save.is_some() {
                let remote = self.clone();
                services.download = Some(Box::new(move |path: &str, bytes: &[u8]| remote.deliver(path, bytes, &ctx)));
            }
        }

        /// A file the app hands over: the workbook's own goes to the server, anything else (and
        /// a workbook the server's format can't take) downloads.
        fn deliver(self: &Rc<Self>, path: &str, bytes: &[u8], ctx: &egui::Context) {
            if let Some(error) = self.params.refused(path) {
                log::warn!("{error}");
                let text = deskwerk_sheets_ui_egui::i18n::current(ctx).tr("Couldn't save {name}: {error}");
                self.saves.alert(text.replace("{name}", &self.params.name).replace("{error}", &error));
                self.post("deskwerk:save-failed", json!({ "status": 415, "error": error }));
                ctx.request_repaint();
            } else if self.params.saves_back(path) {
                let ticket = self.saves.start();
                let (remote, ctx) = (self.clone(), ctx.clone());
                let bytes = bytes.to_vec();
                wasm_bindgen_futures::spawn_local(async move {
                    let outcome = remote.put(bytes).await;
                    remote.saves.finish(ticket, outcome);
                    ctx.request_repaint();
                });
                return;
            }
            if let Err(e) = crate::web::download(path, bytes) {
                log::error!("download of {path} failed: {e}");
            }
        }

        /// Fetch the workbook and queue it for the app, like a dropped file. A failure shows in
        /// the bar under the ribbon and is posted.
        pub fn load(self: &Rc<Self>, inbox: Inbox, ctx: egui::Context) {
            let remote = self.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let name = remote.params.name.clone();
                match remote.get().await {
                    Ok(bytes) => {
                        inbox.lock().unwrap_or_else(|e| e.into_inner()).push((name, bytes));
                        remote.post("deskwerk:loaded", json!({}));
                    }
                    Err((status, error)) => {
                        log::error!("couldn't load {name}: {error}");
                        let text = deskwerk_sheets_ui_egui::i18n::current(&ctx).tr("Couldn't open {name}: {error}");
                        remote.saves.alert(text.replace("{name}", &name).replace("{error}", &error));
                        remote.post("deskwerk:load-failed", json!({ "status": status, "error": error }));
                    }
                }
                ctx.request_repaint();
            });
        }

        /// After each part of a frame (`logic` and `ui`): settle the saves the server answered and
        /// tell the framing page when unsaved changes appear or go away.
        pub fn sync(&self, app: &mut SheetApp) {
            if let Some(dirty) = self.saves.sync(app, &self.params.name) {
                self.dirty.set(dirty);
                self.post("deskwerk:dirty", json!({ "dirty": dirty }));
            }
        }

        /// Whether leaving the page would lose work: unsaved changes, or a save still on its way.
        pub fn unsaved(&self) -> bool {
            self.dirty.get() || self.saves.in_flight() > 0
        }

        pub fn post_ready(&self) {
            self.post("deskwerk:ready", json!({ "app": APP, "version": env!("CARGO_PKG_VERSION") }));
        }

        async fn get(&self) -> Result<Vec<u8>, Failure> {
            let resp = fetch("GET", &self.params.src, self.params.token.as_deref(), None, None).await?;
            if !resp.ok() {
                return Err((resp.status(), load_error(resp.status())));
            }
            let too_big = || (413, format!("the file is larger than {} MiB.", MAX_LOAD_BYTES >> 20));
            let length = resp.headers().get("Content-Length").ok().flatten().and_then(|l| l.trim().parse::<u64>().ok());
            if length.is_some_and(|l| l > MAX_LOAD_BYTES) {
                return Err(too_big());
            }
            let etag = resp.headers().get("ETag").ok().flatten();
            let buffer = resp.array_buffer().map_err(no_answer)?;
            let buffer = JsFuture::from(buffer).await.map_err(no_answer)?;
            let array = js_sys::Uint8Array::new(&buffer);
            if u64::from(array.length()) > MAX_LOAD_BYTES {
                return Err(too_big());
            }
            *self.etag.borrow_mut() = etag;
            Ok(array.to_vec())
        }

        /// `PUT save`, one at a time, posting how it went. The error is the bar's reason.
        async fn put(&self, bytes: Vec<u8>) -> Result<(), String> {
            while self.busy.replace(true) {
                crate::api::sleep(WAIT_MS).await;
            }
            let result = self.put_now(&bytes).await;
            self.busy.set(false);
            match result {
                Ok((etag, x_version)) => {
                    let version = version(x_version.as_deref(), etag.as_deref());
                    self.post("deskwerk:saved", json!({ "version": version, "etag": etag }));
                    Ok(())
                }
                Err((status, error)) => {
                    log::error!("saving {} on the server failed: {error}", self.params.name);
                    self.post("deskwerk:save-failed", json!({ "status": status, "error": error }));
                    Err(error)
                }
            }
        }

        /// The new `ETag` (the server's next `If-Match`) and `X-Version` on success.
        async fn put_now(&self, bytes: &[u8]) -> Result<(Option<String>, Option<String>), Failure> {
            let Some(url) = &self.params.save else { return Err((0, "there is nowhere to save it.".into())) };
            let etag = self.etag.borrow().clone();
            let body = (crate::web::mime_for(&self.params.name), bytes);
            let resp = fetch("PUT", url, self.params.token.as_deref(), etag.as_deref(), Some(body)).await?;
            if !resp.ok() {
                return Err((resp.status(), save_error(resp.status())));
            }
            let etag = resp.headers().get("ETag").ok().flatten();
            let x_version = resp.headers().get("X-Version").ok().flatten();
            *self.etag.borrow_mut() = etag.clone();
            Ok((etag, x_version))
        }

        /// `postMessage` to the framing page (or the page that opened this tab), on its origin
        /// only. Nothing without `origin`.
        fn post(&self, ty: &str, fields: Value) {
            let Some(origin) = &self.params.origin else { return };
            let Some(target) = framing_page() else { return };
            let Ok(data) = js_sys::JSON::parse(&message(ty, fields).to_string()) else { return };
            if let Err(e) = target.post_message(&data, origin) {
                log::warn!("couldn't post {ty}: {e:?}");
            }
        }

        /// Hear `deskwerk:save` from the framing page and queue File › Save for the app.
        pub fn listen(&self, control: Sender<ControlRequest>, ctx: egui::Context) {
            let Some(origin) = self.params.origin.clone() else { return };
            let Some(window) = web_sys::window() else { return };
            let on_message = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |e: web_sys::MessageEvent| {
                let ty = js_sys::Reflect::get(&e.data(), &JsValue::from_str("type")).ok().and_then(|v| v.as_string());
                if super::is_save_request(&origin, &e.origin(), ty.as_deref()) {
                    // Nobody waits for the reply: the save reports itself (`deskwerk:saved`).
                    let (request, _reply) = ControlRequest::new("file.save", json!({}));
                    if control.send(request).is_err() {
                        log::warn!("deskwerk:save: the app is not running");
                    }
                    ctx.request_repaint();
                }
            });
            if let Err(e) = window.add_event_listener_with_callback("message", on_message.as_ref().unchecked_ref()) {
                log::error!("deskwerk:save is unavailable: {e:?}");
            }
            // The listener lives as long as the page.
            on_message.forget();
        }
    }

    /// The parent window when framed, else the window that opened this tab.
    fn framing_page() -> Option<web_sys::Window> {
        let window = web_sys::window()?;
        if let Ok(Some(parent)) = window.parent()
            && !js_sys::Object::is(&parent, &window)
        {
            return Some(parent);
        }
        let opener = window.opener().ok().filter(|o| !o.is_null() && !o.is_undefined())?;
        Some(opener.unchecked_into())
    }

    fn no_answer(e: JsValue) -> Failure {
        log::warn!("request failed: {e:?}");
        (0, save_error(0))
    }

    /// A cross-origin request with the token and no cookies (the server takes the token alone).
    async fn fetch(
        method: &str,
        url: &str,
        token: Option<&str>,
        if_match: Option<&str>,
        body: Option<(&str, &[u8])>,
    ) -> Result<web_sys::Response, Failure> {
        let window = web_sys::window().ok_or((0, "no window".to_string()))?;
        let init = web_sys::RequestInit::new();
        init.set_method(method);
        init.set_mode(web_sys::RequestMode::Cors);
        init.set_credentials(web_sys::RequestCredentials::Omit);
        let headers = web_sys::Headers::new().map_err(no_answer)?;
        if let Some(token) = token {
            headers.set("Authorization", &format!("Bearer {token}")).map_err(no_answer)?;
        }
        if let Some(etag) = if_match {
            headers.set("If-Match", etag).map_err(no_answer)?;
        }
        if let Some((mime, bytes)) = body {
            headers.set("Content-Type", mime).map_err(no_answer)?;
            init.set_body(&js_sys::Uint8Array::from(bytes));
        }
        init.set_headers(&headers);
        let request = web_sys::Request::new_with_str_and_init(url, &init).map_err(no_answer)?;
        let resp = JsFuture::from(window.fetch_with_request(&request)).await.map_err(no_answer)?;
        resp.dyn_into().map_err(no_answer)
    }

    /// Take `token` out of the address bar, keeping the rest.
    fn hide_token() {
        let Some(window) = web_sys::window() else { return };
        let Ok(href) = window.location().href() else { return };
        let Ok(url) = web_sys::Url::new(&href) else { return };
        url.search_params().delete("token");
        if let Ok(history) = window.history()
            && let Err(e) = history.replace_state_with_url(&JsValue::NULL, "", Some(&url.href()))
        {
            log::warn!("couldn't take the token out of the address: {e:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use deskwerk_sheets_engine::Session;
    use deskwerk_sheets_ui_egui::Services;

    use super::*;

    fn parse(query: &[(&str, &str)]) -> Result<Option<Params>, String> {
        Params::parse(|key| query.iter().find(|(k, _)| *k == key).map(|(_, v)| v.to_string()))
    }

    fn server(name: &str) -> Params {
        parse(&[("src", SRC), ("save", SRC), ("name", name)]).unwrap().unwrap()
    }

    const SRC: &str = "https://files.example.com/editor/v1/files/f1/content";

    #[test]
    fn no_src_is_the_normal_app() {
        assert_eq!(parse(&[]), Ok(None));
        assert_eq!(parse(&[("src", "  "), ("token", "t")]), Ok(None));
    }

    #[test]
    fn the_server_link_is_read_whole() {
        let p = parse(&[("src", SRC), ("save", SRC), ("token", "abc.def"), ("name", "Budget.xlsx"), ("origin", "https://files.example.com")])
            .unwrap()
            .unwrap();
        assert_eq!(
            p,
            Params {
                src: SRC.into(),
                save: Some(SRC.into()),
                token: Some("abc.def".into()),
                name: "Budget.xlsx".into(),
                origin: Some("https://files.example.com".into()),
            }
        );
    }

    #[test]
    fn the_name_falls_back_to_the_path_and_is_cleaned() {
        let p = parse(&[("src", "https://h/files/Sales.csv?x=1#y")]).unwrap().unwrap();
        assert_eq!((p.name.as_str(), p.save, p.token, p.origin), ("Sales.csv", None, None, None));
        assert_eq!(parse(&[("src", "https://h/")]).unwrap().unwrap().name, DEFAULT_NAME);
        assert_eq!(parse(&[("src", SRC), ("name", "../a/b\\Q3: plan?.xlsx")]).unwrap().unwrap().name, "Q3 plan.xlsx");
        assert_eq!(parse(&[("src", SRC), ("name", "..")]).unwrap().unwrap().name, "content");
    }

    #[test]
    fn only_https_or_this_machine_is_taken() {
        for bad in ["http://files.example.com/a", "javascript:alert(1)", "file:///etc/passwd", "https://user@evil/a", "//h/a", "https:///a"] {
            assert!(parse(&[("src", bad)]).is_err(), "{bad}");
            assert!(parse(&[("src", SRC), ("save", bad)]).is_err(), "{bad}");
        }
        for good in ["http://localhost:8000/a", "http://127.0.0.1/a", "http://[::1]:8000/a", "HTTPS://Files.Example.com/a"] {
            assert!(parse(&[("src", good)]).unwrap().is_some(), "{good}");
        }
    }

    #[test]
    fn the_origin_is_an_origin() {
        let origin = |o: &str| parse(&[("src", SRC), ("origin", o)]).map(|p| p.and_then(|p| p.origin));
        assert_eq!(origin("https://files.example.com/"), Ok(Some("https://files.example.com".into())));
        assert_eq!(origin("http://localhost:8000"), Ok(Some("http://localhost:8000".into())));
        for bad in ["*", "https://files.example.com/edit", "http://files.example.com", "null"] {
            assert!(origin(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn only_the_workbooks_own_name_saves_back() {
        let p = server("Budget.xlsx");
        assert!(p.saves_back("Budget.xlsx"));
        assert!(p.saves_back("some/dir/Budget.XLSX"));
        for copy in ["Budget.pdf", "Budget.csv", "Budget.html", "Other.xlsx", "budget.xlsx", "Book1.xlsx"] {
            assert!(!p.saves_back(copy), "{copy}");
        }
        assert!(server("data.CSV").saves_back("data.csv"), "Save writes the extension in lower case");
        let read_only = parse(&[("src", SRC), ("name", "Budget.xlsx")]).unwrap().unwrap();
        assert!(!read_only.saves_back("Budget.xlsx"));
        assert_eq!(read_only.refused("Budget.xlsx"), None);
    }

    #[test]
    fn save_keeps_the_opened_format_when_it_can_write_it() {
        for (name, format) in
            [("a.xlsx", Some("xlsx")), ("a.XLSM", Some("xlsm")), ("a.csv", Some("csv")), ("a.tsv", Some("tsv")), ("a.json", Some("json"))]
        {
            assert_eq!(save_format(name), format, "{name}");
        }
        for name in ["a.ods", "a.xls", "a.xlsb", "a.slk", "a.html", "a", "a.xltx"] {
            assert_eq!(save_format(name), None, "{name}");
        }
    }

    #[test]
    fn an_import_only_workbook_is_never_written_back() {
        for (name, format) in [("Plan.ods", "ODS"), ("Plan.XLS", "Excel 97-2003 (.xls)"), ("Plan.xlsb", "XLSB"), ("Plan.slk", "SYLK (.slk)")] {
            let p = server(name);
            assert_eq!(import_only(name), Some(format), "{name}");
            // The .xlsx Save makes of it is the save; it is refused with the reason, never PUT.
            let title = format!("{} (imported).xlsx", name.split('.').next().unwrap());
            for save in [title.as_str(), name] {
                assert!(!p.saves_back(save), "{name}: {save}");
            }
            let error = p.refused(&title).unwrap();
            assert!(error.starts_with(&format!("{name} can't be saved back to the server")), "{error}");
            assert!(error.contains(&format!("opens {format} files but can't write them")), "{error}");
            assert!(error.contains("download as an .xlsx copy"), "{error}");
            // Exports still just download.
            assert_eq!(p.refused("Plan (imported).pdf"), None);
            assert_eq!(p.refused("Plan (imported).csv"), None);
        }
        assert_eq!(server("Budget.xlsx").refused("Budget.xlsx"), None);
    }

    #[test]
    fn statuses_say_what_to_do() {
        assert!(save_error(412).contains("changed on the server"));
        assert!(save_error(423).contains("no longer locked"));
        for status in [412, 423, 401, 403] {
            assert!(save_error(status).contains("Download a copy"), "{status}");
        }
        assert!(save_error(413).contains("too large"));
        assert!(save_error(415).contains("format"));
        assert!(save_error(507).contains("storage"));
        assert!(save_error(0).contains("couldn't be reached"));
        assert!(save_error(500).contains("HTTP 500"));
        assert!(load_error(401).contains("again"));
        assert!(load_error(404).contains("isn't on the server"));
        assert!(load_error(502).contains("HTTP 502"));
    }

    #[test]
    fn the_version_comes_from_x_version_or_the_etag() {
        assert_eq!(version(Some("8"), Some("\"9f2c.7\"")), Some(8));
        assert_eq!(version(None, Some("\"9f2c.7\"")), Some(7));
        assert_eq!(version(Some("x"), Some("W/\"9f2c.12\"")), Some(12));
        assert_eq!(version(None, Some("\"9f2c\"")), None);
        assert_eq!(version(None, None), None);
    }

    #[test]
    fn messages_carry_their_type_and_fields() {
        assert_eq!(message("deskwerk:dirty", json!({"dirty": true})), json!({"type": "deskwerk:dirty", "dirty": true}));
        assert_eq!(message("deskwerk:loaded", json!({})), json!({"type": "deskwerk:loaded"}));
        assert_eq!(message("deskwerk:ready", Value::Null), json!({"type": "deskwerk:ready"}));
    }

    #[test]
    fn only_the_framing_origin_asks_for_a_save() {
        let page = "https://files.example.com";
        assert!(is_save_request(page, page, Some("deskwerk:save")));
        assert!(!is_save_request(page, "https://files.example.com.evil.com", Some("deskwerk:save")));
        assert!(!is_save_request(page, "null", Some("deskwerk:save")));
        assert!(!is_save_request(page, page, Some("deskwerk:open")));
        assert!(!is_save_request(page, page, None));
    }

    fn app() -> SheetApp {
        let mut app = SheetApp::new(Session::new(), Services::default());
        app.session.new_workbook();
        app
    }

    fn edit(app: &mut SheetApp, cell: &str) {
        app.session.run("cell.set", json!({"cell": cell, "input": "1"})).unwrap();
    }

    fn dirty(app: &SheetApp) -> bool {
        app.session.active().unwrap().is_dirty()
    }

    #[test]
    fn a_confirmed_save_marks_the_workbook_it_sent_as_saved() {
        let (mut app, saves) = (app(), SaveQueue::default());
        assert_eq!(saves.sync(&mut app, "Budget.xlsx"), Some(false), "the first frame reports the state");
        edit(&mut app, "A1");
        let ticket = saves.start();
        assert_eq!(saves.sync(&mut app, "Budget.xlsx"), Some(true));
        assert_eq!(saves.in_flight(), 1);
        saves.finish(ticket, Ok(()));
        assert_eq!(saves.sync(&mut app, "Budget.xlsx"), Some(false));
        assert!(!dirty(&app));
        assert_eq!(saves.in_flight(), 0);
        assert_eq!(saves.sync(&mut app, "Budget.xlsx"), None, "nothing changed");
    }

    #[test]
    fn changes_made_while_the_save_is_on_its_way_stay_unsaved() {
        let (mut app, saves) = (app(), SaveQueue::default());
        edit(&mut app, "A1");
        let ticket = saves.start();
        saves.sync(&mut app, "Budget.xlsx");
        let sent = app.session.active().unwrap().wb.clone();
        edit(&mut app, "A2");
        saves.finish(ticket, Ok(()));
        saves.sync(&mut app, "Budget.xlsx");
        let doc = app.session.active().unwrap();
        assert!(doc.is_dirty(), "A2 was typed after the save started");
        assert!(Arc::ptr_eq(&doc.saved, &sent), "the workbook as it was sent is the saved one");
    }

    #[test]
    fn a_refused_save_leaves_the_workbook_unsaved_and_says_why() {
        let (mut app, saves) = (app(), SaveQueue::default());
        edit(&mut app, "A1");
        let ticket = saves.start();
        saves.sync(&mut app, "Budget.xlsx");
        saves.finish(ticket, Err(save_error(412)));
        assert_eq!(saves.sync(&mut app, "Budget.xlsx"), None, "still unsaved");
        assert!(dirty(&app));
        let notice = app.notice.clone().unwrap();
        assert_eq!(notice.kind, NoticeKind::Warning);
        assert!(notice.text.starts_with("Couldn't save Budget.xlsx: the file was changed on the server"), "{}", notice.text);
        // The next save that gets through takes the warning away; another bar stays.
        let ticket = saves.start();
        saves.sync(&mut app, "Budget.xlsx");
        saves.finish(ticket, Ok(()));
        saves.sync(&mut app, "Budget.xlsx");
        assert!(!dirty(&app));
        assert_eq!(app.notice, None);
        app.notice = Some(Notice { text: "macros".into(), kind: NoticeKind::Info, action: None });
        let ticket = saves.start();
        saves.finish(ticket, Ok(()));
        saves.sync(&mut app, "Budget.xlsx");
        assert_eq!(app.notice.as_ref().map(|n| n.text.as_str()), Some("macros"));
    }

    #[test]
    fn a_save_confirmed_while_another_workbook_is_in_front_waits_for_it() {
        let (mut app, saves) = (app(), SaveQueue::default());
        edit(&mut app, "A1");
        let first = app.session.active_index();
        let ticket = saves.start();
        saves.sync(&mut app, "Budget.xlsx");
        app.session.new_workbook();
        saves.finish(ticket, Ok(()));
        saves.sync(&mut app, "Budget.xlsx");
        assert!(app.session.documents()[first].is_dirty(), "not marked while in the background");
        app.session.set_active(first);
        saves.sync(&mut app, "Budget.xlsx");
        assert!(!dirty(&app), "marked once it is in front again");
    }

    #[test]
    fn alerts_show_in_the_bar() {
        let (mut app, saves) = (app(), SaveQueue::default());
        saves.alert("Couldn't open Budget.xlsx: the file isn't on the server.".into());
        saves.sync(&mut app, "Budget.xlsx");
        assert_eq!(app.notice.as_ref().map(|n| n.text.as_str()), Some("Couldn't open Budget.xlsx: the file isn't on the server."));
    }
}
