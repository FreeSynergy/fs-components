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

use super::chrome::{Action, Weight, button, header, locked_button, message_bar};
use super::form::{FormEnv, FormSlots, form_body};
use super::redraw::redraw_worker;
use crate::commands::CommandPlan;
use crate::screen::Screen;
use crate::session::Session;

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
    pub(super) env: Arc<FormEnv>,
    pub(super) live: Live,
    pub(super) slots: Arc<Mutex<FormSlots>>,
    pub(super) position: ListPosition,
    pub(super) about: Arc<Mutex<Option<AboutInfo>>>,
}

pub(super) fn lock(s: &Mutex<Session>) -> std::sync::MutexGuard<'_, Session> {
    s.lock().unwrap_or_else(PoisonError::into_inner)
}

impl AdminSurface {
    /// A surface over `session`, in `look`, with the words of `say`. The messages of the session draw it again.
    pub fn new(
        session: Session,
        look: Look,
        say: Arc<dyn Say>,
        suggest: Option<Arc<dyn Suggest>>,
    ) -> Self {
        let live = Live::new();
        session.messages().subscribe(redraw_worker(
            move || live.refresh_now(),
            std::time::Duration::from_millis(crate::messages::SUCCESS_MS + 100),
        ));
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

    /// Whether a form is open with text that was not sent.
    pub fn has_unsaved(&self) -> bool {
        lock(&self.env.session).has_unsaved_form()
    }

    /// Tell the person that leaving would lose what is typed (ADR 0065 point 5: not silent).
    pub fn warn_unsaved(&self) {
        let board = Arc::clone(lock(&self.env.session).messages());
        board.solve("unsaved"); // one warning, however often it is asked
        board.raise(crate::MessageKind::Error, "unsaved");
        self.set_about(None); // the page with the form is the one to show
    }

    /// Quit was asked and there was no window to close: tell the person (message code `quit-failed`).
    pub fn warn_quit_failed(&self) {
        let board = Arc::clone(lock(&self.env.session).messages());
        board.solve("quit-failed");
        board.raise(crate::MessageKind::Error, "quit-failed");
    }

    /// Draw the page again now.
    pub fn refresh(&self) {
        self.live.refresh_now();
    }

    pub(super) fn act(&self, f: impl Fn(&mut Session) + Send + Sync + 'static) -> Action {
        let (session, live) = (Arc::clone(&self.env.session), self.live);
        Arc::new(move || {
            f(&mut lock(&session));
            live.refresh_now();
        })
    }

    pub(super) fn command_buttons(&self, plans: &[&CommandPlan], session: &Session) -> Vec<Div> {
        plans
            .iter()
            .map(|p| {
                let name = p.name.clone();
                let weight = match (p.kind, p.confirm) {
                    (_, true) => Weight::Danger,
                    (fs_types::CommandKind::Create, _) => Weight::Primary,
                    _ => Weight::Plain,
                };
                if !session.is_supported(&p.name) {
                    // locked, with the word "coming soon"; it unlocks when the core reports the command
                    return div()
                        .flex_row()
                        .items_center()
                        .gap_px(self.env.look.style.spacing.xs)
                        .child(locked_button(
                            &self.env.say.say(&p.label),
                            &format!("cmd-{}", p.name),
                        ))
                        .child(
                            div()
                                .id(format!("soon-{}", p.name))
                                .child(self.env.look.text(
                                    self.env.say.say("command.soon"),
                                    self.env.look.style.text.caption,
                                    self.env.look.palette().text_weak,
                                )),
                        );
                }
                let go = self.act(move |s| {
                    s.begin(&name);
                });
                div().child(button(
                    &self.env.say.say(&p.label),
                    weight,
                    &format!("cmd-{}", p.name),
                    go,
                ))
            })
            .collect()
    }

    fn page(&self, width: f32, height: f32) -> Div {
        let (look, say) = (&self.env.look, &*self.env.say);
        let s = look.style;
        if let Some(about) = self
            .about
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
        {
            return self.about_page(&about);
        }
        let screen = lock(&self.env.session).screen().clone();
        let (title, actions, body): (String, Vec<Div>, Div) = match &screen {
            Screen::List => self.list_screen(width, height),
            Screen::Detail(_) => self.detail_screen(),
            Screen::Command { name, .. } => {
                let title = lock(&self.env.session)
                    .command(name)
                    .map_or(String::new(), |p| say.say(&p.label));
                let save = button(
                    &say.say("action.save"),
                    Weight::Primary,
                    "save",
                    self.act(|s| {
                        s.submit();
                    }),
                );
                let cancel = button(
                    &say.say("action.cancel"),
                    Weight::Plain,
                    "cancel",
                    self.act(Session::cancel),
                );
                (
                    title,
                    vec![div().child(save), div().child(cancel)],
                    form_body(&self.env, &self.slots),
                )
            }
            Screen::Confirm { name, .. } => self.confirm_screen(name),
            Screen::Result(name) => self.result_screen(name),
        };
        // The guard is let go before the messages are read: reading can tell the listeners, and a listener draws.
        let board = Arc::clone(lock(&self.env.session).messages());
        let messages = board.current();
        let session = Arc::clone(&self.env.session);
        let live = self.live;
        let dismiss: Arc<dyn Fn(u64) + Send + Sync> = Arc::new(move |id| {
            lock(&session).messages().dismiss(id);
            live.refresh_now();
        });
        let session = Arc::clone(&self.env.session);
        // Connecting again can take seconds (the program starts a server): it runs on a worker, with no lock and not on
        // the interface thread, so the window does not freeze; the messages tell the window when it is done.
        let retry: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
            let Some(job) = lock(&session).retry_begin() else {
                return; // one try is running already
            };
            let later = Arc::clone(&session);
            let worker = std::thread::Builder::new()
                .name("fs-admin-reconnect".into())
                .spawn(move || {
                    // `run` never panics out and is done with no lock held; only its answers go in under the lock
                    let outcome = job.run();
                    lock(&later).retry_finish(outcome);
                    live.refresh();
                });
            if worker.is_err() {
                // No thread to be had: the failure is told, never silence.
                lock(&session).retry_finish(crate::RetryOutcome::failed(
                    crate::BackendError::Unavailable("no thread to connect again".into()),
                ));
                live.refresh_now();
            }
        });
        div()
            .class("fs-admin-page")
            .flex_col()
            .gap_px(s.spacing.m)
            .padding_x_px(s.spacing.xl)
            .padding_y_px(s.spacing.xl)
            .w(width)
            .h(height)
            .bg(Look::color(look.palette().ground))
            .child(header(look, &title, actions))
            .child(message_bar(look, say, &messages, &dismiss, &retry))
            .child(body)
    }

    pub(super) fn clone_handles(&self) -> Handles {
        Handles {
            about: Arc::clone(&self.about),
            live: self.live,
        }
    }
}

/// The parts of the surface a button needs to close the About page.
pub(super) struct Handles {
    about: Arc<Mutex<Option<AboutInfo>>>,
    live: Live,
}

impl Handles {
    pub(super) fn set_about(&self, about: Option<AboutInfo>) {
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
