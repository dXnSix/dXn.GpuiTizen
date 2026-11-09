use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

/// A fresh directory in the temp dir, removed on drop.
struct Dir(PathBuf);

impl Dir {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "dxn-blob-cache-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&path);
        Self(path)
    }

    fn files(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(&self.0)
            .map(|read| {
                read.filter_map(Result::ok)
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn get(cache: &BlobCache, key: &[u8]) -> Option<Vec<u8>> {
    let size = cache.get(key, &mut []);
    if size == 0 {
        return None;
    }
    let mut out = vec![0; size];
    assert_eq!(cache.get(key, &mut out), size);
    Some(out)
}

#[test]
fn stores_and_loads_like_egl_asks() {
    let dir = Dir::new();
    let cache = BlobCache::open(&dir.0, DEFAULT_MAX_BYTES).unwrap();
    assert_eq!(cache.get(b"shader-1", &mut [0; 16]), 0, "a miss");
    cache.set(b"shader-1", b"compiled program");
    // EGL asks for the size first with too small a buffer, then again.
    let mut small = [0u8; 4];
    assert_eq!(cache.get(b"shader-1", &mut small), 16);
    assert_eq!(small, [0; 4], "nothing copied into a buffer too small");
    assert_eq!(
        get(&cache, b"shader-1").as_deref(),
        Some(&b"compiled program"[..])
    );
    cache.set(b"shader-1", b"recompiled");
    assert_eq!(
        get(&cache, b"shader-1").as_deref(),
        Some(&b"recompiled"[..])
    );
    assert_eq!(
        dir.files(),
        [file_name(b"shader-1")],
        "one file per key, no temporaries"
    );
    assert_eq!(cache.counts().sets, 2);
    // Entries survive a restart.
    drop(cache);
    let reopened = BlobCache::open(&dir.0, DEFAULT_MAX_BYTES).unwrap();
    assert_eq!(
        get(&reopened, b"shader-1").as_deref(),
        Some(&b"recompiled"[..])
    );
}

#[test]
fn the_least_recently_used_entries_are_evicted() {
    let dir = Dir::new();
    let entry = encode(b"k0", &[0; 100]).len() as u64;
    let cache = BlobCache::open(&dir.0, 3 * entry).unwrap();
    for key in [b"k0", b"k1", b"k2"] {
        cache.set(key, &[7; 100]);
    }
    assert!(get(&cache, b"k0").is_some(), "k0 is used again");
    cache.set(b"k3", &[7; 100]);
    assert!(get(&cache, b"k1").is_none(), "the least recently used goes");
    assert!(get(&cache, b"k0").is_some());
    assert!(get(&cache, b"k2").is_some());
    assert!(get(&cache, b"k3").is_some());
    assert_eq!(cache.total_bytes(), 3 * entry);
    assert_eq!(cache.counts().evictions, 1);
    cache.set(b"huge", &vec![1; 10 * entry as usize]);
    assert!(
        get(&cache, b"huge").is_none(),
        "larger than the cache: not stored"
    );
    // A smaller bound on reopening evicts down to it.
    drop(cache);
    let smaller = BlobCache::open(&dir.0, entry).unwrap();
    assert_eq!(smaller.total_bytes(), entry);
}

#[test]
fn corrupt_or_truncated_entries_are_ignored() {
    let dir = Dir::new();
    let cache = BlobCache::open(&dir.0, DEFAULT_MAX_BYTES).unwrap();
    cache.set(b"a", b"value of a");
    cache.set(b"b", b"value of b");
    let path_a = dir.0.join(file_name(b"a"));
    let bytes = fs::read(&path_a).unwrap();
    fs::write(&path_a, &bytes[..bytes.len() - 3]).unwrap();
    assert_eq!(get(&cache, b"a"), None, "truncated");
    assert!(!path_a.exists(), "and removed");
    let path_b = dir.0.join(file_name(b"b"));
    let mut bytes = fs::read(&path_b).unwrap();
    let last = bytes.len() - 10;
    bytes[last] ^= 0xff;
    fs::write(&path_b, &bytes).unwrap();
    assert_eq!(get(&cache, b"b"), None, "a flipped bit");
    fs::write(dir.0.join(file_name(b"c")), b"garbage").unwrap();
    assert_eq!(get(&cache, b"c"), None);
    // Another key's entry under this key's name (a hash collision) is a miss, kept.
    fs::write(dir.0.join(file_name(b"d")), encode(b"not d", b"x")).unwrap();
    assert_eq!(get(&cache, b"d"), None);
    assert!(dir.0.join(file_name(b"d")).exists());
    assert_eq!(
        decode(&encode(b"key", b"value"), b"key"),
        Some(&b"value"[..])
    );
    // A write that never finished is cleaned up on the next start.
    fs::write(dir.0.join("x.blob.tmp-1-2"), b"partial").unwrap();
    drop(cache);
    let _ = BlobCache::open(&dir.0, DEFAULT_MAX_BYTES).unwrap();
    assert!(!dir.0.join("x.blob.tmp-1-2").exists());
}

#[test]
fn concurrent_callers_never_see_half_an_entry() {
    let dir = Dir::new();
    let cache = Arc::new(BlobCache::open(&dir.0, DEFAULT_MAX_BYTES).unwrap());
    let threads: Vec<_> = (0..8u8)
        .map(|t| {
            let cache = Arc::clone(&cache);
            std::thread::spawn(move || {
                for round in 0..50u8 {
                    let shared = vec![round; 1000];
                    cache.set(b"shared", &shared);
                    cache.set(&[t, round], &[t; 64]);
                    if let Some(value) = get(&cache, b"shared") {
                        assert_eq!(value.len(), 1000);
                        assert!(value.iter().all(|&b| b == value[0]), "one whole write");
                    }
                    assert_eq!(get(&cache, &[t, round]).as_deref(), Some(&[t; 64][..]));
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
    assert!(
        dir.files().iter().all(|name| name.ends_with(".blob")),
        "no temporaries left"
    );
    assert_eq!(dir.files().len(), 1 + 8 * 50);
}

#[test]
fn a_lookup_counts_once_and_only_the_copy_refreshes_the_entry() {
    let dir = Dir::new();
    let cache = BlobCache::open(&dir.0, DEFAULT_MAX_BYTES).unwrap();
    assert_eq!(get(&cache, b"absent"), None);
    assert_eq!(
        cache.counts(),
        Counts {
            misses: 1,
            ..Counts::default()
        },
        "a miss ends the lookup at the size probe"
    );

    cache.set(b"shader", b"program");
    let path = dir.0.join(file_name(b"shader"));
    let old = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000);
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(old)
        .unwrap();
    assert_eq!(cache.get(b"shader", &mut []), 7, "the size probe");
    assert_eq!(cache.counts().hits, 0, "a probe is not a hit");
    assert_eq!(
        fs::metadata(&path).unwrap().modified().unwrap(),
        old,
        "a probe does not refresh the entry"
    );
    // The fetch uses what the probe read, even when the file is gone by now.
    fs::remove_file(&path).unwrap();
    let mut out = [0u8; 7];
    assert_eq!(cache.get(b"shader", &mut out), 7);
    assert_eq!(&out, b"program");
    assert_eq!(
        cache.counts(),
        Counts {
            sets: 1,
            hits: 1,
            misses: 1,
            evictions: 0
        }
    );

    cache.set(b"shader", b"program");
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(old)
        .unwrap();
    assert_eq!(get(&cache, b"shader").as_deref(), Some(&b"program"[..]));
    assert_eq!(cache.counts().hits, 2, "one hit per lookup");
    assert!(
        fs::metadata(&path).unwrap().modified().unwrap() > old,
        "the copy refreshes the entry"
    );
}
