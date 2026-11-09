//! The video view: the video full screen and a row of controls over its bottom edge. The control
//! row's focus and what each control does are [`Controls`], tested on the host; on Tizen the
//! commands go to `dxn_gpui_tizen::video::VideoPlayer`.

use std::time::Duration;

use gpui::{
    App, AppContext, Context, FocusHandle, Focusable, InteractiveElement, IntoElement, KeyBinding,
    ParentElement, Render, SharedString, Styled, Window, WindowBounds, WindowOptions, actions, div,
    px, rgb, rgba,
};

actions!(video, [FocusPrevious, FocusNext, Activate, Quit]);

/// The key context the bindings apply in.
pub const CONTEXT: &str = "Video";
/// How far Rewind and Forward jump.
pub const JUMP: Duration = Duration::from_secs(10);
/// The bundled clip, in the app's resources.
pub const CLIP: &str = "clip.mp4";

/// Left/Right move between the controls, Enter uses one, Back (TV) or Escape (desktop) quits.
pub fn bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("left", FocusPrevious, Some(CONTEXT)),
        KeyBinding::new("right", FocusNext, Some(CONTEXT)),
        KeyBinding::new("enter", Activate, Some(CONTEXT)),
        KeyBinding::new("back", Quit, Some(CONTEXT)),
        KeyBinding::new("escape", Quit, Some(CONTEXT)),
    ]
}

/// The controls, left to right.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Rewind,
    PlayPause,
    Forward,
    Mute,
}

pub const CONTROLS: [Control; 4] = [
    Control::Rewind,
    Control::PlayPause,
    Control::Forward,
    Control::Mute,
];

/// What a control asks the player for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Play,
    Pause,
    /// Jump back by [`JUMP`].
    Rewind,
    /// Jump ahead by [`JUMP`].
    Forward,
    Mute(bool),
}

/// The control row: which control has the focus, and the player state the labels show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Controls {
    pub focused: usize,
    pub playing: bool,
    pub muted: bool,
}

impl Default for Controls {
    /// Play/Pause focused; the clip starts playing.
    fn default() -> Self {
        Self {
            focused: 1,
            playing: true,
            muted: false,
        }
    }
}

impl Controls {
    pub fn focused_control(&self) -> Control {
        CONTROLS[self.focused]
    }

    /// Focus moves left, stopping at the first control.
    pub fn previous(&mut self) {
        self.focused = self.focused.saturating_sub(1);
    }

    /// Focus moves right, stopping at the last control.
    pub fn next(&mut self) {
        self.focused = (self.focused + 1).min(CONTROLS.len() - 1);
    }

    /// Uses the focused control: the command for the player, with the labels updated.
    pub fn activate(&mut self) -> Command {
        match self.focused_control() {
            Control::Rewind => Command::Rewind,
            Control::Forward => Command::Forward,
            Control::PlayPause => {
                self.playing = !self.playing;
                if self.playing {
                    Command::Play
                } else {
                    Command::Pause
                }
            }
            Control::Mute => {
                self.muted = !self.muted;
                Command::Mute(self.muted)
            }
        }
    }

    /// Follows what the player did on its own (it started, paused, ended, was interrupted), so
    /// Play/Pause offers what the player can do next.
    pub fn follow_player(&mut self, playing: bool) {
        self.playing = playing;
    }

    /// The label a control shows.
    pub fn label(&self, control: Control) -> &'static str {
        match control {
            Control::Rewind => "−10 s",
            Control::Forward => "+10 s",
            Control::PlayPause if self.playing => "Pause",
            Control::PlayPause => "Play",
            Control::Mute if self.muted => "Unmute",
            Control::Mute => "Mute",
        }
    }
}

/// The app's view.
pub struct VideoApp {
    pub controls: Controls,
    /// Every command sent, in order (what the desktop placeholder would have played).
    pub sent: Vec<Command>,
    pub quit_requested: bool,
    /// A line under the video: the player's state or why it failed.
    pub status: SharedString,
    focus: FocusHandle,
    #[cfg(target_vendor = "tizen")]
    player: gpui::Entity<dxn_gpui_tizen::video::VideoPlayer>,
    #[cfg(target_vendor = "tizen")]
    _notices: gpui::Subscription,
}

