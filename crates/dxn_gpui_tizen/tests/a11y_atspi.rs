//! The accessibility bridge over real AT-SPI (Linux only; run in the `gpuitizen` Lima VM under
//! `dbus-run-session`, with at-spi2-core): the platform's AccessKit adapter publishes a small
//! tree, which is read back over the accessibility bus (names and roles), and an AT-SPI
//! `DoAction` reaches GPUI's action callback on the main thread.
//!
//! `scripts/lima-run.sh dbus-run-session -- cargo test -p dxn_gpui_tizen --test a11y_atspi`
#![cfg(target_os = "linux")]

use std::cell::RefCell;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use dxn_gpui_tizen::a11y::{MainThreadEvent, OnMainThread, Post, UnixAdapter};
use gpui::A11yCallbacks;
use gpui::accesskit::{Action, ActionRequest, Node, NodeId, Role, Tree, TreeId, TreeUpdate};
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

const WINDOW: NodeId = NodeId(1);
const BUTTON: NodeId = NodeId(2);
const LABEL: NodeId = NodeId(3);
const TIMEOUT: Duration = Duration::from_secs(30);
/// `AtspiRole` values (at-spi2-core's `atspi-constants.h`).
const ROLE_FRAME: u32 = 23;
const ROLE_LABEL: u32 = 29;
const ROLE_PUSH_BUTTON: u32 = 43;

thread_local! {
    /// The window's adapter, on the main thread (this test's thread).
    static ADAPTER: RefCell<Option<UnixAdapter>> = const { RefCell::new(None) };
}

fn tree() -> TreeUpdate {
    let mut window = Node::new(Role::Window);
    window.set_label("Long list".to_owned());
    window.set_children(vec![BUTTON, LABEL]);
    let mut button = Node::new(Role::Button);
    button.set_label("Filter".to_owned());
    button.add_action(Action::Click);
    let mut label = Node::new(Role::Label);
    // AccessKit names static text (`Label`) from its value.
    label.set_value("Row 1".to_owned());
    TreeUpdate {
        nodes: vec![(WINDOW, window), (BUTTON, button), (LABEL, label)],
        tree: Some(Tree::new(WINDOW)),
        tree_id: TreeId::ROOT,
        focus: WINDOW,
    }
}

/// An accessible object on the accessibility bus.
#[derive(Clone, Debug)]
struct Object {
    bus: String,
    path: OwnedObjectPath,
}

impl Object {
    fn call<B>(
        &self,
        a11y: &Connection,
        interface: &str,
        method: &str,
        body: &B,
    ) -> zbus::Result<zbus::Message>
    where
        B: serde::ser::Serialize + zbus::zvariant::DynamicType,
    {
        a11y.call_method(
            Some(self.bus.as_str()),
            &self.path,
            Some(interface),
            method,
            body,
        )
    }

    fn children(&self, a11y: &Connection) -> zbus::Result<Vec<Object>> {
        let reply = self.call(a11y, "org.a11y.atspi.Accessible", "GetChildren", &())?;
        let children: Vec<(String, OwnedObjectPath)> = reply.body().deserialize()?;
        Ok(children
            .into_iter()
            .map(|(bus, path)| Object { bus, path })
            .collect())
    }

    fn name(&self, a11y: &Connection) -> zbus::Result<String> {
        let reply = self.call(
            a11y,
            "org.freedesktop.DBus.Properties",
            "Get",
            &("org.a11y.atspi.Accessible", "Name"),
        )?;
        let value: OwnedValue = reply.body().deserialize()?;
        Ok(String::try_from(value).unwrap_or_default())
    }

    fn role(&self, a11y: &Connection) -> zbus::Result<u32> {
        let reply = self.call(a11y, "org.a11y.atspi.Accessible", "GetRole", &())?;
        reply.body().deserialize()
    }
}

/// A main loop on this thread for the tasks the bridge posts.
struct MainLoop {
    tasks: mpsc::Receiver<Box<dyn FnOnce() + Send>>,
}

impl MainLoop {
    /// Runs posted tasks until `done` or the timeout.
    fn run_until(&self, what: &str, mut done: impl FnMut() -> bool) {
        let start = Instant::now();
        while !done() {
            assert!(start.elapsed() < TIMEOUT, "timed out waiting for {what}");
            if let Ok(task) = self.tasks.recv_timeout(Duration::from_millis(50)) {
                task();
            }
        }
    }
}

/// Polls the accessibility bus (running the main loop between tries) until `find` succeeds.
fn poll<T>(main_loop: &MainLoop, what: &str, mut find: impl FnMut() -> Option<T>) -> T {
    let mut found = None;
    main_loop.run_until(what, || {
        found = find();
        found.is_some()
    });
    found.unwrap()
}

