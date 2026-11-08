//! Ecore's main loop behind the dispatcher's [`MainLoop`]. A post from another thread goes through
//! `ecore_main_loop_thread_safe_call_async`; on the main thread that call runs its callback
//! inline, so a post from the main thread is an Ecore job (`ecore_job_add`), which runs on a
//! later loop iteration. Timers are one-shot Ecore timers (main thread).

use std::ffi::c_void;
use std::thread::{self, ThreadId};
use std::time::Duration;

use dxn_gpui_tizen_sys as sys;

use crate::dispatcher::MainLoop;

type Job = Box<dyn FnOnce() + Send>;

pub struct EcoreLoop {
    main_thread: ThreadId,
}

impl EcoreLoop {
    /// Takes the calling thread (the one that runs `ui_app_main`) as the main thread.
    pub fn new() -> Self {
        Self {
            main_thread: thread::current().id(),
        }
    }
}

unsafe extern "C" fn run_job(data: *mut c_void) {
    // SAFETY: `data` is the `Box<Job>` leaked by `post`; Ecore calls this exactly once.
    let job = unsafe { Box::from_raw(data.cast::<Job>()) };
    job();
}

unsafe extern "C" fn fire_timer(data: *mut c_void) -> sys::Eina_Bool {
    // SAFETY: `data` is the `Box<Job>` leaked by `add_timer`; the timer is cancelled after this
    // one call, so Ecore never passes it again.
    let job = unsafe { Box::from_raw(data.cast::<Job>()) };
    job();
    sys::ECORE_CALLBACK_CANCEL
}

impl MainLoop for EcoreLoop {
    fn post(&self, task: Job) {
        let data = Box::into_raw(Box::new(task)).cast::<c_void>();
        if thread::current().id() == self.main_thread {
            // SAFETY: on the main thread; the job runs once on a later iteration and `run_job`
            // takes ownership of `data`.
            unsafe { sys::ecore_job_add(Some(run_job), data) };
        } else {
            // SAFETY: documented as callable from any thread (queued off the main thread);
            // `run_job` takes ownership of `data`.
            unsafe { sys::ecore_main_loop_thread_safe_call_async(Some(run_job), data) };
        }
    }

    fn add_timer(&self, delay: Duration, task: Job) {
        let data = Box::into_raw(Box::new(task)).cast::<c_void>();
        // SAFETY: called on the main thread (the dispatcher posts first from other threads);
        // `fire_timer` takes ownership of `data`.
        unsafe { sys::ecore_timer_add(delay.as_secs_f64(), Some(fire_timer), data) };
    }
}
