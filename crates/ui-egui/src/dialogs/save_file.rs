//! Save (Ctrl+S, the title bar's Save) on a workbook that has never been saved: Excel 365's
//! compact Save file window. It asks for a name and a folder from a short list (recent folders,
//! Documents, Desktop, Browse…), and More options… leads to the full Save As page. Desktop only:
//! the web build downloads the workbook instead.

use std::path::Path;

use egui::{Align2, Color32, Key, Modifiers, RichText, Sense, Stroke, StrokeKind, pos2, vec2};

use super::{Dialog, Json, SheetApp, json};
use crate::icons::{self, Icon};
use crate::theme::{self, Tokens};

/// The dialog's name in `Dialog::open` and `app.dialog`.
pub(crate) const NAME: &str = "saveFile";
/// The folders of recently used files listed before Documents and Desktop.
const RECENT_FOLDERS: usize = 5;
const WIDTH: f32 = 440.0;
/// Characters Windows refuses in a file name.
const INVALID: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// A folder the location list offers.
struct Folder {
    /// Shown above the path: Documents and Desktop are translated, others show their own name.
    label: String,
    path: String,
    known: bool,
}

/// What the user asked for this frame.
enum Action {
    Save,
    Replace,
    Cancel,
    MoreOptions,
}

/// The dialog for the active workbook. `params`: `close` (asked from Save Changes: the workbook
/// closes once saved) and `quit` (the app goes on quitting after that).
pub(super) fn open(app: &SheetApp, params: &Json) -> Dialog {
    let title = app.session.active().map(|d| d.display_title()).unwrap_or_else(|| "Book1".into());
    let name = Path::new(&title).file_stem().and_then(|s| s.to_str()).filter(|s| !s.is_empty()).unwrap_or("Book1").to_string();
    let folders = folder_list(app);
    let folder = folders.iter().find(|f| f.known && f.label == "Documents").or_else(|| folders.first()).map(|f| f.path.clone()).unwrap_or_default();
    let list: Vec<Json> = folders.iter().map(|f| json!({"label": f.label, "path": f.path, "known": f.known})).collect();
    let mut d = Dialog::custom(
        NAME,
        "Save file",
        json!({
            "name": name,
            "folder": folder,
            "folders": list,
            "close": params.get("close").and_then(Json::as_bool) == Some(true),
            "quit": params.get("quit").and_then(Json::as_bool) == Some(true),
        }),
    );
    d.focus_first_field = true;
    d
}

/// The folders to offer: those of recently used files (most recent first), then Documents and
/// Desktop. Only folders that exist are listed, each once.
fn folder_list(app: &SheetApp) -> Vec<Folder> {
    let known: Vec<(String, String)> = match &app.services.known_folders {
        Some(known) => known(),
        None => fallback_known_folders(),
    };
    let mut out: Vec<Folder> = Vec::new();
    let push = |out: &mut Vec<Folder>, folder: Folder| {
        if !out.iter().any(|f| same_folder(&f.path, &folder.path)) {
            out.push(folder);
        }
    };
    let recent = app.ui.recent.iter().filter_map(|file| Path::new(file).parent().and_then(|p| p.to_str()).map(str::to_string));
    for path in recent.filter(|p| !p.is_empty() && Path::new(p).is_dir()) {
        if out.len() >= RECENT_FOLDERS {
            break;
        }
        // A recent file in Documents is listed as Documents, in its own place.
        if known.iter().any(|(_, k)| same_folder(k, &path)) {
            continue;
        }
        push(&mut out, Folder { label: folder_name(&path), path, known: false });
    }
    for (label, path) in known {
        if Path::new(&path).is_dir() {
            push(&mut out, Folder { label, path, known: true });
        }
    }
    out
}

/// Documents and Desktop in the home folder, for hosts that don't name them.
fn fallback_known_folders() -> Vec<(String, String)> {
    let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) else { return Vec::new() };
    let home = std::path::PathBuf::from(home);
    ["Documents", "Desktop"]
        .into_iter()
        .filter_map(|name| {
            let path = home.join(name);
            path.is_dir().then(|| (name.to_string(), path.to_string_lossy().into_owned()))
        })
        .collect()
}

