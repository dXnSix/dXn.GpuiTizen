//! `cargo xtask store-check <tpk or staged dir>` (`STORE-1`, design `d27-store-publication`): what
//! can be checked before the owner submits a package in Seller Office, as one report.
//!
//! - The manifest (`tizen-manifest.xml`): a 10-character package id, a Tizen version
//!   (`x.y.z`, or `x.y.z.n` in a multi-architecture upload), API version 8.0, the `tv-samsung`
//!   profile, a UI application under the package id, and its icon in `shared/res/`, a 512×423 PNG
//!   (Samsung's TV app icon size).
//! - Privileges: only those the app's feature description declares under `## Privileges`.
//! - The native libraries in `lib/`: exactly one export and only allowlisted imports (`SYS-3`,
//!   the same check as `check-symbols`).
//! - Sizes: each library and the whole package, with a warning above a budget (Samsung publishes
//!   no numbers, only that review is strict on size).
//! - The feature description file the submission needs (`store/feature-description.<app>.md`).
//!
//! Everything here is pure: `main.rs` reads the package (a directory or the `.tpk` zip) and runs
//! the symbol check on the libraries.

use std::collections::BTreeMap;
use std::fmt;

/// The package's files: path (with `/`, relative to the package root) to contents.
pub type Files = BTreeMap<String, Vec<u8>>;

/// Samsung's TV app icon size.
pub const ICON_SIZE: (u32, u32) = (512, 423);
/// Above these sizes the report warns (review is strict on size; there are no published numbers).
pub const PACKAGE_BUDGET: u64 = 50 * 1024 * 1024;
pub const LIBRARY_BUDGET: u64 = 40 * 1024 * 1024;

/// How one check ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    Warn,
    Fail,
}

/// One line of the report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub outcome: Outcome,
    pub check: &'static str,
    pub detail: String,
}

/// The report.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub items: Vec<Item>,
}

impl Report {
    fn push(&mut self, outcome: Outcome, check: &'static str, detail: impl Into<String>) {
        self.items.push(Item {
            outcome,
            check,
            detail: detail.into(),
        });
    }

    fn check(&mut self, ok: bool, check: &'static str, detail: impl Into<String>) {
        let outcome = if ok { Outcome::Ok } else { Outcome::Fail };
        self.push(outcome, check, detail);
    }

    /// Whether nothing failed.
    pub fn passed(&self) -> bool {
        self.items.iter().all(|item| item.outcome != Outcome::Fail)
    }

    /// The items of one check.
    pub fn items(&self, check: &str) -> impl Iterator<Item = &Item> {
        self.items.iter().filter(move |item| item.check == check)
    }

    /// The worst outcome of one check (`None` when it did not run).
    pub fn outcome(&self, check: &str) -> Option<Outcome> {
        self.items(check)
            .map(|item| item.outcome)
            .max_by_key(|outcome| match outcome {
                Outcome::Ok => 0,
                Outcome::Warn => 1,
                Outcome::Fail => 2,
            })
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            let mark = match item.outcome {
                Outcome::Ok => "ok  ",
                Outcome::Warn => "warn",
                Outcome::Fail => "FAIL",
            };
            writeln!(f, "{mark} {}: {}", item.check, item.detail)?;
        }
        let failed = self
            .items
            .iter()
            .filter(|i| i.outcome == Outcome::Fail)
            .count();
        let warned = self
            .items
            .iter()
            .filter(|i| i.outcome == Outcome::Warn)
            .count();
        if failed == 0 {
            writeln!(f, "store check passed ({warned} warning(s))")
        } else {
            writeln!(
                f,
                "store check failed: {failed} failure(s), {warned} warning(s)"
            )
        }
    }
}

/// `xml` without its comments.
pub fn strip_comments(xml: &str) -> String {
    let mut out = String::with_capacity(xml.len());
    let mut rest = xml;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        match rest[start..].find("-->") {
            Some(end) => rest = &rest[start + end + 3..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// Where each `<tag ...>` (or `<tag>`, `<tag/>`) of `xml` starts and its attribute text ends.
fn tags<'a>(xml: &'a str, tag: &str) -> Vec<(usize, &'a str, bool)> {
    let open = format!("<{tag}");
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = xml[from..].find(&open) {
        let start = from + at;
        let after = &xml[start + open.len()..];
        from = start + open.len();
        if !after.starts_with(|c: char| c.is_whitespace() || c == '>' || c == '/') {
            continue;
        }
        let Some(end) = after.find('>') else {
            break;
        };
        let inner = &after[..end];
        let self_closing = inner.trim_end().ends_with('/');
        found.push((
            start + open.len() + end + 1,
            inner.trim_end_matches('/'),
            self_closing,
        ));
    }
    found
}

/// The attributes of the first `<tag ...>` of `xml` (`key="value"` or `key='value'`, with or
/// without spaces around `=`).
fn attributes<'a>(xml: &'a str, tag: &str) -> Option<BTreeMap<&'a str, &'a str>> {
    let (_, mut rest, _) = *tags(xml, tag).first()?;
    let mut found = BTreeMap::new();
    loop {
        rest = rest.trim_start();
        let Some(eq) = rest.find('=') else {
            break;
        };
        let key = rest[..eq].trim();
        let value = rest[eq + 1..].trim_start();
        let quote = value.chars().next()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        let end = value[1..].find(quote)?;
        found.insert(key, &value[1..1 + end]);
        rest = &value[end + 2..];
    }
    Some(found)
}

/// The text of every `<tag ...>text</tag>` in `xml` (self-closing tags have none).
fn texts<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let close = format!("</{tag}");
    tags(xml, tag)
        .into_iter()
        .filter(|(_, _, self_closing)| !self_closing)
        .filter_map(|(body, _, _)| {
            let end = xml[body..].find(&close)?;
            Some(xml[body..body + end].trim())
        })
        .collect()
}

