//! Video playback (`VID-2`, design `d25-video`): Tizen's public `capi-media-player`, behind a
//! state machine that the platform shell drives and the host tests with a fake player.
//!
//! - [`Player`] follows the player's states (`player_state_e`) and rejects calls the current state
//!   does not allow before they reach the player, with [`VideoError::WrongState`].
//! - Preparing and seeking are asynchronous and serialized: a `play` while preparing starts
//!   playback once prepared; a seek while another is in flight replaces any queued one, and is
//!   issued when the first completes. Nothing blocks the main loop.
//! - Player errors are typed ([`PlayerError`], from `player_error_e`).
//! - The player calls back on its own threads. The shell posts those callbacks into an [`Inbox`],
//!   which wakes the main loop once per batch; the events reach [`Player::handle`] there, in order.
//!
//! [`Backend`] is the C API seen through Rust: the shell implements it over `player_h`, the tests
//! with a fake.

use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

#[cfg(target_vendor = "tizen")]
pub use crate::tizen::video::VideoPlayer;

/// The player's state (`player_state_e`), plus the asynchronous prepare in progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Created, not prepared (`PLAYER_STATE_IDLE`).
    Idle,
    /// `player_prepare_async` was called; the prepared callback has not come yet.
    Preparing,
    /// Prepared or stopped (`PLAYER_STATE_READY`).
    Ready,
    Playing,
    Paused,
    /// Destroyed: every call fails.
    Destroyed,
}

/// What a call asked for, for errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    SetUri,
    Prepare,
    Play,
    Pause,
    Stop,
    Seek,
    SetVolume,
    SetMuted,
    SetLooping,
    Position,
    Duration,
    Unprepare,
}

/// `player_error_e`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerError {
    OutOfMemory,
    InvalidParameter,
    NoSuchFile,
    InvalidOperation,
    NoSpaceOnDevice,
    FeatureNotSupported,
    SeekFailed,
    InvalidState,
    NotSupportedFile,
    InvalidUri,
    SoundPolicy,
    ConnectionFailed,
    VideoCaptureFailed,
    DrmExpired,
    DrmNoLicense,
    DrmFutureUse,
    DrmNotPermitted,
    ResourceLimit,
    PermissionDenied,
    ServiceDisconnected,
    BufferSpace,
    NotSupportedAudioCodec,
    NotSupportedVideoCodec,
    NotSupportedSubtitle,
    NotSupportedFormat,
    NotAvailable,
    /// A code `player.h` does not list.
    Other(i32),
}

/// `TIZEN_ERROR_PLAYER`.
const TIZEN_ERROR_PLAYER: i32 = -0x0194_0000;
/// `PLAYER_ERROR_CLASS`.
const PLAYER_ERROR_CLASS: i32 = TIZEN_ERROR_PLAYER | 0x20;
/// `TIZEN_ERROR_NOT_SUPPORTED`.
const TIZEN_ERROR_NOT_SUPPORTED: i32 = -1_073_741_824 + 2;

impl PlayerError {
    /// The error for a non-zero `player_error_e` code.
    pub fn from_code(code: i32) -> Self {
        match code {
            -12 => Self::OutOfMemory,
            -22 => Self::InvalidParameter,
            -2 => Self::NoSuchFile,
            -38 => Self::InvalidOperation,
            -28 => Self::NoSpaceOnDevice,
            -13 => Self::PermissionDenied,
            -105 => Self::BufferSpace,
            TIZEN_ERROR_NOT_SUPPORTED => Self::FeatureNotSupported,
            _ => match code.wrapping_sub(PLAYER_ERROR_CLASS) {
                0x01 => Self::SeekFailed,
                0x02 => Self::InvalidState,
                0x03 => Self::NotSupportedFile,
                0x04 => Self::InvalidUri,
                0x05 => Self::SoundPolicy,
                0x06 => Self::ConnectionFailed,
                0x07 => Self::VideoCaptureFailed,
                0x08 => Self::DrmExpired,
                0x09 => Self::DrmNoLicense,
                0x0a => Self::DrmFutureUse,
                0x0b => Self::DrmNotPermitted,
                0x0c => Self::ResourceLimit,
                0x0d => Self::ServiceDisconnected,
                0x0e => Self::NotSupportedAudioCodec,
                0x0f => Self::NotSupportedVideoCodec,
                0x10 => Self::NotSupportedSubtitle,
                0x11 => Self::NotSupportedFormat,
                0x12 => Self::NotAvailable,
                _ => Self::Other(code),
            },
        }
    }

