//! Named chrome colors. Chrome asks a [`Tokens`] for "line" or "selected"
//! instead of reaching into the terminal palette or mixing its own shade, so
//! one place decides what each role looks like in every theme.
use super::theme::{Theme, mix};

/// The roles that have no field of their own on [`Theme`]. Only the default
/// theme authors them; every other theme derives them from its terminal colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Authored {
    pub(crate) rail: u32,
    pub(crate) line: u32,
    pub(crate) faint: u32,
    pub(crate) hover: u32,
    pub(crate) working: u32,
    pub(crate) idle: u32,
    pub(crate) done: u32,
    pub(crate) blocked: u32,
}

/// The soft dark palette the default theme ships.
pub(crate) const SOFT_DARK: Authored = Authored {
    rail: 0x212327,
    line: 0x3a3d43,
    faint: 0x767b84,
    hover: 0x33353a,
    working: 0xd9b66a,
    idle: 0x8d929b,
    done: 0x7fb596,
    blocked: 0xd9786a,
};

/// Packed 24-bit RGB per role.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Tokens {
    pub(crate) accent: u32,
    pub(crate) bg: u32,
    pub(crate) blocked: u32,
    pub(crate) dim: u32,
    pub(crate) done: u32,
    pub(crate) faint: u32,
    pub(crate) hover: u32,
    pub(crate) idle: u32,
    pub(crate) line: u32,
    pub(crate) panel: u32,
    pub(crate) rail: u32,
    pub(crate) selected: u32,
    pub(crate) text: u32,
    pub(crate) working: u32,
}

impl Theme {
    /// The theme's named colors. Roles with a [`Theme`] field follow it, so
    /// contrast and accent settings still apply; the rest are authored for the
    /// default theme and mixed from the background and foreground otherwise.
    pub(crate) fn tokens(&self) -> Tokens {
        let blend = |percent| mix(self.background, self.foreground, percent);
        let derived = Authored {
            rail: self.background,
            line: blend(15),
            faint: blend(40),
            hover: blend(8),
            working: self.palette[3],
            idle: self.muted,
            done: self.palette[2],
            blocked: self.palette[1],
        };
        let authored = self.authored.unwrap_or(derived);
        Tokens {
            accent: self.primary(),
            bg: self.background,
            blocked: authored.blocked,
            dim: self.muted,
            done: authored.done,
            faint: authored.faint,
            hover: authored.hover,
            idle: authored.idle,
            line: authored.line,
            panel: self.surface,
            rail: self.sidebar.unwrap_or(authored.rail),
            selected: self.active,
            text: self.foreground,
            working: authored.working,
        }
    }
}
