use super::{Dot, PULSE_LOW, pulse_opacity, spec};
use crate::config::Theme;
use gpui::{Context, IntoElement, ParentElement, Render, TestAppContext, Window, div};
use herdr_client::protocol::AgentStatus;

const STATUSES: [AgentStatus; 5] = [
    AgentStatus::Working,
    AgentStatus::Idle,
    AgentStatus::Done,
    AgentStatus::Blocked,
    AgentStatus::Unknown,
];

const SELECTORS: [&str; 3] = ["status-dot-pulse", "status-dot-hollow", "status-dot-filled"];

#[test]
fn each_status_takes_its_token_color_and_shape() {
    let tokens = Theme::default().tokens();
    let dot = |status| spec(status, &tokens);
    assert_eq!(
        (
            dot(AgentStatus::Working).color,
            dot(AgentStatus::Working).filled
        ),
        (0xd9b66a, true)
    );
    assert_eq!(
        (dot(AgentStatus::Idle).color, dot(AgentStatus::Idle).filled),
        (0x8d929b, false)
    );
    assert_eq!(
        (dot(AgentStatus::Done).color, dot(AgentStatus::Done).filled),
        (0x7fb596, true)
    );
    assert_eq!(
        (
            dot(AgentStatus::Blocked).color,
            dot(AgentStatus::Blocked).filled
        ),
        (0xd9786a, true)
    );
    assert_eq!(dot(AgentStatus::Unknown).color, tokens.faint);
    assert!(dot(AgentStatus::Unknown).small);
}

#[test]
fn only_a_working_dot_pulses() {
    let tokens = Theme::default().tokens();
    for status in STATUSES {
        assert_eq!(
            spec(status, &tokens).pulses,
            status == AgentStatus::Working,
            "{status:?}"
        );
    }
}

/// Reduced motion holds an animation on its first frame, so the pulse starts
/// at full strength and a held working dot is a solid one.
#[test]
fn the_pulse_starts_full_and_never_fades_out() {
    assert_eq!(pulse_opacity(0.), 1.);
    let lowest = (0..=100)
        .map(|step| pulse_opacity(step as f32 / 100.))
        .fold(1f32, f32::min);
    assert!((lowest - PULSE_LOW).abs() < 0.01, "{lowest}");
}

struct One(Dot);

impl Render for One {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.0.drawn(8.))
    }
}

/// A drawn dot names its shape, so every call site can be checked against
/// the spec without reading pixels.
#[gpui::test]
fn a_drawn_dot_is_the_shape_its_spec_says(cx: &mut TestAppContext) {
    let tokens = Theme::default().tokens();
    for status in STATUSES {
        let dot = spec(status, &tokens);
        let (_, cx) = cx.add_window_view(|_, _| One(dot));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let want = match (dot.pulses, dot.filled) {
            (true, _) => "status-dot-pulse",
            (false, false) => "status-dot-hollow",
            (false, true) => "status-dot-filled",
        };
        assert_eq!(dot.selector(), want, "{status:?}");
        for selector in SELECTORS {
            assert_eq!(
                cx.debug_bounds(selector).is_some(),
                selector == want,
                "{status:?} {selector}"
            );
        }
    }
}
