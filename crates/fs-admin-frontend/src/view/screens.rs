//! The pages of the surface other than the frame: list, detail, confirm, result, about.
//!
//! Pattern: Adapter (session state to elements; the click handlers it builds change the session), part of
//! [`AdminSurface`] (an `impl` split off to keep the file small).

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use fs_components_fields::Look;

use super::chrome::{Weight, button, header, panel};
use super::detail::fields_panel;
use super::list::{ListEvents, list_body};
use super::surface::{AboutInfo, AdminSurface, lock};
use crate::commands::Placement;
use crate::detail::entries;
use crate::session::Session;

impl AdminSurface {
    pub(super) fn list_screen(&self, width: f32, height: f32) -> (String, Vec<Div>, Div) {
        let (look, say) = (&self.env.look, &*self.env.say);
        let s = look.style;
        let session = lock(&self.env.session);
        let title = say.say(&session.description().label);
        let actions = self.command_buttons(&session.commands(Placement::OnList), &session);
        let (open, sort) = (self.opener(), self.sorter());
        let inner_w = (width - 2.0 * s.spacing.xl).max(0.0);
        let inner_h =
            (height - 2.0 * s.spacing.xl - s.heights.control - 3.0 * s.spacing.m).max(0.0);
        // The records are old while the last reading failed: the list says so as long as it is true, not only the
        // message (which goes when the person clicks it away).
        let stale = session.is_stale().then(|| {
            div().id("list-stale").child(look.text(
                say.say("list.stale"),
                s.text.caption,
                look.palette().error,
            ))
        });
        let body = div()
            .flex_col()
            .gap_px(s.spacing.s)
            .w(inner_w)
            .children(stale)
            .child(list_body(
                look,
                say,
                session.list(),
                &self.position,
                (inner_w, inner_h),
                &ListEvents { open, sort },
            ));
        (title, actions, body)
    }

    pub(super) fn opener(&self) -> Arc<dyn Fn(&str) + Send + Sync> {
        let (session, live) = (Arc::clone(&self.env.session), self.live);
        Arc::new(move |id| {
            lock(&session).open_detail(id);
            live.refresh_now();
        })
    }

    pub(super) fn sorter(&self) -> Arc<dyn Fn(usize) + Send + Sync> {
        let (session, live) = (Arc::clone(&self.env.session), self.live);
        Arc::new(move |c| {
            lock(&session).list_mut().cycle_sort(c);
            live.refresh_now();
        })
    }

    pub(super) fn detail_screen(&self) -> (String, Vec<Div>, Div) {
        let (look, say) = (&self.env.look, &*self.env.say);
        let session = lock(&self.env.session);
        let title = session
            .detail()
            .first()
            .and_then(|e| e.values.first().cloned())
            .unwrap_or_else(|| say.say(&session.description().label));
        // A record that could not be read (a failed reload) has no commands: nothing is done to what is not there.
        let loaded = session.record().is_some();
        let mut actions = if loaded {
            self.command_buttons(&session.commands(Placement::OnRecord), &session)
        } else {
            Vec::new()
        };
        actions.push(div().child(button(
            &say.say("action.back"),
            Weight::Plain,
            "back",
            self.act(Session::back_to_list),
        )));
        let body = if loaded {
            fields_panel(look, say, &session.detail())
        } else {
            panel(look).child(div().id("not-loaded").child(look.text(
                say.say("detail.not-loaded"),
                look.style.text.body,
                look.palette().error,
            )))
        };
        (title, actions, body)
    }

    pub(super) fn confirm_screen(&self, name: &str) -> (String, Vec<Div>, Div) {
        let (look, say) = (&self.env.look, &*self.env.say);
        let s = look.style;
        let session = lock(&self.env.session);
        let label = session
            .command(name)
            .map_or(String::new(), |p| say.say(&p.label));
        let yes = button(
            &label,
            Weight::Danger,
            "confirm",
            self.act(|s| {
                s.submit();
            }),
        );
        let no = button(
            &say.say("action.cancel"),
            Weight::Plain,
            "cancel",
            self.act(Session::cancel),
        );
        let body = panel(look)
            .child(look.text(say.say("confirm.sure"), s.text.title, look.palette().text))
            .child(div().flex_row().gap_px(s.spacing.s).child(yes).child(no));
        (label, Vec::new(), body)
    }

    pub(super) fn result_screen(&self, name: &str) -> (String, Vec<Div>, Div) {
        let (look, say) = (&self.env.look, &*self.env.say);
        let session = lock(&self.env.session);
        let (title, fields) = session
            .command(name)
            .map_or((String::new(), Vec::new()), |p| {
                (say.say(&p.label), p.result.clone())
            });
        let values = session
            .result()
            .map(|o| o.values.clone())
            .unwrap_or_default();
        let mut t = fs_types::TypeDescription::new("Result", "result", "result", "result");
        t.fields = fields;
        let record = crate::Record {
            id: String::new(),
            values,
        };
        let back = button(
            &say.say("action.back"),
            Weight::Plain,
            "back",
            self.act(Session::cancel),
        );
        (
            title,
            vec![div().child(back)],
            fields_panel(look, say, &entries(&t, &record)),
        )
    }

    pub(super) fn about_page(&self, about: &AboutInfo) -> Div {
        let (look, say) = (&self.env.look, &*self.env.say);
        let (s, pal) = (look.style, look.palette());
        let me = self.clone_handles();
        let back = button(
            &say.say("action.back"),
            Weight::Plain,
            "about-back",
            Arc::new(move || me.set_about(None)),
        );
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
                    .child(div().w(s.text.body * 12.0).flex_shrink_0().child(look.text(
                        say.say(key),
                        s.text.body,
                        pal.text_weak,
                    )))
                    .child(look.text(value.clone(), s.text.body, pal.text)),
            );
        }
        div()
            .flex_col()
            .gap_px(s.spacing.m)
            .padding_x_px(s.spacing.xl)
            .padding_y_px(s.spacing.xl)
            .w_full()
            .bg(Look::color(pal.ground))
            .child(header(look, &about.name, vec![div().child(back)]))
            .child(body)
    }
}
