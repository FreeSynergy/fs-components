//! The surface the shell shows: one screen of the session at a time.
//!
//! Pattern: Adapter (session state to elements, `Surface` of `fs-components-app`) + Observer (the session's messages
//! and every transition draw the page again). The skeleton is fixed, as ADR 0065 asks: title and actions on top, the
//! messages of the screen, then the body.

use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use fs_components_app::Surface;
use fs_components_fields::{Look, Say, Suggest};
use fs_components_widgets::list_view::ListPosition;
use fs_components_widgets::live::Live;
use fs_components_widgets::{Element, styles};

use super::chrome::{Action, Weight, button, header, message_bar, panel};
use super::detail::fields_panel;
use super::form::{FormEnv, FormSlots, form_body};
use super::list::{ListEvents, list_body};
use crate::commands::{CommandPlan, Placement};
use crate::detail::entries;
use crate::session::{Screen, Session};

/// What the About page says (ADR 0065 point 2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AboutInfo {
    /// `FreeSynergy.<Name>`.
    pub name: String,
    /// `v0.0.N`.
    pub version: String,
    /// Where it came from (source, catalog).
    pub origin: String,
    /// The licence.
    pub licence: String,
    /// The repository.
    pub repository: String,
}

/// The Admin frontend of one data type as a surface of the shell.
pub struct AdminSurface {
    env: Arc<FormEnv>,
    live: Live,
    slots: Arc<Mutex<FormSlots>>,
    position: ListPosition,
    about: Arc<Mutex<Option<AboutInfo>>>,
}

fn lock(s: &Mutex<Session>) -> std::sync::MutexGuard<'_, Session> {
    s.lock().unwrap_or_else(PoisonError::into_inner)
}

impl AdminSurface {
    /// A surface over `session`, in `look`, with the words of `say`. The messages of the session draw it again.
    pub fn new(session: Session, look: Look, say: Arc<dyn Say>, suggest: Option<Arc<dyn Suggest>>) -> Self {
        let live = Live::new();
        session.messages().subscribe(move || live.refresh());
        Self {
            env: Arc::new(FormEnv {
                session: Arc::new(Mutex::new(session)),
                look,
                say,
                suggest,
                findings: Live::new(),
                page: live,
            }),
            live,
            slots: Arc::new(Mutex::new(FormSlots::default())),
            position: ListPosition::default(),
            about: Arc::new(Mutex::new(None)),
        }
    }

    /// The session, to drive it from outside (menus, tests).
    pub fn session(&self) -> &Arc<Mutex<Session>> {
        &self.env.session
    }

    /// Show the About page (`None` goes back to the screen).
    pub fn set_about(&self, about: Option<AboutInfo>) {
        *self.about.lock().unwrap_or_else(PoisonError::into_inner) = about;
        self.live.refresh_now();
    }

    /// Draw the page again now.
    pub fn refresh(&self) {
        self.live.refresh_now();
    }

    fn act(&self, f: impl Fn(&mut Session) + Send + Sync + 'static) -> Action {
        let (session, live) = (Arc::clone(&self.env.session), self.live);
        Arc::new(move || {
            f(&mut lock(&session));
            live.refresh_now();
        })
    }

    fn command_buttons(&self, plans: &[&CommandPlan]) -> Vec<Div> {
        plans
            .iter()
            .map(|p| {
                let name = p.name.clone();
                let weight = match (p.kind, p.confirm) {
                    (_, true) => Weight::Danger,
                    (fs_types::CommandKind::Create, _) => Weight::Primary,
                    _ => Weight::Plain,
                };
                let go = self.act(move |s| {
                    s.begin(&name);
                });
                div().child(button(&self.env.say.say(&p.label), weight, &format!("cmd-{}", p.name), go))
            })
            .collect()
    }

    fn page(&self, width: f32, height: f32) -> Div {
        let (look, say) = (&self.env.look, &*self.env.say);
        let s = look.style;
        if let Some(about) = self.about.lock().unwrap_or_else(PoisonError::into_inner).clone() {
            return self.about_page(&about);
        }
        let screen = lock(&self.env.session).screen().clone();
        let (title, actions, body): (String, Vec<Div>, Div) = match &screen {
            Screen::List => self.list_screen(width, height),
            Screen::Detail(_) => self.detail_screen(),
            Screen::Command { name, .. } => {
                let title = lock(&self.env.session).command(name).map_or(String::new(), |p| say.say(&p.label));
                let save = button(&say.say("action.save"), Weight::Primary, "save", self.act(|s| { s.submit(); }));
                let cancel = button(&say.say("action.cancel"), Weight::Plain, "cancel", self.act(Session::cancel));
                (title, vec![div().child(save), div().child(cancel)], form_body(&self.env, &self.slots))
            }
            Screen::Confirm { name, .. } => self.confirm_screen(name),
            Screen::Result(name) => self.result_screen(name),
        };
        let messages = lock(&self.env.session).messages().current();
        let session = Arc::clone(&self.env.session);
        let live = self.live;
        let dismiss: Arc<dyn Fn(u64) + Send + Sync> = Arc::new(move |id| {
            lock(&session).messages().dismiss(id);
            live.refresh_now();
        });
        div()
            .class("fs-admin-page")
            .flex_col()
            .gap_px(s.spacing.m)
            .padding_x_px(s.spacing.xl).padding_y_px(s.spacing.xl)
            .w(width)
            .h(height)
            .bg(Look::color(look.palette().ground))
            .child(header(look, &title, actions))
            .child(message_bar(look, say, &messages, &dismiss))
            .child(body)
    }

