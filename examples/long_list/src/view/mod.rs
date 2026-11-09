//! The long list's view and key bindings.

use gpui::{
    App, AppContext, Context, FocusHandle, Focusable, InteractiveElement, IntoElement, KeyBinding,
    ParentElement, Render, ScrollStrategy, SharedString, Styled, UniformListScrollHandle, Window,
    WindowBounds, WindowOptions, actions, div, px, rgb, uniform_list,
};

actions!(long_list, [SelectPrevious, SelectNext, Quit]);

/// The number of rows.
pub const ROWS: usize = 1_000;
/// The key context the bindings apply in.
pub const CONTEXT: &str = "LongList";

/// Up/Down move the selection, Back (TV) or Escape (desktop) quits.
pub fn bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("up", SelectPrevious, Some(CONTEXT)),
        KeyBinding::new("down", SelectNext, Some(CONTEXT)),
        KeyBinding::new("back", Quit, Some(CONTEXT)),
        KeyBinding::new("escape", Quit, Some(CONTEXT)),
    ]
}

pub struct LongList {
    pub selected: usize,
    pub quit_requested: bool,
    pub scroll: UniformListScrollHandle,
    focus: FocusHandle,
}

impl LongList {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            selected: 0,
            quit_requested: false,
            scroll: UniformListScrollHandle::new(),
            focus: cx.focus_handle(),
        }
    }

    /// Moves the selection to `row` (clamped) and scrolls it into view.
    pub fn select(&mut self, row: usize, cx: &mut Context<Self>) {
        self.selected = row.min(ROWS - 1);
        self.scroll
            .scroll_to_item(self.selected, ScrollStrategy::Nearest);
        cx.notify();
    }

    fn select_previous(
        &mut self,
        _: &SelectPrevious,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select(self.selected.saturating_sub(1), cx);
    }

    fn select_next(&mut self, _: &SelectNext, _window: &mut Window, cx: &mut Context<Self>) {
        self.select(self.selected + 1, cx);
    }

    fn quit(&mut self, _: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
        self.quit_requested = true;
        cx.quit();
    }
}

/// A row's label.
pub fn label(row: usize) -> SharedString {
    SharedString::from(format!("Row {}", row + 1))
}

impl Focusable for LongList {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for LongList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.selected;
        div()
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::quit))
            .size_full()
            .bg(rgb(0x10141c))
            .p(px(48.))
            .child(
                uniform_list("rows", ROWS, move |range, _window, _cx| {
                    range
                        .map(|row| {
                            let is_selected = row == selected;
                            div()
                                .h(px(72.))
                                .px(px(32.))
                                .flex()
                                .items_center()
                                .rounded(px(12.))
                                .bg(if is_selected {
                                    rgb(0x3b6ee8)
                                } else {
                                    rgb(0x1c2230)
                                })
                                .text_color(rgb(0xf5f7fa))
                                .text_size(px(36.))
                                .child(label(row))
                        })
                        .collect()
                })
                .track_scroll(&self.scroll)
                .size_full(),
            )
    }
}

/// Opens the list's window, focused, with its bindings.
pub fn launch(cx: &mut App) {
    cx.bind_keys(bindings());
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Fullscreen(gpui::Bounds::maximized(None, cx))),
        ..Default::default()
    };
    let opened = cx.open_window(options, |window, cx| {
        let list = cx.new(LongList::new);
        window.focus(&list.focus_handle(cx), cx);
        list
    });
    if let Err(err) = opened {
        eprintln!("cannot open the list's window: {err:#}");
        cx.quit();
    }
}

/// Quits after `frames` frames have been drawn (for scripted runs).
pub fn quit_after_frames(window: &Window, frames: u32) {
    window.on_next_frame(move |window, cx| {
        if frames <= 1 {
            cx.quit();
        } else {
            quit_after_frames(window, frames - 1);
        }
    });
}

#[cfg(test)]
mod tests;
