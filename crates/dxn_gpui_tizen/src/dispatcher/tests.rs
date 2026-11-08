use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use gpui::{PlatformDispatcher, Priority, RunnableMeta, RunnableVariant};

use super::*;

type Job = Box<dyn FnOnce() + Send>;

/// A main loop the test thread pumps by hand, with a virtual clock for timers.
#[derive(Default)]
struct FakeLoop {
    posts: Mutex<VecDeque<Job>>,
    timers: Mutex<Vec<(Duration, Job)>>,
    now: Mutex<Duration>,
    timers_added_on: Mutex<Vec<thread::ThreadId>>,
}

impl MainLoop for FakeLoop {
    fn post(&self, task: Job) {
        self.posts.lock().unwrap().push_back(task);
    }
    fn add_timer(&self, delay: Duration, task: Job) {
        self.timers_added_on
            .lock()
            .unwrap()
            .push(thread::current().id());
        let due = *self.now.lock().unwrap() + delay;
        self.timers.lock().unwrap().push((due, task));
    }
}

impl FakeLoop {
    fn posted(&self) -> usize {
        self.posts.lock().unwrap().len()
    }

    /// Runs the oldest post, as one Ecore main-loop iteration would.
    fn run_one(&self) {
        let next = self.posts.lock().unwrap().pop_front();
        if let Some(job) = next {
            job();
        }
    }

    /// Runs every post (and posts they make) on the calling thread.
    fn run_posts(&self) {
        loop {
            let next = self.posts.lock().unwrap().pop_front();
            match next {
                Some(job) => job(),
                None => return,
            }
        }
    }

    /// Advances the clock and fires the timers that are due, in due order.
    fn advance(&self, by: Duration) {
        let now = {
            let mut now = self.now.lock().unwrap();
            *now += by;
            *now
        };
        let mut due: Vec<(Duration, Job)> = Vec::new();
        {
            let mut timers = self.timers.lock().unwrap();
            let mut i = 0;
            while i < timers.len() {
                if timers[i].0 <= now {
                    due.push(timers.remove(i));
                } else {
                    i += 1;
                }
            }
        }
        due.sort_by_key(|(at, _)| *at);
        for (_, job) in due {
            job();
        }
    }
}

fn recorder() -> (Arc<Mutex<Vec<&'static str>>>, impl Fn(&'static str) -> Job) {
    let log = Arc::new(Mutex::new(Vec::new()));
    let make = {
        let log = Arc::clone(&log);
        move |name: &'static str| -> Job {
            let log = Arc::clone(&log);
            Box::new(move || log.lock().unwrap().push(name))
        }
    };
    (log, make)
}

#[test]
fn main_thread_work_runs_by_priority_and_fifo_within_each() {
    let main_loop = Arc::new(FakeLoop::default());
    let main: MainThread<FakeLoop, Job> = MainThread::new(Arc::clone(&main_loop));
    let (log, job) = recorder();
    main.dispatch(job("low-1"), Priority::Low);
    main.dispatch(job("medium-1"), Priority::Medium);
    main.dispatch(job("high-1"), Priority::High);
    main.dispatch(job("low-2"), Priority::Low);
    main.dispatch(job("high-2"), Priority::High);
    main.dispatch(job("medium-2"), Priority::Medium);
    assert_eq!(main_loop.posted(), 1, "one drain per batch");
    assert!(
        log.lock().unwrap().is_empty(),
        "nothing runs until the main loop does"
    );
    main_loop.run_posts();
    assert_eq!(
        *log.lock().unwrap(),
        ["high-1", "high-2", "medium-1", "medium-2", "low-1", "low-2"]
    );
}

