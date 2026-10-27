//! The Devices layout's worktree group: one collapsible header over the
//! sessions that share a worktree, and the two-line text of a session row.

use crate::{
    HerdrWindow,
    config::TokenStyle,
    sidebar::{
        agents::status_indicator,
        cell::RowContext,
        label_text, line_height,
        tokens::{ResolvedToken, TextRole, TokenKind},
        unified::{self, Facts, Group},
    },
};
use gpui::{prelude::*, *};

/// Dots in a header's summary stop here; the count says the rest.
const SUMMARY_DOTS: usize = 4;

/// A session's two lines: its title, then where it runs. A missing detail
/// leaves one line.
pub(super) fn session_lines(
    facts: &Facts,
    theme: &crate::config::Theme,
) -> Vec<Vec<ResolvedToken>> {
    let tokens = theme.tokens();
    let text = |text: String, role, fg| ResolvedToken {
        kind: TokenKind::Text(text.into(), role),
        style: TokenStyle {
            fg: Some(fg),
            ..TokenStyle::default()
        },
    };
    let mut lines = vec![vec![
        ResolvedToken {
            kind: TokenKind::StateIcon,
            style: TokenStyle::default(),
        },
        text(facts.title.clone(), TextRole::Workspace, tokens.text),
    ]];
    if let Some(detail) = unified::detail_line(facts) {
        lines.push(vec![text(detail, TextRole::Muted, tokens.faint)]);
    }
    lines
}

impl HerdrWindow {
    pub(in crate::sidebar) fn toggle_worktree_group(&mut self, key: &str, cx: &mut Context<Self>) {
        unified::toggle(&mut self.collapsed_worktree_groups, key);
        cx.notify();
    }

    /// The header over a group's sessions: name, branch chip, session count,
    /// and a dot for each state present.
    pub(super) fn worktree_group_header(
        &self,
        endpoint_id: &str,
        group: &Group,
        row_cx: &RowContext<'_>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let (font, theme) = (row_cx.font, row_cx.theme);
        let tokens = theme.tokens();
        let key = group.key.clone();
        let selector = format!("worktree-group-{endpoint_id}-{}", group.key);
        let count: usize = group.members.len();
        let mut dots = div().flex_none().flex().items_start().gap(px(3.));
        let mut drawn = 0;
        for (status, number) in &group.summary {
            for _ in 0..*number {
                if drawn == SUMMARY_DOTS {
                    break;
                }
                dots = dots.child(status_indicator(*status, font, row_cx.indicators));
                drawn += 1;
            }
        }
        div()
            .id(SharedString::from(selector.clone()))
            .debug_selector(move || selector.clone())
            .role(Role::Button)
            .aria_label(format!(
                "{} worktree, {count} sessions, {}",
                group.label,
                if group.collapsed {
                    "collapsed"
                } else {
                    "expanded"
                }
            ))
            .h(px(line_height(font)))
            .pl(px(row_cx.look.content_x() + row_cx.nest))
            .pr(px(row_cx.look.content_x()))
            .flex()
            .items_start()
            .gap(px(6.))
            .cursor_pointer()
            .text_color(rgb(tokens.dim))
            .hover(move |style| style.bg(rgb(tokens.hover)))
            .child(label_text(if group.collapsed {
                "\u{25b8}"
            } else {
                "\u{25be}"
            }))
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_color(rgb(tokens.text))
                    .child(label_text(&group.label)),
            )
            .when_some(group.branch.clone(), |header, branch| {
                header.child(
                    div()
                        .flex_shrink(1.)
                        .min_w_0()
                        .truncate()
                        .px(px(4.))
                        .rounded(px(3.))
                        .bg(rgb(tokens.hover))
                        .text_color(rgb(tokens.dim))
                        .child(label_text(&branch)),
                )
            })
            .child(
                div()
                    .flex_none()
                    .text_color(rgb(tokens.faint))
                    .child(label_text(&count.to_string())),
            )
            .child(div().flex_1())
            .child(dots)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.toggle_worktree_group(&key, cx);
            }))
    }
}
