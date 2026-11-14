use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::*;

/// The pack as committed: `apps/gridcraft-web/fonts`.
fn pack_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/gridcraft-web/fonts")
}

fn pack_bytes(file: &str) -> Vec<u8> {
    std::fs::read(pack_dir().join(file)).unwrap_or_else(|e| panic!("{file}: {e}"))
}

/// The licences the repository accepts for font data (AGENTS.md), as the manifest spells them.
const ALLOWED_LICENSES: &[&str] = &["OFL-1.1", "Apache-2.0", "Bitstream-Vera AND LicenseRef-Arev-Fonts"];

/// SHA-256 (FIPS 180-4), so the manifest check needs no new dependency.
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be,
        0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa,
        0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85,
        0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
        0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f,
        0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut msg = data.to_vec();
    let bits = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bits.to_be_bytes());
    let mut w = [0u32; 64];
    for block in msg.as_chunks::<64>().0 {
        for (i, word) in block.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes(*word);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for (k, wi) in K.iter().zip(w.iter()) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(*k).wrapping_add(*wi);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *x = x.wrapping_add(y);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

#[test]
fn sha256_matches_known_digests() {
    assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert_eq!(
        sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

/// One manifest row: `file family style license license_file source sha256 bytes`.
struct Row {
    license: String,
    license_file: String,
    source: String,
    sha256: String,
    bytes: u64,
}

fn manifest() -> BTreeMap<String, Row> {
    let text = std::fs::read_to_string(pack_dir().join("MANIFEST.tsv")).expect("fonts/MANIFEST.tsv");
    let mut rows = BTreeMap::new();
    let mut lines = text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty());
    assert_eq!(lines.next(), Some("file\tfamily\tstyle\tlicense\tlicense_file\tsource\tsha256\tbytes"));
    for line in lines {
        let cols: Vec<&str> = line.split('\t').collect();
        let [file, _family, _style, license, license_file, source, sha256, bytes] = cols.as_slice() else { panic!("bad manifest row: {line}") };
        let row = Row {
            license: license.to_string(),
            license_file: license_file.to_string(),
            source: source.to_string(),
            sha256: sha256.to_string(),
            bytes: bytes.parse().expect("bytes"),
        };
        assert!(rows.insert(file.to_string(), row).is_none(), "{file} is listed twice");
    }
    rows
}

#[test]
fn the_manifest_covers_every_bundled_file_with_its_source_hash_and_licence() {
    let rows = manifest();
    let dir: BTreeSet<String> = std::fs::read_dir(pack_dir())
        .expect("fonts dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n != "MANIFEST.tsv")
        .collect();
    let listed: BTreeSet<String> = rows.keys().cloned().collect();
    assert_eq!(dir, listed, "every file in fonts/ is in the manifest, and every manifest file is there");
    let table: BTreeSet<String> = FILES.iter().map(|f| f.file.to_string()).collect();
    assert_eq!(table, listed, "font_pack::FILES lists the manifest's files");
    for f in FILES {
        let row = &rows[f.file];
        assert_eq!((row.sha256.as_str(), row.bytes, row.license.as_str()), (f.sha256, f.bytes, f.license), "{}: FILES matches the manifest", f.file);
        assert!(ALLOWED_LICENSES.contains(&f.license), "{}: licence {} is not on the allowlist", f.file, f.license);
        assert!(row.source.starts_with("https://"), "{}: a pinned https source", f.file);
        assert!(!row.source.contains("fonts.googleapis.com") && !row.source.contains("fonts.gstatic.com"), "{}: not from Google's font CDN", f.file);
        let licence = std::fs::read_to_string(pack_dir().join(&row.license_file)).unwrap_or_else(|e| panic!("{}: {}: {e}", f.file, row.license_file));
        let text = licence.to_lowercase();
        let looks_right = match f.license {
            "OFL-1.1" => text.contains("sil open font license"),
            "Apache-2.0" => text.contains("apache license"),
            _ => text.contains("bitstream") && text.contains("arev"),
        };
        assert!(looks_right, "{}: {} doesn't read as {}", f.file, row.license_file, f.license);
        let data = pack_bytes(f.file);
        assert_eq!(data.len() as u64, f.bytes, "{}: size", f.file);
        assert_eq!(sha256_hex(&data), f.sha256, "{}: sha256", f.file);
        assert!(crate::theme::font_parses(&data, 0), "{}: parses as a font", f.file);
    }
}

#[test]
fn every_face_set_uses_pack_files_and_every_file_is_used() {
    let mut used: BTreeSet<&str> = BTreeSet::new();
    for set in SETS {
        assert!(!set.faces.is_empty(), "{} has faces", set.family);
        assert!(set.faces.iter().any(|f| !f.bold && !f.italic), "{} has a regular face", set.family);
        for f in set.faces {
            assert!(pack_file(f.file).is_some(), "{}: {} is in the pack", set.family, f.file);
            used.insert(f.file);
        }
    }
    let all: BTreeSet<&str> = FILES.iter().map(|f| f.file).collect();
    assert_eq!(used, all, "no pack file is shipped unused");
}

#[test]
fn the_eager_set_is_calibris_twin_and_stays_small() {
    let set = face_set(EAGER).expect("the eager family is a pack family");
    assert_eq!(substitute("Calibri").map(|s| s.family), Some(set.family));
    let bytes: u64 = set.faces.iter().filter_map(|f| pack_file(f.file)).map(|f| f.bytes).sum();
    assert!(bytes < 3 << 20, "eager fonts are {bytes} bytes");
}

#[test]
fn office_names_map_to_their_twin_or_closest_open_font() {
    let twins = [
        ("Calibri", "Carlito"),
        ("Cambria", "Caladea"),
        ("Arial", "Liberation Sans"),
        ("Helvetica", "Liberation Sans"),
        ("Times New Roman", "Liberation Serif"),
        ("Courier New", "Liberation Mono"),
        ("Georgia", "Gelasio"),
        ("Segoe UI", "Selawik"),
    ];
    for (name, family) in twins {
        let s = substitute(name).unwrap_or_else(|| panic!("{name} has a substitute"));
        assert_eq!((s.family, s.closeness), (family, Closeness::Twin), "{name}");
    }
    let similar = [
        ("Verdana", "DejaVu Sans"),
        ("Tahoma", "DejaVu Sans Condensed"),
        ("Trebuchet MS", "Fira Sans"),
        ("Comic Sans MS", "Comic Neue"),
        ("Impact", "Anton"),
        ("Consolas", "Inconsolata"),
        ("Lucida Console", "DejaVu Sans Mono"),
        ("Garamond", "EB Garamond"),
        ("Palatino Linotype", "Lora"),
        ("Book Antiqua", "Lora"),
        ("Century Gothic", "Didact Gothic"),
        ("Calibri Light", "Lato Light"),
        ("Aptos", "Inter"),
        ("Candara", "PT Sans"),
        ("Constantia", "PT Serif"),
        ("Corbel", "Source Sans 3"),
        ("Franklin Gothic Book", "Libre Franklin"),
        ("Arial Narrow", "PT Sans Narrow"),
    ];
    for (name, family) in similar {
        let s = substitute(name).unwrap_or_else(|| panic!("{name} has a substitute"));
        assert_eq!((s.family, s.closeness), (family, Closeness::Similar), "{name}");
    }
    // Any capitalisation, and stray spaces, as workbooks spell names loosely.
    assert_eq!(substitute(" times new roman ").map(|s| s.family), Some("Liberation Serif"));
    assert!(substitute("Grossly Missing Font").is_none());
    for s in SUBSTITUTES {
        assert!(face_set(s.family).is_some(), "{} → {} is a pack family", s.name, s.family);
        assert!(!SETS.iter().any(|set| set.listed && set.family.eq_ignore_ascii_case(s.name)), "{} is not also an open font", s.name);
    }
}

#[test]
fn the_requested_open_fonts_are_offered_by_their_own_names() {
    for name in [
        "Roboto",
        "Open Sans",
        "Lato",
        "Montserrat",
        "Noto Sans",
        "Noto Serif",
        "PT Sans",
        "PT Serif",
        "Merriweather",
        "Lora",
        "Playfair Display",
        "Oswald",
        "Raleway",
        "Nunito",
        "Poppins",
        "EB Garamond",
        "Roboto Mono",
        "Roboto Slab",
        "Lexend",
        "Comic Neue",
        "Source Sans 3",
        "Inter",
    ] {
        let set = set_for(name).unwrap_or_else(|| panic!("{name} is in the pack"));
        assert!(set.listed && set.family == name, "{name} draws as itself");
        assert!(crate::ribbon::FONTS.contains(&name), "{name} is in the font list");
        assert!(substitute(name).is_none(), "{name} is not marked as a stand-in");
    }
}

#[test]
fn the_font_list_is_the_office_names_and_the_open_fonts_a_to_z() {
    let mut want: Vec<&str> = SUBSTITUTES.iter().map(|s| s.name).chain(SETS.iter().filter(|s| s.listed).map(|s| s.family)).collect();
    want.sort_by_key(|n| n.to_lowercase());
    want.dedup();
    assert_eq!(crate::ribbon::FONTS, want.as_slice());
}

/// Runs `f` with the pack on, under the font test lock, and turns it off again.
fn with_pack(f: impl FnOnce()) {
    let _g = crate::fonts::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    enable();
    f();
    reset();
    // Leave the standard roles bound, as the other font tests do.
    let _ = crate::theme::font_definitions_with(crate::i18n::Language::En, &[]);
}

fn deliver_set(family: &str) {
    for f in face_set(family).expect("pack family").faces {
        deliver(&format!("pack/{}", f.file), pack_bytes(f.file));
    }
}

#[test]
fn enabling_asks_for_the_eager_files_once() {
    with_pack(|| {
        let fetches = take_fetches();
        let files: Vec<&str> = fetches.iter().filter_map(|f| if let Fetch::Pack { file, .. } = f { Some(*file) } else { None }).collect();
        assert_eq!(files, ["Carlito-Regular.ttf", "Carlito-Bold.ttf", "Carlito-Italic.ttf", "Carlito-BoldItalic.ttf"]);
        assert!(matches!(&fetches[0], Fetch::Pack { rev, .. } if rev.len() == 16), "the URL carries the file's hash");
        enable();
        assert!(take_fetches().is_empty(), "nothing is asked twice");
    });
}

#[test]
fn a_substitute_binds_under_the_workbooks_name_once_it_arrives() {
    with_pack(|| {
        let _ = take_fetches();
        let used = vec!["Times New Roman".to_string()];
        let mut fonts = egui::FontDefinitions::default();
        assert!(bind_used(&mut fonts, &used, &[]).is_empty(), "nothing binds before the file is here");
        let asked: Vec<String> = take_fetches().iter().map(|f| f.key().to_string()).collect();
        assert!(asked.contains(&"pack/LiberationSerif-Regular.ttf".to_string()), "{asked:?}");
        assert_eq!(asked.len(), 4, "the four faces are asked for: {asked:?}");

        let before = generation();
        deliver_set("Liberation Serif");
        assert!(generation() > before, "a delivery tells the app to rebind");
        let mut fonts = egui::FontDefinitions::default();
        let bound = bind_used(&mut fonts, &used, &[]);
        for (b, i) in [(false, false), (true, false), (false, true), (true, true)] {
            let role = crate::fonts::role_key("Times New Roman", b, i);
            assert!(bound.contains(&role), "{role} is bound");
            let chain = &fonts.families[&egui::FontFamily::Name(role.clone().into())];
            assert!(chain[0].contains("pack/LiberationSerif-"), "{role} draws with Liberation Serif: {chain:?}");
            assert!(chain.iter().all(|k| fonts.font_data.contains_key(k)), "{role}: no key without data");
        }
        // An installed font of that name wins: the pack leaves it alone.
        let mut fonts = egui::FontDefinitions::default();
        assert!(bind_used(&mut fonts, &used, &used).is_empty());
    });
}

#[test]
fn junk_bytes_are_never_bound() {
    with_pack(|| {
        deliver("pack/LiberationSans-Regular.ttf", b"<html>not found</html>".to_vec());
        assert!(!is_loaded("pack/LiberationSans-Regular.ttf"));
        let mut fonts = egui::FontDefinitions::default();
        assert!(bind_used(&mut fonts, &["Arial".to_string()], &[]).is_empty());
    });
}

#[test]
fn the_font_definitions_render_a_substituted_name_with_its_twin() {
    with_pack(|| {
        deliver_set("Carlito");
        let used = vec!["Calibri".to_string()];
        let _fonts = crate::theme::font_definitions_with(crate::i18n::Language::En, &used);
        let role = crate::fonts::role_key("Calibri", false, false);
        assert!(crate::fonts::is_bound(&role), "Calibri draws with its own binding");
        assert!(matches!(crate::theme::cell_family("Calibri", false, false), egui::FontFamily::Name(f) if *f == *role));
    });
}

#[test]
fn local_fonts_win_over_the_pack_and_join_the_list() {
    with_pack(|| {
        assert!(substituted("Georgia"));
        let list = font_list();
        assert!(list.iter().any(|(n, marked)| n == "Georgia" && *marked));
        assert!(list.iter().any(|(n, marked)| n == "Roboto" && !*marked), "open fonts are not marked");

        set_local_api(true);
        assert!(local_access_offered());
        request_local_access();
        assert!(take_local_access_request());
        assert!(!take_local_access_request(), "a request is taken once");
        let face = |family: &str, style: &str, id: &str| LocalFace { family: family.into(), style: style.into(), id: id.into() };
        set_local_fonts(vec![face("Georgia", "Regular", "Georgia"), face("Georgia", "Bold", "Georgia-Bold"), face("Zapfino", "Regular", "Zapfino")]);
        assert!(!local_access_offered(), "the offer goes once the fonts are read");
        assert!(!substituted("Georgia"), "the computer's Georgia is the real one");
        let list = font_list();
        assert!(list.iter().any(|(n, marked)| n == "Georgia" && !*marked));
        assert_eq!(list.last().map(|(n, _)| n.as_str()), Some("Zapfino"), "the computer's other fonts follow the list");

        let _ = take_fetches();
        let mut fonts = egui::FontDefinitions::default();
        let _ = bind_used(&mut fonts, &["Georgia".to_string()], &[]);
        let asked: Vec<String> = take_fetches().iter().map(|f| f.key().to_string()).collect();
        assert!(asked.contains(&"local/Georgia".to_string()) && asked.contains(&"local/Georgia-Bold".to_string()), "{asked:?}");
        // The computer's face arrives (any font file will do here): it binds instead of Gelasio.
        deliver("local/Georgia", pack_bytes("Gelasio-Regular.ttf"));
        let mut fonts = egui::FontDefinitions::default();
        let bound = bind_used(&mut fonts, &["Georgia".to_string()], &[]);
        let role = crate::fonts::role_key("Georgia", false, false);
        assert!(bound.contains(&role));
        assert!(fonts.families[&egui::FontFamily::Name(role.into())][0].contains("local/Georgia"));
    });
}

#[test]
fn a_workbook_keeps_the_office_font_name_on_save() {
    with_pack(|| {
        deliver_set("Liberation Serif");
        let mut s = gridcraft_engine::Session::new();
        s.new_workbook();
        s.execute("home.fontName", serde_json::json!({"range": "A1", "name": "Times New Roman"})).expect("set the font");
        let used = vec!["Times New Roman".to_string()];
        let _fonts = crate::theme::font_definitions_with(crate::i18n::Language::En, &used);
        assert!(crate::fonts::is_bound(&crate::fonts::role_key("Times New Roman", false, false)), "drawn with the twin");
        let doc = s.doc().expect("a workbook");
        let bytes = gridcraft_engine::io::save_bytes(&doc.wb, "fonts.xlsx").expect("save");
        let (wb, _) = gridcraft_engine::io::open_bytes("fonts.xlsx", &bytes).expect("open");
        let a1 = gridcraft_engine::core::CellRef::parse("A1").expect("A1");
        let font = &wb.styles.get(wb.active().expect("a sheet").style_id(a1)).font;
        assert_eq!(font.name, "Times New Roman", "the file keeps the chosen name, not the twin's");
        let names: Vec<&str> = wb.styles.iter().map(|(_, st)| st.font.name.as_str()).collect();
        assert!(!names.iter().any(|n| n.contains("Liberation")), "no twin name leaks into the file: {names:?}");
    });
}
