//! The store behind `EGL_ANDROID_blob_cache` (design `d24-platform-egl`): the driver hands the
//! platform its compiled shaders as key/value blobs, and asks for them again on the next start.
//!
//! - One file per key under the cache directory, named by a stable 128-bit hash of the key; the
//!   file holds the whole key (a hash collision is a miss, not a wrong shader), the value and a
//!   checksum.
//! - Writes go to a temporary file renamed over the entry, so a reader never sees half an entry.
//!   Nothing is synced: the driver calls from the main thread while it links, and an entry torn
//!   by a crash fails its checksum and is a miss.
//! - The total size is bounded: the least recently used entries are evicted, by access time,
//!   which persists as the files' modification times.
//! - A corrupt or truncated entry is ignored and removed; any I/O error is a miss.
//! - Drivers may call from any thread: the store is `Sync`.
//! - EGL asks for a value's size, then for the value: the size probe keeps the value it read, so
//!   the lookup reads the file once, and only the copy counts as a hit and refreshes the entry.
//!
//! EGL's callbacks map onto [`BlobCache::set`] and [`BlobCache::get`]; the glue is in the `egl`
//! module.

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::SystemTime;

/// The cache directory's name under the app's cache directory.
pub const DIR_NAME: &str = "egl-blob-cache";
/// The default bound on the cache's total size.
pub const DEFAULT_MAX_BYTES: u64 = 16 * 1024 * 1024;

const MAGIC: [u8; 8] = *b"DXNBLOB1";
/// Magic, key length, value length, checksum.
const OVERHEAD: u64 = 8 + 4 + 4 + 8;

/// Two FNV-1a lanes: stable across runs and builds.
fn hash(bytes: &[u8]) -> [u64; 2] {
    let mut lanes = [0xcbf2_9ce4_8422_2325u64, 0x6c62_272e_07bb_0142];
    for lane in &mut lanes {
        for &byte in bytes {
            *lane ^= u64::from(byte);
            *lane = lane.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    lanes
}

/// The entry file name for `key`.
pub fn file_name(key: &[u8]) -> String {
    let [a, b] = hash(key);
    format!("{a:016x}{b:016x}.blob")
}

fn checksum(key: &[u8], value: &[u8]) -> u64 {
    let mut bytes = Vec::with_capacity(key.len() + value.len() + 8);
    bytes.extend_from_slice(&(key.len() as u64).to_le_bytes());
    bytes.extend_from_slice(key);
    bytes.extend_from_slice(value);
    let [a, b] = hash(&bytes);
    a ^ b.rotate_left(32)
}

/// An entry file's bytes.
pub fn encode(key: &[u8], value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(OVERHEAD as usize + key.len() + value.len());
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&(key.len() as u32).to_le_bytes());
    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    out.extend_from_slice(key);
    out.extend_from_slice(value);
    out.extend_from_slice(&checksum(key, value).to_le_bytes());
    out
}

/// The value of an entry file holding `key`; `None` for another key or a corrupt entry.
pub fn decode<'a>(bytes: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    let header = bytes.get(..16)?;
    if header[..8] != MAGIC {
        return None;
    }
    let key_len = u32::from_le_bytes(header[8..12].try_into().ok()?) as usize;
    let value_len = u32::from_le_bytes(header[12..16].try_into().ok()?) as usize;
    let body = bytes.get(16..)?;
    if body.len() != key_len + value_len + 8 {
        return None;
    }
    let (stored_key, rest) = body.split_at(key_len);
    let (value, sum) = rest.split_at(value_len);
    let sum = u64::from_le_bytes(sum.try_into().ok()?);
    (stored_key == key && sum == checksum(key, value)).then_some(value)
}

#[derive(Clone, Copy, Debug)]
struct Entry {
    size: u64,
    /// The store's clock at the last access (larger is more recent).
    used: u64,
}

#[derive(Debug, Default)]
struct State {
    entries: HashMap<String, Entry>,
    total: u64,
    clock: u64,
    /// The entry a size probe read last (file name and bytes), for the fetch that follows.
    probed: Option<(String, Vec<u8>)>,
}

/// What the store did, for logs and tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub sets: u64,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

