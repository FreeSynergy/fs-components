//! Support for the chat tests: a source the test can change, a tree driven like a window drives it,
//! and a picture.
//!
//! Pattern: Facade over Blinc's tree and router (as in `people_common`), Factory for the source.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::reactive::global_graph;
use blinc_core::{BlincContextState, HookState};
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::event_router::{EventRouter, MouseButton};
use blinc_layout::prelude::RenderTree;
use fs_components_render::{Canvas, draw_to_png};
use fs_components_widgets::agent_tree::AgentNode;
use fs_components_widgets::message_list::{ChatMessage, MessageSource, SourceError};
use fs_components_widgets::{Block, Inline, Sender, SenderKind};
use fs_components_workbenches::llm_chat::{ChatSource, LlmChat, Pending, StopControl, StopState};

static ONE: Mutex<()> = Mutex::new(());

/// A conversation the test can change at any time.
#[derive(Default)]
pub struct Chat {
    pub messages: Mutex<Vec<ChatMessage>>,
    pub agents: Mutex<Vec<AgentNode>>,
    pub questions: Mutex<Vec<Pending>>,
    pub stop: Mutex<Option<StopControl>>,
}

impl MessageSource for Chat {
    fn count(&self) -> usize {
        self.messages.lock().unwrap().len()
    }

    fn messages(&self, first: usize, limit: usize) -> Result<Vec<ChatMessage>, SourceError> {
        Ok(self
            .messages
            .lock()
            .unwrap()
            .iter()
            .skip(first)
            .take(limit)
            .cloned()
            .collect())
    }
}

impl ChatSource for Chat {
    fn key(&self) -> String {
        "room".into()
    }

    fn title(&self) -> String {
        "Release planning".into()
    }

    fn agents(&self) -> Vec<AgentNode> {
        self.agents.lock().unwrap().clone()
    }

    fn questions(&self) -> Vec<Pending> {
        self.questions.lock().unwrap().clone()
    }

    fn stop_control(&self) -> Option<StopControl> {
        self.stop.lock().unwrap().clone()
    }

    fn usage(&self) -> Option<String> {
        Some("1.2k tokens, local".into())
    }
}

/// A message with one paragraph.
pub fn say(key: &str, who: &str, kind: SenderKind, text: &str) -> ChatMessage {
    let sender = Sender::new(who, who).of_kind(kind);
    ChatMessage::new(key, sender, vec![Block::Paragraph(Inline::plain(text))])
}

pub fn stop(state: StopState) -> Option<StopControl> {
    Some(StopControl::new("Stop all agents", state))
}

/// The chat laid out and driven like a window drives it.
pub struct Driven {
    pub tree: RenderTree,
    router: EventRouter,
    size: (f32, f32),
    chat: LlmChat,
    _one: MutexGuard<'static, ()>,
}

fn tree_of(chat: &LlmChat, (w, h): (f32, f32)) -> RenderTree {
    let bundle = fs_components_render::bundle(&fs_components_workbenches::llm_chat::styles());
    blinc_theme::ThemeState::init(bundle.clone(), blinc_theme::ColorScheme::Light);
    if !BlincContextState::is_initialized() {
        BlincContextState::init(
            global_graph(),
            Arc::new(Mutex::new(HookState::new())),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        );
    }
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
    blinc_layout::init_text_measurer();
    let mut tree = RenderTree::from_element(&chat.build(w, h));
    let vars = blinc_theme::ThemeState::get().to_css_variable_map();
    tree.set_stylesheet(
        Stylesheet::parse_with_variables(&bundle.css_sources.join("\n"), &vars).expect("css"),
    );
    tree.apply_stylesheet_layout_overrides();
    tree.apply_stylesheet_base_styles();
    tree.compute_layout(w, h);
    tree
}

impl Driven {
    pub fn new(chat: &LlmChat, w: f32, h: f32) -> Self {
        let one = ONE.lock().unwrap_or_else(PoisonError::into_inner);
        Self {
            tree: tree_of(chat, (w, h)),
            router: EventRouter::new(),
            size: (w, h),
            chat: chat.clone(),
            _one: one,
        }
    }

    /// A window frame: what asked to be drawn again is drawn again.
    pub fn frame(&mut self) {
        let updates = self.tree.take_pending_property_updates();
        self.tree.apply_partial_property_updates(updates);
        self.tree.process_pending_subtree_rebuilds_routed(None);
        self.tree.compute_layout(self.size.0, self.size.1);
    }

    /// Everything is built again, as after a request for a new build.
    pub fn rebuild(&mut self) {
        self.tree = tree_of(&self.chat, self.size);
    }

    pub fn has(&self, id: &str) -> bool {
        self.tree.query_by_id(id).is_some()
    }

    pub fn click(&mut self, id: &str) {
        let node = self
            .tree
            .query_by_id(id)
            .unwrap_or_else(|| panic!("no element {id}"));
        let b = self.tree.get_absolute_bounds(node).expect("bounds");
        let (x, y) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
        for (n, e) in self.router.on_mouse_move(&self.tree, x, y) {
            self.tree.dispatch_event(n, e, x, y);
        }
        for (n, e) in self
            .router
            .on_mouse_down(&self.tree, x, y, MouseButton::Left)
        {
            self.tree.dispatch_event(n, e, x, y);
        }
        for (n, e) in self.router.on_mouse_up(&self.tree, x, y, MouseButton::Left) {
            self.tree.dispatch_event(n, e, x, y);
        }
    }
}

/// Draw the chat to a picture (kept in `FS_COMP_SHOTS` when set).
pub fn shot(chat: &LlmChat, w: u32, h: u32, name: &str) -> image::RgbaImage {
    let dir = std::env::temp_dir().join(format!("fs-llm-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path: PathBuf = dir.join(name);
    let canvas = Canvas::new(w, h, fs_components_workbenches::llm_chat::styles());
    let chat = chat.clone();
    draw_to_png(&canvas, move |w, h| chat.build(w, h), || {}, &path).expect("picture");
    let img = image::open(&path).unwrap().to_rgba8();
    if let Some(keep) = std::env::var_os("FS_COMP_SHOTS") {
        std::fs::create_dir_all(&keep).unwrap();
        std::fs::copy(&path, PathBuf::from(keep).join(name)).unwrap();
    }
    let _ = std::fs::remove_dir_all(&dir);
    img
}
