//! The long list's view and key bindings: the filter field above the rows. Up from the first
//! row (or from an empty list) focuses the field; Down, Enter or Back leave it for the list.

use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    KeyBinding, ParentElement, Render, ScrollStrategy, SharedString, Styled, Subscription,
    UniformListScrollHandle, Window, WindowBounds, WindowOptions, actions, div, px, rgb,
    uniform_list,
};

use crate::filter::{self, FilterField, LeaveField, QueryChanged};

actions!(long_list, [SelectPrevious, SelectNext, Quit]);

/// The number of rows.
pub const ROWS: usize = 1_000;
/// The key context the bindings apply in.
pub const CONTEXT: &str = "LongList";

/// Up/Down move the selection, Back (TV) or Escape (desktop) quits; the filter field's own
/// bindings come with them.
pub fn bindings() -> Vec<KeyBinding> {
    let mut bindings = vec![
        KeyBinding::new("up", SelectPrevious, Some(CONTEXT)),
        KeyBinding::new("down", SelectNext, Some(CONTEXT)),
        KeyBinding::new("back", Quit, Some(CONTEXT)),
        KeyBinding::new("escape", Quit, Some(CONTEXT)),
    ];
    bindings.extend(filter::bindings());
    bindings
}

pub struct LongList {
    /// The selected position in [`LongList::rows`].
    pub selected: usize,
    /// The rows the filter keeps.
    pub rows: Vec<usize>,
    pub quit_requested: bool,
    pub scroll: UniformListScrollHandle,
    pub filter: Entity<FilterField>,
    focus: FocusHandle,
    _filter_changes: Subscription,
}

impl LongList {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let filter = cx.new(FilterField::new);
        let changes = cx.subscribe(&filter, |list, _field, QueryChanged(query), cx| {
            list.set_query(query, cx);
        });
        Self {
            selected: 0,
            rows: filter::rows(""),
            quit_requested: false,
            scroll: UniformListScrollHandle::new(),
            filter,
            focus: cx.focus_handle(),
            _filter_changes: changes,
        }
    }

    /// Narrows the rows to `query` and selects the first.
    fn set_query(&mut self, query: &str, cx: &mut Context<Self>) {
        let rows = filter::rows(query);
        if rows != self.rows {
            self.rows = rows;
            self.select(0, cx);
        }
    }

    /// Moves the selection to `position` (clamped) and scrolls it into view.
    pub fn select(&mut self, position: usize, cx: &mut Context<Self>) {
        self.selected = position.min(self.rows.len().saturating_sub(1));
        self.scroll
            .scroll_to_item(self.selected, ScrollStrategy::Nearest);
        cx.notify();
    }

    /// The selected row, if the filter kept any.
    pub fn selected_row(&self) -> Option<usize> {
        self.rows.get(self.selected).copied()
    }

    fn select_previous(&mut self, _: &SelectPrevious, window: &mut Window, cx: &mut Context<Self>) {
        if !self.focus.is_focused(window) {
            // In the field: Up has nowhere to go.
            return;
        }
        if self.selected == 0 {
            window.focus(&self.filter.focus_handle(cx), cx);
            cx.notify();
        } else {
            self.select(self.selected - 1, cx);
        }
    }

    fn leave_field(&mut self, _: &LeaveField, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        cx.notify();
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.focus.is_focused(window).then_some(self.selected);
        let rows = self.rows.clone();
        div()
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::leave_field))
            .on_action(cx.listener(Self::quit))
            .size_full()
            .bg(rgb(0x10141c))
            .p(px(48.))
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(self.filter.clone())
            .child(
                uniform_list("rows", rows.len(), move |range, _window, _cx| {
                    range
                        .map(|position| {
                            let row = rows[position];
                            let is_selected = Some(position) == selected;
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
                .flex_1()
                .w_full(),
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
