//! System settings (design `d17-settings-fonts`): what `system_settings` reports, in the forms
//! apps use. GPUI has no API for locale or font scale, so apps read [`current`] and
//! [`subscribe`] to changes here.
//!
//! - The locale is BCP 47 (`ko-KR` from `ko_KR.UTF-8`), from the UI language, else the region.
//! - The font scale maps `system_settings`' font size monotonically, normal = 1.0.
//! - The screen reader is `SYSTEM_SETTINGS_KEY_ACCESSIBILITY_TTS`.
//! - High contrast and reduce motion have no `system_settings` key on Tizen 8.0: they stay `None`
//!   until a TV reports them.
//!
//! A key the TV does not support yields `None`, never an error. The Tizen shell reads the settings
//! at start and again after `system_settings` change callbacks, coalescing a burst of callbacks
//! into one read, and publishes them here; subscribers run on the main thread, only when
//! something changed. Off Tizen nothing is published: [`current`] stays empty.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// `system_settings_font_size_e` (`system/system_settings.h`) to a font scale.
pub const FONT_SCALES: [f32; 5] = [
    0.8,  // SYSTEM_SETTINGS_FONT_SIZE_SMALL
    1.0,  // NORMAL
    1.25, // LARGE
    1.5,  // HUGE
    2.0,  // GIANT
];

/// The settings an app sees; `None` where the TV does not report one.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SystemSettings {
    /// BCP 47, such as `ko-KR`.
    pub locale: Option<String>,
    /// An IANA time zone, such as `Asia/Seoul`.
    pub time_zone: Option<String>,
    /// 1.0 for the normal font size.
    pub font_scale: Option<f32>,
    pub screen_reader: Option<bool>,
    pub high_contrast: Option<bool>,
    pub reduce_motion: Option<bool>,
}

/// The values as `system_settings` returns them; `None` for an unsupported key or an error.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RawSettings {
    /// `SYSTEM_SETTINGS_KEY_LOCALE_LANGUAGE`, such as `ko_KR.UTF-8`.
    pub language: Option<String>,
    /// `SYSTEM_SETTINGS_KEY_LOCALE_COUNTRY` (the region format), same form.
    pub country: Option<String>,
    /// `SYSTEM_SETTINGS_KEY_LOCALE_TIMEZONE`.
    pub time_zone: Option<String>,
    /// `SYSTEM_SETTINGS_KEY_FONT_SIZE`.
    pub font_size: Option<i32>,
    /// `SYSTEM_SETTINGS_KEY_ACCESSIBILITY_TTS`.
    pub tts: Option<bool>,
}

impl SystemSettings {
    pub fn from_raw(raw: &RawSettings) -> Self {
        let locale = raw
            .language
            .as_deref()
            .and_then(bcp47)
            .or_else(|| raw.country.as_deref().and_then(bcp47));
        Self {
            locale,
            time_zone: raw
                .time_zone
                .as_deref()
                .map(str::trim)
                .filter(|zone| !zone.is_empty())
                .map(str::to_owned),
            font_scale: raw.font_size.and_then(font_scale),
            screen_reader: raw.tts,
            high_contrast: None,
            reduce_motion: None,
        }
    }
}

/// The font scale for a `system_settings` font size; `None` for an unknown size.
pub fn font_scale(size: i32) -> Option<f32> {
    usize::try_from(size)
        .ok()
        .and_then(|index| FONT_SCALES.get(index).copied())
}

/// A POSIX locale (`language[_TERRITORY][.codeset][@modifier]`) as a BCP 47 tag: `ko_KR.UTF-8`
/// is `ko-KR`, `sr_RS@latin` is `sr-Latn-RS`. `C`, `POSIX` and malformed values are `None`.
pub fn bcp47(posix: &str) -> Option<String> {
    let posix = posix.trim();
    let (rest, modifier) = match posix.split_once('@') {
        Some((rest, modifier)) => (rest, Some(modifier)),
        None => (posix, None),
    };
    let rest = rest
        .split_once('.')
        .map_or(rest, |(locale, _codeset)| locale);
    let mut parts = rest.split(['_', '-']);
    let language = parts.next()?.to_ascii_lowercase();
    if !(2..=3).contains(&language.len()) || !language.bytes().all(|b| b.is_ascii_lowercase()) {
        return None;
    }
    let mut tag = language;
    let script = match modifier.map(str::to_ascii_lowercase).as_deref() {
        Some("latin") => Some("Latn"),
        Some("cyrillic") => Some("Cyrl"),
        Some("devanagari") => Some("Deva"),
        _ => None,
    };
    let mut region = None;
    for part in parts {
        match part.len() {
            4 if part.bytes().all(|b| b.is_ascii_alphabetic()) => {
                let mut script = part.to_ascii_lowercase();
                script[..1].make_ascii_uppercase();
                tag.push('-');
                tag.push_str(&script);
            }
            2 if part.bytes().all(|b| b.is_ascii_alphabetic()) => {
                region = Some(part.to_ascii_uppercase());
            }
            3 if part.bytes().all(|b| b.is_ascii_digit()) => region = Some(part.to_owned()),
            _ => return None,
        }
    }
    if let Some(script) = script {
        tag.push('-');
        tag.push_str(script);
    }
    if let Some(region) = region {
        tag.push('-');
        tag.push_str(&region);
    }
    Some(tag)
}