/// The width and height of a PNG.
pub fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.get(..8)? != SIGNATURE || bytes.get(12..16)? != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes.get(16..20)?.try_into().ok()?);
    let height = u32::from_be_bytes(bytes.get(20..24)?.try_into().ok()?);
    Some((width, height))
}

/// Where the next privilege URI starts (Tizen's `http://tizen.org/privilege/…` or Samsung's
/// `http://developer.samsung.com/privilege/…`).
fn privilege_start(text: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(at) = text[from..].find("http") {
        let start = from + at;
        let uri: String = text[start..]
            .chars()
            .take_while(|c| !c.is_whitespace() && !matches!(c, '`' | ')' | ','))
            .collect();
        if (uri.starts_with("http://") || uri.starts_with("https://"))
            && uri.contains("/privilege/")
        {
            return Some(start);
        }
        from = start + 4;
    }
    None
}

/// The placeholders a feature description template still holds: `[...]` that is not a link.
pub fn placeholders(description: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = description;
    while let Some(start) = rest.find('[') {
        let Some(end) = rest[start..].find(']') else {
            break;
        };
        let inner = &rest[start + 1..start + end];
        let after = &rest[start + end + 1..];
        let link = after.starts_with('(') || after.starts_with('[');
        let checkbox = matches!(inner, " " | "x" | "X");
        if !link && !checkbox && !inner.trim().is_empty() {
            found.push(format!("[{inner}]"));
        }
        rest = after;
    }
    found
}

/// The privileges a feature description declares: the URIs listed under its `## Privileges`
/// heading, up to the next heading.
pub fn declared_privileges(description: &str) -> Vec<String> {
    let mut in_section = false;
    let mut found = Vec::new();
    for line in description.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            in_section = trimmed.trim_start_matches('#').trim() == "Privileges";
            continue;
        }
        if !in_section {
            continue;
        }
        let mut rest = trimmed;
        while let Some(start) = privilege_start(rest) {
            let taken: String = rest[start..]
                .chars()
                .take_while(|c| !c.is_whitespace() && !matches!(c, '`' | ')' | ','))
                .collect();
            rest = &rest[start + taken.len()..];
            found.push(taken.trim_end_matches([':', '.', ';']).to_owned());
        }
    }
    found
}

fn valid_version(version: &str) -> bool {
    let limits: &[u32] = match version.split('.').count() {
        3 => &[255, 255, 65535],
        4 => &[255, 255, 65535, 99999],
        _ => return false,
    };
    version.split('.').zip(limits).all(|(part, limit)| {
        !part.is_empty()
            && part.bytes().all(|b| b.is_ascii_digit())
            && part.parse::<u32>().is_ok_and(|n| n <= *limit)
    })
}

/// What the check needs besides the package.
pub struct Inputs<'a> {
    pub files: &'a Files,
    /// The feature description (`store/feature-description.<app>.md`), if found.
    pub feature_description: Option<&'a str>,
    /// Where it was looked for, for the report.
    pub feature_description_path: &'a str,
}