impl VideoApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        #[cfg(target_vendor = "tizen")]
        {
            use dxn_gpui_tizen::video::{Notice, VideoPlayer};
            let uri = dxn_gpui_tizen::resource_path(CLIP)
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| CLIP.to_owned());
            let player = cx.new(|cx| {
                let mut player = VideoPlayer::new(&uri, None, cx);
                let _ = player.set_looping(true);
                let _ = player.play(cx);
                player
            });
            let status = player
                .read(cx)
                .error()
                .map_or_else(|| "Preparing…".to_owned(), |e| format!("Cannot play: {e}"));
            let notices = cx.subscribe(&player, |app, player, notice: &Notice, cx| {
                let playing = match notice {
                    Notice::Playing => Some(true),
                    Notice::Paused | Notice::Ended => Some(false),
                    Notice::Interrupted(state) => {
                        Some(*state == dxn_gpui_tizen::video::State::Playing)
                    }
                    Notice::Failed(_) => {
                        Some(player.read(cx).state() == dxn_gpui_tizen::video::State::Playing)
                    }
                    Notice::Ready | Notice::Seeked | Notice::Buffering(_) | Notice::Frame => None,
                };
                if let Some(playing) = playing {
                    app.controls.follow_player(playing);
                    cx.notify();
                }
                let status = match notice {
                    Notice::Ready | Notice::Playing => Some("Playing".to_owned()),
                    Notice::Ended => Some("Ended".to_owned()),
                    Notice::Failed(error) => Some(format!("Player error: {error:?}")),
                    Notice::Buffering(percent) => Some(format!("Buffering {percent}%")),
                    Notice::Interrupted(_) => Some("Interrupted: Play opens it again".to_owned()),
                    Notice::Paused => Some("Paused".to_owned()),
                    Notice::Seeked | Notice::Frame => None,
                };
                if let Some(status) = status {
                    app.status = status.into();
                    cx.notify();
                }
            });
            Self {
                controls: Controls::default(),
                sent: Vec::new(),
                quit_requested: false,
                status: status.into(),
                focus: cx.focus_handle(),
                player,
                _notices: notices,
            }
        }
        #[cfg(not(target_vendor = "tizen"))]
        Self {
            controls: Controls::default(),
            sent: Vec::new(),
            quit_requested: false,
            status: "Video plays on the TV".into(),
            focus: cx.focus_handle(),
        }
    }

    /// Sends `command` to the player.
    fn send(&mut self, command: Command, cx: &mut Context<Self>) {
        self.sent.push(command);
        #[cfg(target_vendor = "tizen")]
        self.player.update(cx, |player, cx| {
            let result = match command {
                Command::Play => {
                    // After an interruption the system unprepared the player: open it again.
                    if player.state() == dxn_gpui_tizen::video::State::Idle {
                        let _ = player.prepare(cx);
                    }
                    player.play(cx)
                }
                Command::Pause => player.pause(cx),
                Command::Mute(muted) => player.set_muted(muted),
                Command::Rewind | Command::Forward => player.position().and_then(|position| {
                    let target = if command == Command::Rewind {
                        position.saturating_sub(JUMP)
                    } else {
                        position + JUMP
                    };
                    player.seek(target, cx)
                }),
            };
            if let Err(err) = result {
                log::warn!("{command:?}: {err}");
            }
        });
        cx.notify();
    }

    fn focus_previous(&mut self, _: &FocusPrevious, _: &mut Window, cx: &mut Context<Self>) {
        self.controls.previous();
        cx.notify();
    }

    fn focus_next(&mut self, _: &FocusNext, _: &mut Window, cx: &mut Context<Self>) {
        self.controls.next();
        cx.notify();
    }

    fn activate(&mut self, _: &Activate, _: &mut Window, cx: &mut Context<Self>) {
        let command = self.controls.activate();
        self.send(command, cx);
    }

    fn quit(&mut self, _: &Quit, _: &mut Window, cx: &mut Context<Self>) {
        self.quit_requested = true;
        cx.quit();
    }

    fn video(&self) -> gpui::AnyElement {
        #[cfg(target_vendor = "tizen")]
        {
            dxn_gpui_tizen::video::VideoPlayer::element(self.player.clone()).into_any_element()
        }
        #[cfg(not(target_vendor = "tizen"))]
        {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(0x1c2230))
                .text_color(rgb(0x8a93a6))
                .text_size(px(40.))
                .child("▶ video")
                .into_any_element()
        }
    }
}

impl Focusable for VideoApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for VideoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let controls = self.controls;
        div()
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::focus_previous))
            .on_action(cx.listener(Self::focus_next))
            .on_action(cx.listener(Self::activate))
            .on_action(cx.listener(Self::quit))
            .size_full()
            .relative()
            .bg(rgb(0x000000))
            .child(self.video())
            .child(
                div()
                    .absolute()
                    .bottom(px(48.))
                    .left(px(0.))
                    .right(px(0.))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(16.))
                    .child(
                        div()
                            .flex()
                            .gap(px(24.))
                            .p(px(16.))
                            .rounded(px(24.))
                            .bg(rgba(0x10141cd0))
                            .children(CONTROLS.iter().enumerate().map(|(index, control)| {
                                let focused = index == controls.focused;
                                div()
                                    .w(px(180.))
                                    .h(px(72.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(16.))
                                    .bg(if focused {
                                        rgb(0x3b6ee8)
                                    } else {
                                        rgb(0x2a3242)
                                    })
                                    .text_color(rgb(0xf5f7fa))
                                    .text_size(px(32.))
                                    .child(controls.label(*control))
                            })),
                    )
                    .child(
                        div()
                            .text_color(rgb(0xc7cdd8))
                            .text_size(px(24.))
                            .child(self.status.clone()),
                    ),
            )
    }
}

/// Opens the player's window, focused, with its bindings.
pub fn launch(cx: &mut App) {
    cx.bind_keys(bindings());
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Fullscreen(gpui::Bounds::maximized(None, cx))),
        ..Default::default()
    };
    let opened = cx.open_window(options, |window, cx| {
        let app = cx.new(VideoApp::new);
        window.focus(&app.focus_handle(cx), cx);
        app
    });
    if let Err(err) = opened {
        eprintln!("cannot open the video window: {err:#}");
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