/// Coalesces `system_settings` change callbacks (from any thread) into one read on the main
/// thread.
#[derive(Debug, Default)]
pub struct Coalescer {
    posted: AtomicBool,
}

impl Coalescer {
    pub const fn new() -> Self {
        Self {
            posted: AtomicBool::new(false),
        }
    }

    /// A setting changed: `true` when the caller posts the read to the main loop, `false` when a
    /// posted read will see this change too.
    pub fn changed(&self) -> bool {
        !self.posted.swap(true, Ordering::AcqRel)
    }

    /// The posted read starts: changes from now on post another.
    pub fn reading(&self) {
        self.posted.store(false, Ordering::Release);
    }
}

/// A subscription made with [`subscribe`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

type Subscriber = Box<dyn FnMut(&SystemSettings)>;

/// The current settings and their subscribers (one per main thread).
#[derive(Default)]
pub struct SettingsHub {
    current: SystemSettings,
    next_id: u64,
    subscribers: Vec<(SubscriptionId, Subscriber)>,
}

impl SettingsHub {
    pub fn current(&self) -> &SystemSettings {
        &self.current
    }

    pub fn subscribe(&mut self, subscriber: Subscriber) -> SubscriptionId {
        let id = SubscriptionId(self.next_id);
        self.next_id += 1;
        self.subscribers.push((id, subscriber));
        id
    }

    pub fn unsubscribe(&mut self, id: SubscriptionId) {
        self.subscribers.retain(|(sub, _)| *sub != id);
    }

    /// New settings were read. When they differ from the current ones, they become current and
    /// the subscribers to call are returned (taken out, so a subscriber may subscribe or
    /// unsubscribe); give them back with [`SettingsHub::restore`].
    #[must_use]
    pub fn publish(
        &mut self,
        settings: SystemSettings,
    ) -> Option<Vec<(SubscriptionId, Subscriber)>> {
        if self.current == settings {
            return None;
        }
        self.current = settings;
        Some(std::mem::take(&mut self.subscribers))
    }

    /// Puts back the subscribers [`SettingsHub::publish`] took, ahead of those added meanwhile,
    /// minus those unsubscribed meanwhile.
    pub fn restore(
        &mut self,
        mut taken: Vec<(SubscriptionId, Subscriber)>,
        unsubscribed: &[SubscriptionId],
    ) {
        taken.retain(|(id, _)| !unsubscribed.contains(id));
        taken.append(&mut self.subscribers);
        self.subscribers = taken;
    }
}

thread_local! {
    static HUB: RefCell<SettingsHub> = RefCell::default();
    static UNSUBSCRIBED: RefCell<Vec<SubscriptionId>> = const { RefCell::new(Vec::new()) };
}

/// The settings as last read (empty off Tizen, and before the platform starts). Main thread.
pub fn current() -> SystemSettings {
    HUB.with(|hub| hub.borrow().current.clone())
}

/// Calls `f` on the main thread with the new settings whenever they change.
pub fn subscribe(f: impl FnMut(&SystemSettings) + 'static) -> SubscriptionId {
    HUB.with(|hub| hub.borrow_mut().subscribe(Box::new(f)))
}

pub fn unsubscribe(id: SubscriptionId) {
    HUB.with(|hub| hub.borrow_mut().unsubscribe(id));
    UNSUBSCRIBED.with(|ids| ids.borrow_mut().push(id));
}

/// Publishes newly read settings on the main thread and calls the subscribers if they changed.
#[cfg_attr(not(target_vendor = "tizen"), allow(dead_code))]
pub(crate) fn publish(settings: SystemSettings) {
    UNSUBSCRIBED.with(|ids| ids.borrow_mut().clear());
    let taken = HUB.with(|hub| hub.borrow_mut().publish(settings.clone()));
    let Some(mut taken) = taken else {
        return;
    };
    for (_, subscriber) in &mut taken {
        subscriber(&settings);
    }
    let unsubscribed = UNSUBSCRIBED.with(|ids| std::mem::take(&mut *ids.borrow_mut()));
    HUB.with(|hub| hub.borrow_mut().restore(taken, &unsubscribed));
}

#[cfg(test)]
mod tests;