#[test]
fn work_posted_from_other_threads_runs_on_the_main_thread() {
    let main_loop = Arc::new(FakeLoop::default());
    let main = Arc::new(MainThread::<FakeLoop, Job>::new(Arc::clone(&main_loop)));
    let main_id = thread::current().id();
    let (ran_on, ran) = mpsc::channel();
    let posters: Vec<_> = (0..4)
        .map(|i| {
            let main = Arc::clone(&main);
            let ran_on = ran_on.clone();
            thread::spawn(move || {
                assert!(!main.is_main_thread());
                for j in 0..25 {
                    let ran_on = ran_on.clone();
                    main.dispatch(
                        Box::new(move || ran_on.send((i, j, thread::current().id())).unwrap()),
                        Priority::Medium,
                    );
                }
            })
        })
        .collect();
    for poster in posters {
        poster.join().unwrap();
    }
    drop(ran_on);
    assert!(main.is_main_thread());
    main_loop.run_posts();
    let runs: Vec<(i32, i32, thread::ThreadId)> = ran.iter().collect();
    assert_eq!(runs.len(), 100);
    assert!(
        runs.iter().all(|(_, _, id)| *id == main_id),
        "every runnable ran on the main thread"
    );
    for poster in 0..4 {
        let order: Vec<i32> = runs
            .iter()
            .filter(|(i, _, _)| *i == poster)
            .map(|(_, j, _)| *j)
            .collect();
        assert_eq!(order, (0..25).collect::<Vec<_>>(), "FIFO per poster");
    }
}

#[test]
fn work_dispatched_while_draining_runs_in_a_later_batch() {
    let main_loop = Arc::new(FakeLoop::default());
    let main = Arc::new(MainThread::<FakeLoop, Job>::new(Arc::clone(&main_loop)));
    let (log, job) = recorder();
    // A task that reschedules itself forever.
    fn spin(main: Arc<MainThread<FakeLoop, Job>>, log: Arc<Mutex<Vec<&'static str>>>) {
        let again = Arc::clone(&main);
        main.dispatch(
            Box::new(move || {
                log.lock().unwrap().push("spin");
                spin(again, log);
            }),
            Priority::High,
        );
    }
    spin(Arc::clone(&main), Arc::clone(&log));
    // While the batch runs, an input event reaches Ecore's queue.
    let ecore = Arc::clone(&main_loop);
    let event = Mutex::new(Some(job("ecore-event")));
    let queued = job("queued");
    main.dispatch(
        Box::new(move || {
            queued();
            if let Some(event) = event.lock().unwrap().take() {
                ecore.post(event);
            }
        }),
        Priority::Low,
    );
    // Ecore runs one post at a time.
    main_loop.run_one();
    assert_eq!(
        *log.lock().unwrap(),
        ["spin", "queued"],
        "one batch: the work queued when it started"
    );
    main_loop.run_one();
    assert_eq!(
        log.lock().unwrap().last().copied(),
        Some("ecore-event"),
        "Ecore ran between batches"
    );
    main_loop.run_one();
    assert_eq!(
        *log.lock().unwrap(),
        ["spin", "queued", "ecore-event", "spin"]
    );
    assert_eq!(
        main_loop.posted(),
        1,
        "the next batch is posted, not run inline"
    );
}

/// A thread-bound (foreground) GPUI runnable, like `spawn_local`'s.
fn local_runnable(ran: Arc<std::sync::atomic::AtomicBool>) -> RunnableVariant {
    let (runnable, task) = async_task::Builder::new()
        .metadata(RunnableMeta::new_with_callers_location())
        .spawn_local(
            move |_| async move {
                ran.store(true, std::sync::atomic::Ordering::SeqCst);
            },
            |_runnable| {},
        );
    task.detach();
    runnable
}

#[test]
fn a_foreground_runnable_from_another_thread_after_shutdown_is_not_dropped_there() {
    let main_loop = Arc::new(FakeLoop::default());
    let dispatcher = Arc::new(TizenDispatcher::new(Arc::clone(&main_loop)));
    dispatcher.shut_down();
    let ran = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let runnable = local_runnable(Arc::clone(&ran));
    let other = Arc::clone(&dispatcher);
    let result = thread::spawn(move || {
        // Dropping a spawn_local runnable here would panic (and abort the app on a TV).
        other.dispatch_on_main_thread(runnable, Priority::High);
    })
    .join();
    assert!(result.is_ok(), "no panic on the worker");
    let after = local_runnable(Arc::clone(&ran));
    let other = Arc::clone(&dispatcher);
    assert!(
        thread::spawn(move || other.dispatch_after(Duration::from_millis(1), after))
            .join()
            .is_ok()
    );
    main_loop.run_posts();
    main_loop.advance(Duration::from_secs(1));
    assert!(
        !ran.load(std::sync::atomic::Ordering::SeqCst),
        "it never runs"
    );
}

