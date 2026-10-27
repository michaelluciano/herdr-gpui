use super::*;
use crate::config::Config;

/// The cell grid follows the shaped advance of "M" at the font size, so the
/// bundled face (about 0.6em wide) sets its own columns without a constant.
#[gpui::test]
fn cell_width_is_shaped_from_the_default_font_and_scales_with_size(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| Empty);
    let terminal = Config::default().terminal;
    assert_eq!(terminal.family, "B612 Mono");
    let font = terminal.font();
    let widths = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    for size in [14., 28.] {
        let (font, widths) = (font.clone(), widths.clone());
        let line_height = terminal.line_height();
        cx.draw(Point::default(), gpui::size(px(800.), px(600.)), |_, _| {
            canvas(
                |_, _, _| (),
                move |_, _, window, cx| {
                    let mut painter = TerminalPainter::default();
                    painter.set_appearance(size, line_height, Theme::default());
                    let measured = painter.cell_width(&font, window, cx);
                    let shaped = window
                        .text_system()
                        .shape_line(
                            "M".into(),
                            px(size),
                            &[TextRun {
                                len: 1,
                                font: font.clone(),
                                color: Hsla::default(),
                                background_color: None,
                                underline: None,
                                strikethrough: None,
                            }],
                            None,
                        )
                        .width
                        .to_f64() as f32;
                    widths.borrow_mut().push((measured, shaped));
                },
            )
        });
    }
    let widths = widths.borrow();
    assert_eq!(widths.len(), 2);
    for (measured, shaped) in widths.iter() {
        assert!(
            *measured > 0. && measured == shaped,
            "{measured} vs {shaped}"
        );
    }
    assert!(
        widths[1].0 > widths[0].0,
        "the cell grows with the font size"
    );
}

#[test]
fn the_default_cell_height_leaves_room_for_the_font() {
    let terminal = Config::default().terminal;
    // B612 Mono's ascent plus descent is 1.215em; the default row is 1.43em.
    assert!(terminal.line_height() >= terminal.size * 1.215);
    assert_eq!(terminal.line_height(), CELL_HEIGHT);
}