    /// The `player_error_e` code.
    pub fn code(self) -> i32 {
        let class = |n: i32| PLAYER_ERROR_CLASS | n;
        match self {
            Self::OutOfMemory => -12,
            Self::InvalidParameter => -22,
            Self::NoSuchFile => -2,
            Self::InvalidOperation => -38,
            Self::NoSpaceOnDevice => -28,
            Self::PermissionDenied => -13,
            Self::BufferSpace => -105,
            Self::FeatureNotSupported => TIZEN_ERROR_NOT_SUPPORTED,
            Self::SeekFailed => class(0x01),
            Self::InvalidState => class(0x02),
            Self::NotSupportedFile => class(0x03),
            Self::InvalidUri => class(0x04),
            Self::SoundPolicy => class(0x05),
            Self::ConnectionFailed => class(0x06),
            Self::VideoCaptureFailed => class(0x07),
            Self::DrmExpired => class(0x08),
            Self::DrmNoLicense => class(0x09),
            Self::DrmFutureUse => class(0x0a),
            Self::DrmNotPermitted => class(0x0b),
            Self::ResourceLimit => class(0x0c),
            Self::ServiceDisconnected => class(0x0d),
            Self::NotSupportedAudioCodec => class(0x0e),
            Self::NotSupportedVideoCodec => class(0x0f),
            Self::NotSupportedSubtitle => class(0x10),
            Self::NotSupportedFormat => class(0x11),
            Self::NotAvailable => class(0x12),
            Self::Other(code) => code,
        }
    }
}

/// Why a call failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoError {
    /// The call is not valid in the player's state; it did not reach the player.
    WrongState { call: Call, state: State },
    /// `prepare` before `set_uri`.
    NoUri,
    /// The player refused the call.
    Player { call: Call, error: PlayerError },
    /// The media could not be opened (see the player's error).
    NotOpened,
}

impl fmt::Display for VideoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongState { call, state } => write!(f, "{call:?} is not valid while {state:?}"),
            Self::NoUri => write!(f, "no media to prepare: set a URI first"),
            Self::Player { call, error } => write!(f, "the player refused {call:?}: {error:?}"),
            Self::NotOpened => write!(f, "the media could not be opened"),
        }
    }
}

impl std::error::Error for VideoError {}

/// What the player reported on one of its threads, delivered on the main thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// `player_prepared_cb`.
    Prepared,
    /// `player_seek_completed_cb`.
    SeekCompleted,
    /// `player_completed_cb`.
    Completed,
    /// `player_error_cb` with its code.
    Error(i32),
    /// `player_buffering_cb` with its percentage.
    Buffering(i32),
    /// `player_interrupted_cb`: the player was paused or unprepared (a resource conflict).
    Interrupted,
    /// A decoded frame waits for the texture output (`video_frames::FrameSlot`).
    Frame,
}

/// What the app hears about, after [`Player::handle`] applied an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    /// Prepared: [`Player::duration`] is known.
    Ready,
    /// Playback started (also after a `play` while preparing or seeking).
    Playing,
    /// Playback paused after the seeks it waited for.
    Paused,
    /// Every requested seek has completed.
    Seeked,
    /// The end was reached (not looping): the player stopped.
    Ended,
    /// A player error; the state is what the player reported afterwards.
    Failed(PlayerError),
    /// Buffering progress, 0-100.
    Buffering(u8),
    /// Interrupted by the system (another app took the decoder): now in the given state.
    Interrupted(State),
    /// A new frame for the texture output: repaint the video.
    Frame,
}

