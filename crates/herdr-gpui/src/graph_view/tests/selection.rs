use super::paint::{bounds, history, open_with};
use crate::config::Theme;
use crate::graph_view::render::row::{row_background, selected_edge_color};

#[test]
fn a_selected_row_uses_the_sidebars_selected_token() {
    let theme = Theme::default();
    assert_eq!(row_background(true, &theme), theme.tokens().selected);
    assert_eq!(row_background(false, &theme), theme.background);
    assert_ne!(row_background(true, &theme), row_background(false, &theme));
}

#[test]
fn the_selected_edge_uses_the_sidebars_accent_token() {
    let theme = Theme::default();
    assert_eq!(selected_edge_color(&theme), theme.tokens().accent);
}

#[gpui::test]
fn only_the_selected_row_draws_an_accent_edge(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = super::tab::window(cx, Some(folder.path()));
    open_with(&view, history(), cx);
    let edge = bounds(cx, "graph-row-selected-edge");
    let row = bounds(cx, "graph-row-0");
    assert!(edge.size.width >= gpui::px(2.0), "edge too thin: {edge:?}");
    assert!(
        edge.origin.y >= row.origin.y && edge.bottom() <= row.bottom(),
        "the edge {edge:?} is not inside the selected row {row:?}"
    );
}