#[test]
fn dispatch_after_fires_no_earlier_than_its_delay() {
    let main_loop = Arc::new(FakeLoop::default());
    let main = Arc::new(MainThread::<FakeLoop, Job>::new(Arc::clone(&main_loop)));
    let (log, job) = recorder();
    main.dispatch_after(Duration::from_millis(50), job("after-50"));
    main.dispatch_after(Duration::from_millis(10), job("after-10"));
    main_loop.advance(Duration::from_millis(9));
    assert!(log.lock().unwrap().is_empty());
    main_loop.advance(Duration::from_millis(1));
    assert_eq!(*log.lock().unwrap(), ["after-10"]);
    main_loop.advance(Duration::from_millis(39));
    assert_eq!(*log.lock().unwrap(), ["after-10"]);
    main_loop.advance(Duration::from_millis(1));
    assert_eq!(*log.lock().unwrap(), ["after-10", "after-50"]);

    // From another thread the timer is added on the main thread, through a post.
    let other = Arc::clone(&main);
    let delayed = job("from-worker");
    thread::spawn(move || other.dispatch_after(Duration::from_millis(5), delayed))
        .join()
        .unwrap();
    assert_eq!(
        main_loop.timers.lock().unwrap().len(),
        0,
        "not added off the main thread"
    );
    main_loop.run_posts();
    assert!(
        main_loop
            .timers_added_on
            .lock()
            .unwrap()
            .iter()
            .all(|id| *id == thread::current().id())
    );
    main_loop.advance(Duration::from_millis(5));
    assert_eq!(log.lock().unwrap().last().copied(), Some("from-worker"));
}

#[test]
fn nothing_runs_after_shutdown() {
    let main_loop = Arc::new(FakeLoop::default());
    let main: MainThread<FakeLoop, Job> = MainThread::new(Arc::clone(&main_loop));
    let (log, job) = recorder();
    main.dispatch(job("queued"), Priority::High);
    main.dispatch_after(Duration::from_millis(1), job("timer"));
    main.shut_down();
    main.dispatch(job("after"), Priority::High);
    main.dispatch_after(Duration::from_millis(1), job("timer-after"));
    main_loop.run_posts();
    main_loop.advance(Duration::from_secs(1));
    assert!(log.lock().unwrap().is_empty(), "{:?}", log.lock().unwrap());
}

/// A GPUI runnable that sends `value` when it runs.
fn runnable(value: u32, sender: mpsc::Sender<(u32, thread::ThreadId)>) -> RunnableVariant {
    let (runnable, task) = async_task::Builder::new()
        .metadata(RunnableMeta::new_with_callers_location())
        .spawn(
            move |_| async move {
                sender.send((value, thread::current().id())).unwrap();
            },
            |_runnable| {},
        );
    task.detach();
    runnable
}

#[test]
fn the_platform_dispatcher_runs_gpui_runnables_on_the_right_threads() {
    let main_loop = Arc::new(FakeLoop::default());
    let dispatcher = TizenDispatcher::new(Arc::clone(&main_loop));
    let main_id = thread::current().id();
    assert!(dispatcher.is_main_thread());

    let (sender, ran) = mpsc::channel();
    dispatcher.dispatch_on_main_thread(runnable(1, sender.clone()), Priority::Low);
    dispatcher.dispatch_on_main_thread(runnable(2, sender.clone()), Priority::High);
    main_loop.run_posts();
    assert_eq!(ran.recv().unwrap(), (2, main_id));
    assert_eq!(ran.recv().unwrap(), (1, main_id));

    dispatcher.dispatch(runnable(3, sender.clone()), Priority::Medium);
    let (value, worker) = ran.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(value, 3);
    assert_ne!(worker, main_id, "background work runs on the worker pool");

    dispatcher.dispatch_after(Duration::from_millis(20), runnable(4, sender.clone()));
    main_loop.advance(Duration::from_millis(19));
    assert!(ran.try_recv().is_err());
    main_loop.advance(Duration::from_millis(1));
    assert_eq!(ran.recv().unwrap(), (4, main_id));

    dispatcher.shut_down();
    dispatcher.dispatch_on_main_thread(runnable(5, sender), Priority::High);
    main_loop.run_posts();
    assert!(ran.try_recv().is_err(), "nothing runs after shutdown");
}