/// `capi-media-player` as the state machine uses it. Calls return the player's error code on
/// failure. The asynchronous calls report through events the shell delivers to
/// [`Player::handle`].
pub trait Backend {
    fn set_uri(&mut self, uri: &str) -> Result<(), i32>;
    /// `player_prepare_async`: [`Event::Prepared`] follows.
    fn prepare_async(&mut self) -> Result<(), i32>;
    fn unprepare(&mut self) -> Result<(), i32>;
    fn start(&mut self) -> Result<(), i32>;
    fn pause(&mut self) -> Result<(), i32>;
    fn stop(&mut self) -> Result<(), i32>;
    /// `player_set_play_position`: [`Event::SeekCompleted`] follows.
    fn seek(&mut self, milliseconds: i32, accurate: bool) -> Result<(), i32>;
    fn set_volume(&mut self, volume: f32) -> Result<(), i32>;
    fn set_muted(&mut self, muted: bool) -> Result<(), i32>;
    fn set_looping(&mut self, looping: bool) -> Result<(), i32>;
    fn position(&mut self) -> Result<i32, i32>;
    fn duration(&mut self) -> Result<i32, i32>;
    /// `player_get_state`, after an interruption.
    fn state(&mut self) -> Result<State, i32>;
    /// `player_destroy`.
    fn destroy(&mut self);
}

/// The seek in flight and the one waiting for it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Seeks {
    in_flight: bool,
    queued: Option<(i32, bool)>,
    /// Play (true) or pause (false) once the seeks are done: the player refuses both while a
    /// seek is in flight (`player.h`).
    then: Option<bool>,
}

/// A media player on the main thread.
pub struct Player<B: Backend> {
    backend: B,
    state: State,
    /// The media, kept to prepare again after the system unprepared the player.
    uri: Option<String>,
    /// Whether the player holds `uri` (it may not after going back to idle).
    uri_set: bool,
    /// `player_prepare_async` was called and its callback has not come: a late callback still
    /// counts after an error dropped the state to idle (the player reports idle while preparing).
    prepare_pending: bool,
    play_when_ready: bool,
    seeks: Seeks,
    looping: bool,
}

fn milliseconds(position: Duration) -> i32 {
    position.as_millis().min(i32::MAX as u128) as i32
}