/// A bounded on-disk key/value store for the driver's shader blobs.
#[derive(Debug)]
pub struct BlobCache {
    dir: PathBuf,
    max_bytes: u64,
    state: Mutex<State>,
    sets: AtomicU64,
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

static TEMP: AtomicU64 = AtomicU64::new(0);

impl BlobCache {
    /// Opens (creating) the store in `dir`, bounded to `max_bytes`, indexing the entries already
    /// there in order of their modification times.
    pub fn open(dir: impl Into<PathBuf>, max_bytes: u64) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        let mut found: Vec<(SystemTime, String, u64)> = Vec::new();
        for entry in fs::read_dir(&dir)?.filter_map(Result::ok) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if !metadata.is_file() {
                continue;
            }
            if name.ends_with(".blob") {
                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                found.push((modified, name, metadata.len()));
            } else if name.contains(".tmp-") {
                // A write that never finished.
                let _ = fs::remove_file(entry.path());
            }
        }
        found.sort();
        let mut state = State::default();
        for (_, name, size) in found {
            state.clock += 1;
            state.total += size;
            state.entries.insert(
                name,
                Entry {
                    size,
                    used: state.clock,
                },
            );
        }
        let cache = Self {
            dir,
            max_bytes,
            state: Mutex::new(state),
            sets: AtomicU64::new(0),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        };
        cache.evict(&mut cache.lock());
        Ok(cache)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn counts(&self) -> Counts {
        Counts {
            sets: self.sets.load(Ordering::Relaxed),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
        }
    }

    /// The total size of the entries.
    pub fn total_bytes(&self) -> u64 {
        self.lock().total
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Stores `value` under `key` (EGL's `set` callback). Values larger than the whole cache are
    /// not stored.
    pub fn set(&self, key: &[u8], value: &[u8]) {
        let bytes = encode(key, value);
        let size = bytes.len() as u64;
        if size > self.max_bytes {
            return;
        }
        let name = file_name(key);
        let path = self.dir.join(&name);
        let temp = self.dir.join(format!(
            "{name}.tmp-{}-{}",
            std::process::id(),
            TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        let written = fs::File::create(&temp)
            .and_then(|mut file| file.write_all(&bytes))
            .and_then(|()| fs::rename(&temp, &path));
        if written.is_err() {
            let _ = fs::remove_file(&temp);
            return;
        }
        self.sets.fetch_add(1, Ordering::Relaxed);
        let mut state = self.lock();
        if state
            .probed
            .as_ref()
            .is_some_and(|(probed, _)| *probed == name)
        {
            state.probed = None;
        }
        state.clock += 1;
        let used = state.clock;
        if let Some(old) = state.entries.insert(name, Entry { size, used }) {
            state.total -= old.size;
        }
        state.total += size;
        self.evict(&mut state);
    }

    /// The value stored under `key` (EGL's `get` callback): its size, copied into `out` when `out`
    /// holds it, or 0 for a miss.
    pub fn get(&self, key: &[u8], out: &mut [u8]) -> usize {
        let name = file_name(key);
        let path = self.dir.join(&name);
        let probed = {
            let mut state = self.lock();
            match &state.probed {
                Some((probed, _)) if *probed == name => state.probed.take().map(|(_, bytes)| bytes),
                _ => None,
            }
        };
        let bytes = match probed.map_or_else(|| fs::read(&path), Ok) {
            Ok(bytes) => bytes,
            Err(_) => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                return 0;
            }
        };
        let Some(value) = decode(&bytes, key) else {
            // Another key with the same name keeps its file; a corrupt one goes.
            if bytes.get(..8) != Some(&MAGIC[..]) || !holds_key(&bytes) {
                let _ = fs::remove_file(&path);
                let mut state = self.lock();
                if let Some(old) = state.entries.remove(&name) {
                    state.total -= old.size;
                }
            }
            self.misses.fetch_add(1, Ordering::Relaxed);
            return 0;
        };
        let len = value.len();
        if len > out.len() {
            // A size probe: the fetch that follows uses these bytes.
            self.lock().probed = Some((name, bytes));
            return len;
        }
        out[..len].copy_from_slice(value);
        self.hits.fetch_add(1, Ordering::Relaxed);
        let mut state = self.lock();
        state.clock += 1;
        let used = state.clock;
        if let Some(entry) = state.entries.get_mut(&name) {
            entry.used = used;
        }
        drop(state);
        // The access time survives restarts as the file's modification time.
        if let Ok(file) = fs::File::options().write(true).open(&path) {
            let _ = file.set_modified(SystemTime::now());
        }
        len
    }

    /// Removes least recently used entries until the total fits.
    fn evict(&self, state: &mut State) {
        while state.total > self.max_bytes {
            let Some((name, entry)) = state
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.used)
                .map(|(name, entry)| (name.clone(), *entry))
            else {
                break;
            };
            let _ = fs::remove_file(self.dir.join(&name));
            if state
                .probed
                .as_ref()
                .is_some_and(|(probed, _)| *probed == name)
            {
                state.probed = None;
            }
            state.entries.remove(&name);
            state.total -= entry.size;
            self.evictions.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Whether `bytes` is a whole, checksummed entry for some key.
fn holds_key(bytes: &[u8]) -> bool {
    let Some(header) = bytes.get(..16) else {
        return false;
    };
    let key_len = u32::from_le_bytes(header[8..12].try_into().unwrap_or_default()) as usize;
    let Some(key) = bytes.get(16..16 + key_len) else {
        return false;
    };
    decode(bytes, key).is_some()
}

#[cfg(test)]
mod tests;
