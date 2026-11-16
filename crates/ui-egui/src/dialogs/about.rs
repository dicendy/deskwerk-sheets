/// Help ▸ About Deskwerk Sheets: for now an empty window, its title and a Close button only.
pub(super) fn about(ui: &mut egui::Ui, open: &mut bool) {
    let lang = crate::i18n::current(ui.ctx());
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(format!(" {} ", lang.tr("Close"))).clicked() {
                *open = false;
            }
        });
    });
}