/// Two spellings of one folder: a trailing separator is ignored, and case too on Windows.
fn same_folder(a: &str, b: &str) -> bool {
    let trim = |s: &str| s.trim_end_matches(['/', '\\']).to_string();
    let (a, b) = (trim(a), trim(b));
    if cfg!(windows) { a.to_lowercase() == b.to_lowercase() } else { a == b }
}

/// A folder's own name (the last part of its path), or the path itself for a drive's root.
fn folder_name(path: &str) -> String {
    Path::new(path).file_name().and_then(|n| n.to_str()).map(str::to_string).unwrap_or_else(|| path.to_string())
}

fn folders(d: &Dialog) -> Vec<Folder> {
    d.values
        .get("folders")
        .and_then(Json::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|f| {
                    Some(Folder {
                        label: f.get("label")?.as_str()?.to_string(),
                        path: f.get("path")?.as_str()?.to_string(),
                        known: f.get("known").and_then(Json::as_bool) == Some(true),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn text<'a>(d: &'a Dialog, key: &str) -> &'a str {
    d.values.get(key).and_then(Json::as_str).unwrap_or("")
}

fn flag(d: &Dialog, key: &str) -> bool {
    d.values.get(key).and_then(Json::as_bool) == Some(true)
}

/// The name as typed, without the .xlsx the field already shows.
fn file_stem(d: &Dialog) -> String {
    let name = text(d, "name").trim();
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".xlsx") && name.len() > 5 { name[..name.len() - 5].trim_end().to_string() } else { name.to_string() }
}

/// A name Windows accepts: none of `<>:"/\|?*`, no control characters, not ending in a dot.
fn valid_name(name: &str) -> bool {
    !name.is_empty() && !name.chars().any(|c| INVALID.contains(&c) || c.is_control()) && !name.ends_with('.') && name != ".."
}

/// The label a folder shows: Documents and Desktop in the interface language.
fn shown_label(lang: crate::i18n::Language, f: &Folder) -> String {
    if f.known { lang.tr(&f.label).to_string() } else { f.label.clone() }
}

/// Draws the dialog. It stays in `app.dialog` while open; Save, Cancel, × and More options…
/// close it.
pub(super) fn show(app: &mut SheetApp, ctx: &egui::Context, mut d: Dialog) {
    let lang = app.ui.language;
    let t = Tokens::get(ctx);
    let replacing = d.values.get("replace").and_then(Json::as_str).map(str::to_string);
    let mut action: Option<Action> = None;
    // Keys first, before the name box sees Enter: not while a message or the folder list is up.
    if app.message.is_none() && !egui::Popup::is_any_open(ctx) {
        let escape = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        let enter = !escape && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
        match (replacing.is_some(), escape, enter) {
            (true, true, _) => {
                d.values.remove("replace");
            }
            (true, _, true) => action = Some(Action::Replace),
            (false, true, _) => action = Some(Action::Cancel),
            (false, _, true) => action = Some(Action::Save),
            _ => {}
        }
    }
    let frame = egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin::same(20)).corner_radius(8.0);
    egui::Modal::new(egui::Id::new("save_file_dialog")).frame(frame).show(ctx, |ui| {
        ui.set_width(WIDTH);
        if let Some(a) = form(app, ui, &mut d, &t) {
            action = Some(a);
        }
    });
    if let Some(path) = d.values.get("replace").and_then(Json::as_str).map(str::to_string) {
        let frame = egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin::same(20)).corner_radius(8.0);
        egui::Modal::new(egui::Id::new("save_file_replace")).frame(frame).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.label(RichText::new(lang.tr("Save file")).font(theme::ui_font(16.0)).color(t.accent));
            ui.add_space(8.0);
            let file = Path::new(&path).file_name().and_then(|n| n.to_str()).unwrap_or(&path).to_string();
            let question = lang.tr("A file named {name} already exists. Do you want to replace it?").replace("{name}", &file);
            ui.add(egui::Label::new(question).wrap());
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if outlined(ui, lang.tr("Cancel"), t.input_border, t.text).clicked() {
                        d.values.remove("replace");
                    }
                    if filled(ui, lang.tr("Save file|Replace"), &t, true).clicked() {
                        action = Some(Action::Replace);
                    }
                });
            });
        });
    }
    let mut open = true;
    match action {
        None => {}
        Some(Action::Cancel) => open = false,
        Some(Action::Save) => {
            let stem = file_stem(&d);
            let folder = text(&d, "folder").to_string();
            if stem.is_empty() || folder.is_empty() {
                // Nothing to save yet: Enter on an empty name does nothing, as Save is disabled.
            } else if !valid_name(&stem) {
                app.message = Some(("Save file".into(), "This file name isn't valid.".into()));
            } else {
                let path = Path::new(&folder).join(format!("{stem}.xlsx")).to_string_lossy().into_owned();
                if Path::new(&path).exists() {
                    d.values.insert("replace".into(), json!(path));
                } else {
                    open = !write(app, ctx, &d, &path);
                }
            }
        }
        Some(Action::Replace) => {
            if let Some(path) = d.values.remove("replace").and_then(|p| p.as_str().map(str::to_string)) {
                open = !write(app, ctx, &d, &path);
            }
        }
        Some(Action::MoreOptions) => {
            more_options(app, ctx, &d);
            return;
        }
    }
    if open {
        app.dialog = Some(d);
    } else if app.dialog.is_none() {
        app.dialog = d.back.take().map(|b| *b);
    }
}

