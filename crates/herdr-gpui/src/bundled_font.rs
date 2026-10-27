//! B612 Mono, the font every face defaults to, embedded so it draws the same on
//! every machine. Licensed under the SIL OFL 1.1: `fonts/OFL.txt`, source in
//! `fonts/README.md`.

use std::borrow::Cow;

/// The family name inside the font files, which is what a config names.
pub(crate) const FAMILY: &str = "B612 Mono";

// keep-sorted start
const FACES: [&[u8]; 4] = [
    include_bytes!("../fonts/B612Mono-Bold.ttf"),
    include_bytes!("../fonts/B612Mono-BoldItalic.ttf"),
    include_bytes!("../fonts/B612Mono-Italic.ttf"),
    include_bytes!("../fonts/B612Mono-Regular.ttf"),
];
// keep-sorted end

/// Hands the bundled faces to the text system through `add`. An error leaves
/// the family unregistered; config resolution then falls back to the platform
/// default, so the caller only needs to report it.
pub(crate) fn register(
    add: impl FnOnce(Vec<Cow<'static, [u8]>>) -> gpui::Result<()>,
) -> gpui::Result<()> {
    add(FACES.iter().map(|bytes| Cow::Borrowed(*bytes)).collect())
}

/// Registers the faces with the app's text system. Run before any config is
/// resolved against the installed families.
pub(crate) fn install(cx: &gpui::App) {
    let text_system = cx.text_system();
    if let Err(error) = register(|faces| text_system.add_fonts(faces)) {
        tracing::warn!(%error, "Could not load the bundled B612 Mono font; using the platform default");
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_bundled_faces_reach_the_text_system() {
        let mut received = Vec::new();
        register(|faces| {
            received = faces;
            Ok(())
        })
        .unwrap();
        assert_eq!(received.len(), 4);
        for face in &received {
            // An sfnt TrueType header, naming the family a config asks for.
            assert_eq!(face[..4], [0, 1, 0, 0]);
            assert!(face.windows(FAMILY.len()).any(|w| w == FAMILY.as_bytes()));
        }
    }

    #[test]
    fn a_rejected_font_is_an_error_for_the_caller_to_report() {
        let rejected = register(|_| anyhow::bail!("Could not load an embedded font."));
        assert!(rejected.is_err());
    }
}
