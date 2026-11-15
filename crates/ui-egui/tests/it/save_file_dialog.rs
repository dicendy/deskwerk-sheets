//! Save on a workbook that was never saved asks in a compact Save file window, as Excel 365
//! does: a name, a folder from a short list, More options… for the full Save As page. The Save
//! Changes question shows Excel's three answers, and its Save goes through the same window.

use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers};
use egui_kittest::kittest::Queryable as _;
use gridcraft_ui_egui::i18n::Language;
use gridcraft_ui_egui::{Services, SheetApp};
use serde_json::json;

type H = egui_kittest::Harness<'static, SheetApp>;

/// Folders standing in for Documents, Desktop, a recent file's folder and a folder Browse… picks.
struct Places {
    root: PathBuf,
    documents: PathBuf,
    desktop: PathBuf,
    recent: PathBuf,
    picked: PathBuf,
}

impl Places {
    fn new(tag: &str) -> Places {
        let root = std::env::temp_dir().join(format!("gridcraft-save-file-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let p = Places {
            documents: root.join("Documents"),
            desktop: root.join("Desktop"),
            recent: root.join("Reports"),
            picked: root.join("Picked"),
            root,
        };
        for dir in [&p.documents, &p.desktop, &p.recent, &p.picked] {
            std::fs::create_dir_all(dir).unwrap();
        }
        p
    }
}

impl Drop for Places {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn s(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// A desktop app (it has the system Save As) with the test folders as Documents and Desktop.
fn harness(p: &Places) -> H {
    let (documents, desktop, picked) = (s(&p.documents), s(&p.desktop), s(&p.picked));
    let services = Services {
        pick_save: Some(Box::new(|_| None)),
        pick_folder: Some(Box::new(move |_| Some(picked.clone()))),
        known_folders: Some(Box::new(move || vec![("Documents".into(), documents.clone()), ("Desktop".into(), desktop.clone())])),
        ..Default::default()
    };
    let mut h = crate::support::services_harness(services, 4);
    h.state_mut().run("cell.set", json!({"cell": "A1", "input": "42"})).unwrap();
    h.run_steps(2);
    h
}

fn dialog_name(h: &H) -> Option<String> {
    h.state().dialog.as_ref().map(|d| d.name().to_string())
}

fn value(h: &H, key: &str) -> serde_json::Value {
    h.state().dialog.as_ref().and_then(|d| d.values.get(key).cloned()).unwrap_or_default()
}

fn save_shortcut(h: &mut H) {
    h.key_press_modifiers(Modifiers::COMMAND, Key::S);
    h.run_steps(4);
}

fn click(h: &mut H, role: Role, label: &str) {
    h.get_by_role_and_label(role, label).click();
    h.run_steps(3);
}

fn is_xlsx(path: &Path) -> bool {
    std::fs::read(path).is_ok_and(|b| b.starts_with(b"PK"))
}

#[test]
fn save_on_a_new_workbook_asks_for_a_name_ready_to_type_and_enter_saves_it() {
    let p = Places::new("enter");
    let mut h = harness(&p);
    save_shortcut(&mut h);
    assert_eq!(dialog_name(&h).as_deref(), Some("saveFile"), "Ctrl+S opens the Save file window, not the system dialog");
    assert_eq!(value(&h, "name"), json!("Book1"));
    assert_eq!(value(&h, "folder"), json!(s(&p.documents)), "Documents is the folder at first");
    assert!(h.query_by_label("Save file").is_some(), "the window is titled Save file");
    assert!(h.query_by_label(".xlsx").is_none(), "the extension is shown, not a field or label of its own");
    let name = h.get_by_role_and_label(Role::TextInput, "File name");
    assert!(name.is_focused(), "the name has the keyboard");
    assert_eq!(name.value().as_deref(), Some("Book1"));
    // The whole name is selected: typing replaces it.
    for c in "Q3 report".chars() {
        h.input_mut().events.push(Event::Text(c.to_string()));
        h.run_steps(1);
    }
    assert_eq!(value(&h, "name"), json!("Q3 report"));
    h.key_press(Key::Enter);
    h.run_steps(4);
    let path = p.documents.join("Q3 report.xlsx");
    assert!(is_xlsx(&path), "Enter writes <folder>/<name>.xlsx");
    assert!(h.state().dialog.is_none());
    assert!(h.state().message.is_none());
    let doc = h.state().session.active().unwrap();
    assert_eq!(doc.path.as_deref(), Some(s(&path).as_str()));
    assert!(!doc.is_dirty());
    assert_eq!(h.state().ui.recent.first(), Some(&s(&path)), "the file is in the Recent list");
    // Saved once, Save writes the same file again without asking.
    h.state_mut().run("cell.set", json!({"cell": "A2", "input": "7"})).unwrap();
    save_shortcut(&mut h);
    assert!(h.state().dialog.is_none());
    assert!(!h.state().session.active().unwrap().is_dirty());
}

#[test]
fn escape_cancel_and_the_close_button_leave_without_saving() {
    let p = Places::new("cancel");
    let mut h = harness(&p);
    for how in ["Escape", "Cancel", "Close"] {
        h.state_mut().run_or_alert("file.save", json!({}));
        h.run_steps(4);
        assert_eq!(dialog_name(&h).as_deref(), Some("saveFile"), "{how}");
        match how {
            "Escape" => {
                h.key_press(Key::Escape);
                h.run_steps(3);
            }
            // The window's own × (the title bar has a Close button of its own).
            "Close" => {
                h.get_all_by_role_and_label(Role::Button, "Close").last().unwrap().click();
                h.run_steps(3);
            }
            label => click(&mut h, Role::Button, label),
        }
        assert!(h.state().dialog.is_none(), "{how} closes the window");
    }
    assert_eq!(std::fs::read_dir(&p.documents).unwrap().count(), 0, "nothing was written");
    assert!(h.state().session.active().unwrap().is_dirty());
}

#[test]
fn an_existing_file_is_replaced_only_once_confirmed() {
    let p = Places::new("replace");
    let existing = p.documents.join("Book1.xlsx");
    std::fs::write(&existing, b"old").unwrap();
    let mut h = harness(&p);
    save_shortcut(&mut h);
    click(&mut h, Role::Button, "Save");
    assert_eq!(value(&h, "replace"), json!(s(&existing)), "asked before replacing");
    assert!(h.query_by_label("A file named Book1.xlsx already exists. Do you want to replace it?").is_some());
    assert_eq!(std::fs::read(&existing).unwrap(), b"old");
    // Esc answers the question, not the window.
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert_eq!(dialog_name(&h).as_deref(), Some("saveFile"));
    assert!(value(&h, "replace").is_null());
    assert_eq!(std::fs::read(&existing).unwrap(), b"old");
    click(&mut h, Role::Button, "Save");
    click(&mut h, Role::Button, "Replace");
    assert!(is_xlsx(&existing), "Replace writes the workbook over the file");
    assert!(h.state().dialog.is_none());
}

#[test]
fn the_folder_list_has_recent_folders_documents_desktop_and_browse() {
    let p = Places::new("folders");
    let mut h = harness(&p);
    h.state_mut().ui.recent = vec![s(&p.recent.join("Budget.xlsx")), s(&p.documents.join("Notes.xlsx"))];
    save_shortcut(&mut h);
    let labels: Vec<String> = value(&h, "folders").as_array().unwrap().iter().map(|f| f["label"].as_str().unwrap().to_string()).collect();
    assert_eq!(labels, ["Reports", "Documents", "Desktop"], "a recent file in Documents is listed as Documents, once");
    let location = h.get_by_role_and_label(Role::ComboBox, "Choose a location");
    assert_eq!(location.value().as_deref(), Some("Documents"));
    location.click();
    h.run_steps(3);
    for label in ["Reports", "Documents", "Desktop", "Browse…"] {
        assert!(h.query_by_role_and_label(Role::Button, label).is_some(), "the list offers {label}");
    }
    click(&mut h, Role::Button, "Desktop");
    assert_eq!(value(&h, "folder"), json!(s(&p.desktop)));
    h.get_by_role_and_label(Role::ComboBox, "Choose a location").click();
    h.run_steps(3);
    click(&mut h, Role::Button, "Browse…");
    assert_eq!(value(&h, "folder"), json!(s(&p.picked)), "Browse… picks a folder with the system picker");
    click(&mut h, Role::Button, "Save");
    assert!(is_xlsx(&p.picked.join("Book1.xlsx")));
}

#[test]
fn a_name_windows_refuses_is_reported_and_an_empty_one_cannot_be_saved() {
    let p = Places::new("invalid");
    let mut h = harness(&p);
    save_shortcut(&mut h);
    h.state_mut().dialog.as_mut().unwrap().values.insert("name".into(), json!("a/b"));
    h.run_steps(2);
    click(&mut h, Role::Button, "Save");
    assert!(h.state().message.as_ref().is_some_and(|(_, m)| m == "This file name isn't valid."));
    assert_eq!(dialog_name(&h).as_deref(), Some("saveFile"), "the window stays to fix the name");
    h.state_mut().message = None;
    h.state_mut().dialog.as_mut().unwrap().values.insert("name".into(), json!("  "));
    h.run_steps(2);
    click(&mut h, Role::Button, "Save");
    assert_eq!(dialog_name(&h).as_deref(), Some("saveFile"));
    assert_eq!(std::fs::read_dir(&p.documents).unwrap().count(), 0);
    // A typed extension is the one the field shows, not part of the name.
    h.state_mut().dialog.as_mut().unwrap().values.insert("name".into(), json!("Plan.xlsx"));
    h.run_steps(2);
    click(&mut h, Role::Button, "Save");
    assert!(is_xlsx(&p.documents.join("Plan.xlsx")));
    assert!(!p.documents.join("Plan.xlsx.xlsx").exists());
}

#[test]
fn more_options_opens_the_save_as_page() {
    let p = Places::new("more");
    let mut h = harness(&p);
    save_shortcut(&mut h);
    click(&mut h, Role::Link, "More options…");
    assert!(h.state().dialog.is_none());
    assert_eq!(h.state().backstage.as_ref().map(|b| b.page.id()), Some("saveAs"));
}

#[test]
fn the_window_and_its_folders_are_translated() {
    let p = Places::new("ru");
    let mut h = harness(&p);
    h.state_mut().ui.language = Language::Ru;
    h.run_steps(2);
    save_shortcut(&mut h);
    assert!(h.query_by_label("Сохранение файла").is_some());
    assert!(h.query_by_role_and_label(Role::TextInput, "Имя файла").is_some());
    assert!(h.query_by_label("Дополнительные параметры…").is_some());
    assert!(h.query_by_role_and_label(Role::Button, "Сохранить").is_some());
    assert!(h.query_by_role_and_label(Role::Button, "Отмена").is_some());
    let location = h.get_by_role_and_label(Role::ComboBox, "Выберите расположение");
    assert_eq!(location.value().as_deref(), Some("Документы"));
    location.click();
    h.run_steps(3);
    assert!(h.query_by_role_and_label(Role::Button, "Рабочий стол").is_some());
    assert!(h.query_by_role_and_label(Role::Button, "Обзор…").is_some());
}

#[test]
fn save_changes_answers_in_excel_order_and_save_asks_where_then_closes() {
    let p = Places::new("close");
    let mut h = harness(&p);
    h.state_mut().run("file.new", json!({})).unwrap();
    h.state_mut().run("cell.set", json!({"cell": "A1", "input": "7"})).unwrap();
    h.run_steps(2);
    assert_eq!(h.state().session.documents().len(), 2);
    h.state_mut().run_or_alert("file.close", json!({}));
    h.run_steps(3);
    assert_eq!(dialog_name(&h).as_deref(), Some("saveChanges"));
    // Save, Don't Save, Cancel from left to right.
    let left = |h: &H, label: &str| h.get_by_role_and_label(Role::Button, label).rect().left();
    assert!(left(&h, "Save") < left(&h, "Don't Save") && left(&h, "Don't Save") < left(&h, "Cancel"));
    click(&mut h, Role::Button, "Save");
    assert_eq!(dialog_name(&h).as_deref(), Some("saveFile"), "a workbook never saved asks where to save it");
    let name = value(&h, "name").as_str().unwrap().to_string();
    click(&mut h, Role::Button, "Save");
    assert!(is_xlsx(&p.documents.join(format!("{name}.xlsx"))));
    assert_eq!(h.state().session.documents().len(), 1, "saved, the workbook closes");
    assert!(h.state().dialog.is_none());
}

#[test]
fn cancel_in_the_save_file_window_keeps_the_workbook_open() {
    let p = Places::new("keep");
    let mut h = harness(&p);
    h.state_mut().run("file.new", json!({})).unwrap();
    h.state_mut().run("cell.set", json!({"cell": "A1", "input": "7"})).unwrap();
    h.run_steps(2);
    h.state_mut().run_or_alert("file.close", json!({}));
    h.run_steps(3);
    click(&mut h, Role::Button, "Save");
    click(&mut h, Role::Button, "Cancel");
    assert!(h.state().dialog.is_none());
    assert_eq!(h.state().session.documents().len(), 2);
}

#[test]
fn save_changes_is_translated() {
    let p = Places::new("ru-close");
    let mut h = harness(&p);
    h.state_mut().ui.language = Language::Ru;
    h.state_mut().run("file.new", json!({})).unwrap();
    h.state_mut().run("cell.set", json!({"cell": "A1", "input": "7"})).unwrap();
    h.run_steps(2);
    h.state_mut().run_or_alert("file.close", json!({}));
    h.run_steps(3);
    let title = h.state().session.active().unwrap().display_title();
    assert!(h.query_by_label(&format!("Сохранить изменения в файле {title}?")).is_some());
    for label in ["Сохранить", "Не сохранять", "Отмена"] {
        assert!(h.query_by_role_and_label(Role::Button, label).is_some(), "{label}");
    }
}