/// Checks the package. `symbols` runs the symbol check on a library (its name and bytes) and
/// returns the violations.
pub fn check(
    inputs: &Inputs<'_>,
    mut symbols: impl FnMut(&str, &[u8]) -> anyhow::Result<Vec<String>>,
) -> Report {
    let mut report = Report::default();
    let files = inputs.files;

    // The manifest.
    let manifest = files
        .get("tizen-manifest.xml")
        .map(|bytes| strip_comments(&String::from_utf8_lossy(bytes)));
    let Some(manifest) = manifest else {
        report.check(
            false,
            "manifest",
            "no tizen-manifest.xml at the package root",
        );
        return report;
    };
    let root = attributes(&manifest, "manifest").unwrap_or_default();
    let package_id = root.get("package").copied().unwrap_or("");
    report.check(
        package_id.len() == 10 && package_id.bytes().all(|b| b.is_ascii_alphanumeric()),
        "package id",
        format!("`{package_id}` (10 letters and digits)"),
    );
    let version = root.get("version").copied().unwrap_or("");
    report.check(
        valid_version(version),
        "version",
        format!("`{version}` ([0-255].[0-255].[0-65535], plus .[0-99999] per architecture)"),
    );
    let api = root.get("api-version").copied().unwrap_or("");
    report.check(api == "8.0", "api version", format!("`{api}` (8.0)"));
    let profile = attributes(&manifest, "profile")
        .and_then(|a| a.get("name").map(|n| n.to_string()))
        .unwrap_or_default();
    report.check(
        profile == "tv-samsung",
        "profile",
        format!("`{profile}` (tv-samsung)"),
    );
    let app = attributes(&manifest, "ui-application").unwrap_or_default();
    let app_id = app.get("appid").copied().unwrap_or("");
    report.check(
        app_id.starts_with(&format!("{package_id}.")) && app_id.len() > package_id.len() + 1,
        "app id",
        format!("`{app_id}` (<package id>.<name>)"),
    );
    let exec = app.get("exec").copied().unwrap_or("");
    report.check(
        !exec.is_empty() && files.contains_key(&format!("bin/{exec}")),
        "exec",
        format!("`{exec}` in bin/"),
    );

    // The icon.
    match texts(&manifest, "icon").first() {
        None => report.check(false, "icon", "no <icon> in the manifest"),
        Some(icon) => match files.get(&format!("shared/res/{icon}")) {
            None => report.check(false, "icon", format!("shared/res/{icon} is missing")),
            Some(bytes) => match png_size(bytes) {
                Some(size) => report.check(
                    size == ICON_SIZE,
                    "icon",
                    format!(
                        "{icon} is {}×{} ({}×{} required)",
                        size.0, size.1, ICON_SIZE.0, ICON_SIZE.1
                    ),
                ),
                None => report.check(false, "icon", format!("{icon} is not a PNG")),
            },
        },
    }

    // The feature description and the privileges it declares.
    let declared = match inputs.feature_description {
        Some(text) if !text.trim().is_empty() => {
            let left = placeholders(text);
            report.check(
                left.is_empty(),
                "feature description",
                if left.is_empty() {
                    inputs.feature_description_path.to_owned()
                } else {
                    format!(
                        "{} still holds the template's placeholders: {}",
                        inputs.feature_description_path,
                        left.join(", ")
                    )
                },
            );
            declared_privileges(text)
        }
        Some(_) => {
            report.check(
                false,
                "feature description",
                format!("{} is empty", inputs.feature_description_path),
            );
            Vec::new()
        }
        None => {
            report.check(
                false,
                "feature description",
                format!(
                    "{} is missing (the submission needs it)",
                    inputs.feature_description_path
                ),
            );
            Vec::new()
        }
    };
    let privileges = texts(&manifest, "privilege");
    if privileges.is_empty() {
        report.check(true, "privileges", "none");
    }
    for privilege in privileges {
        report.check(
            declared.iter().any(|d| d == privilege),
            "privileges",
            if declared.iter().any(|d| d == privilege) {
                format!("{privilege} (declared)")
            } else {
                format!("{privilege} is not declared in the feature description's ## Privileges")
            },
        );
    }

    // The native libraries.
    let libraries: Vec<(&String, &Vec<u8>)> = files
        .iter()
        .filter(|(path, _)| path.starts_with("lib/") && path.ends_with(".so"))
        .collect();
    if libraries.is_empty() {
        report.check(false, "symbols", "no native library in lib/");
    }
    for (path, bytes) in &libraries {
        match symbols(path, bytes) {
            Ok(violations) if violations.is_empty() => report.check(
                true,
                "symbols",
                format!("{path}: one export, allowlisted imports"),
            ),
            Ok(violations) => {
                for violation in violations {
                    report.check(false, "symbols", format!("{path}: {violation}"));
                }
            }
            Err(err) => report.check(false, "symbols", format!("{path}: {err:#}")),
        }
        let size = bytes.len() as u64;
        report.push(
            if size > LIBRARY_BUDGET {
                Outcome::Warn
            } else {
                Outcome::Ok
            },
            "size",
            format!("{path}: {}", mib(size)),
        );
    }
    let total: u64 = files.values().map(|bytes| bytes.len() as u64).sum();
    report.push(
        if total > PACKAGE_BUDGET {
            Outcome::Warn
        } else {
            Outcome::Ok
        },
        "size",
        format!(
            "package: {} in {} files (budget {})",
            mib(total),
            files.len(),
            mib(PACKAGE_BUDGET)
        ),
    );
    report
}

