#![allow(clippy::expect_used)]

//! Active handoff rows are a light chip with dark text (the theme's pair,
//! inverted); history rows keep the dimmed look.
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
fn active_row_inverts_the_theme_pair_and_reads_at_aa() {
    for theme in themes() {
        let colors = row_colors(&theme, None);
        assert_eq!(colors.bg, Some(theme.foreground));
        assert_eq!(colors.id_text, theme.background);
        let id = ratio(colors.id_text, theme.foreground);
        let detail = ratio(colors.detail_text, theme.foreground);
        assert!(id >= 4.5, "id contrast {id}");
        assert!(detail >= 4.5, "detail contrast {detail}");
        assert!(colors.hover_bg.is_some());
        let hover = ratio(colors.detail_text, colors.hover_bg.expect("hover"));
        assert!(hover >= 4.5, "hover contrast {hover}");
    }
}

#[test]
fn history_row_keeps_the_dimmed_look() {
    for theme in themes() {
        for shelf in [Shelf::PickedUp, Shelf::Superseded] {
            let colors = row_colors(&theme, Some(shelf));
            assert_eq!(colors.bg, None);
            assert_eq!(colors.hover_bg, None);
            assert_eq!(colors.id_text, theme.muted);
            assert_eq!(colors.detail_text, theme.muted);
        }
    }
}
