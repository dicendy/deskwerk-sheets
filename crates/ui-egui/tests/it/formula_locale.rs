//! Language selection must affect UI entry, never the canonical engine/file boundary.
use deskwerk_sheets_engine::{
    Session,
    core::{CellError, CellRef, Value},
};
use deskwerk_sheets_ui_egui::{SheetApp, UiState, i18n::Language};
use serde_json::json;

fn app() -> SheetApp {
    let mut session = Session::new();
    session.new_workbook();
    let mut app = crate::common::app(session);
    app.ui.language = Language::En;
    app
}
fn formula(app: &SheetApp) -> String {
    app.session.active().unwrap().wb.active().unwrap().cell(CellRef::new(0, 0)).unwrap().formula.as_ref().unwrap().text.to_string()
}
fn enter(app: &mut SheetApp, text: &str) -> bool {
    app.begin_edit(Some(text.into()), false);
    app.commit_edit(0, 0, false, false)
}

#[test]
fn language_controls_entry_display_persistence_and_xlsx_roundtrip() {
    let mut app = app();
    assert!(!enter(&mut app, "=SUM(1;2)"));
    app.run("app.language.set", json!({"language":"es"})).unwrap();
    assert!(enter(&mut app, "=SI(VERDADERO;SUMA(1,5;2,25);0)"));
    assert_eq!(formula(&app), "IF(TRUE,SUM(1.5,2.25),0)");
    let saved = app.run("file.save", json!({"format":"xlsx"})).unwrap();
    app.run("file.open", json!({"name":"locale.xlsx", "base64":saved["base64"]})).unwrap();
    app.begin_edit(None, true);
    assert_eq!(app.editor.as_ref().unwrap().text, "=SI(VERDADERO;SUMA(1,5;2,25);0)");
    assert!(app.commit_edit(0, 0, false, false));
    assert_eq!(formula(&app), "IF(TRUE,SUM(1.5,2.25),0)");
    let state: UiState = serde_json::from_str(&serde_json::to_string(&app.ui).unwrap()).unwrap();
    assert_eq!(state.language, Language::Es);
    app.run("app.language.set", json!({"language":"en"})).unwrap();
    app.begin_edit(None, false);
    assert_eq!(app.editor.as_ref().unwrap().text, "=IF(TRUE,SUM(1.5,2.25),0)");
}

#[test]
fn language_does_not_change_api_syntax_or_an_edit_already_in_progress() {
    let mut app = app();
    app.run("app.language.set", json!({"language":"es"})).unwrap();
    assert!(app.run("cell.set", json!({"cell":"A1","input":"=SUM(1,2)"})).is_ok());
    assert!(app.run("cell.set", json!({"cell":"A2","input":"=SUMA(1;2)"})).is_err());
    app.begin_edit(Some("=SUMA(1,5;2)".into()), false);
    app.run("app.language.set", json!({"language":"en"})).unwrap();
    assert!(app.commit_edit(0, 0, false, false));
    assert_eq!(formula(&app), "SUM(1.5,2)");
}

#[test]
fn defined_and_local_names_survive_localized_editing() {
    let mut app = app();
    app.run("formulas.defineName", json!({"name":"FALSO", "refersTo":"=7"})).unwrap();
    app.run("app.language.set", json!({"language":"es"})).unwrap();
    assert!(enter(&mut app, "=LET(SUMA;LAMBDA(x;x+1);SUMA(2))+FALSO"));
    let original = formula(&app);
    app.begin_edit(None, false);
    assert_eq!(app.editor.as_ref().unwrap().text, "=LET(SUMA;LAMBDA(x;x+1);SUMA(2))+FALSO");
    assert!(app.commit_edit(0, 0, false, false));
    assert_eq!(formula(&app), original);
    assert_eq!(app.session.active().unwrap().wb.active().unwrap().value(CellRef::new(0, 0)), Value::Number(10.0));
}

#[test]
fn localized_argument_hints_ignore_decimal_commas_arrays_and_nested_calls() {
    let mut app = app();
    app.run("app.language.set", json!({"language":"es"})).unwrap();
    for (text, expected) in [("=SI(1,5;SUMA(2;3);", 2), ("=SUMA({1,5\\2;3\\4};", 1), ("=SUMA(Table1[[a],[b]];", 1)] {
        app.begin_edit(Some(text.into()), false);
        assert_eq!(app.editor.as_ref().unwrap().current_function().unwrap().1, expected);
    }
}

