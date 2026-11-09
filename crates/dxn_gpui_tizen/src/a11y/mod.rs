//! Accessibility through AccessKit's Unix adapter (design `d18-a11y`), as `gpui_linux` uses it,
//! publishing GPUI's tree over standard AT-SPI.
//!
//! AccessKit calls its activation, action and deactivation handlers on its own thread; the bridge
//! posts each to the main thread, where GPUI's [`A11yCallbacks`] run. Activation therefore answers
//! AccessKit with no tree at once (the adapter waits), and the initial tree GPUI's activation
//! callback returns reaches the adapter from the main thread. Tree updates are passed on only once
//! the adapter has its initial tree, and stop again at deactivation. The window's bounds are the
//! full-screen window at the origin in physical pixels.
//!
//! The bridge and [`TreeState`] are platform-neutral; [`UnixAdapter`] (every Linux, so the Lima VM
//! runs the AT-SPI integration test) wraps `accesskit_unix::Adapter`.

use std::sync::{Arc, Mutex, PoisonError};

use gpui::A11yCallbacks;
use gpui::accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Rect, TreeUpdate,
};

/// Posts a task to the main thread (callable from any thread).
pub type Post = Arc<dyn Fn(Box<dyn FnOnce() + Send>) + Send + Sync>;

/// What the bridge hands the window on the main thread.
#[derive(Debug)]
pub enum MainThreadEvent {
    /// GPUI's activation callback returned the initial tree: give it to the adapter.
    InitialTree(TreeUpdate),
    /// The screen reader went away: tree updates stop until the next initial tree.
    Deactivated,
}

/// Receives [`MainThreadEvent`]s; called on the main thread only, from posted tasks.
pub type OnMainThread = Arc<dyn Fn(MainThreadEvent) + Send + Sync>;

struct Bridge {
    callbacks: Mutex<A11yCallbacks>,
    post: Post,
    on_main_thread: OnMainThread,
}

impl Bridge {
    fn callbacks<R>(&self, f: impl FnOnce(&A11yCallbacks) -> R) -> R {
        f(&self
            .callbacks
            .lock()
            .unwrap_or_else(PoisonError::into_inner))
    }
}

/// AccessKit's activation handler.
pub struct Activation(Arc<Bridge>);
/// AccessKit's action handler.
pub struct Action(Arc<Bridge>);
/// AccessKit's deactivation handler.
pub struct Deactivation(Arc<Bridge>);

impl ActivationHandler for Activation {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        let bridge = Arc::clone(&self.0);
        (self.0.post)(Box::new(move || {
            if let Some(tree) = bridge.callbacks(|c| (c.activation)()) {
                (bridge.on_main_thread)(MainThreadEvent::InitialTree(tree));
            }
        }));
        None
    }
}

impl ActionHandler for Action {
    fn do_action(&mut self, request: ActionRequest) {
        let bridge = Arc::clone(&self.0);
        (self.0.post)(Box::new(move || bridge.callbacks(|c| (c.action)(request))));
    }
}

impl DeactivationHandler for Deactivation {
    fn deactivate_accessibility(&mut self) {
        let bridge = Arc::clone(&self.0);
        (self.0.post)(Box::new(move || {
            bridge.callbacks(|c| (c.deactivation)());
            (bridge.on_main_thread)(MainThreadEvent::Deactivated);
        }));
    }
}

/// The three AccessKit handlers for GPUI's callbacks.
pub fn handlers(
    callbacks: A11yCallbacks,
    post: Post,
    on_main_thread: OnMainThread,
) -> (Activation, Action, Deactivation) {
    let bridge = Arc::new(Bridge {
        callbacks: Mutex::new(callbacks),
        post,
        on_main_thread,
    });
    (
        Activation(Arc::clone(&bridge)),
        Action(Arc::clone(&bridge)),
        Deactivation(bridge),
    )
}

/// Whether the adapter has its initial tree (main thread).
#[derive(Clone, Copy, Debug, Default)]
pub struct TreeState {
    initialized: bool,
}

impl TreeState {
    pub fn initialized(&self) -> bool {
        self.initialized
    }

    /// A main-thread event: returns the tree to give the adapter now, if any.
    pub fn event(&mut self, event: MainThreadEvent) -> Option<TreeUpdate> {
        match event {
            MainThreadEvent::InitialTree(tree) => {
                self.initialized = true;
                Some(tree)
            }
            MainThreadEvent::Deactivated => {
                self.initialized = false;
                None
            }
        }
    }

    /// GPUI pushed an update: passed on only after the initial tree.
    pub fn update(&self, update: TreeUpdate) -> Option<TreeUpdate> {
        self.initialized.then_some(update)
    }
}

/// The window's bounds for AccessKit: the full-screen window at the origin, in physical pixels.
pub fn window_bounds(pixels: (u32, u32)) -> Rect {
    Rect::new(0.0, 0.0, f64::from(pixels.0), f64::from(pixels.1))
}

/// GPUI's window hooks over `accesskit_unix` (main thread).
#[cfg(target_os = "linux")]
pub struct UnixAdapter {
    adapter: accesskit_unix::Adapter,
    tree: TreeState,
}

#[cfg(target_os = "linux")]
impl UnixAdapter {
    /// `a11y_init`: an adapter for a window of `pixels`, focused.
    pub fn new(
        callbacks: A11yCallbacks,
        post: Post,
        on_main_thread: OnMainThread,
        pixels: (u32, u32),
    ) -> Self {
        let (activation, action, deactivation) = handlers(callbacks, post, on_main_thread);
        let mut adapter = accesskit_unix::Adapter::new(activation, action, deactivation);
        let bounds = window_bounds(pixels);
        adapter.set_root_window_bounds(bounds, bounds);
        adapter.update_window_focus_state(true);
        Self {
            adapter,
            tree: TreeState::default(),
        }
    }

    pub fn initialized(&self) -> bool {
        self.tree.initialized()
    }

    /// A [`MainThreadEvent`] from the bridge.
    pub fn event(&mut self, event: MainThreadEvent) {
        if let Some(tree) = self.tree.event(event) {
            self.adapter.update_if_active(|| tree);
        }
    }

    /// `a11y_tree_update`.
    pub fn tree_update(&mut self, update: TreeUpdate) {
        if let Some(update) = self.tree.update(update) {
            self.adapter.update_if_active(|| update);
        }
    }

    /// `a11y_update_window_bounds`.
    pub fn set_bounds(&mut self, pixels: (u32, u32)) {
        let bounds = window_bounds(pixels);
        self.adapter.set_root_window_bounds(bounds, bounds);
    }

    /// The window gained or lost the system's focus (the app resumed or paused).
    pub fn set_focused(&mut self, focused: bool) {
        self.adapter.update_window_focus_state(focused);
    }
}

#[cfg(test)]
mod tests;
