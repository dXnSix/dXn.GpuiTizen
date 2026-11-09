use std::cell::RefCell;
use std::rc::Rc;

use super::*;

#[test]
fn posix_locales_become_bcp47() {
    assert_eq!(bcp47("ko_KR.UTF-8").as_deref(), Some("ko-KR"));
    assert_eq!(bcp47("en_US").as_deref(), Some("en-US"));
    assert_eq!(bcp47("de").as_deref(), Some("de"));
    assert_eq!(bcp47("pt_br.utf8").as_deref(), Some("pt-BR"));
    assert_eq!(bcp47("sr_RS@latin").as_deref(), Some("sr-Latn-RS"));
    assert_eq!(bcp47("uz_UZ.UTF-8@cyrillic").as_deref(), Some("uz-Cyrl-UZ"));
    assert_eq!(bcp47("zh_Hant_TW").as_deref(), Some("zh-Hant-TW"));
    assert_eq!(bcp47("es_419").as_deref(), Some("es-419"));
    assert_eq!(bcp47("en-GB").as_deref(), Some("en-GB"), "already BCP 47");
    assert_eq!(bcp47(" fil_PH ").as_deref(), Some("fil-PH"));
    for bad in [
        "",
        "C",
        "POSIX",
        "C.UTF-8",
        "english_US",
        "k1_KR",
        "ko_KOREA",
        "ko__KR",
    ] {
        assert_eq!(bcp47(bad), None, "{bad:?}");
    }
}

#[test]
fn the_font_scale_is_monotonic_with_normal_at_one() {
    assert_eq!(font_scale(1), Some(1.0));
    let scales: Vec<f32> = (0..5).map(|size| font_scale(size).unwrap()).collect();
    assert!(
        scales.windows(2).all(|pair| pair[0] < pair[1]),
        "{scales:?}"
    );
    assert!(scales[0] < 1.0);
    assert_eq!(font_scale(-1), None);
    assert_eq!(font_scale(5), None);
}

#[test]
fn raw_values_become_settings() {
    let raw = RawSettings {
        language: Some("ko_KR.UTF-8".into()),
        country: Some("en_US.UTF-8".into()),
        time_zone: Some(" Asia/Seoul ".into()),
        font_size: Some(2),
        tts: Some(true),
    };
    assert_eq!(
        SystemSettings::from_raw(&raw),
        SystemSettings {
            locale: Some("ko-KR".into()),
            time_zone: Some("Asia/Seoul".into()),
            font_scale: Some(1.25),
            screen_reader: Some(true),
            high_contrast: None,
            reduce_motion: None,
        }
    );
}

#[test]
fn unsupported_keys_yield_none() {
    assert_eq!(
        SystemSettings::from_raw(&RawSettings::default()),
        SystemSettings::default()
    );
    let raw = RawSettings {
        language: Some("C".into()),
        country: Some("de_AT.UTF-8".into()),
        time_zone: Some("".into()),
        font_size: Some(9),
        tts: None,
    };
    let settings = SystemSettings::from_raw(&raw);
    assert_eq!(
        settings.locale.as_deref(),
        Some("de-AT"),
        "the region when the language is unusable"
    );
    assert_eq!(settings.time_zone, None);
    assert_eq!(settings.font_scale, None, "an unknown size");
    assert_eq!(settings.screen_reader, None);
    assert_eq!(settings.high_contrast, None);
    assert_eq!(settings.reduce_motion, None);
}

#[test]
fn a_burst_of_changes_posts_one_read() {
    let coalescer = Coalescer::new();
    assert!(coalescer.changed(), "the first change posts a read");
    assert!(!coalescer.changed());
    assert!(!coalescer.changed());
    coalescer.reading();
    assert!(
        coalescer.changed(),
        "a change during the read posts another"
    );
    assert!(!coalescer.changed());
}

#[test]
fn changes_coalesce_across_threads() {
    let coalescer = std::sync::Arc::new(Coalescer::new());
    let posts: usize = (0..8)
        .map(|_| {
            let coalescer = std::sync::Arc::clone(&coalescer);
            std::thread::spawn(move || (0..100).filter(|_| coalescer.changed()).count())
        })
        .map(|thread| thread.join().unwrap())
        .sum();
    assert_eq!(posts, 1);
}

fn settings(locale: &str) -> SystemSettings {
    SystemSettings {
        locale: Some(locale.into()),
        ..SystemSettings::default()
    }
}

#[test]
fn subscribers_hear_only_changes() {
    let mut hub = SettingsHub::default();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let log = Rc::clone(&seen);
    hub.subscribe(Box::new(move |s: &SystemSettings| {
        log.borrow_mut().push(s.locale.clone())
    }));
    let deliver = |hub: &mut SettingsHub, s: SystemSettings| {
        if let Some(mut taken) = hub.publish(s.clone()) {
            for (_, f) in &mut taken {
                f(&s);
            }
            hub.restore(taken, &[]);
        }
    };
    deliver(&mut hub, settings("ko-KR"));
    deliver(&mut hub, settings("ko-KR"));
    deliver(&mut hub, settings("en-US"));
    assert_eq!(*seen.borrow(), [Some("ko-KR".into()), Some("en-US".into())]);
    assert_eq!(hub.current(), &settings("en-US"));
}

#[test]
fn publication_runs_subscribers_that_may_subscribe_and_unsubscribe() {
    let seen = Rc::new(RefCell::new(Vec::<String>::new()));
    let log = Rc::clone(&seen);
    let first = subscribe(move |s| log.borrow_mut().push(format!("first {:?}", s.locale)));
    let log = Rc::clone(&seen);
    let added = Rc::new(RefCell::new(None));
    let slot = Rc::clone(&added);
    let second = subscribe(move |_| {
        // Unsubscribe the first and add a third, from inside a delivery.
        unsubscribe(first);
        if slot.borrow().is_none() {
            let log = Rc::clone(&log);
            *slot.borrow_mut() = Some(subscribe(move |s| {
                log.borrow_mut().push(format!("third {:?}", s.locale))
            }));
        }
    });
    publish(settings("ko-KR"));
    assert_eq!(current(), settings("ko-KR"));
    assert_eq!(*seen.borrow(), ["first Some(\"ko-KR\")"]);
    publish(settings("en-US"));
    assert_eq!(
        *seen.borrow(),
        ["first Some(\"ko-KR\")", "third Some(\"en-US\")"]
    );
    unsubscribe(second);
    unsubscribe(added.borrow().unwrap());
    publish(settings("de-DE"));
    assert_eq!(seen.borrow().len(), 2, "nobody left");
}
