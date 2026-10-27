# Bundled fonts

B612 Mono (Regular, Bold, Italic, Bold Italic), the default UI and terminal
font. Embedded by `src/bundled_font.rs` and registered with the text system at
startup.

- Source: https://github.com/polarsys/b612, `fonts/ttf/B612Mono-*.ttf`
- Commit: `48ac6ba67ecab8123e8e36d6aa05367db0c7b638` (master when fetched, 2026-10-26)
- License: SIL Open Font License 1.1, `OFL.txt` here (copied from the same commit).
  Copyright 2012 The B612 Project Authors.
- Files are unmodified.

Coverage: 590 glyphs, Latin, Greek and Cyrillic. It has no box-drawing, block or
braille glyphs; the terminal cascade supplies those from the platform monospace.
