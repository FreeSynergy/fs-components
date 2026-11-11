//! A scene on a Blinc canvas.
//!
//! Pattern: Visitor over the scene — the only code in the widget that talks to Blinc's drawing API.
//! Primitives outside the view are skipped before a path is built; colours come from `look`.

use blinc_core::{
    Brush, ClipShape, DrawContext, FontWeight, Path, Point, Rect, Stroke, TextAlign, TextBaseline,
    TextStyle,
};

use super::look::colour;
use super::model::{Paint, TilePoint};
use super::scene::{Placed, Scene};

/// Size of the pin of a marker, in pixels.
pub const PIN: f32 = 9.0;

/// Draw `scene`, clipped to its view.
pub fn draw(ctx: &mut dyn DrawContext, scene: &Scene) {
    ctx.push_clip(ClipShape::Rect(Rect::new(
        0.0,
        0.0,
        scene.width,
        scene.height,
    )));
    for tile in &scene.tiles {
        areas(ctx, tile);
    }
    for tile in &scene.tiles {
        lines(ctx, tile);
    }
    for tile in &scene.tiles {
        dots(ctx, tile);
    }
    for label in &scene.labels {
        let paint = scene.tiles[label.tile].geometry.labels[label.index].paint;
        text(ctx, &label.text, (label.x, label.y), label.size, paint);
    }
    for placed in &scene.markers {
        marker(ctx, placed);
    }
    ctx.pop_clip();
}

fn screen(tile: &Placed, points: &[TilePoint]) -> Vec<[f32; 2]> {
    points.iter().map(|p| tile.to_screen(*p)).collect()
}

fn ring(path: Path, points: &[[f32; 2]], close: bool) -> Path {
    let mut it = points.iter();
    let Some(first) = it.next() else {
        return path;
    };
    let mut path = path.move_to(first[0], first[1]);
    for p in it {
        path = path.line_to(p[0], p[1]);
    }
    if close { path.close() } else { path }
}

fn areas(ctx: &mut dyn DrawContext, tile: &Placed) {
    for &i in &tile.areas {
        let area = &tile.geometry.areas[i];
        let rings: Vec<Vec<[f32; 2]>> = area.rings.iter().map(|r| screen(tile, r)).collect();
        let path = rings.iter().fold(Path::new(), |p, r| ring(p, r, true));
        ctx.fill_path(&path, Brush::Solid(colour(area.fill)));
        if let Some((paint, width)) = area.outline {
            let stroke = Stroke::new(width);
            ctx.stroke_path(&path, &stroke, Brush::Solid(colour(paint)));
        }
    }
}

fn lines(ctx: &mut dyn DrawContext, tile: &Placed) {
    for &i in &tile.lines {
        let line = &tile.geometry.lines[i];
        let points = screen(tile, &line.points);
        let stroke = Stroke::new(line.width);
        let path = ring(Path::new(), &points, false);
        ctx.stroke_path(&path, &stroke, Brush::Solid(colour(line.paint)));
    }
}

fn dots(ctx: &mut dyn DrawContext, tile: &Placed) {
    for &i in &tile.dots {
        let dot = &tile.geometry.dots[i];
        let [x, y] = tile.to_screen(dot.at);
        let brush = Brush::Solid(colour(dot.paint));
        ctx.fill_circle(Point::new(x, y), dot.radius, brush);
    }
}

fn text(ctx: &mut dyn DrawContext, words: &str, (x, y): (f32, f32), size: f32, paint: Paint) {
    let style = TextStyle {
        size,
        color: colour(paint),
        weight: FontWeight::Regular,
        align: TextAlign::Left,
        baseline: TextBaseline::Top,
        ..TextStyle::default()
    };
    let half = words.chars().count() as f32 * size * 0.56 / 2.0;
    ctx.draw_text(words, Point::new(x - half, y - size / 2.0), &style);
}

/// A pin: a ring with a dot. Marked ones are drawn as a square with a cross, so that they cannot be
/// taken for ordinary ones, and carry no word.
fn marker(ctx: &mut dyn DrawContext, placed: &super::scene::PlacedMarker) {
    let m = &placed.marker;
    let tone = if m.marked {
        super::model::Tone::Warning
    } else if m.selected {
        super::model::Tone::Error
    } else {
        super::model::Tone::Primary
    };
    let fill = colour(Paint::tone(tone));
    let edge = colour(Paint::tone(super::model::Tone::Background));
    let at = Point::new(placed.x, placed.y);
    if m.marked {
        let h = PIN;
        let square = ring(
            Path::new(),
            &[
                [placed.x - h, placed.y - h],
                [placed.x + h, placed.y - h],
                [placed.x + h, placed.y + h],
                [placed.x - h, placed.y + h],
            ],
            true,
        );
        ctx.fill_path(&square, Brush::Solid(fill));
        ctx.stroke_path(&square, &Stroke::new(2.0), Brush::Solid(edge));
        let cross = ring(
            Path::new(),
            &[
                [placed.x - 5.0, placed.y - 5.0],
                [placed.x + 5.0, placed.y + 5.0],
            ],
            false,
        );
        let cross = ring(
            cross,
            &[
                [placed.x - 5.0, placed.y + 5.0],
                [placed.x + 5.0, placed.y - 5.0],
            ],
            false,
        );
        ctx.stroke_path(&cross, &Stroke::new(2.0), Brush::Solid(edge));
        return;
    }
    let r = if m.selected { PIN + 3.0 } else { PIN };
    ctx.fill_circle(at, r, Brush::Solid(fill));
    ctx.stroke_circle(at, r, &Stroke::new(2.5), Brush::Solid(edge));
    ctx.fill_circle(at, 3.0, Brush::Solid(edge));
    if let Some(label) = &m.label {
        let style = TextStyle {
            size: 13.0,
            color: colour(Paint::tone(super::model::Tone::Text)),
            baseline: TextBaseline::Top,
            ..TextStyle::default()
        };
        ctx.draw_text(
            label,
            Point::new(placed.x + r + 4.0, placed.y - 8.0),
            &style,
        );
    }
}
