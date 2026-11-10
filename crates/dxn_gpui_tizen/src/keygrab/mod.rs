//! TV key grabs (design `d14-keys`): while the window is shown, the color, media and channel keys
//! are grabbed with `ecore_wl2_window_keygrab_set` (topmost), so they reach the app instead of the
//! TV; they are released when the app is hidden (paused) or terminates, and grabbed again when it
//! resumes. [`RunOptions::grab_keys`](crate::RunOptions::grab_keys) chooses the set at start, by
//! Tizen key name (`XF86Red`) or GPUI key name (`red`); [`grab`], [`release`] and [`set`] change
//! it while the app runs ([`runtime`]). The keys of [`ON_REQUEST`] (the remote's Info key) are
//! grabbed only when asked for. `Return`, `XF86Exit` and `XF86Home` are never grabbed: Return's
//! long press is Exit, which force-terminates the app, and Exit and Home belong to the TV.

use std::cell::RefCell;

use crate::keys::NAMED_KEYS;

pub mod runtime;

pub use runtime::{GrabCalls, GrabChange, GrabSet};

/// The keys grabbed by default: colors, media and channels.
pub const DEFAULT_GRABS: &[&str] = &[
    "XF86Red",
    "XF86Green",
    "XF86Yellow",
    "XF86Blue",
    "XF86AudioPlay",
    "XF86AudioPause",
    "XF86PlayBack",
    "XF86AudioStop",
    "XF86AudioRewind",
    "XF86AudioForward",
    "XF86AudioNext",
    "XF86AudioPrev",
    "XF86RaiseChannel",
    "XF86LowerChannel",
];

/// Keys grabbed only when asked for (through the options or at run time), never by default: the
/// remote's Info key.
pub const ON_REQUEST: &[&str] = &["XF86Info"];

/// Every key that can be grabbed, in grab order: [`DEFAULT_GRABS`], then [`ON_REQUEST`].
pub fn grabbable() -> impl Iterator<Item = &'static str> {
    DEFAULT_GRABS.iter().chain(ON_REQUEST).copied()
}

/// Keys that are never grabbed.
pub const NEVER_GRABBED: &[&str] = &["Return", "XF86Exit", "XF86Home"];

/// Why a requested key is not grabbed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// `Return`, `XF86Exit` or `XF86Home`.
    Forbidden,
    /// Neither one of [`DEFAULT_GRABS`] nor one of [`ON_REQUEST`].
    NotGrabbable,
}

/// The keys to grab, and the requested keys that were refused (for a warning in the log).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GrabPlan {
    pub keys: Vec<&'static str>,
    pub refused: Vec<(String, Refusal)>,
}

/// The Tizen key name for a Tizen or GPUI key name.
fn tizen_name(name: &str) -> &str {
    NAMED_KEYS
        .iter()
        .find(|(_, gpui)| *gpui == name)
        .map_or(name, |(tizen, _)| tizen)
}

/// The grabbable keys among `names` (Tizen or GPUI names), in [`grabbable`] order and without
/// duplicates, and the names that were refused.
pub fn validate<S: AsRef<str>>(names: impl IntoIterator<Item = S>) -> GrabPlan {
    let mut wanted = Vec::new();
    let mut refused = Vec::new();
    for name in names {
        let name = name.as_ref();
        let tizen = tizen_name(name.trim());
        // Both spellings of the forbidden keys: `Return` and its GPUI name `enter`, ...
        if NEVER_GRABBED.contains(&tizen) || tizen == "KP_Enter" {
            refused.push((name.to_owned(), Refusal::Forbidden));
        } else if let Some(key) = grabbable().find(|key| *key == tizen) {
            wanted.push(key);
        } else {
            refused.push((name.to_owned(), Refusal::NotGrabbable));
        }
    }
    let keys = grabbable().filter(|key| wanted.contains(key)).collect();
    GrabPlan { keys, refused }
}