#[test]
/// Spanish autocomplete offers `SUMA` instead of `SUM`; accepting it inserts `=SUMA(`, and English switches back.
fn localized_autocomplete_is_rendered_and_inserts_the_selected_name() {
    let mut app = app();
    app.run("app.language.set", json!({"language":"es"})).unwrap();
    let mut h = crate::common::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        app,
    );
    h.run_steps(4);
    h.state_mut().begin_edit(Some("=SU".into()), true);
    h.run_steps(4);
    h.input_mut().events.push(egui::Event::Text("M".into()));
    h.run_steps(4);
    assert!(h.state().editor.as_ref().unwrap().autocomplete.iter().any(|n| n == "SUMA"));
    assert!(!h.state().editor.as_ref().unwrap().autocomplete.iter().any(|n| n == "SUM"));
    if let Ok(path) = std::env::var("DESKWERK_SHEETS_LOCALE_SCREENSHOT") {
        h.render().unwrap().save(path).unwrap();
    }
    let ed = h.state_mut().editor.as_mut().unwrap();
    ed.ac_index = ed.autocomplete.iter().position(|n| n == "SUMA").unwrap();
    assert!(ed.accept_autocomplete());
    assert_eq!(ed.text, "=SUMA(");
    h.run_steps(4);
    h.input_mut().events.push(egui::Event::Text("1,5;".into()));
    h.run_steps(4);
    assert_eq!(h.state().editor.as_ref().unwrap().current_function(), Some(("SUMA".into(), 1)));
    if let Ok(path) = std::env::var("DESKWERK_SHEETS_LOCALE_SCREENSHOT") {
        h.render().unwrap().save(std::path::Path::new(&path).with_file_name("spanish-hints.png")).unwrap();
        h.state_mut().cancel_edit();
        h.state_mut().ui.ribbon_tab = "View".into();
        h.run_steps(4);
        h.render().unwrap().save(std::path::Path::new(&path).with_file_name("spanish-setting.png")).unwrap();
    }
    h.state_mut().cancel_edit();
    h.state_mut().run("app.language.set", json!({"language":"en"})).unwrap();
    h.state_mut().begin_edit(Some("=SU".into()), true);
    h.run_steps(4);
    h.input_mut().events.push(egui::Event::Text("M".into()));
    h.run_steps(4);
    assert!(h.state().editor.as_ref().unwrap().autocomplete.iter().any(|n| n == "SUM"));
    assert!(!h.state().editor.as_ref().unwrap().autocomplete.iter().any(|n| n == "SUMA"));
}

#[test]
fn locale_preserves_text_cells_and_existing_entry_conveniences() {
    let mut app = app();
    app.run("app.language.set", json!({"language":"es"})).unwrap();
    assert!(enter(&mut app, "=SUMA(1,5;2"));
    assert_eq!(formula(&app), "SUM(1.5,2)");
    assert!(enter(&mut app, "-1,5"));
    assert_eq!(app.session.active().unwrap().wb.active().unwrap().value(CellRef::new(0, 0)), Value::Number(-1.5));
    app.run("home.numberFormat", json!({"format":"@"})).unwrap();
    assert!(enter(&mut app, "=SUM(1,2)"));
    assert_eq!(app.session.active().unwrap().wb.active().unwrap().value(CellRef::new(0, 0)), Value::text("=SUM(1,2)"));
    app.begin_edit(None, true);
    assert_eq!(app.editor.as_ref().unwrap().text, "=SUM(1,2)");
}

#[test]
fn empty_cell_autosum_places_caret_inside_parentheses() {
    let mut app = app();
    app.run("formulas.autoSum", json!({"enter": false})).unwrap();
    let ed = app.editor.as_ref().unwrap();
    assert_eq!(ed.text, "=SUM()");
    assert_eq!(ed.caret, ed.text.chars().count() - 1);
}

