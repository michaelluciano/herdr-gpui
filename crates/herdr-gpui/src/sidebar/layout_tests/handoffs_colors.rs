#![allow(clippy::expect_used)]

//! Active handoff rows sit on the sidebar's own background with white text,
//! the detail line a step dimmer; history rows are gray.
use super::super::handoffs::row_colors;
use crate::{config::Theme, contrast::ratio, handoffs::Shelf};

fn themes() -> Vec<Theme> {
    vec![
        Theme::default(),
        Theme::builtin("Catppuccin Latte").expect("latte"),
        Theme::builtin("Catppuccin Mocha").expect("mocha"),
    ]
}

#[test]
fn active_row_is_white_text_on_the_sidebar_background_and_reads_at_aa() {
    for theme in themes() {
        let colors = row_colors(&theme, None);
        assert_eq!(colors.id_text, theme.foreground);
        assert_ne!(colors.detail_text, colors.id_text, "detail is dimmer");
        assert_ne!(colors.detail_text, theme.muted, "but not the history gray");
        let id = ratio(colors.id_text, theme.background);
        let detail = ratio(colors.detail_text, theme.background);
        assert!(id >= 4.5, "id contrast {id}");
        assert!(detail >= 4.5, "detail contrast {detail}");
        assert!(id > detail, "detail {detail} reads below id {id}");
    }
}

#[test]
fn history_row_is_gray() {
    for theme in themes() {
        for shelf in [Shelf::PickedUp, Shelf::Superseded, Shelf::Dismissed] {
            let colors = row_colors(&theme, Some(shelf));
            assert_eq!(colors.id_text, theme.muted);
            assert_eq!(colors.detail_text, theme.muted);
        }
    }
}
