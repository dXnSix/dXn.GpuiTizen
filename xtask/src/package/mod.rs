//! `cargo xtask package`: a TPK tree for one app and target under `target/tpk/<app>-<target>/`,
//! signed with `tz pack` only when the owner names a certificate profile.
//!
//! The app table, manifest rendering and the staged layout are pure functions tested by
//! `package::tests`; the cross build, `dotnet build`, file copies and `tz` stay in `main.rs`.

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::targets::Target;

/// The host assembly every TPK runs (`exec` in the manifest).
pub const HOST_ASSEMBLY: &str = "Dxn.GpuiTizen.Host.dll";
/// The name the app's library gets inside the TPK (the host's `DllImport`).
pub const LIBRARY_NAME: &str = "libdxn_app.so";

/// An app of this repository that can be packaged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct App {
    /// The `--app` name.
    pub name: &'static str,
    /// The Cargo package that builds its `cdylib`.
    pub package: &'static str,
    /// The 10-character Tizen package id.
    pub package_id: &'static str,
    /// The app id suffix (`<package id>.<suffix>`).
    pub app_suffix: &'static str,
    /// The label on the TV.
    pub label: &'static str,
    /// Files (relative to the repository) copied into the TPK's `res/`.
    pub resources: &'static [&'static str],
}

pub const APPS: &[App] = &[
    App {
        name: "diag",
        package: "dxn_gpui_tizen_diag",
        package_id: "DxnGpuiTzD",
        app_suffix: "Diag",
        label: "GpuiTizen Diag",
        resources: &[],
    },
    App {
        name: "counter",
        package: "example_counter",
        package_id: "DxnGpuiTzC",
        app_suffix: "Counter",
        label: "GpuiTizen Counter",
        resources: &[],
    },
    App {
        name: "long_list",
        package: "example_long_list",
        package_id: "DxnGpuiTzL",
        app_suffix: "LongList",
        label: "GpuiTizen Long List",
        resources: &[],
    },
    App {
        name: "video",
        package: "example_video",
        package_id: "DxnGpuiTzV",
        app_suffix: "Video",
        label: "GpuiTizen Video",
        resources: &["examples/video/res/clip.mp4"],
    },
];

impl App {
    pub fn find(name: &str) -> Result<&'static App> {
        match APPS.iter().find(|app| app.name == name) {
            Some(app) => Ok(app),
            None => {
                let names: Vec<&str> = APPS.iter().map(|a| a.name).collect();
                bail!("unknown app `{name}`; expected one of {}", names.join(", "))
            }
        }
    }

    /// The Tizen app id: `<package id>.<suffix>`.
    pub fn app_id(&self) -> String {
        format!("{}.{}", self.package_id, self.app_suffix)
    }
}

/// A Tizen package id is exactly 10 ASCII alphanumeric characters.
pub fn validate_package_id(id: &str) -> Result<()> {
    if id.len() != 10 || !id.bytes().all(|b| b.is_ascii_alphanumeric()) {
        bail!("package id `{id}` must be exactly 10 alphanumeric characters");
    }
    Ok(())
}

/// The values the manifest template takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestValues {
    pub package_id: String,
    pub app_id: String,
    pub label: String,
    pub version: String,
}

