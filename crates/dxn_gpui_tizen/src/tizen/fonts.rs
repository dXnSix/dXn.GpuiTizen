//! The text system (design `d17-settings-fonts`): gpui_wgpu's cosmic-text system without system
//! fonts, given only the allowed families' files from fontconfig's directories (`crate::fonts`),
//! memory-mapped and kept mapped for the process lifetime.

use std::borrow::Cow;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::Arc;

use gpui::PlatformTextSystem;
use gpui_wgpu::CosmicTextSystem;

use crate::fonts;

/// A read-only mapping of a whole file.
struct Mapping {
    ptr: *mut libc::c_void,
    len: usize,
}

impl Mapping {
    fn open(path: &Path) -> Option<Self> {
        let path = CString::new(path.as_os_str().as_bytes()).ok()?;
        // SAFETY: a NUL-terminated path; the descriptor is closed once mapped (the mapping keeps
        // the file), and a failed or empty mapping is never used.
        unsafe {
            let fd = libc::open(path.as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC);
            if fd < 0 {
                return None;
            }
            let mut stat: libc::stat = std::mem::zeroed();
            let len = if libc::fstat(fd, &mut stat) == 0 {
                usize::try_from(stat.st_size).unwrap_or(0)
            } else {
                0
            };
            let ptr = if len == 0 {
                libc::MAP_FAILED
            } else {
                libc::mmap(
                    std::ptr::null_mut(),
                    len,
                    libc::PROT_READ,
                    libc::MAP_PRIVATE,
                    fd,
                    0,
                )
            };
            libc::close(fd);
            (ptr != libc::MAP_FAILED).then_some(Self { ptr, len })
        }
    }

    /// Keeps the mapping for the rest of the process.
    fn leak(self) -> &'static [u8] {
        let bytes = self.as_ref();
        // SAFETY: never unmapped from here on, so the bytes live as long as the process.
        let bytes: &'static [u8] =
            unsafe { std::slice::from_raw_parts(bytes.as_ptr(), bytes.len()) };
        std::mem::forget(self);
        bytes
    }
}

impl AsRef<[u8]> for Mapping {
    fn as_ref(&self) -> &[u8] {
        // SAFETY: a live read-only mapping of `len` bytes.
        unsafe { std::slice::from_raw_parts(self.ptr.cast::<u8>(), self.len) }
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        // SAFETY: the mapping from `open`, unmapped once.
        unsafe { libc::munmap(self.ptr, self.len) };
    }
}

/// The platform's text system with the allowed families loaded.
pub(crate) fn text_system(requested: Option<&[String]>) -> Arc<CosmicTextSystem> {
    let root = Path::new("/");
    let report = fonts::font_dirs(root, Path::new(fonts::CONFIG));
    for missing in &report.missing {
        log::warn!("fontconfig include {} is missing", missing.display());
    }
    if report.fallback {
        log::warn!(
            "no font directories configured: using {}",
            fonts::FALLBACK_DIR
        );
    }
    let files = fonts::font_files(root, &report.dirs);
    let allowed = fonts::allowed_families(requested);
    let selected = fonts::select(&files, &allowed, Mapping::open);
    let ui = fonts::ui_family(&selected, &allowed).unwrap_or_else(|| "sans-serif".to_owned());
    log::info!(
        "fonts: {} of {} files for {allowed:?}, UI family {ui:?}",
        selected.len(),
        files.len()
    );
    let system = CosmicTextSystem::new_without_system_fonts(&ui);
    let data: Vec<Cow<'static, [u8]>> = selected
        .iter()
        .filter_map(|font| Mapping::open(&font.path))
        .map(|mapping| Cow::Borrowed(mapping.leak()))
        .collect();
    if let Err(err) = system.add_fonts(data) {
        log::error!("loading the fonts failed: {err:#}");
    }
    Arc::new(system)
}
