//! The font pack built into the desktop app: the same files the web build serves from `fonts/`
//! ([`gridcraft_ui_egui::font_pack`]), embedded in the binary so no package needs extra files and
//! nothing is downloaded. Their licences ship in each package's `font-licences` folder (see
//! `packaging/env.sh` and `packaging/windows/package.ps1`), and are listed in NOTICE and
//! ATTRIBUTION.md.
//!
//! The pack only draws a font name the computer doesn't have: installed fonts are used directly,
//! so the desktop app has no "Use this computer's fonts" action.

use gridcraft_ui_egui::font_pack::{self, Fetch};

/// `embedded(file)`: a pack file's bytes, from `apps/gridcraft-web/fonts/<file>`.
macro_rules! pack {
    ($($file:literal),* $(,)?) => {
        fn embedded(file: &str) -> Option<&'static [u8]> {
            match file {
                $($file => Some(include_bytes!(concat!("../../gridcraft-web/fonts/", $file)).as_slice()),)*
                _ => None,
            }
        }
    };
}

pack![
    "Anton-Regular.ttf",
    "Caladea-Bold.ttf",
    "Caladea-BoldItalic.ttf",
    "Caladea-Italic.ttf",
    "Caladea-Regular.ttf",
    "Carlito-Bold.ttf",
    "Carlito-BoldItalic.ttf",
    "Carlito-Italic.ttf",
    "Carlito-Regular.ttf",
    "ComicNeue-Bold.ttf",
    "ComicNeue-BoldItalic.ttf",
    "ComicNeue-Italic.ttf",
    "ComicNeue-Regular.ttf",
    "DejaVuSans-Bold.ttf",
    "DejaVuSans-BoldItalic.ttf",
    "DejaVuSans-Italic.ttf",
    "DejaVuSans-Regular.ttf",
    "DejaVuSansCondensed-Bold.ttf",
    "DejaVuSansCondensed-BoldItalic.ttf",
    "DejaVuSansCondensed-Italic.ttf",
    "DejaVuSansCondensed-Regular.ttf",
    "DejaVuSansMono-Bold.ttf",
    "DejaVuSansMono-BoldItalic.ttf",
    "DejaVuSansMono-Italic.ttf",
    "DejaVuSansMono-Regular.ttf",
    "DidactGothic-Regular.ttf",
    "EBGaramond-Bold.ttf",
    "EBGaramond-BoldItalic.ttf",
    "EBGaramond-Italic.ttf",
    "EBGaramond-Regular.ttf",
    "FiraSans-Bold.ttf",
    "FiraSans-BoldItalic.ttf",
    "FiraSans-Italic.ttf",
    "FiraSans-Regular.ttf",
    "Gelasio-Bold.ttf",
    "Gelasio-BoldItalic.ttf",
    "Gelasio-Italic.ttf",
    "Gelasio-Regular.ttf",
    "Inconsolata-Bold.ttf",
    "Inconsolata-Regular.ttf",
    "Inter-Bold.ttf",
    "Inter-BoldItalic.ttf",
    "Inter-Italic.ttf",
    "Inter-Regular.ttf",
    "Lato-Bold.ttf",
    "Lato-BoldItalic.ttf",
    "Lato-Italic.ttf",
    "Lato-Light.ttf",
    "Lato-LightItalic.ttf",
    "Lato-Regular.ttf",
    "Lexend-Bold.ttf",
    "Lexend-Regular.ttf",
    "LiberationMono-Bold.ttf",
    "LiberationMono-BoldItalic.ttf",
    "LiberationMono-Italic.ttf",
    "LiberationMono-Regular.ttf",
    "LiberationSans-Bold.ttf",
    "LiberationSans-BoldItalic.ttf",
    "LiberationSans-Italic.ttf",
    "LiberationSans-Regular.ttf",
    "LiberationSerif-Bold.ttf",
    "LiberationSerif-BoldItalic.ttf",
    "LiberationSerif-Italic.ttf",
    "LiberationSerif-Regular.ttf",
    "LibreFranklin-Bold.ttf",
    "LibreFranklin-BoldItalic.ttf",
    "LibreFranklin-Italic.ttf",
    "LibreFranklin-Regular.ttf",
    "Lora-Bold.ttf",
    "Lora-BoldItalic.ttf",
    "Lora-Italic.ttf",
    "Lora-Regular.ttf",
    "Merriweather-Bold.ttf",
    "Merriweather-BoldItalic.ttf",
    "Merriweather-Italic.ttf",
    "Merriweather-Regular.ttf",
    "Montserrat-Bold.ttf",
    "Montserrat-BoldItalic.ttf",
    "Montserrat-Italic.ttf",
    "Montserrat-Regular.ttf",
    "NotoSans-Bold.ttf",
    "NotoSans-BoldItalic.ttf",
    "NotoSans-Italic.ttf",
    "NotoSans-Regular.ttf",
    "NotoSerif-Bold.ttf",
    "NotoSerif-BoldItalic.ttf",
    "NotoSerif-Italic.ttf",
    "NotoSerif-Regular.ttf",
    "Nunito-Bold.ttf",
    "Nunito-BoldItalic.ttf",
    "Nunito-Italic.ttf",
    "Nunito-Regular.ttf",
    "OpenSans-Bold.ttf",
    "OpenSans-BoldItalic.ttf",
    "OpenSans-Italic.ttf",
    "OpenSans-Regular.ttf",
    "Oswald-Bold.ttf",
    "Oswald-Regular.ttf",
    "PTSans-Bold.ttf",
    "PTSans-BoldItalic.ttf",
    "PTSans-Italic.ttf",
    "PTSans-Regular.ttf",
    "PTSansNarrow-Bold.ttf",
    "PTSansNarrow-Regular.ttf",
    "PTSerif-Bold.ttf",
    "PTSerif-BoldItalic.ttf",
    "PTSerif-Italic.ttf",
    "PTSerif-Regular.ttf",
    "PlayfairDisplay-Bold.ttf",
    "PlayfairDisplay-BoldItalic.ttf",
    "PlayfairDisplay-Italic.ttf",
    "PlayfairDisplay-Regular.ttf",
    "Poppins-Bold.ttf",
    "Poppins-BoldItalic.ttf",
    "Poppins-Italic.ttf",
    "Poppins-Regular.ttf",
    "Raleway-Bold.ttf",
    "Raleway-BoldItalic.ttf",
    "Raleway-Italic.ttf",
    "Raleway-Regular.ttf",
    "Roboto-Bold.ttf",
    "Roboto-BoldItalic.ttf",
    "Roboto-Italic.ttf",
    "Roboto-Regular.ttf",
    "RobotoMono-Bold.ttf",
    "RobotoMono-BoldItalic.ttf",
    "RobotoMono-Italic.ttf",
    "RobotoMono-Regular.ttf",
    "RobotoSlab-Bold.ttf",
    "RobotoSlab-Regular.ttf",
    "Selawik-Bold.ttf",
    "Selawik-Regular.ttf",
    "SourceSans3-Bold.ttf",
    "SourceSans3-BoldItalic.ttf",
    "SourceSans3-Italic.ttf",
    "SourceSans3-Regular.ttf",
];

/// Turns the pack on (Calibri's twin is asked for straight away).
pub fn start(ctx: &egui::Context) {
    font_pack::enable();
    pump(ctx);
}

/// Each frame: hands the pack the files it asked for. Local Font Access is web-only, so only pack
/// files are ever asked for here.
pub fn pump(ctx: &egui::Context) {
    let fetches = font_pack::take_fetches();
    if fetches.is_empty() {
        return;
    }
    for fetch in fetches {
        match &fetch {
            Fetch::Pack { file, .. } => match embedded(file) {
                Some(bytes) => font_pack::deliver(fetch.key(), bytes.to_vec()),
                None => font_pack::failed(fetch.key()),
            },
            Fetch::Local { .. } => font_pack::failed(fetch.key()),
        }
    }
    ctx.request_repaint();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pack_file_is_embedded() {
        for f in font_pack::FILES {
            assert_eq!(embedded(f.file).and_then(|b| u64::try_from(b.len()).ok()), Some(f.bytes), "{}", f.file);
        }
        assert_eq!(embedded("missing.ttf"), None);
    }
}