/// The dialog's window: title and ×, the name, the folder, then More options…, Save and Cancel.
fn form(app: &SheetApp, ui: &mut egui::Ui, d: &mut Dialog, t: &Tokens) -> Option<Action> {
    let lang = app.ui.language;
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(RichText::new(lang.tr("Save file")).font(theme::ui_font(20.0)).color(t.accent));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let close = crate::widgets::icon_button(ui, Icon::Close, t.text_dim, "Close", vec2(28.0, 28.0));
            close.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, lang.tr("Close")));
            if close.clicked() {
                action = Some(Action::Cancel);
            }
        });
    });
    ui.add_space(14.0);

    let label = ui.label(RichText::new(lang.tr("File name")).color(t.text));
    ui.add_space(2.0);
    name_box(ui, d, t, label.id);
    ui.add_space(14.0);

    let label = ui.label(RichText::new(lang.tr("Choose a location")).color(t.text));
    ui.add_space(2.0);
    location(app, ui, d, t, label.id);
    ui.add_space(18.0);
    ui.separator();
    ui.add_space(10.0);

    let can_save = !file_stem(d).is_empty() && !text(d, "folder").is_empty();
    ui.horizontal(|ui| {
        let more = ui.add(egui::Link::new(RichText::new(lang.tr("More options…")).color(t.accent)));
        if more.clicked() {
            action = Some(Action::MoreOptions);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if outlined(ui, lang.tr("Cancel"), t.input_border, t.text).clicked() {
                action = Some(Action::Cancel);
            }
            if filled(ui, lang.tr("Save"), t, can_save).clicked() {
                action = Some(Action::Save);
            }
        });
    });
    action
}

/// A button filled with the accent colour, white text: the dialog's default answer.
fn filled(ui: &mut egui::Ui, label: &str, t: &Tokens, enabled: bool) -> egui::Response {
    let button = egui::Button::new(RichText::new(label).color(t.on_accent)).fill(t.accent).min_size(vec2(84.0, 30.0)).corner_radius(4.0);
    ui.add_enabled(enabled, button)
}

/// A button with no fill and a 1 px border.
fn outlined(ui: &mut egui::Ui, label: &str, border: Color32, text: Color32) -> egui::Response {
    let button = egui::Button::new(RichText::new(label).color(text))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, border))
        .min_size(vec2(84.0, 30.0))
        .corner_radius(4.0);
    ui.add(button)
}

