//! Painting the gutter of one graph row from its [`Segment`]s.
use crate::{config::Theme, graph_view::geometry::Segment};
use gpui::{prelude::*, *};

/// Indices into the theme's palette, one per lane color: blue, green,
/// yellow, magenta, red, cyan (the bright ANSI colors).
const LANE_PALETTE: [usize; 6] = [12, 10, 11, 13, 9, 14];
const STROKE: f32 = 1.5;

/// The color of `lane`, cycling through the palette.
pub(super) fn lane_color(theme: &Theme, lane: usize) -> Hsla {
    let index = LANE_PALETTE[lane % LANE_PALETTE.len()];
    rgb(theme
        .palette
        .get(index)
        .copied()
        .unwrap_or(theme.foreground))
    .into()
}

/// A canvas that paints `segments` into the bounds it is given.
pub(super) fn gutter_canvas(segments: Vec<Segment>, theme: &Theme) -> impl IntoElement {
    let colors: Vec<Hsla> = (0..LANE_PALETTE.len())
        .map(|lane| lane_color(theme, lane))
        .collect();
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let at = |x: f32, y: f32| point(bounds.left() + px(x), bounds.top() + px(y));
            let color = |lane: usize| colors[lane % colors.len()];
            for segment in &segments {
                match *segment {
                    Segment::Line {
                        lane,
                        x0,
                        x1,
                        y0,
                        y1,
                    } => stroke(window, at(x0, y0), at(x1, y1), color(lane)),
                    Segment::Dangling {
                        lane,
                        x0,
                        x1,
                        y0,
                        y1,
                    } => stroke(window, at(x0, y0), at(x1, y1), color(lane).opacity(0.5)),
                    Segment::Dot { cx, cy, lane, r } => {
                        let size = size(px(r * 2.0), px(r * 2.0));
                        let dot = Bounds::new(at(cx - r, cy - r), size);
                        window.paint_quad(fill(dot, color(lane)).corner_radii(Corners::all(px(r))));
                    }
                }
            }
        },
    )
    .absolute()
    .size_full()
}

fn stroke(window: &mut Window, from: Point<Pixels>, to: Point<Pixels>, color: Hsla) {
    let mut path = PathBuilder::stroke(px(STROKE));
    path.move_to(from);
    path.line_to(to);
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
}
