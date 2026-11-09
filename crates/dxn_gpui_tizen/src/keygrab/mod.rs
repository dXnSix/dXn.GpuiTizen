//! TV key grabs (design `d14-keys`): while the window is shown, the color, media and channel keys
//! are grabbed with `ecore_wl2_window_keygrab_set` (topmost), so they reach the app instead of the
//! TV; they are released when the app is hidden (paused) or terminates.
//! [`RunOptions::grab_keys`](crate::RunOptions::grab_keys) can narrow the set, by Tizen key name
//! (`XF86Red`) or GPUI key name (`red`). `Return`, `XF86Exit` and `XF86Home` are never grabbed:
//! Return's long press is Exit, which force-terminates the app, and Exit and Home belong to the TV.

use crate::keys::NAMED_KEYS;

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

/// Keys that are never grabbed.
pub const NEVER_GRABBED: &[&str] = &["Return", "XF86Exit", "XF86Home"];

/// Why a requested key is not grabbed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// `Return`, `XF86Exit` or `XF86Home`.
    Forbidden,
    /// Not one of [`DEFAULT_GRABS`]: the options only narrow the set.
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

/// The grab plan for `requested` (`None`: [`DEFAULT_GRABS`]), in [`DEFAULT_GRABS`] order and
/// without duplicates.
pub fn plan(requested: Option<&[String]>) -> GrabPlan {
    let Some(requested) = requested else {
        return GrabPlan {
            keys: DEFAULT_GRABS.to_vec(),
            refused: Vec::new(),
        };
    };
    let mut wanted = Vec::new();
    let mut refused = Vec::new();
    for name in requested {
        let tizen = tizen_name(name.trim());
        // Both spellings of the forbidden keys: `Return` and its GPUI name `enter`, ...
        if NEVER_GRABBED.contains(&tizen) || tizen == "KP_Enter" {
            refused.push((name.clone(), Refusal::Forbidden));
        } else if let Some(key) = DEFAULT_GRABS.iter().find(|key| **key == tizen) {
            wanted.push(*key);
        } else {
            refused.push((name.clone(), Refusal::NotGrabbable));
        }
    }
    let keys = DEFAULT_GRABS
        .iter()
        .copied()
        .filter(|key| wanted.contains(key))
        .collect();
    GrabPlan { keys, refused }
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

#[cfg(test)]
mod tests;