impl<B: Backend> Player<B> {
    /// A player in the idle state (`player_create` succeeded).
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            state: State::Idle,
            uri: None,
            uri_set: false,
            prepare_pending: false,
            play_when_ready: false,
            seeks: Seeks::default(),
            looping: false,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    /// Whether a seek is in flight.
    pub fn seeking(&self) -> bool {
        self.seeks.in_flight
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    fn require(&self, call: Call, allowed: &[State]) -> Result<(), VideoError> {
        if allowed.contains(&self.state) {
            Ok(())
        } else {
            Err(VideoError::WrongState {
                call,
                state: self.state,
            })
        }
    }

    fn player<T>(call: Call, result: Result<T, i32>) -> Result<T, VideoError> {
        result.map_err(|code| VideoError::Player {
            call,
            error: PlayerError::from_code(code),
        })
    }

    /// The media to play (a path or URL), while idle.
    pub fn set_uri(&mut self, uri: &str) -> Result<(), VideoError> {
        self.require(Call::SetUri, &[State::Idle])?;
        Self::player(Call::SetUri, self.backend.set_uri(uri))?;
        self.uri = Some(uri.to_owned());
        self.uri_set = true;
        Ok(())
    }

    /// Starts preparing the media set last; [`Notice::Ready`] follows. After the system
    /// unprepared the player (an interruption, a failed prepare), this prepares it again.
    pub fn prepare(&mut self) -> Result<(), VideoError> {
        self.require(Call::Prepare, &[State::Idle])?;
        let Some(uri) = self.uri.clone() else {
            return Err(VideoError::NoUri);
        };
        if self.prepare_pending {
            // Cancels the earlier prepare (allowed while idle since Tizen 5.0).
            let _ = self.backend.unprepare();
            self.prepare_pending = false;
            self.uri_set = false;
        }
        if !self.uri_set {
            Self::player(Call::SetUri, self.backend.set_uri(&uri))?;
            self.uri_set = true;
        }
        Self::player(Call::Prepare, self.backend.prepare_async())?;
        self.state = State::Preparing;
        self.prepare_pending = true;
        Ok(())
    }

    /// Plays; while preparing, once prepared; while seeking, once the seeks are done.
    pub fn play(&mut self) -> Result<(), VideoError> {
        match self.state {
            State::Preparing => {
                self.play_when_ready = true;
                Ok(())
            }
            _ if self.seeks.in_flight => {
                self.require(Call::Play, &[State::Ready, State::Paused, State::Playing])?;
                self.seeks.then = Some(true);
                Ok(())
            }
            State::Playing => Ok(()),
            _ => {
                self.require(Call::Play, &[State::Ready, State::Paused])?;
                Self::player(Call::Play, self.backend.start())?;
                self.state = State::Playing;
                Ok(())
            }
        }
    }

    /// Pauses; while preparing, cancels a pending `play`; while seeking, once the seeks are done.
    pub fn pause(&mut self) -> Result<(), VideoError> {
        match self.state {
            State::Preparing => {
                self.play_when_ready = false;
                Ok(())
            }
            _ if self.seeks.in_flight => {
                self.require(Call::Pause, &[State::Playing, State::Paused])?;
                self.seeks.then = Some(false);
                Ok(())
            }
            State::Paused => Ok(()),
            _ => {
                self.require(Call::Pause, &[State::Playing])?;
                Self::player(Call::Pause, self.backend.pause())?;
                self.state = State::Paused;
                Ok(())
            }
        }
    }

    /// Stops: back to the start, ready to play again; seeks in flight are forgotten.
    pub fn stop(&mut self) -> Result<(), VideoError> {
        self.require(Call::Stop, &[State::Playing, State::Paused])?;
        Self::player(Call::Stop, self.backend.stop())?;
        self.state = State::Ready;
        self.seeks = Seeks::default();
        Ok(())
    }

    /// Seeks to `position`; [`Notice::Seeked`] follows once no seek is left. A seek while another
    /// is in flight replaces any queued one.
    pub fn seek(&mut self, position: Duration, accurate: bool) -> Result<(), VideoError> {
        self.require(Call::Seek, &[State::Ready, State::Playing, State::Paused])?;
        let target = (milliseconds(position), accurate);
        if self.seeks.in_flight {
            self.seeks.queued = Some(target);
            return Ok(());
        }
        Self::player(Call::Seek, self.backend.seek(target.0, target.1))?;
        self.seeks.in_flight = true;
        Ok(())
    }

    /// Volume, 0.0 to 1.0, on both channels.
    pub fn set_volume(&mut self, volume: f32) -> Result<(), VideoError> {
        self.require(
            Call::SetVolume,
            &[
                State::Idle,
                State::Preparing,
                State::Ready,
                State::Playing,
                State::Paused,
            ],
        )?;
        Self::player(
            Call::SetVolume,
            self.backend.set_volume(volume.clamp(0., 1.)),
        )
    }

    pub fn set_muted(&mut self, muted: bool) -> Result<(), VideoError> {
        self.require(
            Call::SetMuted,
            &[
                State::Idle,
                State::Preparing,
                State::Ready,
                State::Playing,
                State::Paused,
            ],
        )?;
        Self::player(Call::SetMuted, self.backend.set_muted(muted))
    }

    pub fn set_looping(&mut self, looping: bool) -> Result<(), VideoError> {
        self.require(
            Call::SetLooping,
            &[
                State::Idle,
                State::Preparing,
                State::Ready,
                State::Playing,
                State::Paused,
            ],
        )?;
        Self::player(Call::SetLooping, self.backend.set_looping(looping))?;
        self.looping = looping;
        Ok(())
    }

    /// The playback position.
    pub fn position(&mut self) -> Result<Duration, VideoError> {
        self.require(
            Call::Position,
            &[State::Ready, State::Playing, State::Paused],
        )?;
        let ms = Self::player(Call::Position, self.backend.position())?;
        Ok(Duration::from_millis(ms.max(0) as u64))
    }

    /// The media's duration (known once prepared).
    pub fn duration(&mut self) -> Result<Duration, VideoError> {
        self.require(
            Call::Duration,
            &[State::Ready, State::Playing, State::Paused],
        )?;
        let ms = Self::player(Call::Duration, self.backend.duration())?;
        Ok(Duration::from_millis(ms.max(0) as u64))
    }

    /// Back to idle (a new URI can be set, or the same one prepared again); cancels a prepare in
    /// progress.
    pub fn unprepare(&mut self) -> Result<(), VideoError> {
        self.require(
            Call::Unprepare,
            &[
                State::Preparing,
                State::Ready,
                State::Playing,
                State::Paused,
            ],
        )?;
        Self::player(Call::Unprepare, self.backend.unprepare())?;
        self.back_to_idle();
        Ok(())
    }

    fn back_to_idle(&mut self) {
        self.state = State::Idle;
        self.uri_set = false;
        self.prepare_pending = false;
        self.play_when_ready = false;
        self.seeks = Seeks::default();
    }

    /// Destroys the player; later calls fail and events are ignored.
    pub fn destroy(&mut self) {
        if self.state != State::Destroyed {
            self.backend.destroy();
            self.state = State::Destroyed;
        }
    }

    /// Applies an event the player reported, on the main thread, and says what the app should
    /// hear (several notices for one event when a queued `play` runs).
    pub fn handle(&mut self, event: Event) -> Vec<Notice> {
        let mut notices = Vec::new();
        match (event, self.state) {
            (_, State::Destroyed) => {}
            // Prepared, also after an error had the state drop to idle while preparing.
            (Event::Prepared, State::Preparing | State::Idle) if self.prepare_pending => {
                self.prepare_pending = false;
                self.state = State::Ready;
                notices.push(Notice::Ready);
                if std::mem::take(&mut self.play_when_ready) {
                    match self.play() {
                        Ok(()) => notices.push(Notice::Playing),
                        Err(VideoError::Player { error, .. }) => {
                            notices.push(Notice::Failed(error))
                        }
                        Err(_) => {}
                    }
                }
            }
            // A prepare cancelled by `unprepare`.
            (Event::Prepared, _) => {}
            (Event::SeekCompleted, _) => {
                self.seeks.in_flight = false;
                if let Some((ms, accurate)) = self.seeks.queued.take() {
                    match self.backend.seek(ms, accurate) {
                        Ok(()) => self.seeks.in_flight = true,
                        Err(code) => {
                            notices.push(Notice::Failed(PlayerError::from_code(code)));
                        }
                    }
                }
                if !self.seeks.in_flight {
                    notices.push(Notice::Seeked);
                    notices.extend(self.after_seeks());
                }
            }
            (Event::Completed, State::Playing) => {
                if !self.looping && self.backend.stop().is_ok() {
                    self.state = State::Ready;
                    self.seeks = Seeks::default();
                }
                notices.push(Notice::Ended);
            }
            (Event::Completed, _) => notices.push(Notice::Ended),
            (Event::Error(code), _) => {
                // A failed seek never completes (`PLAYER_ERROR_SEEK_FAILED` comes here instead),
                // and after any other error the player answers no seek either.
                self.seeks = Seeks::default();
                self.sync_state();
                notices.push(Notice::Failed(PlayerError::from_code(code)));
            }
            (Event::Buffering(percent), _) => {
                notices.push(Notice::Buffering(percent.clamp(0, 100) as u8));
            }
            (Event::Interrupted, _) => {
                self.seeks = Seeks::default();
                self.sync_state();
                notices.push(Notice::Interrupted(self.state));
            }
            (Event::Frame, _) => notices.push(Notice::Frame),
        }
        notices
    }

    /// Plays or pauses as asked while the seeks were in flight.
    fn after_seeks(&mut self) -> Option<Notice> {
        let play = self.seeks.then.take()?;
        let result = if play { self.play() } else { self.pause() };
        match result {
            Ok(()) if play => Some(Notice::Playing),
            Ok(()) => Some(Notice::Paused),
            Err(VideoError::Player { error, .. }) => Some(Notice::Failed(error)),
            Err(_) => None,
        }
    }

    /// Takes the player's own state after it changed it (an error or an interruption). The
    /// player reports idle while it prepares: a prepare in flight drops to idle, and its late
    /// callback still makes the player ready.
    fn sync_state(&mut self) {
        let Ok(state) = self.backend.state() else {
            return;
        };
        match state {
            State::Idle => {
                let pending = self.prepare_pending;
                self.back_to_idle();
                self.prepare_pending = pending;
            }
            // Ready while a prepare is in flight: the prepared callback is on its way.
            State::Ready if self.state == State::Preparing => {}
            state => self.state = state,
        }
    }
}

impl<B: Backend> Drop for Player<B> {
    fn drop(&mut self) {
        self.destroy();
    }
}

/// Where a player's pictures go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    /// The TV's video plane under the window, seen through a hole GPUI paints where the video
    /// is: hardware decoding up to 4K, no copies. Needs a window that can turn translucent.
    Overlay,
    /// Decoded frames converted to RGBA and drawn as a texture: for video inside transformed or
    /// clipped UI, and where the overlay is not available.
    Texture,
}

