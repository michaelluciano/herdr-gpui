use super::*;
use crate::config::fonts::{LINUX_FONTS, MACOS_FONTS, PLATFORM_FONTS, WINDOWS_FONTS};
use std::collections::BTreeSet;

fn installed(families: &[&str]) -> BTreeSet<String> {
    families.iter().map(|family| (*family).to_owned()).collect()
}

#[test]
fn installed_defaults_are_kept() {
    let fonts = installed(&["DejaVu Sans Mono", "DejaVu Sans", "Noto Sans Mono"]);
    assert_eq!(LINUX_FONTS.substitute("DejaVu Sans Mono", &fonts), None);
    assert_eq!(LINUX_FONTS.substitute("DejaVu Sans", &fonts), None);
    let fonts = installed(&["Cascadia Mono", "Consolas"]);
    assert_eq!(WINDOWS_FONTS.substitute("Cascadia Mono", &fonts), None);
}

#[test]
fn missing_linux_defaults_take_the_first_known_alternative() {
    let fonts = installed(&["Ubuntu Mono", "Liberation Mono", "Cantarell", "Noto Sans"]);
    assert_eq!(
        LINUX_FONTS
            .substitute("DejaVu Sans Mono", &fonts)
            .as_deref(),
        Some("Liberation Mono")
    );
    assert_eq!(
        LINUX_FONTS.substitute("DejaVu Sans", &fonts).as_deref(),
        Some("Noto Sans")
    );
}

#[test]
fn omarchy_gets_its_jetbrains_mono_nerd_font() {
    // Omarchy's default font, alongside the Noto fonts Arch also installs.
    let fonts = installed(&[
        "CaskaydiaMono Nerd Font",
        "JetBrainsMono Nerd Font",
        "JetBrainsMono Nerd Font Mono",
        "Noto Sans",
        "Noto Sans Mono",
        "Noto Color Emoji",
    ]);
    assert_eq!(
        LINUX_FONTS
            .substitute("DejaVu Sans Mono", &fonts)
            .as_deref(),
        Some("JetBrainsMono Nerd Font")
    );
    assert_eq!(
        LINUX_FONTS.substitute("DejaVu Sans", &fonts).as_deref(),
        Some("Noto Sans")
    );
}

#[test]
fn missing_linux_monospace_falls_back_to_an_installed_text_mono_family() {
    // No listed family: any text `Mono` face beats a proportional fallback.
    let fonts = installed(&[
        "Symbols Nerd Font Mono",
        "Iosevka Nerd Font Propo",
        "Iosevka Nerd Font Mono",
        "Noto Color Emoji",
    ]);
    assert_eq!(
        LINUX_FONTS
            .substitute("DejaVu Sans Mono", &fonts)
            .as_deref(),
        Some("Iosevka Nerd Font Mono")
    );
    // With no sans family, the UI reads in monospace text, not a missing face.
    assert_eq!(
        LINUX_FONTS.substitute("DejaVu Sans", &fonts).as_deref(),
        Some("Iosevka Nerd Font Mono")
    );
}

#[test]
fn windows_without_cascadia_uses_consolas() {
    // Windows 10 without Windows Terminal has no Cascadia Mono.
    let fonts = installed(&["Segoe UI", "Lucida Console", "Consolas"]);
    assert_eq!(
        WINDOWS_FONTS.substitute("Cascadia Mono", &fonts).as_deref(),
        Some("Consolas")
    );
}

#[test]
fn chosen_aliased_and_unmatched_families_are_left_alone() {
    let fonts = installed(&["Noto Sans Mono", "Consolas"]);
    assert_eq!(LINUX_FONTS.substitute("Iosevka", &fonts), None);
    assert_eq!(
        LINUX_FONTS.substitute("DejaVu Sans Mono", &installed(&["Zapfino"])),
        None
    );
    // GPUI resolves `.SystemUIFont` itself; it is never listed as installed.
    assert_eq!(WINDOWS_FONTS.substitute(".SystemUIFont", &fonts), None);
    // macOS always ships its defaults, so resolution never checks them.
    assert!(!MACOS_FONTS.is_replaceable("Menlo"));
    assert_eq!(MACOS_FONTS.substitute("Menlo", &fonts), None);
}

