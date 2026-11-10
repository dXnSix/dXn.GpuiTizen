//! Key grabs that change while the app runs: [`GrabSet`] holds the keys the app wants grabbed
//! and whether the window is shown, and turns each change into the grab and release calls the
//! shell makes. Keys asked for while the window is hidden are grabbed when it is shown; keys
//! released while it is hidden are not grabbed again. Pause releases every key and resume grabs
//! them again; after terminate nothing is grabbed.

use super::{GrabCall, GrabPlan, KeyGrabs, Refusal, grabbable, validate};

/// The calls the shell makes: `ecore_wl2_window_keygrab_unset` for every key of `release`, then
/// `ecore_wl2_window_keygrab_set` (topmost) for every key of `grab`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GrabCalls {
    pub release: Vec<&'static str>,
    pub grab: Vec<&'static str>,
}

impl GrabCalls {
    pub fn is_empty(&self) -> bool {
        self.release.is_empty() && self.grab.is_empty()
    }

    fn of(call: Option<GrabCall>, keys: &[&'static str]) -> Self {
        match call {
            Some(GrabCall::Grab) => Self {
                release: Vec::new(),
                grab: keys.to_vec(),
            },
            Some(GrabCall::Release) => Self {
                release: keys.to_vec(),
                grab: Vec::new(),
            },
            None => Self::default(),
        }
    }
}

/// What a change made at run time asks of the shell, and the names it refused.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GrabChange {
    pub calls: GrabCalls,
    pub refused: Vec<(String, Refusal)>,
}

/// The keys the app wants grabbed (in [`grabbable`] order) and whether they are grabbed now.
#[derive(Clone, Debug)]
pub struct GrabSet {
    keys: Vec<&'static str>,
    state: KeyGrabs,
}

impl GrabSet {
    /// `keys` wanted (in any order; [`validate`]d keys), nothing grabbed yet.
    pub fn new(keys: Vec<&'static str>) -> Self {
        Self {
            keys: ordered(|key| keys.contains(&key)),
            state: KeyGrabs::default(),
        }
    }

    /// The keys grabbed while the window is shown.
    pub fn keys(&self) -> &[&'static str] {
        &self.keys
    }

    /// Whether the window is shown, so the keys are grabbed now.
    pub fn grabbed(&self) -> bool {
        self.state.grabbed()
    }

    /// The window was shown (opened, or the app resumed): grab every key.
    pub fn shown(&mut self) -> GrabCalls {
        GrabCalls::of(self.state.shown(), &self.keys)
    }

    /// The window was hidden (the app paused): release every key; they stay wanted.
    pub fn hidden(&mut self) -> GrabCalls {
        GrabCalls::of(self.state.hidden(), &self.keys)
    }

    /// The app terminates: release every key, and never grab again.
    pub fn terminated(&mut self) -> GrabCalls {
        GrabCalls::of(self.state.terminated(), &self.keys)
    }

    /// Wants `names` as well.
    pub fn grab<S: AsRef<str>>(&mut self, names: impl IntoIterator<Item = S>) -> GrabChange {
        let GrabPlan { keys, refused } = validate(names);
        let wanted = ordered(|key| self.keys.contains(&key) || keys.contains(&key));
        self.change(wanted, refused)
    }

    /// No longer wants `names`.
    pub fn release<S: AsRef<str>>(&mut self, names: impl IntoIterator<Item = S>) -> GrabChange {
        let GrabPlan { keys, refused } = validate(names);
        let wanted = ordered(|key| self.keys.contains(&key) && !keys.contains(&key));
        self.change(wanted, refused)
    }

    /// Wants exactly `names`.
    pub fn set<S: AsRef<str>>(&mut self, names: impl IntoIterator<Item = S>) -> GrabChange {
        let GrabPlan { keys, refused } = validate(names);
        self.change(keys, refused)
    }

    fn change(&mut self, wanted: Vec<&'static str>, refused: Vec<(String, Refusal)>) -> GrabChange {
        let calls = if self.grabbed() {
            GrabCalls {
                release: self
                    .keys
                    .iter()
                    .copied()
                    .filter(|key| !wanted.contains(key))
                    .collect(),
                grab: wanted
                    .iter()
                    .copied()
                    .filter(|key| !self.keys.contains(key))
                    .collect(),
            }
        } else {
            GrabCalls::default()
        };
        self.keys = wanted;
        GrabChange { calls, refused }
    }
}

/// The grabbable keys `keep` keeps, in [`grabbable`] order.
fn ordered(keep: impl Fn(&'static str) -> bool) -> Vec<&'static str> {
    grabbable().filter(|key| keep(key)).collect()
}

#[cfg(test)]
mod tests;