impl Output {
    /// The output for a request: `None` asks for the overlay where the window can show it.
    pub fn choose(requested: Option<Output>, translucency_available: bool) -> Output {
        match requested {
            Some(output) => output,
            None if translucency_available => Output::Overlay,
            None => Output::Texture,
        }
    }
}

/// A rectangle in the window's device pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl DeviceRect {
    /// Logical `x, y, width, height` at `scale`, each edge rounded to a device pixel: the grid
    /// GPUI snaps a hole to (`gpui::fast::hole`), so the video and its hole line up.
    pub fn snapped(x: f32, y: f32, width: f32, height: f32, scale: f32) -> Self {
        let left = (x * scale).round() as i32;
        let top = (y * scale).round() as i32;
        let right = ((x + width) * scale).round() as i32;
        let bottom = ((y + height) * scale).round() as i32;
        Self {
            x: left,
            y: top,
            width: (right - left).max(0),
            height: (bottom - top).max(0),
        }
    }
}

/// What the overlay's exported surface is told.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayOp {
    Move { x: i32, y: i32 },
    Resize { width: i32, height: i32 },
    Show,
    Hide,
}

/// Where the overlay's video surface is, so each frame tells it only what changed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OverlayPlacement {
    placed: Option<DeviceRect>,
    shown: bool,
}

