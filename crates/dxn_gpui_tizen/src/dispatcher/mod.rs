//! GPUI's `PlatformDispatcher` over a [`MainLoop`] abstraction.
//!
//! On Tizen the main loop is Ecore's: [`MainLoop::post`] is
//! `ecore_main_loop_thread_safe_call_async` (callable from any thread) and
//! [`MainLoop::add_timer`] an Ecore timer (called on the main thread). Main-thread runnables wait
//! in three FIFO queues and are drained high before medium before low, one post per batch;
//! background runnables run on a worker pool; `dispatch_after` fires its runnable on the main
//! thread no earlier than its delay. After [`TizenDispatcher::shut_down`] nothing runs. The
//! Ecore-backed loop lives in the Tizen shell; `dispatcher::tests` drives a fake.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread::{self, ThreadId};
use std::time::Duration;

use gpui::{PlatformDispatcher, Priority, RunnableVariant};

/// The main loop a dispatcher posts to.
pub trait MainLoop: Send + Sync + 'static {
    /// Runs `task` on the main thread soon. Callable from any thread.
    fn post(&self, task: Box<dyn FnOnce() + Send>);
    /// Runs `task` on the main thread once `delay` has passed. Called on the main thread only.
    fn add_timer(&self, delay: Duration, task: Box<dyn FnOnce() + Send>);
}

/// Something a dispatcher runs: GPUI's runnables, or closures in tests.
pub trait Task: Send + 'static {
    fn run(self);
}

impl Task for RunnableVariant {
    fn run(self) {
        RunnableVariant::run(self);
    }
}

impl Task for Box<dyn FnOnce() + Send> {
    fn run(self) {
        self()
    }
}

/// Main-thread work by priority, each queue FIFO.
struct MainQueues<T> {
    high: VecDeque<T>,
    medium: VecDeque<T>,
    low: VecDeque<T>,
}

impl<T> Default for MainQueues<T> {
    fn default() -> Self {
        Self {
            high: VecDeque::new(),
            medium: VecDeque::new(),
            low: VecDeque::new(),
        }
    }
}

impl<T> MainQueues<T> {
    fn push(&mut self, priority: Priority, task: T) {
        match priority {
            Priority::High | Priority::RealtimeAudio => self.high.push_back(task),
            Priority::Medium => self.medium.push_back(task),
            Priority::Low => self.low.push_back(task),
        }
    }

    fn len(&self) -> usize {
        self.high.len() + self.medium.len() + self.low.len()
    }

    fn pop(&mut self) -> Option<T> {
        self.high
            .pop_front()
            .or_else(|| self.medium.pop_front())
            .or_else(|| self.low.pop_front())
    }
}

/// The main-thread half of a dispatcher, generic over the task type so tests can use closures.
///
/// Each posted drain runs only the work queued when it starts, then posts another drain for the
/// rest, so Ecore handles input and frame callbacks between batches even when tasks keep
/// rescheduling themselves.
pub struct MainThread<L: MainLoop, T: Task> {
    inner: Arc<Inner<L, T>>,
    main_thread: ThreadId,
}

struct Inner<L, T> {
    main_loop: Arc<L>,
    queues: Mutex<MainQueues<T>>,
    /// A drain is posted and has not started yet: further work joins it.
    drain_posted: AtomicBool,
    shut_down: AtomicBool,
}

impl<L: MainLoop, T: Task> Inner<L, T> {
    fn post_drain(self: &Arc<Self>) {
        if !self.drain_posted.swap(true, Ordering::AcqRel) {
            let inner = Arc::clone(self);
            self.main_loop.post(Box::new(move || inner.drain()));
        }
    }

    fn drain(self: &Arc<Self>) {
        // The batch is the work queued now, in priority order; what it dispatches waits for the
        // next drain. The lock is not held while tasks run, and `drain_posted` stays set, so
        // nothing posts the next drain before this batch ends.
        let batch: Vec<T> = {
            let mut queues = self.queues.lock().unwrap_or_else(PoisonError::into_inner);
            std::iter::from_fn(|| queues.pop()).collect()
        };
        let mut batch = batch.into_iter();
        for task in batch.by_ref() {
            if self.shut_down.load(Ordering::Acquire) {
                break;
            }
            task.run();
        }
        // After a shutdown mid-batch the rest is dropped here, on the main thread.
        drop(batch);
        // Work dispatched during the batch (which saw a drain still posted) or after the flag
        // clears gets exactly one next drain, posted behind whatever Ecore queued meanwhile.
        self.drain_posted.store(false, Ordering::Release);
        let more = self
            .queues
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
            > 0;
        if more && !self.shut_down.load(Ordering::Acquire) {
            self.post_drain();
        }
    }
}

impl<L: MainLoop, T: Task> MainThread<L, T> {
    /// Takes the calling thread as the main thread.
    pub fn new(main_loop: Arc<L>) -> Self {
        Self {
            inner: Arc::new(Inner {
                main_loop,
                queues: Mutex::new(MainQueues::default()),
                drain_posted: AtomicBool::new(false),
                shut_down: AtomicBool::new(false),
            }),
            main_thread: thread::current().id(),
        }
    }