/// The name box: the name, selected when the dialog opens, with ".xlsx" greyed at its right end
/// (shown, not part of the text).
fn name_box(ui: &mut egui::Ui, d: &mut Dialog, t: &Tokens, label: egui::Id) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
    let background = ui.painter().add(egui::Shape::Noop);
    let ext = ui.painter().text(pos2(rect.right() - 10.0, rect.center().y), Align2::RIGHT_CENTER, ".xlsx", theme::ui_font(14.0), t.text_dim);
    let edit_rect = egui::Rect::from_min_max(pos2(rect.left() + 10.0, rect.top() + 2.0), pos2(ext.left() - 6.0, rect.bottom() - 2.0));
    let mut name = text(d, "name").to_string();
    let edit = egui::TextEdit::singleline(&mut name)
        .id(egui::Id::new("save_file_name"))
        .frame(egui::Frame::NONE)
        .margin(egui::Margin::symmetric(0, 6))
        .font(theme::ui_font(14.0))
        .text_color(t.text)
        .desired_width(edit_rect.width());
    let r = ui.put(edit_rect, edit).labelled_by(label);
    if r.changed() {
        d.values.insert("name".into(), json!(name));
    }
    super::show::take_first_focus(d, &r);
    let border = if r.has_focus() { Stroke::new(1.5, t.accent) } else { Stroke::new(1.0, t.input_border) };
    ui.painter().set(background, egui::epaint::RectShape::new(rect, 4.0, t.input_bg, border, StrokeKind::Inside));
}

/// Paints a folder entry: the folder icon, its name, and its full path under it in smaller text.
fn paint_folder(ui: &egui::Ui, rect: egui::Rect, label: &str, path: &str, t: &Tokens, right_margin: f32) {
    let painter = ui.painter();
    icons::paint(painter, egui::Rect::from_center_size(pos2(rect.left() + 20.0, rect.center().y), vec2(20.0, 20.0)), Icon::Folder, t.text);
    let width = (rect.width() - 40.0 - right_margin).max(10.0);
    let line = |text: &str, size: f32, color: Color32| {
        let mut job = egui::text::LayoutJob::simple_singleline(text.to_string(), theme::ui_font(size), color);
        job.wrap = egui::text::TextWrapping::truncate_at_width(width);
        painter.layout_job(job)
    };
    if path.is_empty() {
        let galley = line(label, 14.0, t.text);
        painter.galley(pos2(rect.left() + 40.0, rect.center().y - galley.size().y / 2.0), galley, t.text);
        return;
    }
    let name = line(label, 14.0, t.text);
    let sub = line(path, 11.5, t.text_dim);
    let top = rect.center().y - (name.size().y + sub.size().y) / 2.0;
    let name_height = name.size().y;
    painter.galley(pos2(rect.left() + 40.0, top), name, t.text);
    painter.galley(pos2(rect.left() + 40.0, top + name_height), sub, t.text_dim);
}

/// The folder button and its list: recent folders, Documents, Desktop and Browse…
fn location(app: &SheetApp, ui: &mut egui::Ui, d: &mut Dialog, t: &Tokens, label: egui::Id) {
    let lang = app.ui.language;
    let list = folders(d);
    let current = text(d, "folder").to_string();
    let shown = list
        .iter()
        .find(|f| same_folder(&f.path, &current))
        .map(|f| shown_label(lang, f))
        .unwrap_or_else(|| if current.is_empty() { lang.tr("Browse…").to_string() } else { folder_name(&current) });
    let (rect, button) = ui.allocate_exact_size(vec2(ui.available_width(), 48.0), Sense::click());
    let open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&button));
    let fill = if button.is_pointer_button_down_on() || open {
        t.pressed
    } else if button.hovered() {
        t.hover
    } else {
        t.input_bg
    };
    let border = if button.has_focus() || open { t.accent } else { t.input_border };
    ui.painter().rect(rect, 4.0, fill, Stroke::new(1.0, border), StrokeKind::Inside);
    paint_folder(ui, rect, &shown, &current, t, 32.0);
    icons::paint(ui.painter(), egui::Rect::from_center_size(pos2(rect.right() - 18.0, rect.center().y), vec2(16.0, 16.0)), Icon::Chevron, t.text_dim);
    let name = lang.tr("Choose a location").to_string();
    let button = button.labelled_by(label);
    button.widget_info(|| {
        let mut info = egui::WidgetInfo::labeled(egui::Role::ComboBox, true, &name);
        info.current_text_value = Some(shown.clone());
        info
    });
    let width = rect.width();
    egui::Popup::menu(&button).width(width).accessible_name(name.clone()).close_behavior(egui::PopupCloseBehavior::CloseOnClick).show(|ui| {
        ui.set_width(width - 12.0);
        for f in &list {
            let label = shown_label(lang, f);
            if folder_row(ui, &label, &f.path, same_folder(&f.path, &current), t).clicked() {
                d.values.insert("folder".into(), json!(f.path));
            }
        }
        if let Some(pick) = &app.services.pick_folder {
            if !list.is_empty() {
                ui.separator();
            }
            let browse = lang.tr("Browse…");
            let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click());
            if r.hovered() {
                ui.painter().rect_filled(rect, 4.0, t.hover);
            }
            icons::paint(
                ui.painter(),
                egui::Rect::from_center_size(pos2(rect.left() + 20.0, rect.center().y), vec2(18.0, 18.0)),
                Icon::OpenFolder,
                t.text,
            );
            ui.painter().text(pos2(rect.left() + 40.0, rect.center().y), Align2::LEFT_CENTER, browse, theme::ui_font(14.0), t.text);
            r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, browse));
            if r.clicked()
                && let Some(picked) = pick(&current)
            {
                add_folder(d, &picked);
            }
        }
    });
}

