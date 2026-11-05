#![allow(clippy::expect_used)]

//! Active handoff rows sit on the sidebar's own background with white text,
//! the detail line a step dimmer; history rows are gray.
use super::super::handoffs::{pin_ink, row_colors};
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

#[test]
fn the_pin_ink_is_accent_and_solid_when_pinned_and_a_faint_muted_when_not() {
    for theme in themes() {
        let (pinned, pinned_opacity) = pin_ink(&theme, true);
        let (open, open_opacity) = pin_ink(&theme, false);
        // The accent itself, unless it needs a nudge to reach 3:1.
        let accent = theme.tokens().accent;
        if ratio(accent, theme.background) >= 3.0 {
            assert_eq!(pinned, accent);
        }
        assert_eq!(pinned_opacity, 1.0);
        assert_eq!(open, theme.muted);
        assert!(open_opacity < 0.5, "the open lock recedes: {open_opacity}");
        assert_ne!(pinned, open, "colours differ");
        assert_ne!(pinned_opacity, open_opacity, "opacities differ");
    }
}

#[test]
fn the_pinned_lock_keeps_three_to_one_against_the_background() {
    for theme in themes() {
        let (color, opacity) = pin_ink(&theme, true);
        assert_eq!(opacity, 1.0, "full opacity, so the colour is what shows");
        for background in [theme.background, theme.tokens().rail] {
            let seen = ratio(color, background);
            println!("pinned lock {color:06x} on {background:06x}: {seen:.2}:1");
            assert!(seen >= 3.0, "contrast {seen} on {background:x}");
        }
    }
}