#[test]
fn the_tree_is_published_over_atspi_and_do_action_reaches_gpui() {
    let session = Connection::session().expect("a session bus (run under dbus-run-session)");
    // What a screen reader does: turn accessibility on (activates at-spi-bus-launcher).
    session
        .call_method(
            Some("org.a11y.Bus"),
            "/org/a11y/bus",
            Some("org.freedesktop.DBus.Properties"),
            "Set",
            &("org.a11y.Status", "IsEnabled", Value::from(true)),
        )
        .expect("org.a11y.Status.IsEnabled (at-spi2-core installed?)");
    let address: String = session
        .call_method(
            Some("org.a11y.Bus"),
            "/org/a11y/bus",
            Some("org.a11y.Bus"),
            "GetAddress",
            &(),
        )
        .and_then(|reply| reply.body().deserialize())
        .expect("the accessibility bus address");
    let a11y = zbus::blocking::connection::Builder::address(address.as_str())
        .and_then(|builder| builder.build())
        .expect("the accessibility bus");

    let main_thread = thread::current().id();
    let (task_tx, tasks) = mpsc::channel::<Box<dyn FnOnce() + Send>>();
    let task_tx = Mutex::new(task_tx);
    let post: Post = Arc::new(move |task| {
        let _ = task_tx.lock().unwrap().send(task);
    });
    let on_main_thread: OnMainThread = Arc::new(|event: MainThreadEvent| {
        ADAPTER.with(|adapter| {
            if let Some(adapter) = adapter.borrow_mut().as_mut() {
                adapter.event(event);
            }
        });
    });
    let (action_tx, actions) = mpsc::channel::<(ActionRequest, ThreadId)>();
    let action_tx = Mutex::new(action_tx);
    let activations = Arc::new(Mutex::new(Vec::<ThreadId>::new()));
    let seen_activations = Arc::clone(&activations);
    let callbacks = A11yCallbacks {
        activation: Box::new(move || {
            seen_activations
                .lock()
                .unwrap()
                .push(thread::current().id());
            Some(tree())
        }),
        action: Box::new(move |request| {
            let _ = action_tx
                .lock()
                .unwrap()
                .send((request, thread::current().id()));
        }),
        deactivation: Box::new(|| {}),
    };
    let adapter = UnixAdapter::new(callbacks, post, on_main_thread, (1920, 1080));
    ADAPTER.with(|slot| *slot.borrow_mut() = Some(adapter));
    let main_loop = MainLoop { tasks };

    main_loop.run_until("the initial tree", || {
        ADAPTER.with(|a| a.borrow().as_ref().is_some_and(UnixAdapter::initialized))
    });
    assert_eq!(
        *activations.lock().unwrap(),
        [main_thread],
        "GPUI's activation callback ran once, on the main thread"
    );

    let registry = Object {
        bus: "org.a11y.atspi.Registry".into(),
        path: OwnedObjectPath::try_from("/org/a11y/atspi/accessible/root").unwrap(),
    };
    let window = poll(&main_loop, "the window on the accessibility bus", || {
        let apps = registry.children(&a11y).ok()?;
        apps.iter()
            .filter_map(|app| app.children(&a11y).ok())
            .flatten()
            .find(|window| window.name(&a11y).is_ok_and(|name| name == "Long list"))
    });
    assert_eq!(window.role(&a11y).unwrap(), ROLE_FRAME);
    let children = window.children(&a11y).unwrap();
    let read: Vec<(String, u32)> = children
        .iter()
        .map(|child| (child.name(&a11y).unwrap(), child.role(&a11y).unwrap()))
        .collect();
    assert_eq!(
        read,
        [
            ("Filter".to_owned(), ROLE_PUSH_BUTTON),
            ("Row 1".to_owned(), ROLE_LABEL),
        ]
    );

    let button = &children[0];
    let done: bool = button
        .call(&a11y, "org.a11y.atspi.Action", "DoAction", &(0i32))
        .and_then(|reply| reply.body().deserialize())
        .expect("Action.DoAction");
    assert!(done);
    let mut action = None;
    main_loop.run_until("the action callback", || {
        action = actions.try_recv().ok();
        action.is_some()
    });
    let (request, thread) = action.unwrap();
    assert_eq!(request.action, Action::Click);
    assert_eq!(request.target_node, BUTTON);
    assert_eq!(
        thread, main_thread,
        "GPUI's action callback ran on the main thread"
    );
}