#[test]
fn resolution_replaces_missing_platform_defaults_only() -> anyhow::Result<()> {
    let mut config = Config::parse("[terminal]\nfamily = 'Iosevka'")?;
    let default = Config::default();
    config.resolve_fonts_with(
        || ["Noto Sans Mono", "Noto Sans", "Consolas"].map(str::to_owned),
        |_| false,
    );
    let monospace = if cfg!(target_os = "linux") {
        "Noto Sans Mono"
    } else if cfg!(windows) {
        "Consolas"
    } else {
        "Menlo"
    };
    let sans = if cfg!(target_os = "linux") {
        "Noto Sans"
    } else {
        PLATFORM_FONTS.sans
    };
    assert_eq!(config.sidebar.family, monospace);
    assert_eq!(config.tabs.family, monospace);
    assert_eq!(config.terminal.family, "Iosevka");
    assert_eq!(config.ui.family, sans);
    // Defaults themselves are untouched; only the loaded copy is resolved.
    assert_eq!(default.terminal.family, crate::bundled_font::FAMILY);
    Ok(())
}

fn bundled_installed(extra: &[&str]) -> Vec<String> {
    [crate::bundled_font::FAMILY]
        .iter()
        .chain(extra)
        .map(|family| (*family).to_owned())
        .collect()
}

#[test]
fn every_face_defaults_to_the_bundled_font() -> anyhow::Result<()> {
    for config in [Config::default(), Config::parse("")?] {
        for face in [&config.sidebar, &config.tabs, &config.terminal, &config.ui] {
            assert_eq!(face.family, "B612 Mono");
        }
    }
    Ok(())
}

#[test]
fn the_bundled_font_stays_when_registered_and_a_user_family_still_wins() -> anyhow::Result<()> {
    let mut config = Config::parse("[terminal]\nfamily = 'Iosevka'")?;
    config.resolve_fonts_with(|| bundled_installed(&["Iosevka", "Menlo"]), |_| false);
    assert_eq!(config.terminal.family, "Iosevka");
    for face in [&config.sidebar, &config.tabs, &config.ui] {
        assert_eq!(face.family, "B612 Mono");
    }
    assert!(config.missing_fonts.is_empty());
    Ok(())
}

#[test]
fn a_bundled_font_that_failed_to_load_falls_back_to_the_platform_default() {
    let mut config = Config::default();
    config.resolve_fonts_with(
        || ["Noto Sans Mono", "Noto Sans", "Consolas"].map(str::to_owned),
        |_| false,
    );
    let monospace = if cfg!(target_os = "linux") {
        "Noto Sans Mono"
    } else if cfg!(windows) {
        "Consolas"
    } else {
        "Menlo"
    };
    let sans = if cfg!(target_os = "linux") {
        "Noto Sans"
    } else {
        PLATFORM_FONTS.sans
    };
    assert_eq!(config.terminal.family, monospace);
    assert_eq!(config.sidebar.family, monospace);
    assert_eq!(config.tabs.family, monospace);
    assert_eq!(config.ui.family, sans);
    assert!(
        config.missing_fonts.is_empty(),
        "the fallback is silent here"
    );
}

#[test]
fn the_terminal_cascade_reaches_the_platform_monospace_for_box_drawing() -> anyhow::Result<()> {
    // B612 Mono has no box-drawing, block or braille glyphs; the platform
    // monospace draws them.
    let monospace = PLATFORM_FONTS.monospace;
    let mut config = Config::default();
    config.resolve_fonts_with(
        || bundled_installed(&[monospace, "Symbols Nerd Font Mono"]),
        |_| false,
    );
    assert_eq!(
        config.terminal.fallbacks.as_deref(),
        Some(&[monospace.to_owned(), "Symbols Nerd Font Mono".to_owned()][..])
    );
    // A cascade the user wrote, even an empty one, is kept as written.
    let mut config = Config::parse("[terminal]\nfallback = []")?;
    config.resolve_fonts_with(|| bundled_installed(&[monospace]), |_| false);
    assert_eq!(config.terminal.fallbacks, Some(vec![]));
    Ok(())
}
