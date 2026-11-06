//! The agent tree: attention, collapse, depth, size, pictures.
//!
//! Pattern: model tests on the pure forest, then layouts and pixels of the same view a window draws.

mod common;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use common::{Input, draw, layout, serial};
use fs_components_widgets::SenderKind;
use fs_components_widgets::agent_tree::{
    AgentEnd, AgentNode, AgentState, AgentTree, Attention, Forest, Outcome,
};

fn ai(key: &str) -> AgentNode {
    AgentNode::new(key, format!("Agent {key}")).of_kind(SenderKind::Ai)
}

fn sample() -> Vec<AgentNode> {
    vec![
        ai("root")
            .state(AgentState::Working)
            .summary("Plan the release"),
        ai("a")
            .under("root")
            .state(AgentState::Working)
            .summary("Collect the changes"),
        ai("a1")
            .under("a")
            .settled(AgentEnd::Failed)
            .summary("The tests broke"),
        ai("b")
            .under("root")
            .state(AgentState::Question)
            .summary("May I delete the old build?"),
        ai("c")
            .under("root")
            .settled(AgentEnd::Delivered)
            .summary("Notes written"),
        AgentNode::new("tool", "indexer")
            .under("c")
            .of_kind(SenderKind::Software)
            .state(AgentState::Working),
        AgentNode::new("who", "Someone")
            .under("c")
            .settled(AgentEnd::Cancelled),
    ]
}

fn forest(nodes: Vec<AgentNode>) -> Forest {
    let mut f = Forest::new();
    for n in nodes {
        f.apply(n);
    }
    f
}

#[test]
fn only_a_question_a_delivery_and_a_failure_ask_for_attention() {
    assert_eq!(ai("x").state(AgentState::Working).attention(), None);
    assert_eq!(ai("x").state(AgentState::Forming).attention(), None);
    assert_eq!(
        ai("x").state(AgentState::Question).attention(),
        Some(Attention::Question)
    );
    assert_eq!(
        ai("x").settled(AgentEnd::Failed).attention(),
        Some(Attention::Failure)
    );
    assert_eq!(
        ai("x").settled(AgentEnd::Delivered).attention(),
        Some(Attention::Delivery)
    );
    assert_eq!(
        ai("x")
            .settled(AgentEnd::Delivered)
            .acknowledged(true)
            .attention(),
        None
    );
    assert_eq!(ai("x").settled(AgentEnd::Cancelled).attention(), None);
    assert_eq!(ai("x").settled(AgentEnd::Open).attention(), None);
}

#[test]
fn an_update_replaces_by_key_and_keeps_the_place() {
    let mut f = forest(sample());
    assert!(!f.apply(ai("a").under("root").state(AgentState::Question)));
    assert_eq!(f.len(), 7);
    let rows = f.rows(&HashSet::new());
    let keys: Vec<_> = rows.iter().map(|r| r.node.key.as_str()).collect();
    assert_eq!(keys, ["root", "a", "a1", "b", "c", "tool", "who"]);
    assert_eq!(rows[1].node.state, AgentState::Question);
}

#[test]
fn a_closed_parent_hides_its_children_but_not_what_is_wrong_below() {
    let f = forest(sample());
    let closed: HashSet<String> = ["root".to_string()].into();
    let rows = f.rows(&closed);
    assert_eq!(rows.len(), 1);
    let c = rows[0].below;
    assert_eq!((c.questions, c.failures, c.deliveries), (1, 1, 1));
    assert!(rows[0].collapsed && rows[0].has_children);
    // closing only "a" hides a1 and says its failure
    let rows = f.rows(&["a".to_string()].into());
    assert!(rows.iter().all(|r| r.node.key != "a1"));
    let a = rows.iter().find(|r| r.node.key == "a").unwrap();
    assert_eq!(a.below.failures, 1);
    // an open parent carries no badge of its own, its children show theirs
    assert!(!f.rows(&HashSet::new())[0].collapsed);
}

