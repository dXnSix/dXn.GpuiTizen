use std::sync::mpsc;
use std::thread::{self, ThreadId};

use gpui::accesskit::{self, Node, NodeId, Role, Tree, TreeId};

use super::*;

/// A tree of one window node labelled `label`.
fn tree(label: &str) -> TreeUpdate {
    let mut window = Node::new(Role::Window);
    window.set_label(label.to_owned());
    TreeUpdate {
        nodes: vec![(NodeId(0), window)],
        tree: Some(Tree::new(NodeId(0))),
        tree_id: TreeId::ROOT,
        focus: NodeId(0),
    }
}

fn label(update: &TreeUpdate) -> Option<String> {
    update.nodes[0].1.label().map(str::to_owned)
}

/// What GPUI's callbacks saw, and on which thread.
#[derive(Debug, PartialEq)]
enum Seen {
    Activation(ThreadId),
    Action(accesskit::Action, NodeId, ThreadId),
    Deactivation(ThreadId),
}

/// A main loop: tasks posted from any thread, run when the test (the main thread) drains them.
struct Fixture {
    tasks: mpsc::Receiver<Box<dyn FnOnce() + Send>>,
    seen: mpsc::Receiver<Seen>,
    events: mpsc::Receiver<MainThreadEvent>,
}

type Handlers = (Activation, Action, Deactivation);

impl Fixture {
    fn new(initial: Option<TreeUpdate>) -> (Self, Handlers) {
        let (task_tx, tasks) = mpsc::channel::<Box<dyn FnOnce() + Send>>();
        let (seen_tx, seen) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        let (a, b, c) = (seen_tx.clone(), seen_tx.clone(), seen_tx);
        let callbacks = A11yCallbacks {
            activation: Box::new(move || {
                a.send(Seen::Activation(thread::current().id())).unwrap();
                initial.clone()
            }),
            action: Box::new(move |request| {
                b.send(Seen::Action(
                    request.action,
                    request.target_node,
                    thread::current().id(),
                ))
                .unwrap();
            }),
            deactivation: Box::new(move || {
                c.send(Seen::Deactivation(thread::current().id())).unwrap();
            }),
        };
        let task_tx = Mutex::new(task_tx);
        let post: Post = Arc::new(move |task| task_tx.lock().unwrap().send(task).unwrap());
        let event_tx = Mutex::new(event_tx);
        let on_main: OnMainThread =
            Arc::new(move |event| event_tx.lock().unwrap().send(event).unwrap());
        (
            Self {
                tasks,
                seen,
                events,
            },
            handlers(callbacks, post, on_main),
        )
    }

    /// Runs the posted tasks on this thread; returns how many ran.
    fn run_main_loop(&self) -> usize {
        self.tasks.try_iter().map(|task| task()).count()
    }

    fn seen(&self) -> Vec<Seen> {
        self.seen.try_iter().collect()
    }

    fn events(&self) -> Vec<MainThreadEvent> {
        self.events.try_iter().collect()
    }
}

fn request(action: accesskit::Action, node: u64) -> ActionRequest {
    ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: NodeId(node),
        data: None,
    }
}

#[test]
fn activation_runs_gpui_on_the_main_thread_and_hands_over_the_initial_tree() {
    let (fixture, (mut activation, _, _)) = Fixture::new(Some(tree("Long list")));
    let main = thread::current().id();
    let answer = thread::spawn(move || activation.request_initial_tree())
        .join()
        .unwrap();
    assert!(answer.is_none(), "AccessKit waits for the tree");
    assert!(
        fixture.seen().is_empty(),
        "GPUI is not called on AccessKit's thread"
    );
    assert_eq!(fixture.run_main_loop(), 1);
    assert_eq!(fixture.seen(), [Seen::Activation(main)]);
    let events = fixture.events();
    assert!(
        matches!(&events[..], [MainThreadEvent::InitialTree(t)] if label(t).as_deref() == Some("Long list")),
        "{events:?}"
    );
}

#[test]
fn no_initial_tree_means_no_event() {
    let (fixture, (mut activation, _, _)) = Fixture::new(None);
    activation.request_initial_tree();
    fixture.run_main_loop();
    assert_eq!(fixture.seen().len(), 1);
    assert!(fixture.events().is_empty());
}

#[test]
fn actions_reach_gpui_on_the_main_thread_in_order() {
    let (fixture, (_, mut action, _)) = Fixture::new(None);
    let main = thread::current().id();
    thread::spawn(move || {
        action.do_action(request(accesskit::Action::Click, 7));
        action.do_action(request(accesskit::Action::Focus, 8));
    })
    .join()
    .unwrap();
    assert!(fixture.seen().is_empty());
    assert_eq!(fixture.run_main_loop(), 2);
    assert_eq!(
        fixture.seen(),
        [
            Seen::Action(accesskit::Action::Click, NodeId(7), main),
            Seen::Action(accesskit::Action::Focus, NodeId(8), main),
        ]
    );
}

#[test]
fn deactivation_reaches_gpui_then_stops_updates() {
    let (fixture, (_, _, mut deactivation)) = Fixture::new(None);
    let main = thread::current().id();
    thread::spawn(move || deactivation.deactivate_accessibility())
        .join()
        .unwrap();
    fixture.run_main_loop();
    assert_eq!(fixture.seen(), [Seen::Deactivation(main)]);
    assert!(matches!(
        &fixture.events()[..],
        [MainThreadEvent::Deactivated]
    ));
}

#[test]
fn updates_pass_only_between_the_initial_tree_and_deactivation() {
    let mut state = TreeState::default();
    assert!(!state.initialized());
    assert!(
        state.update(tree("early")).is_none(),
        "before the initial tree"
    );
    let initial = state.event(MainThreadEvent::InitialTree(tree("initial")));
    assert_eq!(initial.as_ref().and_then(label).as_deref(), Some("initial"));
    assert!(state.initialized());
    let update = state.update(tree("update"));
    assert_eq!(update.as_ref().and_then(label).as_deref(), Some("update"));
    assert!(state.event(MainThreadEvent::Deactivated).is_none());
    assert!(!state.initialized());
    assert!(state.update(tree("late")).is_none(), "after deactivation");
    assert!(
        state
            .event(MainThreadEvent::InitialTree(tree("again")))
            .is_some()
    );
    assert!(state.update(tree("again")).is_some(), "reactivated");
}

#[test]
fn the_window_is_full_screen_at_the_origin_in_physical_pixels() {
    assert_eq!(
        window_bounds((1920, 1080)),
        Rect::new(0.0, 0.0, 1920.0, 1080.0)
    );
    assert_eq!(
        window_bounds((3840, 2160)),
        Rect::new(0.0, 0.0, 3840.0, 2160.0)
    );
}
