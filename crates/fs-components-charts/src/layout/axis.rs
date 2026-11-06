//! One positional axis: its scale, its ticks and their labels.
//!
//! Pattern: Strategy by field type (continuous number, time, categories) behind one `Axis` type.
//! The scale is chosen as Vega-Lite chooses it: numbers get a linear scale that includes zero and
//! is niced, times a time scale without zero or nice, categories a band scale for bars and a
//! point scale for everything else.

use super::cats::Cats;
use crate::data::{Cell, time::MIN};
use crate::labels::Labels;
use crate::scale::format::{number, tick_label};
use crate::scale::timeticks::{pick, ticks as time_ticks};
use crate::scale::{Band, DomainRule, Linear};
use crate::spec::{Aggregate, Channel, ChannelDef, FieldType, MarkKind, Title};

/// The scale of an axis.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Scale {
    /// Numbers.
    Linear(Linear),
    /// Instants, with the offset (minutes) of the data's zone.
    Time(Linear, i32),
    /// Categories with bands.
    Band(Band),
    /// Categories at points.
    Point(Band),
}

/// A tick with its label; `pos` is pixels from the start of the axis.
#[derive(Clone, Debug, PartialEq)]
pub struct Tick {
    /// Position along the axis.
    pub pos: f32,
    /// The label.
    pub label: String,
}

/// An axis ready to draw.
#[allow(clippy::struct_excessive_bools)] // plain switches of one axis: grid, labels, visible, percent
#[derive(Clone, Debug)]
pub struct Axis {
    /// x or y.
    pub channel: Channel,
    /// The scale.
    pub scale: Scale,
    /// Ticks and labels, in order.
    pub ticks: Vec<Tick>,
    /// Categories, for a discrete axis.
    pub cats: Option<Cats>,
    /// Title, if shown.
    pub title: Option<String>,
    /// Grid lines at the ticks.
    pub grid: bool,
    /// Tick labels shown.
    pub labels: bool,
    /// The axis is drawn at all.
    pub visible: bool,
    /// Length in pixels.
    pub len: f32,
    /// Values are fractions shown as percent (normalised stack).
    pub percent: bool,
}

/// What the data says about an axis.
pub enum Source {
    /// Smallest and largest valid value; `None` when there is none.
    Range(Option<(f64, f64)>),
    /// The categories.
    Cats(Cats),
}

/// How the axis is used.
pub struct Use {
    /// The mark: bars get band scales.
    pub mark: MarkKind,
    /// The offset (minutes) for a time axis.
    pub offset: i32,
    /// The value axis of a normalised stack.
    pub percent: bool,
    /// Zero must be in the domain.
    pub force_zero: bool,
}

impl Axis {
    /// Build the axis of `def` over `len` pixels.
    pub fn new(
        channel: Channel,
        def: &ChannelDef,
        src: Source,
        len: f32,
        u: &Use,
        labels: &Labels,
    ) -> Self {
        let n = (len / 40.0).ceil().max(1.0) as usize;
        let (scale, ticks, cats) = match src {
            Source::Cats(cats) => {
                let rng = (0.0, f64::from(len));
                let band = if u.mark == MarkKind::Bar {
                    Scale::Band(Band::bars(cats.len(), rng))
                } else {
                    Scale::Point(Band::points(cats.len(), rng))
                };
                let (Scale::Band(b) | Scale::Point(b)) = &band else {
                    unreachable!()
                };
                let ticks = cats
                    .names
                    .iter()
                    .enumerate()
                    .map(|(i, name)| Tick {
                        pos: b.centre(i) as f32,
                        label: name.clone(),
                    })
                    .collect();
                (band, ticks, Some(cats))
            }
            Source::Range(r) => continuous(channel, def, r, len, n, u, labels),
        };
        let visible = def.axis.visible;
        Self {
            channel,
            scale,
            ticks,
            cats,
            title: title(def, labels),
            grid: def.axis.grid.unwrap_or(!def.ftype.is_discrete()) && visible,
            labels: def.axis.labels,
            visible,
            len,
            percent: u.percent,
        }
    }

    /// Pixel position of a cell along the axis; `None` for a value that is not on it.
    pub fn place(&self, cell: &Cell, labels: &Labels) -> Option<f32> {
        match (&self.scale, &self.cats) {
            (Scale::Linear(s) | Scale::Time(s, _), _) => cell.num().map(|v| s.map(v) as f32),
            (Scale::Band(b) | Scale::Point(b), Some(c)) => {
                c.index_of(cell, labels).map(|i| b.centre(i) as f32)
            }
            _ => None,
        }
    }
}

fn continuous(
    channel: Channel,
    def: &ChannelDef,
    range: Option<(f64, f64)>,
    len: f32,
    n: usize,
    u: &Use,
    labels: &Labels,
) -> (Scale, Vec<Tick>, Option<Cats>) {
    // y runs upward on screen
    let r = if channel == Channel::Y {
        (f64::from(len), 0.0)
    } else {
        (0.0, f64::from(len))
    };
    let (min, max) = range.unwrap_or((0.0, 1.0));
    if def.ftype == FieldType::Temporal {
        let (lo, hi) = def
            .scale
            .domain
            .map_or_else(|| time_domain(min, max), |d| (d[0], d[1]));
        let s = Linear {
            d0: lo,
            d1: hi,
            r0: r.0,
            r1: r.1,
        };
        let ts = time_ticks(lo, hi, pick(hi - lo, n), u.offset);
        let ticks = ts
            .iter()
            .map(|&t| Tick {
                pos: s.map(t) as f32,
                label: tick_label(t + f64::from(u.offset) * MIN, labels),
            })
            .collect();
        return (Scale::Time(s, u.offset), ticks, None);
    }
    let rule = DomainRule {
        zero: def.scale.zero.unwrap_or(def.scale.domain.is_none()) || u.force_zero,
        nice: def.scale.nice.unwrap_or(true),
        fixed: def.scale.domain,
    };
    let s = Linear::from_data(min, max, rule, r);
    let t = s.ticks(n);
    let decimals = crate::scale::ticks::step_for(s.d0.min(s.d1), s.d1.max(s.d0), n)
        .map_or(0, crate::scale::ticks::Step::decimals);
    let ticks = t
        .iter()
        .map(|&v| {
            let label = if u.percent {
                format!("{}%", number(v * 100.0, decimals.saturating_sub(2), labels))
            } else {
                number(v, decimals, labels)
            };
            Tick {
                pos: s.map(v) as f32,
                label,
            }
        })
        .collect();
    (Scale::Linear(s), ticks, None)
}

/// A time domain; one instant gets half a day on each side.
fn time_domain(min: f64, max: f64) -> (f64, f64) {
    if min == max {
        (min - 43_200_000.0, max + 43_200_000.0)
    } else {
        (min, max)
    }
}

/// The axis title from the spec, or made from the field (and aggregate).
fn title(def: &ChannelDef, l: &Labels) -> Option<String> {
    match &def.axis.title {
        Title::Hidden => None,
        Title::Text(t) => Some(t.clone()),
        Title::Auto => {
            let f = def.field.clone().unwrap_or_default();
            Some(match def.aggregate {
                None => f,
                Some(Aggregate::Count) => l.count_title.clone(),
                Some(a) => {
                    let t = match a {
                        Aggregate::Sum => &l.sum_title,
                        Aggregate::Mean => &l.mean_title,
                        Aggregate::Min => &l.min_title,
                        _ => &l.max_title,
                    };
                    Labels::fill(t, &[("field", &f)])
                }
            })
        }
    }
}