#[test]
fn orphans_and_cycles_are_shown_not_dropped() {
    let f = forest(vec![
        ai("lost").under("nobody"),
        ai("x").under("y"),
        ai("y").under("x"),
        ai("self").under("self"),
        ai("dup"),
    ]);
    let rows = f.rows(&HashSet::new());
    assert_eq!(rows.len(), 5, "agents were dropped");
    assert!(rows.iter().all(|r| r.depth <= 1));
}

#[test]
fn a_chain_ten_thousand_deep_and_many_nodes_do_not_crash() {
    let mut f = Forest::new();
    f.apply(ai("n0"));
    for i in 1..20_000 {
        f.apply(ai(&format!("n{i}")).under(format!("n{}", i - 1)));
    }
    f.apply(ai("n19999").settled(AgentEnd::Failed).under("n19998"));
    let rows = f.rows(&HashSet::new());
    assert_eq!(rows.len(), 20_000);
    assert_eq!(rows[19_999].depth, 19_999);
    assert_eq!(rows[0].below.failures, 1);
    let rows = f.rows(&["n0".to_string()].into());
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].below.failures, 1,
        "the failure hid behind the collapse"
    );
}

#[test]
fn thousands_of_agents_build_only_the_rows_in_view() {
    let _g = serial();
    let tree = AgentTree::new().id("big");
    for i in 0..5_000 {
        let parent = if i == 0 {
            None
        } else {
            Some(format!("k{}", (i - 1) / 3))
        };
        let mut n = ai(&format!("k{i}"));
        n.parent = parent;
        tree.apply(n);
    }
    let laid = layout(tree.build(600.0, 400.0), 600.0, 400.0);
    assert!(laid.query_by_id("big-row-k0").is_some());
    assert!(
        laid.query_by_id("big-row-k4000").is_none(),
        "rows far out of view were built"
    );
    assert!(laid.query_by_id("big-head").is_some());
}

#[test]
fn a_closed_branch_says_something_below_needs_attention() {
    let _g = serial();
    let tree = AgentTree::new().id("t");
    for n in sample() {
        tree.apply(n);
    }
    let open = layout(tree.build(640.0, 420.0), 640.0, 420.0);
    assert!(open.query_by_id("t-below-root").is_none());
    let img_open = draw(tree.build(640.0, 420.0), 640, 420, "agent-tree-open.png");
    tree.toggle("root");
    let closed = layout(tree.build(640.0, 420.0), 640.0, 420.0);
    assert!(
        closed.query_by_id("t-below-root").is_some(),
        "no hint on the closed parent"
    );
    assert!(closed.query_by_id("t-row-a1").is_none());
    let img_closed = draw(tree.build(640.0, 420.0), 640, 420, "agent-tree-closed.png");
    assert_ne!(img_open, img_closed);
    // the line above the tree counts everything, closed or not
    assert_eq!(tree.counts().total(), 3);
}

#[test]
fn clicks_toggle_and_select_and_report_only() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let tree = AgentTree::new()
        .id("k")
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    for n in sample() {
        tree.apply(n);
    }
    let mut laid = layout(tree.build(640.0, 420.0), 640.0, 420.0);
    let mut input = Input::new();
    input.click(&mut laid, "k-main-b");
    input.click(&mut laid, "k-toggle-a");
    let seen = got.lock().unwrap().clone();
    assert_eq!(seen[0], Outcome::Selected { key: "b".into() });
    assert_eq!(
        seen[1],
        Outcome::Toggled {
            key: "a".into(),
            open: false
        }
    );
    // nothing was decided: the question is still a question
    assert_eq!(tree.get("b").unwrap().state, AgentState::Question);
}

#[test]
fn deep_rows_stay_inside_the_box() {
    let _g = serial();
    let tree = AgentTree::new().id("d");
    tree.apply(ai("n0"));
    for i in 1..40 {
        tree.apply(
            ai(&format!("n{i}"))
                .under(format!("n{}", i - 1))
                .summary("a long line of summary text that must be clipped, not run out"),
        );
    }
    let img = draw(tree.build(420.0, 300.0), 420, 300, "agent-tree-deep.png");
    let (_, right) = common::extent_x(&img, 0, 300).expect("ink");
    assert!(right < 420, "ran out of the box: {right}");
}