    pub fn is_main_thread(&self) -> bool {
        thread::current().id() == self.main_thread
    }

    /// Queues `task` and posts one drain for every batch of work. After shutdown the task is
    /// forgotten, not dropped: a foreground runnable must not be dropped off its own thread.
    pub fn dispatch(&self, task: T, priority: Priority) {
        if self.inner.shut_down.load(Ordering::Acquire) {
            std::mem::forget(task);
            return;
        }
        self.inner
            .queues
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(priority, task);
        self.inner.post_drain();
    }

    /// Runs `task` on the main thread once `delay` has passed. From another thread the timer is
    /// added through a post, because main-loop timers are main-thread only.
    pub fn dispatch_after(&self, delay: Duration, task: T) {
        if self.inner.shut_down.load(Ordering::Acquire) {
            std::mem::forget(task);
            return;
        }
        let inner = Arc::clone(&self.inner);
        let fire: Box<dyn FnOnce() + Send> = Box::new(move || {
            if inner.shut_down.load(Ordering::Acquire) {
                // Timers fire on the main thread, but keep the rule uniform.
                std::mem::forget(task);
            } else {
                task.run();
            }
        });
        if self.is_main_thread() {
            self.inner.main_loop.add_timer(delay, fire);
        } else {
            let main_loop = Arc::clone(&self.inner.main_loop);
            self.inner
                .main_loop
                .post(Box::new(move || main_loop.add_timer(delay, fire)));
        }
    }

    /// Drops queued work (on the main thread, where foreground runnables belong); nothing
    /// dispatched afterwards runs.
    pub fn shut_down(&self) {
        self.inner.shut_down.store(true, Ordering::Release);
        let mut queues = self
            .inner
            .queues
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *queues = MainQueues::default();
    }
}

/// The background half: a worker pool draining its own priority queues (high first, FIFO within
/// each priority).
pub struct Background {
    shared: Arc<(Mutex<MainQueues<RunnableVariant>>, Condvar)>,
    _workers: Vec<thread::JoinHandle<()>>,
}

const MIN_WORKERS: usize = 2;

impl Background {
    pub fn new() -> Self {
        let shared: Arc<(Mutex<MainQueues<RunnableVariant>>, Condvar)> = Arc::default();
        let count =
            thread::available_parallelism().map_or(MIN_WORKERS, |n| n.get().max(MIN_WORKERS));
        let workers = (0..count)
            .filter_map(|i| {
                let shared = Arc::clone(&shared);
                thread::Builder::new()
                    .name(format!("gpui-worker-{i}"))
                    .spawn(move || worker(&shared))
                    .ok()
            })
            .collect();
        Self {
            shared,
            _workers: workers,
        }
    }

    pub fn dispatch(&self, runnable: RunnableVariant, priority: Priority) {
        let (queues, ready) = &*self.shared;
        queues
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(priority, runnable);
        ready.notify_one();
    }
}

fn worker(shared: &(Mutex<MainQueues<RunnableVariant>>, Condvar)) {
    let (queues, ready) = shared;
    loop {
        let runnable = {
            let mut queues = queues.lock().unwrap_or_else(PoisonError::into_inner);
            loop {
                if let Some(runnable) = queues.pop() {
                    break runnable;
                }
                queues = ready.wait(queues).unwrap_or_else(PoisonError::into_inner);
            }
        };
        runnable.run();
    }
}

impl Default for Background {
    fn default() -> Self {
        Self::new()
    }
}

/// GPUI's dispatcher on Tizen.
pub struct TizenDispatcher<L: MainLoop> {
    main: MainThread<L, RunnableVariant>,
    background: Background,
}

impl<L: MainLoop> TizenDispatcher<L> {
    /// Takes the calling thread (the one that runs `ui_app_main`) as the main thread.
    pub fn new(main_loop: Arc<L>) -> Self {
        Self {
            main: MainThread::new(main_loop),
            background: Background::new(),
        }
    }

    /// Stops all main-thread work (the app is terminating).
    pub fn shut_down(&self) {
        self.main.shut_down();
    }
}

impl<L: MainLoop> PlatformDispatcher for TizenDispatcher<L> {
    fn is_main_thread(&self) -> bool {
        self.main.is_main_thread()
    }

    fn dispatch(&self, runnable: RunnableVariant, priority: Priority) {
        self.background.dispatch(runnable, priority);
    }

    fn dispatch_on_main_thread(&self, runnable: RunnableVariant, priority: Priority) {
        self.main.dispatch(runnable, priority);
    }

    fn dispatch_after(&self, duration: Duration, runnable: RunnableVariant) {
        self.main.dispatch_after(duration, runnable);
    }

    fn spawn_realtime(&self, f: Box<dyn FnOnce() + Send>) {
        // No realtime threads for third-party TV apps; a plain thread keeps the work off the main
        // loop.
        let _ = thread::Builder::new().name("gpui-realtime".into()).spawn(f);
    }
}

#[cfg(test)]
mod tests;
