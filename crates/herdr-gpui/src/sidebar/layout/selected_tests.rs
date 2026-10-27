use super::*;
use crate::config::{Density, Style};

#[test]
fn a_selected_fill_row_has_a_two_pixel_accent_edge() {
    let theme = Theme::default();
    let look = for_mode(LayoutMode::Classic {
        density: Density::Normal,
        style: Style::Flat,
    });
    assert_eq!(look.selected_edge(&theme), Some((2., 0xa99be0)));
    assert_eq!(look.selected_fill(&theme), 0x363849);
    assert_eq!(look.hover_fill(&theme), 0x33353a);
}

#[test]
fn the_edge_follows_the_system_accent() {
    let theme = Theme::default().with_accent(Some(0x00ff00));
    let look = for_mode(LayoutMode::Classic {
        density: Density::Normal,
        style: Style::Flat,
    });
    assert_eq!(look.selected_edge(&theme), Some((2., 0x00ff00)));
}

#[test]
fn outline_rows_keep_their_border_and_get_no_edge() {
    let look = for_mode(LayoutMode::Classic {
        density: Density::Normal,
        style: Style::Rounded,
    });
    assert_eq!(look.selected_edge(&Theme::default()), None);
}