impl ManifestValues {
    pub fn for_app(app: &App, version: &str) -> Self {
        Self {
            package_id: app.package_id.to_owned(),
            app_id: app.app_id(),
            label: app.label.to_owned(),
            version: version.to_owned(),
        }
    }
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Renders `host/tizen-manifest.template.xml`; fails on an invalid package id, a version that is
/// not `x.y.z`, or a placeholder left unfilled.
pub fn render_manifest(template: &str, values: &ManifestValues) -> Result<String> {
    validate_package_id(&values.package_id)?;
    let parts: Vec<&str> = values.version.split('.').collect();
    if parts.len() != 3
        || !parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
    {
        bail!("version `{}` must be x.y.z", values.version);
    }
    // The header comment names the placeholders; it is not part of the rendered manifest.
    let body = match (template.find("<!--"), template.find("-->")) {
        (Some(start), Some(end)) if start < end => {
            format!(
                "{}{}",
                &template[..start],
                template[end + 3..].trim_start_matches('\n')
            )
        }
        _ => template.to_owned(),
    };
    let rendered = body
        .replace("{{PACKAGE_ID}}", &xml_escape(&values.package_id))
        .replace("{{APP_ID}}", &xml_escape(&values.app_id))
        .replace("{{LABEL}}", &xml_escape(&values.label))
        .replace("{{VERSION}}", &xml_escape(&values.version));
    if let Some(start) = rendered.find("{{") {
        let rest = &rendered[start..];
        let end = rest.find("}}").map_or(rest.len(), |e| e + 2);
        bail!(
            "manifest template has an unfilled placeholder `{}`",
            &rest[..end]
        );
    }
    Ok(rendered)
}

/// The staging directory: `<root>/target/tpk/<app>-<target>/`.
pub fn staging_dir(root: &Path, app: &App, target: Target) -> PathBuf {
    let arch = match target {
        Target::Armv7 => "armv7",
        Target::X86_64 => "x86_64",
        Target::Aarch64 => "aarch64",
    };
    root.join("target/tpk").join(format!("{}-{arch}", app.name))
}

/// One item of the staged TPK tree, relative to the staging directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    /// Copy `from` to the relative path.
    Copy { from: PathBuf, to: PathBuf },
    /// An empty directory.
    Dir(PathBuf),
    /// A file with generated contents.
    Write { to: PathBuf, contents: String },
}

/// The staged layout (flutter-tizen's single-architecture layout: the library in `lib/`, the
/// app's `resources` in `res/`).
pub fn layout(
    host_output: &[PathBuf],
    library: &Path,
    resources: &[PathBuf],
    icon: &Path,
    manifest: String,
) -> Vec<Entry> {
    let mut entries: Vec<Entry> = host_output
        .iter()
        .filter_map(|file| {
            let name = file.file_name()?;
            Some(Entry::Copy {
                from: file.clone(),
                to: Path::new("bin").join(name),
            })
        })
        .collect();
    entries.push(Entry::Copy {
        from: library.to_path_buf(),
        to: Path::new("lib").join(LIBRARY_NAME),
    });
    entries.push(Entry::Dir(PathBuf::from("res")));
    entries.extend(resources.iter().filter_map(|file| {
        Some(Entry::Copy {
            from: file.clone(),
            to: Path::new("res").join(file.file_name()?),
        })
    }));
    entries.push(Entry::Copy {
        from: icon.to_path_buf(),
        to: PathBuf::from("shared/res/icon.png"),
    });
    entries.push(Entry::Write {
        to: PathBuf::from("tizen-manifest.xml"),
        contents: manifest,
    });
    entries
}

/// The host build outputs a TPK ships: the assembly and its `.deps.json` (no PDB in Release).
pub fn host_files(output_dir: &Path) -> Vec<PathBuf> {
    [HOST_ASSEMBLY, "Dxn.GpuiTizen.Host.deps.json"]
        .iter()
        .map(|name| output_dir.join(name))
        .collect()
}

/// `tz pack` for a staged tree, run only with `--sign <profile>`.
pub fn sign_command(staging: &Path, profile: &str) -> Vec<String> {
    vec![
        "tz".to_owned(),
        "pack".to_owned(),
        "-t".to_owned(),
        "tpk".to_owned(),
        "-s".to_owned(),
        profile.to_owned(),
        staging.display().to_string(),
    ]
}

/// What `--sign` is for, when it is absent.
pub const UNSIGNED_NOTE: &str = "staged only: signing a TPK needs the owner's Samsung certificate \
     profile (`cargo xtask package ... --sign <profile>`, which runs `tz pack -t tpk -s <profile>`)";

#[cfg(test)]
mod tests;
