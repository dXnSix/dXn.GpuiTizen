//! The counter's view and key bindings.

use gpui::{
    App, AppContext, Context, FocusHandle, Focusable, InteractiveElement, IntoElement, KeyBinding,
    ParentElement, Render, SharedString, Styled, Window, WindowBounds, WindowOptions, actions, div,
    px, rgb,
};

actions!(counter, [FocusLeft, FocusRight, Press, Quit]);

/// The key context the bindings apply in.
pub const CONTEXT: &str = "Counter";

/// Left/Right move the focus, Enter presses, Back (TV) or Escape (desktop) quits.
pub fn bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("left", FocusLeft, Some(CONTEXT)),
        KeyBinding::new("right", FocusRight, Some(CONTEXT)),
        KeyBinding::new("enter", Press, Some(CONTEXT)),
        KeyBinding::new("back", Quit, Some(CONTEXT)),
        KeyBinding::new("escape", Quit, Some(CONTEXT)),
    ]
}

/// The two buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Minus,
    Plus,
}

pub struct Counter {
    pub count: i64,
    pub focused: Button,
    pub quit_requested: bool,
    focus: FocusHandle,
}

impl Counter {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            count: 0,
            focused: Button::Plus,
            quit_requested: false,
            focus: cx.focus_handle(),
        }
    }

    fn focus_left(&mut self, _: &FocusLeft, _window: &mut Window, cx: &mut Context<Self>) {
        self.focused = Button::Minus;
        cx.notify();
    }

    fn focus_right(&mut self, _: &FocusRight, _window: &mut Window, cx: &mut Context<Self>) {
        self.focused = Button::Plus;
        cx.notify();
    }

    fn press(&mut self, _: &Press, _window: &mut Window, cx: &mut Context<Self>) {
        self.count += match self.focused {
            Button::Minus => -1,
            Button::Plus => 1,
        };
        cx.notify();
    }

    fn quit(&mut self, _: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
        self.quit_requested = true;
        cx.quit();
    }

    fn button(&self, button: Button, label: &'static str) -> impl IntoElement {
        let focused = self.focused == button;
        div()
            .w(px(160.))
            .h(px(96.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(16.))
            .border_4()
            .border_color(if focused {
                rgb(0xf5f7fa)
            } else {
                rgb(0x2a3140)
            })
            .bg(if focused {
                rgb(0x3b6ee8)
            } else {
                rgb(0x1c2230)
            })
            .text_color(rgb(0xf5f7fa))
            .text_size(px(48.))
            .child(label)
    }
}

impl Focusable for Counter {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::focus_left))
            .on_action(cx.listener(Self::focus_right))
            .on_action(cx.listener(Self::press))
            .on_action(cx.listener(Self::quit))
            .size_full()
            .bg(rgb(0x10141c))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(48.))
            .child(
                div()
                    .text_color(rgb(0xf5f7fa))
                    .text_size(px(120.))
                    .child(SharedString::from(self.count.to_string())),
            )
            .child(
                div()
                    .flex()
                    .gap(px(48.))
                    .child(self.button(Button::Minus, "−"))
                    .child(self.button(Button::Plus, "+")),
            )
    }
}

/// Opens the counter's window, focused, with its bindings.
pub fn launch(cx: &mut App) {
    cx.bind_keys(bindings());
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Fullscreen(gpui::Bounds::maximized(None, cx))),
        ..Default::default()
    };
    let opened = cx.open_window(options, |window, cx| {
        let counter = cx.new(Counter::new);
        window.focus(&counter.focus_handle(cx), cx);
        counter
    });
    if let Err(err) = opened {
        eprintln!("cannot open the counter's window: {err:#}");
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