/// A folder in the open list, named for assistive technology by its label.
fn folder_row(ui: &mut egui::Ui, label: &str, path: &str, selected: bool, t: &Tokens) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::click());
    if r.hovered() {
        ui.painter().rect_filled(rect, 4.0, t.hover);
    } else if selected {
        ui.painter().rect_filled(rect, 4.0, t.accent_soft);
    }
    paint_folder(ui, rect, label, path, t, 8.0);
    r.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, selected, label));
    r
}

/// A folder picked with Browse…: chosen, and listed first from now on while the dialog is open.
fn add_folder(d: &mut Dialog, path: &str) {
    let mut list: Vec<Json> = d.values.get("folders").and_then(Json::as_array).cloned().unwrap_or_default();
    if !list.iter().any(|f| f.get("path").and_then(Json::as_str).is_some_and(|p| same_folder(p, path))) {
        list.insert(0, json!({"label": folder_name(path), "path": path, "known": false}));
    }
    d.values.insert("folders".into(), Json::Array(list));
    d.values.insert("folder".into(), json!(path));
}

/// Writes the workbook to `path`; a failure is shown and the dialog stays. A save asked for by
/// Save Changes closes the workbook afterwards, and a quit goes on to the next one.
fn write(app: &mut SheetApp, ctx: &egui::Context, d: &Dialog, path: &str) -> bool {
    if let Err(e) = app.run("file.save", json!({"path": path})) {
        app.message = Some(("Save file".into(), crate::clean_error(&e)));
        return false;
    }
    app.ui.recent.retain(|p| p != path);
    app.ui.recent.insert(0, path.to_string());
    app.ui.recent.truncate(20);
    after_save(app, ctx, d);
    true
}

/// Save Changes asked for this save: the saved workbook closes, and a quit goes on.
fn after_save(app: &mut SheetApp, ctx: &egui::Context, d: &Dialog) {
    if !flag(d, "close") || app.session.active().is_none_or(|doc| doc.is_dirty()) {
        return;
    }
    app.run_or_alert("file.close", json!({"force": true, "document": true}));
    if flag(d, "quit") {
        app.continue_quit(ctx);
    }
}

/// More options…: the File › Save As page, or the system Save As window where there is none
/// (or while Save Changes waits for the answer, which the page can't give).
fn more_options(app: &mut SheetApp, ctx: &egui::Context, d: &Dialog) {
    if flag(d, "close") || !app.chrome.file_tab {
        app.open_dialog("saveAs", json!({"format": "xlsx"}));
        after_save(app, ctx, d);
    } else {
        app.open_backstage(crate::backstage::Page::SaveAs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_windows_refuses_are_not_valid() {
        for name in ["Book1", "Q3 report", "отчёт", "a.b"] {
            assert!(valid_name(name), "{name}");
        }
        for name in ["", "a/b", "a\\b", "a:b", "a*b", "a?b", "a\"b", "a<b", "a>b", "a|b", "a.", "..", "a\u{1}b"] {
            assert!(!valid_name(name), "{name:?}");
        }
    }

    #[test]
    fn the_same_folder_matches_with_or_without_a_trailing_separator() {
        assert!(same_folder("/home/u/Documents", "/home/u/Documents/"));
        assert!(!same_folder("/home/u/Documents", "/home/u/Desktop"));
        assert_eq!(folder_name("/home/u/Reports"), "Reports");
    }
}
