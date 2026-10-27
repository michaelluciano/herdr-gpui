use super::*;

#[test]
fn the_default_theme_resolves_the_soft_dark_palette_through_tokens() {
    let tokens = Theme::default().tokens();
    for (name, got, want) in [
        ("bg", tokens.bg, 0x26282c),
        ("panel", tokens.panel, 0x2b2d31),
        ("rail", tokens.rail, 0x212327),
        ("line", tokens.line, 0x3a3d43),
        ("text", tokens.text, 0xd8dadf),
        ("dim", tokens.dim, 0x9fa3ab),
        ("faint", tokens.faint, 0x767b84),
        ("working", tokens.working, 0xd9b66a),
        ("idle", tokens.idle, 0x8d929b),
        ("done", tokens.done, 0x7fb596),
        ("blocked", tokens.blocked, 0xd9786a),
        ("accent", tokens.accent, 0xa99be0),
        ("hover", tokens.hover, 0x33353a),
        ("selected", tokens.selected, 0x363849),
    ] {
        assert_eq!(got, want, "{name}: {got:06x} != {want:06x}");
    }
}

#[test]
fn the_default_terminal_background_is_the_soft_dark_bg() {
    assert_eq!(Theme::default().background, 0x26282c);
    assert_eq!(Theme::default().cursor, Theme::default().foreground);
}

#[test]
fn a_system_accent_replaces_only_the_accent_token() {
    let theme = Theme::default().with_accent(Some(0x123456));
    assert_eq!(theme.tokens().accent, 0x123456);
    assert_eq!(theme.tokens().selected, 0x363849);
}

#[test]
fn other_themes_derive_tokens_from_their_own_colors() -> anyhow::Result<()> {
    let default = Theme::default().tokens();
    for name in Theme::BUILTIN_NAMES
        .iter()
        .filter(|name| **name != "Default")
    {
        let theme = Theme::builtin(name).ok_or_else(|| anyhow::anyhow!("missing {name}"))?;
        let tokens = theme.tokens();
        assert_eq!(tokens.bg, theme.background, "{name}");
        assert_eq!(tokens.panel, theme.surface, "{name}");
        assert_eq!(tokens.text, theme.foreground, "{name}");
        assert_eq!(tokens.dim, theme.muted, "{name}");
        assert_eq!(tokens.selected, theme.active, "{name}");
        assert_eq!(tokens.accent, theme.primary(), "{name}");
        assert_ne!(tokens.line, tokens.bg, "{name}");
        assert_ne!(tokens.faint, tokens.bg, "{name}");
        assert_ne!(tokens.bg, default.bg, "{name}");
        assert_ne!(
            tokens.accent, 0xa99be0,
            "{name} must not inherit the default accent"
        );
    }
    Ok(())
}

#[test]
fn every_status_token_exists_and_differs_from_the_background() -> anyhow::Result<()> {
    let nord = Theme::builtin("Nord").ok_or_else(|| anyhow::anyhow!("missing Nord"))?;
    for theme in [Theme::default(), nord] {
        let tokens = theme.tokens();
        for color in [tokens.working, tokens.idle, tokens.done, tokens.blocked] {
            assert_ne!(color, tokens.bg);
        }
    }
    Ok(())
}
