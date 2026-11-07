//! From validated spec and clean data to a scene: the layout.
//!
//! Pattern: Facade (one function keeps the order: scales, margins by a fixed-point loop, marks, axes, legend, notes). The room an axis needs depends on its
//! labels, which depend on the length of the axis; the loop runs until the margins stop changing
//! (at most four rounds). Pure: the text measure is given, nothing here knows Blinc.

use super::arc;
use super::axis_items as ai;
use super::cats::{Cats, categories};
use super::ctx::Ctx;
use super::legend;
use super::measure::{Measure, Metrics};
use super::notes::notes;
use super::sanitize;
use crate::data::Prepared;
use crate::labels::Labels;
use crate::scene::{Anchor, Baseline, Item, Paint, Scene, TextSize};
use crate::spec::{Channel, MarkKind, Spec};

/// Colour categories above this number are refused: the palette cannot tell more apart.
pub const MAX_SERIES: usize = 8;

/// The colour categories of the data.
pub fn series(spec: &Spec, data: &Prepared, labels: &Labels) -> Cats {
    spec.encoding
        .color
        .as_ref()
        .map_or_else(Cats::default, |d| {
            categories(d, Channel::Color, &data.data, labels, None)
        })
}

/// Lay out the chart in a `w` x `h` box. A size that is not a number is read as 1 pixel and one
/// above a million pixels as a million.
pub fn build(
    spec: &Spec,
    data: &Prepared,
    labels: &Labels,
    m: &dyn Measure,
    k: &Metrics,
    w: f32,
    h: f32,
) -> Scene {
    let size = |v: f32| if v.is_nan() { 1.0 } else { v.clamp(1.0, 1e6) };
    let (w, h) = (size(w), size(h));
    let scene = build_in(spec, data, labels, m, k, (w, h), false);
    if scene.omitted > 0 && data.data.len() <= k.max_marks {
        // something was left out that nobody expected: lay out again with a line for the note
        return build_in(spec, data, labels, m, k, (w, h), true);
    }
    scene
}

fn build_in(
    spec: &Spec,
    data: &Prepared,
    labels: &Labels,
    m: &dyn Measure,
    k: &Metrics,
    size: (f32, f32),
    reserve_note: bool,
) -> Scene {
    let (w, h) = size;
    let mut scene = Scene {
        width: w,
        height: h,
        ..Scene::default()
    };
    let series = series(spec, data, labels);
    let c = Ctx {
        spec,
        data,
        labels,
        series: &series,
        max_marks: k.max_marks,
    };
    let pad = k.pad;
    let mut top = pad;
    if let Some(t) = &spec.title {
        let text = ai::fit(t, w - 2.0 * pad, k.title, true, m);
        scene.items.push(Item::Text {
            at: [w / 2.0, pad],
            text,
            size: TextSize::Title,
            bold: true,
            paint: Paint::Text,
            anchor: Anchor::Middle,
            baseline: Baseline::Top,
            angle: 0.0,
        });
        top += k.line(TextSize::Title) + k.title_gap * 2.0;
    }
    let lines = notes(&data.report, labels, m, k, w - 2.0 * pad);
    let reserve = usize::from(reserve_note || data.data.len() > k.max_marks);
    let note_h = (lines.len() + reserve) as f32 * k.line(TextSize::Label);
    let bottom_room = pad + note_h;
    for (i, text) in lines.iter().enumerate() {
        let y = h - pad - note_h + i as f32 * k.line(TextSize::Label);
        scene.items.push(Item::Text {
            at: [pad, y],
            text: text.clone(),
            size: TextSize::Label,
            bold: false,
            paint: Paint::Muted,
            anchor: Anchor::Start,
            baseline: Baseline::Top,
            angle: 0.0,
        });
    }
    if data.data.is_empty() || no_positions(spec, data) {
        let at = [w / 2.0, (top + h - bottom_room) / 2.0];
        scene.items.push(Item::Text {
            at,
            text: labels.empty.clone(),
            size: TextSize::AxisTitle,
            bold: false,
            paint: Paint::Muted,
            anchor: Anchor::Middle,
            baseline: Baseline::Middle,
            angle: 0.0,
        });
        scene.plot = [pad, top, w - 2.0 * pad, h - top - bottom_room];
        return scene;
    }
    let legend_w = legend::width(&c, m, k);
    if spec.mark.kind == MarkKind::Arc {
        let plot = [
            pad,
            top,
            (w - 2.0 * pad - legend_w).max(1.0),
            (h - top - bottom_room).max(1.0),
        ];
        scene.plot = plot;
        legend_at(&c, plot, w, m, k, &mut scene);
        arc::draw(&c, plot, &mut scene);
        sanitize::clean(&mut scene);
        omitted_note(&mut scene, labels, k);
        return scene;
    }
    super::cartesian::layout(&c, m, k, [pad, top, bottom_room, legend_w], &mut scene);
    sanitize::clean(&mut scene);
    omitted_note(&mut scene, labels, k);
    scene
}

pub(super) fn legend_at(
    c: &Ctx<'_>,
    plot: [f32; 4],
    w: f32,
    m: &dyn Measure,
    k: &Metrics,
    scene: &mut Scene,
) {
    let lw = legend::width(c, m, k);
    if lw > 0.0 {
        let y = plot[1] + ((plot[3] - legend::height(c, k)) / 2.0).max(0.0);
        legend::draw(c, w - k.pad - lw + legend::MARGIN, y, m, k, scene);
    }
}

/// Say how many marks were left out, in the line reserved under the chart.
fn omitted_note(scene: &mut Scene, labels: &Labels, k: &Metrics) {
    if scene.omitted == 0 {
        return;
    }
    let text = Labels::fill(
        &labels.note_omitted,
        &[("count", &scene.omitted.to_string())],
    );
    let y = scene.height - k.pad - k.line(TextSize::Label);
    scene.items.push(Item::Text {
        at: [k.pad, y],
        text,
        size: TextSize::Label,
        bold: false,
        paint: Paint::Muted,
        anchor: Anchor::Start,
        baseline: Baseline::Top,
        angle: 0.0,
    });
}

/// A line or area whose rows all lack a usable position (every date unreadable, say): there is
/// nothing to put on a scale, and an axis made up from nothing would mislead.
fn no_positions(spec: &Spec, data: &Prepared) -> bool {
    [Channel::X, Channel::Y].iter().any(|&ch| {
        spec.encoding
            .get(ch)
            .is_some_and(|d| !d.ftype.is_discrete())
            && data.data.iter().all(|r| r.cells[ch.index()].is_missing())
    })
}
