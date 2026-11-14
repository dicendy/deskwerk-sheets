//! Typing faster than the frames come (a slow browser tab, a fast typist) loses no keys: the
//! characters typed in the pass that starts a cell edit, and in the passes before the editor has
//! the focus, all go into the cell in order, and Enter and a shortcut typed in the same pass act
//! after the text is in.

use egui::{Event, Key, Modifiers};
use gridcraft_engine::core::{CellRef, Value};
use std::sync::{Arc, Mutex};

type Harness = crate::common::Harness;

fn key(key: Key, modifiers: Modifiers) -> [Event; 2] {
    [true, false].map(|pressed| Event::Key { key, physical_key: Some(key), pressed, repeat: false, modifiers })
}

fn chars(text: &str) -> Vec<Event> {
    text.chars().map(|c| Event::Text(c.to_string())).collect()
}

/// Ctrl+S as a Windows or Linux keyboard sends it.
fn ctrl() -> Modifiers {
    Modifiers { ctrl: true, command: true, ..Modifiers::NONE }
}

/// Sends `events` in a single pass, then lets the app settle.
fn one_pass(h: &mut Harness, events: impl IntoIterator<Item = Event>) {
    h.input_mut().events.extend(events);
    h.step();
    h.run_steps(4);
}

fn value(h: &Harness, cell: &str) -> Value {
    h.state().session.active().unwrap().wb.active().unwrap().value(CellRef::parse(cell).unwrap())
}

fn active(h: &Harness) -> String {
    h.state().session.active().unwrap().selection.active.a1()
}

#[test]
fn a_word_typed_in_one_pass_keeps_every_character() {
    let mut h = crate::support::blank_harness(4);
    let mut events = chars("Wort1x74");
    events.extend(key(Key::Enter, Modifiers::NONE));
    one_pass(&mut h, events);
    assert!(h.state().editor.is_none(), "Enter in the same pass commits the edit");
    assert_eq!(value(&h, "A1"), Value::text("Wort1x74"));
    assert_eq!(active(&h), "A2");
}

#[test]
fn characters_typed_before_the_editor_has_focus_are_kept() {
    let mut h = crate::support::blank_harness(4);
    // The pass that starts the edit, then the next one, each with a few keys.
    h.input_mut().events.extend(chars("Wo"));
    h.step();
    h.input_mut().events.extend(chars("rt"));
    h.step();
    assert_eq!(h.state().editor.as_ref().map(|e| e.text.as_str()), Some("Wort"));
    let mut rest = chars("1x74");
    rest.extend(key(Key::Enter, Modifiers::NONE));
    one_pass(&mut h, rest);
    assert_eq!(value(&h, "A1"), Value::text("Wort1x74"));
}

#[test]
fn text_typed_after_enter_in_the_same_pass_goes_into_the_next_cell() {
    let mut h = crate::support::blank_harness(4);
    let mut events = chars("ab");
    events.extend(key(Key::Enter, Modifiers::NONE));
    events.extend(chars("xy"));
    events.extend(key(Key::Tab, Modifiers::NONE));
    events.extend(chars("q"));
    one_pass(&mut h, events);
    assert_eq!(value(&h, "A1"), Value::text("ab"));
    assert_eq!(value(&h, "A2"), Value::text("xy"));
    assert_eq!(h.state().editor.as_ref().map(|e| (e.cell.a1(), e.text.as_str())), Some(("B2".to_string(), "q")));
}

#[test]
fn f2_then_typing_in_the_same_pass_adds_to_the_entry() {
    let mut h = crate::support::blank_harness(4);
    h.state_mut().run("cell.set", serde_json::json!({"cell": "A1", "input": "ab"})).unwrap();
    h.run_steps(2);
    let mut events = key(Key::F2, Modifiers::NONE).to_vec();
    events.extend(chars("cd"));
    events.extend(key(Key::Enter, Modifiers::NONE));
    one_pass(&mut h, events);
    assert_eq!(value(&h, "A1"), Value::text("abcd"), "F2 edits the entry; the characters after it must not replace it");
}

#[test]
fn enter_and_ctrl_s_in_the_same_pass_save_the_typed_entry() {
    let mut h = crate::support::blank_harness(4);
    let downloaded = Arc::new(Mutex::new(Vec::new()));
    let sink = downloaded.clone();
    h.state_mut().services.download = Some(Box::new(move |_, bytes| *sink.lock().unwrap() = bytes.to_vec()));
    let mut events = chars("Wort1x74");
    events.extend(key(Key::Enter, Modifiers::NONE));
    events.extend(key(Key::S, ctrl()));
    one_pass(&mut h, events);
    assert_eq!(value(&h, "A1"), Value::text("Wort1x74"));
    let bytes = downloaded.lock().unwrap().clone();
    assert!(!bytes.is_empty(), "Ctrl+S after Enter in the same pass must save");
    let (saved, _) = gridcraft_engine::io::open_bytes("saved.xlsx", &bytes).unwrap();
    assert_eq!(saved.sheet(0).unwrap().value(CellRef::parse("A1").unwrap()), Value::text("Wort1x74"));
}
