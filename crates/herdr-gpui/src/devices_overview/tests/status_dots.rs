use crate::{
    config::{FontConfig, StatusIcons, Theme},
    devices_overview::Look,
    sidebar::Indicators,
    status_dot::spec,
};
use gpui::{Context, IntoElement, ParentElement, Render, TestAppContext, Window, div};
use herdr_client::protocol::AgentStatus;

struct Dot {
    status: AgentStatus,
    theme: Theme,
    font: FontConfig,
}

impl Render for Dot {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let look = Look {
            theme: &self.theme,
            ui: &self.font,
            mono: &self.font,
            indicators: Indicators::new(None, StatusIcons::Dots, false, false, &self.theme),
        };
        div().child(look.dot(self.status))
    }
}

/// The Overview page, the activity card and the device table all draw agent
/// state with `Look::dot`, so it must be the shared dot: same shape as the
/// spec, and the spec's color.
#[gpui::test]
fn the_overview_dot_is_the_shared_one(cx: &mut TestAppContext) {
    let theme = Theme::default();
    for status in [
        AgentStatus::Working,
        AgentStatus::Idle,
        AgentStatus::Done,
        AgentStatus::Blocked,
    ] {
        let want = spec(status, &theme.tokens());
        let font = FontConfig {
            family: "Menlo".into(),
            size: 12.,
            fallbacks: None,
            line_height_multiple: None,
            weight: gpui::FontWeight::NORMAL,
        };
        let indicators = Indicators::new(None, StatusIcons::Dots, false, false, &theme);
        assert_eq!(indicators.color(status), theme.ink(want.color));
        let (_, cx) = cx.add_window_view(|_, _| Dot {
            status,
            theme: theme.clone(),
            font,
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        for selector in ["status-dot-pulse", "status-dot-hollow", "status-dot-filled"] {
            assert_eq!(
                cx.debug_bounds(selector).is_some(),
                selector == want.selector(),
                "{status:?} {selector}"
            );
        }
    }
}
