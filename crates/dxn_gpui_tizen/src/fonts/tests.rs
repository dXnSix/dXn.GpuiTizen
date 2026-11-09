use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

/// A fixture sysroot in the temp directory, removed on drop.
struct Tree(PathBuf);

impl Tree {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "dxn-fonts-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn file(&self, path: &str, data: impl AsRef<[u8]>) -> &Self {
        let path = self.0.join(path.trim_start_matches('/'));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, data).unwrap();
        self
    }

    fn dirs(&self) -> DirsReport {
        font_dirs(&self.0, Path::new(CONFIG))
    }

    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        std::fs::read(self.0.join(path.strip_prefix("/").unwrap())).ok()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn paths(paths: &[&str]) -> Vec<PathBuf> {
    paths.iter().map(PathBuf::from).collect()
}

fn strings(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

/// A minimal sfnt face (`head`, `hhea`, `maxp`, `name`) with the given name records, as
/// ttf-parser needs it.
fn tables(names: &[(u16, &str)]) -> Vec<([u8; 4], Vec<u8>)> {
    let mut head = vec![0u8; 54];
    head[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    head[12..16].copy_from_slice(&0x5F0F_3CF5u32.to_be_bytes());
    head[18..20].copy_from_slice(&1000u16.to_be_bytes());
    let mut hhea = vec![0u8; 36];
    hhea[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    hhea[34..36].copy_from_slice(&1u16.to_be_bytes());
    let mut maxp = 0x0000_5000u32.to_be_bytes().to_vec();
    maxp.extend_from_slice(&1u16.to_be_bytes());
    let strings: Vec<Vec<u8>> = names
        .iter()
        .map(|(_, text)| text.encode_utf16().flat_map(u16::to_be_bytes).collect())
        .collect();
    let mut name = Vec::new();
    let count = names.len() as u16;
    for value in [0u16, count, 6 + 12 * count] {
        name.extend_from_slice(&value.to_be_bytes());
    }
    let mut offset = 0u16;
    for ((id, _), bytes) in names.iter().zip(&strings) {
        for value in [3u16, 1, 0x0409, *id, bytes.len() as u16, offset] {
            name.extend_from_slice(&value.to_be_bytes());
        }
        offset += bytes.len() as u16;
    }
    for bytes in &strings {
        name.extend_from_slice(bytes);
    }
    vec![
        (*b"head", head),
        (*b"hhea", hhea),
        (*b"maxp", maxp),
        (*b"name", name),
    ]
}

/// A font file: one face, or a collection of several.
fn font(faces: &[&[(u16, &str)]]) -> Vec<u8> {
    let collection = faces.len() > 1;
    let header = if collection { 12 + 4 * faces.len() } else { 0 };
    let mut out = vec![0u8; header];
    if collection {
        out[0..4].copy_from_slice(b"ttcf");
        out[4..8].copy_from_slice(&0x0001_0000u32.to_be_bytes());
        out[8..12].copy_from_slice(&(faces.len() as u32).to_be_bytes());
    }
    for (index, names) in faces.iter().enumerate() {
        let start = out.len();
        if collection {
            out[12 + 4 * index..16 + 4 * index].copy_from_slice(&(start as u32).to_be_bytes());
        }
        let tables = tables(names);
        let directory = 12 + 16 * tables.len();
        out.extend_from_slice(&0x0001_0000u32.to_be_bytes());
        for value in [tables.len() as u16, 64, 2, 0] {
            out.extend_from_slice(&value.to_be_bytes());
        }
        let mut offset = start + directory;
        let mut records = Vec::new();
        for (tag, data) in &tables {
            records.extend_from_slice(tag);
            records.extend_from_slice(&0u32.to_be_bytes());
            records.extend_from_slice(&(offset as u32).to_be_bytes());
            records.extend_from_slice(&(data.len() as u32).to_be_bytes());
            offset += data.len().next_multiple_of(4);
        }
        out.extend_from_slice(&records);
        for (_, data) in &tables {
            out.extend_from_slice(data);
            out.resize(out.len().next_multiple_of(4), 0);
        }
    }
    out
}

fn face(family: &str) -> Vec<u8> {
    font(&[&[(1, family)]])
}

#[test]
fn the_configuration_names_dirs_and_includes() {
    let text = r#"<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
  <!-- <dir>/commented/out</dir> -->
  <dir>/usr/share/fonts</dir>
  <dir prefix="default"> fonts/local </dir>
  <dir prefix="xdg">fonts</dir>
  <dir>~/.fonts</dir>
  <dir>/opt/a &amp; b</dir>
  <include ignore_missing="yes">conf.d</include>
  <include>/etc/fonts/local.conf</include>
  <match target="font"><edit name="dir"><string>x</string></edit></match>
  <cachedir>/var/cache/fontconfig</cachedir>
  <dir/>
</fontconfig>"#;
    let entries = parse_config(text, Path::new("/etc/fonts"));
    assert_eq!(
        entries.dirs,
        paths(&["/usr/share/fonts", "/etc/fonts/fonts/local", "/opt/a & b"])
    );
    assert_eq!(
        entries.includes,
        [
            (PathBuf::from("/etc/fonts/conf.d"), true),
            (PathBuf::from("/etc/fonts/local.conf"), false),
        ]
    );
}

#[test]
fn includes_are_followed_and_missing_ones_skipped() {
    let tree = Tree::new();
    tree.file(
        CONFIG,
        r#"<fontconfig>
  <dir>/usr/share/fonts</dir>
  <include ignore_missing="yes">conf.d</include>
  <include ignore_missing="yes">/etc/fonts/absent.conf</include>
  <include>local.conf</include>
  <include>fonts.conf</include>
</fontconfig>"#,
    )
    .file(
        "/etc/fonts/conf.d/20-tv.conf",
        "<fontconfig><dir>/opt/usr/share/fonts</dir><dir>/usr/share/fonts</dir></fontconfig>",
    )
    .file(
        "/etc/fonts/conf.d/10-first.conf",
        "<fontconfig><dir>/usr/share/fallback-fonts</dir></fontconfig>",
    )
    .file(
        "/etc/fonts/conf.d/README",
        "<fontconfig><dir>/no</dir></fontconfig>",
    )
    .file(
        "/etc/fonts/conf.d/x-late.conf",
        "<fontconfig><dir>/no</dir></fontconfig>",
    );
    let report = tree.dirs();
    assert_eq!(
        report.dirs,
        paths(&[
            "/usr/share/fonts",
            "/usr/share/fallback-fonts",
            "/opt/usr/share/fonts",
        ]),
        "NN*.conf files in name order, duplicates once"
    );
    assert_eq!(
        report.missing,
        paths(&["/etc/fonts/local.conf"]),
        "only an include not marked ignore_missing is reported"
    );
    assert!(!report.fallback);
}

#[test]
fn without_a_configuration_or_dirs_the_fallback_dir_is_used() {
    let tree = Tree::new();
    let report = tree.dirs();
    assert_eq!(report.dirs, paths(&[FALLBACK_DIR]));
    assert!(report.fallback);
    assert_eq!(report.missing, paths(&[CONFIG]));
    let tree = Tree::new();
    tree.file(
        CONFIG,
        "<fontconfig><include ignore_missing=\"yes\">conf.d</include></fontconfig>",
    );
    let report = tree.dirs();
    assert_eq!(report.dirs, paths(&[FALLBACK_DIR]));
    assert!(report.missing.is_empty());
}

#[test]
fn font_files_are_found_recursively_by_extension() {
    let tree = Tree::new();
    tree.file("/usr/share/fonts/a.ttf", "x")
        .file("/usr/share/fonts/sub/b.OTF", "x")
        .file("/usr/share/fonts/sub/deeper/c.ttc", "x")
        .file("/usr/share/fonts/fonts.dir", "x")
        .file("/usr/share/fonts/sub/README", "x")
        .file("/opt/fonts/d.otc", "x");
    let files = font_files(
        &tree.0,
        &paths(&["/usr/share/fonts", "/opt/fonts", "/absent"]),
    );
    assert_eq!(
        files,
        paths(&[
            "/opt/fonts/d.otc",
            "/usr/share/fonts/a.ttf",
            "/usr/share/fonts/sub/b.OTF",
            "/usr/share/fonts/sub/deeper/c.ttc",
        ])
    );
}

#[test]
fn families_come_from_every_face_of_the_name_table() {
    assert_eq!(families(&face("BreezeSans")), ["BreezeSans"]);
    let typographic = font(&[&[(1, "BreezeSans Light"), (16, "BreezeSans")]]);
    assert_eq!(
        families(&typographic),
        ["BreezeSans", "BreezeSans Light"],
        "the typographic family first"
    );
    let collection = font(&[&[(1, "TizenSansKorean")], &[(1, "TizenSansJapanese")]]);
    assert_eq!(
        families(&collection),
        ["TizenSansKorean", "TizenSansJapanese"]
    );
    assert!(families(b"not a font").is_empty());
    assert!(families(&[]).is_empty());
}

#[test]
fn only_allowed_families_and_their_fallbacks_are_selected() {
    let tree = Tree::new();
    tree.file(
        CONFIG,
        "<fontconfig><dir>/usr/share/fonts</dir></fontconfig>",
    )
    .file("/usr/share/fonts/breeze/BreezeSans.ttf", face("BreezeSans"))
    .file(
        "/usr/share/fonts/breeze/BreezeSansKorean.ttf",
        face("Breeze Sans Korean"),
    )
    .file(
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        face("DejaVu Sans"),
    )
    .file("/usr/share/fonts/broken.ttf", "garbage");
    let dirs = tree.dirs().dirs;
    let files = font_files(&tree.0, &dirs);
    assert_eq!(files.len(), 4);
    let allowed = allowed_families(None);
    let selected = select(&files, &allowed, |path| tree.read(path));
    assert_eq!(
        selected,
        [
            Selected {
                path: "/usr/share/fonts/breeze/BreezeSans.ttf".into(),
                families: strings(&["BreezeSans"]),
            },
            Selected {
                path: "/usr/share/fonts/breeze/BreezeSansKorean.ttf".into(),
                families: strings(&["Breeze Sans Korean"]),
            },
        ],
        "the default families, by prefix, ignoring case and spaces"
    );
    assert_eq!(
        ui_family(&selected, &allowed).as_deref(),
        Some("BreezeSans")
    );
    let dejavu = select(&files, &strings(&["dejavu sans"]), |path| tree.read(path));
    assert_eq!(dejavu.len(), 1);
    assert_eq!(dejavu[0].families, ["DejaVu Sans"]);
}

#[test]
fn absent_families_select_nothing() {
    let tree = Tree::new();
    tree.file("/usr/share/fonts/DejaVuSans.ttf", face("DejaVu Sans"));
    let files = font_files(&tree.0, &tree.dirs().dirs);
    assert_eq!(files.len(), 1, "found through the fallback dir");
    let allowed = allowed_families(None);
    let selected = select(&files, &allowed, |path| tree.read(path));
    assert!(selected.is_empty());
    assert_eq!(
        ui_family(&selected, &allowed).as_deref(),
        Some("SamsungOneUI"),
        "the first allowed family names the fallback"
    );
    assert!(select(&files, &[], |path| tree.read(path)).is_empty());
    assert!(select(&files, &strings(&[""]), |path| tree.read(path)).is_empty());
    assert!(
        select(&files, &allowed, |_| None::<Vec<u8>>).is_empty(),
        "unreadable files"
    );
}

#[test]
fn the_options_replace_the_default_families() {
    assert_eq!(allowed_families(None), DEFAULT_FAMILIES);
    assert_eq!(
        allowed_families(Some(&strings(&["Noto Sans"]))),
        ["Noto Sans"]
    );
    let selected = [Selected {
        path: "/f.ttf".into(),
        families: strings(&["TizenSansKorean"]),
    }];
    assert_eq!(
        ui_family(&selected, &strings(&["TizenSans"])).as_deref(),
        Some("TizenSansKorean"),
        "no exact match: the first selected family"
    );
}