#[test]
/// Insert Function and AutoSum use `=SUMA(`, while user-supplied formula text is not translated twice.
fn canonical_engine_templates_enter_the_selected_formula_language() {
    let mut app = app();
    app.run("app.language.set", json!({"language":"es"})).unwrap();
    app.run("formulas.insertFunction", json!({"name":"SUM"})).unwrap();
    assert_eq!(app.editor.as_ref().unwrap().text, "=SUMA(");
    app.cancel_edit();
    app.run("cell.set", json!({"cell":"A1","input":"2"})).unwrap();
    app.run("selection.set", json!({"range":"A2"})).unwrap();
    app.run("formulas.autoSum", json!({"enter":false})).unwrap();
    assert!(app.editor.as_ref().unwrap().text.starts_with("=SUMA("));
    app.cancel_edit();
    // Text supplied by the user is already in the selected UI syntax; do not translate it twice.
    app.begin_edit(Some("=SUMA(1,5;2)".into()), false);
    assert_eq!(app.editor.as_ref().unwrap().text, "=SUMA(1,5;2)");
    assert!(app.commit_edit(0, 0, false, false));
    if let Ok(path) = std::env::var("DESKWERK_SHEETS_LOCALE_SCREENSHOT") {
        app.ui.ribbon_tab = "Formulas".into();
        app.run("formulas.insertFunction", json!({"name":"SUM"})).unwrap();
        let mut h = crate::common::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(
            |ui, app: &mut SheetApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(5);
        h.render().unwrap().save(std::path::Path::new(&path).with_file_name("spanish-ribbon-template.png")).unwrap();
    }
}

#[test]
fn portuguese_interface_retains_its_canonical_formula_fallback() {
    let mut app = app();
    app.run("app.language.set", json!({"code":"pt-BR"})).unwrap();
    assert_eq!(app.ui.language, Language::PtBr);
    assert!(enter(&mut app, "=SUM(1.5,2.25)"));
    app.begin_edit(None, true);
    assert_eq!(app.editor.as_ref().unwrap().text, "=SUM(1.5,2.25)");
    assert!(!enter(&mut app, "=SUMA(1;2)"));
    let restored: UiState = serde_json::from_str(&serde_json::to_string(&app.ui).unwrap()).unwrap();
    assert_eq!(restored.language, Language::PtBr);
}

#[test]
fn czech_interface_retains_its_canonical_formula_fallback() {
    // The Czech table translates interface labels only: formulas keep canonical English names,
    // `,` separators and `.` decimals until a Czech formula locale exists.
    let mut app = app();
    app.run("app.language.set", json!({"code":"cs-CZ"})).unwrap();
    assert_eq!(app.ui.language, Language::Cs);
    assert!(enter(&mut app, "=SUM(1.5,2.25)"));
    app.begin_edit(None, true);
    assert_eq!(app.editor.as_ref().unwrap().text, "=SUM(1.5,2.25)");
    assert!(!enter(&mut app, "=SUMA(1;2)"));
    let restored: UiState = serde_json::from_str(&serde_json::to_string(&app.ui).unwrap()).unwrap();
    assert_eq!(restored.language, Language::Cs);
}

#[test]
fn user_defined_prefix_is_hidden_while_editing_and_kept_on_commit() {
    // Excel shows `_xludf.EOMONTH()` as `=EOMONTH()`; the cell still holds the user-defined call
    // (#726), and committing the edit unchanged must not turn it into the built-in.
    let mut app = app();
    app.run("cell.set", json!({"cell":"A1","input":"=_xludf.EOMONTH()&_xludf.CONCAT(\"_xludf.x\")"})).unwrap();
    app.begin_edit(None, true);
    assert_eq!(app.editor.as_ref().unwrap().text, "=EOMONTH()&CONCAT(\"_xludf.x\")");
    assert!(app.commit_edit(0, 0, false, false));
    assert_eq!(formula(&app), "_xludf.EOMONTH()&_xludf.CONCAT(\"_xludf.x\")");
    assert_eq!(app.session.active().unwrap().wb.active().unwrap().value(CellRef::new(0, 0)), Value::Error(CellError::Name));
    app.run("app.language.set", json!({"language":"es"})).unwrap();
    app.begin_edit(None, true);
    assert_eq!(app.editor.as_ref().unwrap().text, "=EOMONTH()&CONCAT(\"_xludf.x\")");
    assert!(app.commit_edit(0, 0, false, false));
    assert_eq!(formula(&app), "_xludf.EOMONTH()&_xludf.CONCAT(\"_xludf.x\")");
}

#[test]
fn turkish_interface_retains_its_canonical_formula_fallback() {
    let mut app = app();
    app.run("app.language.set", json!({"code":"tr-TR"})).unwrap();
    assert_eq!(app.ui.language, Language::Tr);
    assert!(enter(&mut app, "=SUM(1.5,2.25)"));
    app.begin_edit(None, true);
    assert_eq!(app.editor.as_ref().unwrap().text, "=SUM(1.5,2.25)");
    assert!(!enter(&mut app, "=TOPLA(1;2)"));
    let restored: UiState = serde_json::from_str(&serde_json::to_string(&app.ui).unwrap()).unwrap();
    assert_eq!(restored.language, Language::Tr);
}

#[test]
fn german_entry_display_and_xlsx_roundtrip() {
    let mut app = app();
    app.run("app.language.set", json!({"language":"de"})).unwrap();
    let typed = "=WENNFEHLER(SVERWEIS(1;B1:C2;2;FALSCH);#NV)+zählenwenn(B1:B2;\">0\")+SUMME(1,5;2,25)";
    assert!(enter(&mut app, typed));
    assert_eq!(formula(&app), "IFERROR(VLOOKUP(1,B1:C2,2,FALSE),#N/A)+COUNTIF(B1:B2,\">0\")+SUM(1.5,2.25)");
    let saved = app.run("file.save", json!({"format":"xlsx"})).unwrap();
    app.run("file.open", json!({"name":"de.xlsx", "base64":saved["base64"]})).unwrap();
    app.begin_edit(None, true);
    let shown = "=WENNFEHLER(SVERWEIS(1;B1:C2;2;FALSCH);#NV)+ZÄHLENWENN(B1:B2;\">0\")+SUMME(1,5;2,25)";
    assert_eq!(app.editor.as_ref().unwrap().text, shown);
    assert!(app.commit_edit(0, 0, false, false));
    assert_eq!(formula(&app), "IFERROR(VLOOKUP(1,B1:C2,2,FALSE),#N/A)+COUNTIF(B1:B2,\">0\")+SUM(1.5,2.25)");
    // A leading + before a name that starts with an umlaut is a formula too.
    assert!(enter(&mut app, "+ÜBERSETZEN(\"a\")"));
    assert_eq!(formula(&app), "+TRANSLATE(\"a\")");
    // Typed numbers take the decimal comma.
    assert!(enter(&mut app, "1,5"));
    assert_eq!(app.session.active().unwrap().wb.active().unwrap().value(CellRef::new(0, 0)), Value::Number(1.5));
}

#[test]
fn only_german_takes_a_non_ascii_letter_after_a_sign_as_a_formula() {
    let mut app = app();
    app.run("app.language.set", json!({"language":"es"})).unwrap();
    assert!(enter(&mut app, "-Árbol"));
    assert_eq!(app.session.active().unwrap().wb.active().unwrap().value(CellRef::new(0, 0)), Value::text("-Árbol"));
}

#[test]
fn german_and_french_numbers_and_dates_follow_the_language_unless_chosen() {
    let value = |app: &SheetApp| app.session.active().unwrap().wb.active().unwrap().value(CellRef::new(0, 0));
    let mut app = app();
    assert_eq!(app.session.locale(), deskwerk_sheets_engine::core::Locale::En);
    app.run("app.language.set", json!({"language":"de"})).unwrap();
    assert_eq!(app.session.locale(), deskwerk_sheets_engine::core::Locale::De);
    assert!(enter(&mut app, "31.12.2026"));
    assert_eq!(value(&app), Value::Number(46387.0));
    app.begin_edit(None, true);
    assert_eq!(app.editor.as_ref().unwrap().text, "31.12.2026");
    app.cancel_edit();
    assert!(enter(&mut app, "1.234,56"));
    assert_eq!(value(&app), Value::Number(1234.56));
    // A chosen locale outlasts a language change and is saved with the preferences.
    app.run("app.locale.set", json!({"locale":"en"})).unwrap();
    app.run("app.language.set", json!({"language":"fr"})).unwrap();
    assert_eq!(app.session.locale(), deskwerk_sheets_engine::core::Locale::En);
    let restored: UiState = serde_json::from_str(&serde_json::to_string(&app.ui).unwrap()).unwrap();
    assert_eq!(restored.number_locale.as_deref(), Some("en"));
    assert!(app.run("app.locale.set", json!({"locale":"xx"})).is_err());
    app.run("app.locale.set", json!({"locale":"auto"})).unwrap();
    assert_eq!(app.session.locale(), deskwerk_sheets_engine::core::Locale::Fr);
    assert!(enter(&mut app, "31/12/2026"));
    assert_eq!(value(&app), Value::Number(46387.0));
}

#[test]
fn french_entry_display_and_xlsx_roundtrip() {
    let mut app = app();
    app.run("app.language.set", json!({"language":"fr"})).unwrap();
    let typed = "=SIERREUR(RECHERCHEV(1;B1:C2;2;FAUX);#N/A)+nb.si(B1:B2;\">0\")+SOMME(1,5;2,25)+ESTERREUR(#VALEUR!)";
    assert!(enter(&mut app, typed));
    let canonical = "IFERROR(VLOOKUP(1,B1:C2,2,FALSE),#N/A)+COUNTIF(B1:B2,\">0\")+SUM(1.5,2.25)+ISERROR(#VALUE!)";
    assert_eq!(formula(&app), canonical);
    let saved = app.run("file.save", json!({"format":"xlsx"})).unwrap();
    app.run("file.open", json!({"name":"fr.xlsx", "base64":saved["base64"]})).unwrap();
    app.begin_edit(None, true);
    let shown = "=SIERREUR(RECHERCHEV(1;B1:C2;2;FAUX);#N/A)+NB.SI(B1:B2;\">0\")+SOMME(1,5;2,25)+ESTERREUR(#VALEUR!)";
    assert_eq!(app.editor.as_ref().unwrap().text, shown);
    assert!(app.commit_edit(0, 0, false, false));
    assert_eq!(formula(&app), canonical);
    // Typed numbers take the decimal comma.
    assert!(enter(&mut app, "1,5"));
    assert_eq!(app.session.active().unwrap().wb.active().unwrap().value(CellRef::new(0, 0)), Value::Number(1.5));
}

fn value(app: &SheetApp, a1: &str) -> Value {
    app.session.active().unwrap().wb.active().unwrap().value(CellRef::parse(a1).unwrap())
}

#[test]
/// German and French read pasted text and opened CSV files in their conventions; English as before.
fn pasted_text_and_opened_csv_follow_the_language() {
    let mut app = app();
    app.paste_external("1.234,56\t1,234.5".into());
    assert_eq!(value(&app, "A1"), Value::text("1.234,56"));
    assert_eq!(value(&app, "B1"), Value::Number(1234.5));
    app.run("app.language.set", json!({"language":"de"})).unwrap();
    app.run("selection.set", json!({"range":"A2"})).unwrap();
    app.paste_external("1.234,56\t31.12.2026\t12,5 %".into());
    assert_eq!(value(&app, "A2"), Value::Number(1234.56));
    assert_eq!(value(&app, "B2"), Value::Number(46387.0));
    assert_eq!(value(&app, "C2"), Value::Number(0.125));
    app.run("app.language.set", json!({"language":"fr"})).unwrap();
    app.run("selection.set", json!({"range":"A3"})).unwrap();
    app.paste_external("1\u{202F}234,56\t31/12/2026".into());
    assert_eq!(value(&app, "A3"), Value::Number(1234.56));
    assert_eq!(value(&app, "B3"), Value::Number(46387.0));

    // A file from the command line opens before the first frame: it still reads in the language.
    let dir = std::env::temp_dir().join(format!("deskwerk-sheets-locale-csv-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("de.csv");
    std::fs::write(&path, "Betrag;Datum\n-1.234,56;31.12.2026\n").unwrap();
    let mut app = self::app();
    app.ui.language = Language::De;
    app.open_path(&path.to_string_lossy());
    assert_eq!(value(&app, "A2"), Value::Number(-1234.56));
    assert_eq!(value(&app, "B2"), Value::Number(46387.0));
    std::fs::remove_dir_all(&dir).unwrap();
}