    fn list_screen(&self, width: f32, height: f32) -> (String, Vec<Div>, Div) {
        let (look, say) = (&self.env.look, &*self.env.say);
        let s = look.style;
        let session = lock(&self.env.session);
        let title = say.say(&session.description().label);
        let actions = self.command_buttons(&session.commands(Placement::OnList));
        let (open, sort) = (self.opener(), self.sorter());
        let inner_w = (width - 2.0 * s.spacing.xl).max(0.0);
        let inner_h = (height - 2.0 * s.spacing.xl - s.heights.control - 3.0 * s.spacing.m).max(0.0);
        let body = div().flex_col().gap_px(s.spacing.s).w(inner_w).child(list_body(
            look,
            say,
            session.list(),
            &self.position,
            (inner_w, inner_h),
            &ListEvents { open, sort },
        ));
        (title, actions, body)
    }

    fn opener(&self) -> Arc<dyn Fn(&str) + Send + Sync> {
        let (session, live) = (Arc::clone(&self.env.session), self.live);
        Arc::new(move |id| {
            lock(&session).open_detail(id);
            live.refresh_now();
        })
    }

    fn sorter(&self) -> Arc<dyn Fn(usize) + Send + Sync> {
        let (session, live) = (Arc::clone(&self.env.session), self.live);
        Arc::new(move |c| {
            lock(&session).list_mut().cycle_sort(c);
            live.refresh_now();
        })
    }

    fn detail_screen(&self) -> (String, Vec<Div>, Div) {
        let (look, say) = (&self.env.look, &*self.env.say);
        let session = lock(&self.env.session);
        let title = session
            .detail()
            .first()
            .and_then(|e| e.values.first().cloned())
            .unwrap_or_else(|| say.say(&session.description().label));
        let mut actions = self.command_buttons(&session.commands(Placement::OnRecord));
        actions.push(div().child(button(&say.say("action.back"), Weight::Plain, "back", self.act(Session::open_list))));
        (title, actions, fields_panel(look, say, &session.detail()))
    }

    fn confirm_screen(&self, name: &str) -> (String, Vec<Div>, Div) {
        let (look, say) = (&self.env.look, &*self.env.say);
        let s = look.style;
        let session = lock(&self.env.session);
        let label = session.command(name).map_or(String::new(), |p| say.say(&p.label));
        let yes = button(&label, Weight::Danger, "confirm", self.act(|s| { s.submit(); }));
        let no = button(&say.say("action.cancel"), Weight::Plain, "cancel", self.act(Session::cancel));
        let body = panel(look)
            .child(look.text(say.say("confirm.sure"), s.text.title, look.palette().text))
            .child(div().flex_row().gap_px(s.spacing.s).child(yes).child(no));
        (label, Vec::new(), body)
    }

    fn result_screen(&self, name: &str) -> (String, Vec<Div>, Div) {
        let (look, say) = (&self.env.look, &*self.env.say);
        let session = lock(&self.env.session);
        let (title, fields) = session.command(name).map_or((String::new(), Vec::new()), |p| (say.say(&p.label), p.result.clone()));
        let values = session.result().map(|o| o.values.clone()).unwrap_or_default();
        let mut t = fs_types::TypeDescription::new("Result", "result", "result", "result");
        t.fields = fields;
        let record = crate::Record { id: String::new(), values };
        let back = button(&say.say("action.back"), Weight::Plain, "back", self.act(Session::cancel));
        (title, vec![div().child(back)], fields_panel(look, say, &entries(&t, &record)))
    }

    fn about_page(&self, about: &AboutInfo) -> Div {
        let (look, say) = (&self.env.look, &*self.env.say);
        let (s, pal) = (look.style, look.palette());
        let me = self.clone_handles();
        let back = button(&say.say("action.back"), Weight::Plain, "about-back", Arc::new(move || me.set_about(None)));
        let mut body = panel(look);
        for (key, value) in [
            ("about.version", &about.version),
            ("about.origin", &about.origin),
            ("about.licence", &about.licence),
            ("about.repository", &about.repository),
        ] {
            body = body.child(
                div()
                    .flex_row()
                    .gap_px(s.spacing.m)
                    .child(div().w(s.text.body * 12.0).flex_shrink_0().child(look.text(say.say(key), s.text.body, pal.text_weak)))
                    .child(look.text(value.clone(), s.text.body, pal.text)),
            );
        }
        div()
            .flex_col()
            .gap_px(s.spacing.m)
            .padding_x_px(s.spacing.xl).padding_y_px(s.spacing.xl)
            .w_full()
            .bg(Look::color(pal.ground))
            .child(header(look, &about.name, vec![div().child(back)]))
            .child(body)
    }

    fn clone_handles(&self) -> Handles {
        Handles { about: Arc::clone(&self.about), live: self.live }
    }
}

/// The parts of the surface a button needs to close the About page.
struct Handles {
    about: Arc<Mutex<Option<AboutInfo>>>,
    live: Live,
}

impl Handles {
    fn set_about(&self, about: Option<AboutInfo>) {
        *self.about.lock().unwrap_or_else(PoisonError::into_inner) = about;
        self.live.refresh_now();
    }
}

impl Surface for AdminSurface {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let this = Arc::new(Self {
            env: Arc::clone(&self.env),
            live: self.live,
            slots: Arc::clone(&self.slots),
            position: self.position.clone(),
            about: Arc::clone(&self.about),
        });
        Box::new(self.live.region("page", move || this.page(width, height)))
    }

    fn styles(&self) -> String {
        styles()
    }
}
