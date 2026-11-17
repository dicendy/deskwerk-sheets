use crate::widgets::{ButtonKind, button_row, push_button};
/// Help ▸ About Deskwerk Sheets: for now an empty window, its title and a Close button only.
pub(super) fn about(ui: &mut egui::Ui, open: &mut bool) {
    let lang = crate::i18n::current(ui.ctx());
    ui.add_space(8.0);
    button_row(ui, |ui| {
        if ui.add(push_button(format!(" {} ", lang.tr("Close")), ButtonKind::Neutral)).clicked() {
            *open = false;
        }
    });
}