fn mib(bytes: u64) -> String {
    format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
}

/// The feature description path for a package id: `store/feature-description.<app>.md` for this
/// repository's apps.
pub fn feature_description_for(package_id: &str) -> Option<String> {
    crate::package::APPS
        .iter()
        .find(|app| app.package_id == package_id)
        .map(|app| format!("store/feature-description.{}.md", app.name))
}

/// The package id in a manifest, to find its feature description.
pub fn package_id(manifest: &str) -> Option<String> {
    attributes(&strip_comments(manifest), "manifest")?
        .get("package")
        .map(|id| id.to_string())
}

/// The ELF machine of a library (`e_machine`), little-endian ELF only.
pub fn elf_machine(bytes: &[u8]) -> Option<u16> {
    if bytes.get(..4)? != b"\x7fELF" || *bytes.get(5)? != 1 {
        return None;
    }
    Some(u16::from_le_bytes(bytes.get(18..20)?.try_into().ok()?))
}

/// The CPU feature a library of this ELF machine needs (ARM, AArch64, x86-64).
pub fn feature_for_machine(machine: u16) -> Option<&'static str> {
    match machine {
        40 => Some("http://tizen.org/feature/platform.core.cpu.arch.armv7"),
        183 => Some("http://tizen.org/feature/platform.core.cpu.arch.armv8"),
        62 => Some("http://tizen.org/feature/platform.core.cpu.arch.x86_64"),
        _ => None,
    }
}

/// One package of a multi-architecture upload.
pub struct UploadPackage<'a> {
    /// Its name in the `.zip`.
    pub name: &'a str,
    pub files: &'a Files,
}

/// Checks a multi-architecture upload (Seller Office's launch checklist): 1 to 10 packages, one
/// package id and `x.y.z` for all, a fourth version number unique to each, and in each the CPU
/// feature of its library's architecture. `each` checks one package on its own; its items come
/// after the upload's, prefixed with the package's name.
pub fn check_upload(
    packages: &[UploadPackage<'_>],
    mut each: impl FnMut(&Files) -> Report,
) -> Report {
    let mut report = Report::default();
    report.check(
        (1..=crate::package::MAX_PACKAGES_PER_ZIP).contains(&packages.len()),
        "upload",
        format!(
            "{} package(s) (1 to {})",
            packages.len(),
            crate::package::MAX_PACKAGES_PER_ZIP
        ),
    );
    let mut ids = Vec::new();
    let mut bases = Vec::new();
    let mut numbers = Vec::new();
    for package in packages {
        let manifest = package
            .files
            .get("tizen-manifest.xml")
            .map(|bytes| strip_comments(&String::from_utf8_lossy(bytes)))
            .unwrap_or_default();
        let root = attributes(&manifest, "manifest").unwrap_or_default();
        ids.push(root.get("package").copied().unwrap_or("").to_owned());
        let version = root.get("version").copied().unwrap_or("");
        match version.rsplit_once('.') {
            Some((base, number)) if version.split('.').count() == 4 => {
                bases.push(base.to_owned());
                numbers.push(number.to_owned());
            }
            _ => report.check(
                false,
                "upload",
                format!("{}: version `{version}` has no fourth number", package.name),
            ),
        }
        let features: Vec<String> = tags(&manifest, "feature")
            .iter()
            .filter_map(|(_, inner, _)| {
                attributes(&format!("<feature {inner}>"), "feature")?
                    .get("name")
                    .map(|n| n.to_string())
            })
            .filter(|name| name.contains("platform.core.cpu.arch"))
            .collect();
        let machines: Vec<&'static str> = package
            .files
            .iter()
            .filter(|(path, _)| path.starts_with("lib/") && path.ends_with(".so"))
            .filter_map(|(_, bytes)| elf_machine(bytes).and_then(feature_for_machine))
            .collect();
        let matches = !machines.is_empty()
            && machines.iter().all(|m| features.iter().any(|f| f == m))
            && features.len() == 1;
        report.check(
            matches,
            "upload",
            format!(
                "{}: CPU feature {:?} for its library's {:?}",
                package.name, features, machines
            ),
        );
    }
    let same = |values: &[String]| values.windows(2).all(|w| w[0] == w[1]);
    report.check(
        same(&ids) && same(&bases),
        "upload",
        format!("one package id and version for all: {ids:?} {bases:?}"),
    );
    let mut unique = numbers.clone();
    unique.sort();
    unique.dedup();
    report.check(
        unique.len() == numbers.len(),
        "upload",
        format!("fourth version numbers unique per package: {numbers:?}"),
    );
    for package in packages {
        for item in each(package.files).items {
            report.items.push(Item {
                detail: format!("{}: {}", package.name, item.detail),
                ..item
            });
        }
    }
    report
}

#[cfg(test)]
mod tests;
