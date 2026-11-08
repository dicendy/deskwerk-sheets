use super::Dialog;

/// Help ▸ About Deskwerk Sheets: the app, its contributors and the AI models that helped. The credits
/// are compiled in (`crate::credits`, docs/contributors.md).
pub(super) fn about(ui: &mut egui::Ui, d: &mut Dialog) {
    let lang = crate::i18n::current(ui.ctx());
    ui.horizontal(|ui| {
        for tab in ["About", "Contributors", "Models"] {
            if ui.selectable_label(d.tab == tab, lang.tr(tab)).clicked() {
                d.tab = tab.to_string();
            }
        }
    });
    ui.separator();
    match d.tab.as_str() {
        "Contributors" => crate::credits::contributors_ui(ui),
        "Models" => crate::credits::models_ui(ui),
        _ => {
            ui.heading("Deskwerk Sheets");
            ui.label(lang.tr("Version {version}").replace("{version}", env!("CARGO_PKG_VERSION")));
            ui.add_space(4.0);
            ui.label(lang.tr("A clean-room, open-source, Rust-native spreadsheet."));
            ui.label(lang.tr("Based on GridCraft by the ArtCraft team. Not affiliated with or endorsed by them."));
            ui.hyperlink_to("github.com/dicendy/deskwerk-sheets", "https://github.com/dicendy/deskwerk-sheets");
        }
    }
}
