//! Keyboard events held back for the next pass, so keys typed faster than the frames come keep
//! their order across a change of keyboard owner.
//!
//! A slow frame (a busy browser tab) can bring a whole burst of typing at once: `Wort` Enter
//! Ctrl+S. The first character starts a cell edit from the grid, the rest of the word belongs to
//! the cell editor, Enter commits it, and Ctrl+S is the grid's again, but the grid has already
//! read this pass's input by the time the editor commits. So the cell editor acts on its input
//! up to the first key that can end or leave the edit, and the keys after it wait here for the
//! next pass, where they go ahead of that pass's own input.

use egui::{Event, Key};

fn held_id(ctx: &egui::Context) -> egui::Id {
    egui::Id::unique("deskwerk.held_keys").with(ctx.viewport_id())
}

/// Keyboard input, as opposed to pointer, touch and window events (which egui has already
/// turned into this pass's pointer state, so they stay where they are).
pub(crate) fn is_keyboard(ev: &Event) -> bool {
    matches!(ev, Event::Key { .. } | Event::Text(_) | Event::Paste(_) | Event::Copy | Event::Cut | Event::Ime(_) | Event::PasteImage(_))
}

/// A key the cell editor acts on itself rather than typing it, and after which the keyboard may
/// belong to someone else: Enter, Tab and Esc (commit or cancel), the arrows and Page keys (which
/// commit in Enter mode), F3 and F4, and any Ctrl, Cmd or Alt combination (a shortcut).
fn ends_typing(ev: &Event) -> bool {
    let Event::Key { key, pressed: true, modifiers: m, .. } = ev else { return false };
    m.ctrl
        || m.command
        || m.alt
        || matches!(
            key,
            Key::Enter
                | Key::Tab
                | Key::Escape
                | Key::ArrowUp
                | Key::ArrowDown
                | Key::ArrowLeft
                | Key::ArrowRight
                | Key::PageUp
                | Key::PageDown
                | Key::F3
                | Key::F4
        )
}

/// Puts the keys held in the previous pass ahead of this pass's input. Call once a pass, before
/// any widget reads the keyboard.
pub(crate) fn release(ctx: &egui::Context) {
    let id = held_id(ctx);
    let Some(held) = ctx.data_mut(|d| d.remove_temp::<Vec<Event>>(id)) else { return };
    if held.is_empty() {
        return;
    }
    ctx.input_mut(|i| {
        let fresh = std::mem::take(&mut i.events);
        i.events = held;
        i.events.extend(fresh);
    });
}

/// Leaves this pass's keyboard input up to and including the first key that ends typing (see
/// `ends_typing`), and holds the keyboard events after it for the next pass.
pub(crate) fn hold_after_first_command_key(ctx: &egui::Context) {
    let later = ctx.input_mut(|i| {
        let first = i.events.iter().position(ends_typing)?;
        let mut later = Vec::new();
        let mut kept = Vec::with_capacity(i.events.len());
        for (n, ev) in std::mem::take(&mut i.events).into_iter().enumerate() {
            if n > first && is_keyboard(&ev) { later.push(ev) } else { kept.push(ev) }
        }
        i.events = kept;
        Some(later).filter(|l| !l.is_empty())
    });
    let Some(later) = later else { return };
    let id = held_id(ctx);
    ctx.data_mut(|d| {
        let held = d.get_temp_mut_or_default::<Vec<Event>>(id);
        // Anything still held from before goes first (it was typed first).
        held.extend(later);
    });
    ctx.request_repaint();
}

/// Drops from this pass's input the keyboard events up to `last` (an index into
/// `InputState::events`), and `last` itself when `including`: the grid has acted on them, and
/// the cell editor it just opened takes the keys after them in this same pass.
pub(crate) fn drop_handled(ctx: &egui::Context, last: usize, including: bool) {
    ctx.input_mut(|i| {
        let mut n = 0usize;
        i.events.retain(|ev| {
            let handled = n < last || (including && n == last);
            n += 1;
            !(handled && is_keyboard(ev))
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Modifiers;

    fn key(key: Key, modifiers: Modifiers) -> Event {
        Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers }
    }

    fn text(s: &str) -> Event {
        Event::Text(s.into())
    }

    #[test]
    fn keys_after_the_first_command_key_wait_for_the_next_pass_in_order() {
        let ctx = egui::Context::default();
        let typed = vec![text("a"), text("b"), key(Key::Enter, Modifiers::NONE), key(Key::S, Modifiers::COMMAND), text("c")];
        let input = egui::RawInput { events: typed, ..Default::default() };
        let _ = ctx.run_ui(input, |ui| {
            hold_after_first_command_key(ui.ctx());
            let now = ui.input(|i| i.events.clone());
            assert_eq!(now, vec![text("a"), text("b"), key(Key::Enter, Modifiers::NONE)]);
        });
        let input = egui::RawInput { events: vec![text("d")], ..Default::default() };
        let _ = ctx.run_ui(input, |ui| {
            release(ui.ctx());
            let now = ui.input(|i| i.events.clone());
            assert_eq!(now, vec![key(Key::S, Modifiers::COMMAND), text("c"), text("d")]);
        });
    }

    #[test]
    fn plain_typing_is_not_held() {
        let ctx = egui::Context::default();
        let typed = vec![text("a"), key(Key::Backspace, Modifiers::NONE), text("b")];
        let input = egui::RawInput { events: typed.clone(), ..Default::default() };
        let _ = ctx.run_ui(input, |ui| {
            hold_after_first_command_key(ui.ctx());
            assert_eq!(ui.input(|i| i.events.clone()), typed);
        });
    }

    #[test]
    fn handled_keys_are_dropped_and_pointer_events_kept() {
        let ctx = egui::Context::default();
        let pos = egui::pos2(1.0, 1.0);
        let typed = vec![key(Key::ArrowDown, Modifiers::NONE), Event::PointerMoved(pos), text("W"), text("o")];
        let input = egui::RawInput { events: typed, ..Default::default() };
        let _ = ctx.run_ui(input, |ui| {
            drop_handled(ui.ctx(), 2, true);
            assert_eq!(ui.input(|i| i.events.clone()), vec![Event::PointerMoved(pos), text("o")]);
        });
    }
}