/// The grab plan for `requested` (`None`: [`DEFAULT_GRABS`]), in [`grabbable`] order and without
/// duplicates.
pub fn plan(requested: Option<&[String]>) -> GrabPlan {
    match requested {
        None => GrabPlan {
            keys: DEFAULT_GRABS.to_vec(),
            refused: Vec::new(),
        },
        Some(requested) => validate(requested),
    }
}

/// What the shell does with the plan's keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrabCall {
    /// `ecore_wl2_window_keygrab_set` (topmost) for every key.
    Grab,
    /// `ecore_wl2_window_keygrab_unset` for every key.
    Release,
}

/// Whether the keys are grabbed now.
#[derive(Clone, Copy, Debug, Default)]
pub struct KeyGrabs {
    grabbed: bool,
    terminated: bool,
}

impl KeyGrabs {
    pub fn grabbed(&self) -> bool {
        self.grabbed
    }

    /// The window was shown (opened, or the app resumed).
    pub fn shown(&mut self) -> Option<GrabCall> {
        if self.grabbed || self.terminated {
            return None;
        }
        self.grabbed = true;
        Some(GrabCall::Grab)
    }

    /// The window was hidden (the app paused).
    pub fn hidden(&mut self) -> Option<GrabCall> {
        if !self.grabbed {
            return None;
        }
        self.grabbed = false;
        Some(GrabCall::Release)
    }

    /// The app terminates: release, and never grab again.
    pub fn terminated(&mut self) -> Option<GrabCall> {
        self.terminated = true;
        self.hidden()
    }
}

thread_local! {
    /// The keys the app wants grabbed, and whether they are grabbed now (main thread).
    static GRABS: RefCell<GrabSet> = RefCell::new(GrabSet::new(DEFAULT_GRABS.to_vec()));
}

/// Grabs `keys` (Tizen or GPUI names) as well as the keys grabbed already: now when the window
/// is shown, else when it is shown next. They stay grabbed across pause and resume. Returns the
/// names that were refused ([`validate`]); the rest are grabbed. Call on the main thread (GPUI's
/// foreground thread). Off Tizen only the set changes.
pub fn grab<S: AsRef<str>>(keys: impl IntoIterator<Item = S>) -> Vec<(String, Refusal)> {
    change(|grabs| grabs.grab(keys))
}

/// Releases `keys` (Tizen or GPUI names): now when they are grabbed, and they are not grabbed
/// again on resume. Returns the names that were refused; a grabbable key that was not grabbed is
/// not refused. Main thread.
pub fn release<S: AsRef<str>>(keys: impl IntoIterator<Item = S>) -> Vec<(String, Refusal)> {
    change(|grabs| grabs.release(keys))
}

/// Grabs exactly `keys` (Tizen or GPUI names): grabs the new ones and releases those not among
/// them. Returns the names that were refused. Main thread.
pub fn set<S: AsRef<str>>(keys: impl IntoIterator<Item = S>) -> Vec<(String, Refusal)> {
    change(|grabs| grabs.set(keys))
}

/// The keys grabbed while the window is shown (Tizen names, in [`grabbable`] order). Main thread.
pub fn grabbed_keys() -> Vec<&'static str> {
    GRABS.with(|grabs| grabs.borrow().keys().to_vec())
}

fn change(f: impl FnOnce(&mut GrabSet) -> GrabChange) -> Vec<(String, Refusal)> {
    let GrabChange { calls, refused } = GRABS.with(|grabs| f(&mut grabs.borrow_mut()));
    perform(&calls);
    refused
}

/// Runs `f` on the main thread's grab set (the shell: start, show, hide, terminate).
#[cfg_attr(not(target_vendor = "tizen"), allow(dead_code))]
pub(crate) fn with_grabs<R>(f: impl FnOnce(&mut GrabSet) -> R) -> R {
    GRABS.with(|grabs| f(&mut grabs.borrow_mut()))
}

#[cfg(target_vendor = "tizen")]
fn perform(calls: &GrabCalls) {
    crate::tizen::perform_grabs(calls);
}

#[cfg(not(target_vendor = "tizen"))]
fn perform(_calls: &GrabCalls) {}

#[cfg(test)]
mod tests;