impl OverlayPlacement {
    /// The calls that put the video at `rect` (painted this frame); an empty rect hides it.
    pub fn place(&mut self, rect: DeviceRect) -> Vec<OverlayOp> {
        if rect.width <= 0 || rect.height <= 0 {
            return self.hide();
        }
        let mut ops = Vec::new();
        let previous = self.placed.replace(rect);
        if previous.is_none_or(|p| (p.x, p.y) != (rect.x, rect.y)) {
            ops.push(OverlayOp::Move {
                x: rect.x,
                y: rect.y,
            });
        }
        if previous.is_none_or(|p| (p.width, p.height) != (rect.width, rect.height)) {
            ops.push(OverlayOp::Resize {
                width: rect.width,
                height: rect.height,
            });
        }
        if !self.shown {
            self.shown = true;
            ops.push(OverlayOp::Show);
        }
        ops
    }

    /// The calls that hide the video (not painted this frame).
    pub fn hide(&mut self) -> Vec<OverlayOp> {
        if std::mem::take(&mut self.shown) {
            vec![OverlayOp::Hide]
        } else {
            Vec::new()
        }
    }
}

/// Identifies a player for the events posted for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(pub u64);

/// Runs a closure on the main thread soon.
type Post = Box<dyn Fn(Box<dyn FnOnce() + Send>) + Send + Sync>;

/// Takes the player's callbacks on any thread and hands them to the main thread in order.
///
/// `post` runs a closure on the main thread soon (Ecore's `ecore_main_loop_thread_safe_call_async`
/// through the platform's main loop); one post wakes the main thread for every event queued until
/// it drains. `route` runs there with each event.
pub struct Inbox {
    queue: Mutex<VecDeque<(PlayerId, Event)>>,
    post: Post,
    route: fn(PlayerId, Event),
}

impl Inbox {
    pub fn new(
        post: impl Fn(Box<dyn FnOnce() + Send>) + Send + Sync + 'static,
        route: fn(PlayerId, Event),
    ) -> Arc<Self> {
        Arc::new(Self {
            queue: Mutex::new(VecDeque::new()),
            post: Box::new(post),
            route,
        })
    }

    /// Queues `event` for player `id` (any thread).
    pub fn post(self: &Arc<Self>, id: PlayerId, event: Event) {
        let wake = {
            let mut queue = self.queue.lock().unwrap_or_else(PoisonError::into_inner);
            queue.push_back((id, event));
            queue.len() == 1
        };
        if wake {
            let inbox = Arc::clone(self);
            (self.post)(Box::new(move || inbox.drain()));
        }
    }

    /// Routes every queued event (main thread).
    fn drain(&self) {
        loop {
            let next = self
                .queue
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .pop_front();
            match next {
                Some((id, event)) => (self.route)(id, event),
                None => return,
            }
        }
    }
}

#[cfg(test)]
mod tests;
